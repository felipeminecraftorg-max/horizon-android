//! Frame generation (`frameGen` in `horizon-graphics.txt`, off by default), in two modes:
//!
//! - **Target FPS**: the player picks a frame rate; while the real frames reach it, nothing
//!   happens, and below it made frames fill in only the frames that are missing, up to two
//!   after each real one. At 30 real frames a second and a target of 90, two are made after
//!   each real frame; at 50 and a target of 70, one after about every other real frame.
//! - **Adaptive**: the target is the frame rate the game usually reaches here, learned as
//!   it runs (it rises at once and settles slowly), so a drop in a heavy place is filled
//!   back up to it, one made frame at most after each real one.
//!
//! As the FrameGen mod and VR's reprojection do, a made frame is the last real frame moved
//! to where the camera is now (the newest touch input, so turning feels as fast as the
//! frame rate), through that frame's depth (`frame_warp.wgsl`). It costs one full-screen
//! pass instead of a whole frame, and adds no latency, where interpolation (FSR 3, Horizon
//! on PC) waits for the next real frame. Two copies of each real frame are kept while
//! frames may be made: the world as the screen showed it before the hand, and the frame
//! with the hand and screen effects but before the HUD. Where they differ (the hand, the
//! effects) the made frame keeps the pixels where they were, since those move with the
//! camera; the HUD is drawn fresh over every made frame. Mobs and animations stand still in
//! a made frame, and a fast turn folds the scenery at the edge into what the last frame
//! never saw. Frames past the display's refresh rate are never made: they would not show.

use glam::{DMat4, DVec3};
use std::time::{Duration, Instant};

/// How far (0..1 per channel) the overlay may differ from the world before a pixel counts
/// as the hand's or an effect's: 8-bit rounding stays under it.
const OVERLAY_THRESHOLD: f32 = 0.03;
/// Real frames this much slower than the target make frames; faster ones never do.
const SLACK: f64 = 1.03;
/// The camera moved further than this (blocks) since the real frame (a teleport, a
/// respawn): the next frame is real.
const MAX_TRAVEL: f64 = 4.0;

/// `frame_warp.wgsl`'s `Warp`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WarpUniform {
    to_previous: [[f32; 4]; 4],
    to_current: [[f32; 4]; 4],
    params: [f32; 4],
}

/// The two kept copies of the last real frame, at the screen's size.
struct Kept {
    size: [u32; 2],
    world: wgpu::Texture,
    world_view: wgpu::TextureView,
    overlay: wgpu::Texture,
    overlay_view: wgpu::TextureView,
}

/// What the last real frame left for the next made one.
struct Source {
    view_proj: DMat4,
    camera: DVec3,
    size: [u32; 2],
    group: wgpu::BindGroup,
    /// `world_post`'s target generation the bind group's depth view belongs to.
    generation: u64,
}

pub struct FrameWarp {
    layout: wgpu::BindGroupLayout,
    pipeline: Option<wgpu::RenderPipeline>,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    format: wgpu::TextureFormat,
    kept: Option<Kept>,
    source: Option<Source>,
    /// The setting, and Target FPS's frame rate (0: the display's refresh rate).
    pub mode: crate::graphics::FrameGen,
    pub target_fps: u32,
    /// Why no frame is made now, for the performance overlay (`status`).
    why: Why,
    /// The target and display rate of the last frame, for `status`.
    shown_target: u32,
    shown_refresh: u32,
    /// This frame keeps its copies (`keep_world` then `keep_overlay`).
    capturing: bool,
    world_kept: bool,
    /// The frame being drawn is a made one (`next_frame`).
    pub generate: bool,
    pacer: Pacer,
}

impl FrameWarp {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let format = format.remove_srgb_suffix();
        let texture = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let float = wgpu::TextureSampleType::Float { filterable: true };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Horizon frame warp bindings"),
            entries: &[
                texture(0, float),
                texture(1, float),
                texture(2, wgpu::TextureSampleType::Depth),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Horizon frame warp"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../frame_warp.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Horizon frame warp"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        // As TAA: a backend that cannot load from depth textures (OpenGL's GLSL
        // translator) cannot build it, and the setting then does nothing.
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Horizon frame warp"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_warp"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_warp"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let internal = pollster::block_on(internal.pop());
        let pipeline = match internal.or(pollster::block_on(validation.pop())) {
            None => Some(pipeline),
            Some(e) => {
                log::warn!("Horizon renderer: adaptive frame generation unavailable ({e})");
                None
            }
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Horizon frame warp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Horizon frame warp"),
            size: std::mem::size_of::<WarpUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            layout,
            pipeline,
            sampler,
            uniform,
            format,
            kept: None,
            source: None,
            mode: crate::graphics::FrameGen::Off,
            target_fps: 0,
            why: Why::Off,
            shown_target: 0,
            shown_refresh: 0,
            capturing: false,
            world_kept: false,
            generate: false,
            pacer: Pacer::new(),
        }
    }

    /// The device can make frames at all.
    pub fn supported(&self) -> bool {
        self.pipeline.is_some()
    }

    /// The settings; any change (render distance, scale) makes Adaptive learn the usual
    /// frame rate again.
    pub fn set(&mut self, mode: crate::graphics::FrameGen, target_fps: u32) {
        self.mode = mode;
        self.target_fps = target_fps;
        self.pacer.baseline = 0.0;
    }

    pub fn enabled(&self) -> bool {
        self.mode != crate::graphics::FrameGen::Off
    }

    /// The world needs a target of its own (`world_post`) while this is on, so that its
    /// depth outlives the frame, and the frame a texture that can be copied.
    pub fn wants_world_target(&self) -> bool {
        self.enabled() && self.supported()
    }

    /// The frame rate to keep, never above the display's: frames past the refresh rate
    /// are never shown. Adaptive keeps a little under the rate the game usually reaches,
    /// so that the ordinary wobble of frame times makes nothing.
    fn target(&self, refresh: u32) -> u32 {
        use crate::graphics::FrameGen;
        match self.mode {
            FrameGen::Off => refresh,
            FrameGen::Target => match self.target_fps {
                0 => refresh,
                fps => fps.clamp(30, refresh.max(30)),
            },
            FrameGen::Adaptive => ((self.pacer.baseline * 0.95) as u32).clamp(30, refresh.max(30)),
        }
    }

    /// Called once at the start of every frame the game draws, before it builds the frame:
    /// whether this one is to be made. `allowed`: a world is shown with nothing over it that
    /// a made frame would get wrong (a menu, the title screen, a shader pack's own passes).
    pub fn next_frame(&mut self, allowed: bool, refresh_hz: Option<u32>, size: [u32; 2]) -> bool {
        use crate::graphics::FrameGen;
        let now = Instant::now();
        self.pacer.frame_started(now);
        let refresh = refresh_hz.filter(|&hz| hz >= 30).unwrap_or(60);
        let target = self.target(refresh);
        (self.shown_target, self.shown_refresh) = (target, refresh);
        let interval = 1.0 / f64::from(target);
        // Target FPS makes up to two frames after a real one; Adaptive only fills dips.
        let run = if self.mode == FrameGen::Target { 2 } else { 1 };
        let kept = self.source.as_ref().is_some_and(|s| s.size == size)
            && self.kept.as_ref().is_some_and(|k| k.size == size);
        self.why = if !self.enabled() {
            Why::Off
        } else if !self.supported() {
            Why::Unsupported
        } else if !allowed {
            Why::Paused
        } else if self.why == Why::CannotCopy {
            Why::CannotCopy
        } else if self.pacer.real <= interval * SLACK {
            Why::AboveTarget
        } else {
            Why::Making
        };
        self.generate = if self.why == Why::Making && kept {
            self.pacer.decide(interval, run)
        } else {
            self.pacer.skip();
            false
        };
        // Copies are kept while frames may be made: near the target or below it.
        self.capturing = self.enabled()
            && allowed
            && self.supported()
            && !self.generate
            && self.pacer.real * SLACK * SLACK > interval;
        if !self.capturing && !self.generate {
            self.source = None;
        }
        self.world_kept = false;
        self.generate
    }

    /// A line for the performance overlay while the setting is on: the target, real and
    /// made frames shown over the last second, or why none are made.
    pub fn status(&self) -> Option<String> {
        use crate::graphics::FrameGen;
        let (made, real) = self.pacer.shown();
        let mode = match self.mode {
            FrameGen::Off => return None,
            FrameGen::Target => "target",
            FrameGen::Adaptive => "adaptive",
        };
        Some(match self.why {
            Why::Off => return None,
            Why::Unsupported => "FG: not supported on this GPU".to_owned(),
            Why::CannotCopy => "FG: this screen can't keep frames".to_owned(),
            Why::Paused => format!("FG {mode}: paused"),
            Why::AboveTarget | Why::Making => {
                // As the FrameGen mod's HUD: say when even made frames fall short.
                let short = self.why == Why::Making
                    && (made + real) * 10 < self.shown_target * 9
                    && made > 0;
                format!(
                    "FG {mode} {}: {real} real + {made} made, {} Hz{}",
                    self.shown_target,
                    self.shown_refresh,
                    if short { " (out of reach)" } else { "" }
                )
            }
        })
    }

    fn ensure(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        if self.kept.as_ref().is_some_and(|k| k.size == size) {
            return;
        }
        self.source = None;
        let make = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: size[0],
                    height: size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let world = make("Frame warp world");
        let overlay = make("Frame warp overlay");
        self.kept = Some(Kept {
            size,
            world_view: world.create_view(&Default::default()),
            overlay_view: overlay.create_view(&Default::default()),
            world,
            overlay,
        });
    }

    fn copy(encoder: &mut wgpu::CommandEncoder, frame: &wgpu::Texture, into: &wgpu::Texture) {
        encoder.copy_texture_to_texture(
            frame.as_image_copy(),
            into.as_image_copy(),
            wgpu::Extent3d {
                width: into.width().min(frame.width()),
                height: into.height().min(frame.height()),
                depth_or_array_layers: 1,
            },
        );
    }

    /// Whether a real frame of `frame` (the texture the frame is drawn into) can be kept;
    /// a frame that cannot be copied at all says so in `status`.
    fn can_keep(&mut self, frame: &wgpu::Texture) -> bool {
        if !self.capturing {
            return false;
        }
        let copyable = frame.usage().contains(wgpu::TextureUsages::COPY_SRC)
            && frame.format().remove_srgb_suffix() == self.format;
        if !copyable {
            self.why = Why::CannotCopy;
        } else if self.why == Why::CannotCopy {
            self.why = Why::Making;
        }
        copyable
    }

    /// Keeps the world as the screen shows it, after the world post pass and before the
    /// hand.
    pub fn keep_world(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::Texture,
        size: [u32; 2],
    ) {
        if !self.can_keep(frame) {
            return;
        }
        self.ensure(device, size);
        if let Some(kept) = &self.kept {
            Self::copy(encoder, frame, &kept.world);
            self.world_kept = true;
        }
    }

    /// Keeps the frame with the hand and screen effects, before the HUD, and what made it:
    /// its unjittered `view_proj` (relative to `camera`) and the world's `depth`.
    #[allow(clippy::too_many_arguments)]
    pub fn keep_overlay(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::Texture,
        size: [u32; 2],
        view_proj: DMat4,
        camera: DVec3,
        depth: &wgpu::TextureView,
        generation: u64,
    ) {
        if !self.world_kept || !self.can_keep(frame) {
            self.source = None;
            return;
        }
        let Some(kept) = &self.kept else { return };
        Self::copy(encoder, frame, &kept.overlay);
        let group = match self.source.take() {
            Some(source) if source.generation == generation && source.size == size => source.group,
            _ => device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Horizon frame warp"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&kept.world_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&kept.overlay_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.uniform.as_entire_binding(),
                    },
                ],
            }),
        };
        self.source = Some(Source {
            view_proj,
            camera,
            size,
            group,
            generation,
        });
    }

    /// Draws the made frame into `view` for a camera at `camera` with `view_proj`; false
    /// when nothing was kept for it (the caller then skips the frame).
    pub fn draw(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        view_proj: DMat4,
        camera: DVec3,
    ) -> bool {
        let (Some(pipeline), Some(source)) = (&self.pipeline, &self.source) else {
            return false;
        };
        if camera.distance(source.camera) > MAX_TRAVEL {
            return false;
        }
        let to_previous = source.view_proj
            * DMat4::from_translation(camera - source.camera)
            * view_proj.inverse();
        let to_current = view_proj
            * DMat4::from_translation(source.camera - camera)
            * source.view_proj.inverse();
        let still = to_previous.abs_diff_eq(DMat4::IDENTITY, 1e-7);
        let uniform = WarpUniform {
            to_previous: to_previous.as_mat4().to_cols_array_2d(),
            to_current: to_current.as_mat4().to_cols_array_2d(),
            params: [OVERLAY_THRESHOLD, if still { 1.0 } else { 0.0 }, 0.0, 0.0],
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&uniform));
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Horizon frame warp"),
            color_attachments: &attachments,
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &source.group, &[]);
        pass.draw(0..3, 0..1);
        true
    }

    /// This real frame keeps nothing (a menu's effect, a shader pack): no frame may be
    /// made from an older one.
    pub fn discard(&mut self) {
        self.world_kept = false;
        self.source = None;
    }

    /// The frame that was to be made could not be (nothing kept for this size).
    pub fn unmade(&mut self) {
        self.generate = false;
        self.pacer.unmade();
    }
}

/// Why frames are or are not made, for the overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Why {
    Off,
    Unsupported,
    /// A menu, the title screen, a shader pack's passes, HDR or a screenshot.
    Paused,
    /// The frame texture cannot be copied (no `COPY_SRC`, another format).
    CannotCopy,
    /// The real frames reach the target.
    AboveTarget,
    Making,
}

/// What a kind of frame took last time, and how many of each were shown.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Real,
    Made,
}

/// Decides which frames are made from how long frames take. Real frames take `real`
/// seconds on average and made ones `made`; to show one frame every `interval`, `x` made
/// frames are needed per real one, where `(1 + x) / (real + x * made) = 1 / interval`.
struct Pacer {
    started: Option<Instant>,
    /// The frame being drawn, and the one before it.
    kind: Kind,
    previous: Kind,
    /// Made frames in a row up to the last one.
    run: u32,
    real: f64,
    made: f64,
    /// The real frame rate the game usually reaches (Adaptive's): it follows a rise at
    /// once and a drop over most of a minute; settings changes start it again.
    baseline: f64,
    /// Made frames owed: grows by `x` per real frame, and a made frame pays one.
    owed: f64,
    counted: [u32; 2],
    shown: [u32; 2],
    since: Instant,
}

impl Pacer {
    fn new() -> Self {
        Self {
            started: None,
            kind: Kind::Real,
            previous: Kind::Real,
            run: 0,
            real: 1.0 / 60.0,
            made: 0.002,
            baseline: 0.0,
            owed: 0.0,
            counted: [0; 2],
            shown: [0; 2],
            since: Instant::now(),
        }
    }

    /// The last frame ended where this one starts: its time goes to its kind.
    fn frame_started(&mut self, now: Instant) {
        if let Some(started) = self.started {
            // A frame over a quarter second (a pause, a resize) says nothing about pace.
            let took = now.duration_since(started).min(Duration::from_millis(250));
            let seconds = took.as_secs_f64();
            match self.kind {
                Kind::Real => {
                    self.real += (seconds - self.real) * 0.15;
                    self.counted[0] += 1;
                    let fps = 1.0 / self.real.max(1e-4);
                    // Up within a few frames; down with a half-life of about 2300
                    // real frames (most of a minute), so a heavy place stays filled.
                    let rate = if fps > self.baseline { 0.1 } else { 0.0003 };
                    self.baseline += (fps - self.baseline) * rate;
                }
                Kind::Made => {
                    self.made += (seconds - self.made) * 0.25;
                    self.counted[1] += 1;
                }
            }
        }
        self.started = Some(now);
        if now.duration_since(self.since) >= Duration::from_secs(1) {
            self.shown = self.counted;
            self.counted = [0; 2];
            self.since = now;
        }
        self.run = if self.kind == Kind::Made {
            self.run + 1
        } else {
            0
        };
        self.previous = self.kind;
        self.kind = Kind::Real;
    }

    /// Whether the frame starting now is made, with at most `max_run` in a row.
    fn decide(&mut self, interval: f64, max_run: u32) -> bool {
        if self.previous == Kind::Real {
            if self.real <= interval * SLACK {
                self.owed = 0.0;
                return false;
            }
            let made = self.made.clamp(0.0005, interval * 0.8);
            let share = ((self.real - interval) / (interval - made)).clamp(0.0, f64::from(max_run));
            self.owed = (self.owed + share).min(f64::from(max_run) + 0.5);
        } else if self.run >= max_run {
            return false;
        }
        if self.owed >= 1.0 {
            self.owed -= 1.0;
            self.kind = Kind::Made;
            true
        } else {
            false
        }
    }

    /// No frame may be made now (none kept, or not allowed).
    fn skip(&mut self) {
        self.owed = 0.0;
    }

    /// The made frame could not be drawn: the time it took counts as a real frame's.
    fn unmade(&mut self) {
        self.kind = Kind::Real;
    }

    fn shown(&self) -> (u32, u32) {
        (self.shown[1], self.shown[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real frames of `real_ms`, made ones of 2 ms, against `target`: made and real frames
    /// shown per second.
    fn run(real_ms: f64, target: f64, max_run: u32) -> (u32, u32) {
        let mut pacer = Pacer::new();
        let mut t = Instant::now();
        for _ in 0..4000 {
            pacer.frame_started(t);
            let made = pacer.decide(1.0 / target, max_run);
            t += Duration::from_secs_f64(if made { 0.002 } else { real_ms / 1000.0 });
        }
        pacer.shown()
    }

    #[test]
    fn makes_only_the_missing_frames() {
        // 50 real frames and a target of 70: about 22 made, about 70 shown.
        let (made, real) = run(20.0, 70.0, 1);
        assert!((65..=75).contains(&(made + real)), "{made} + {real}");
        assert!((15..=30).contains(&made), "{made}");
    }

    #[test]
    fn makes_nothing_at_the_target() {
        assert_eq!(run(1000.0 / 72.0, 70.0, 2).0, 0);
    }

    #[test]
    fn target_fps_makes_two_after_a_slow_real_frame() {
        // 30 real frames against 90: two made after each, about 80 shown.
        let (made, real) = run(33.3, 90.0, 2);
        assert!(made >= real * 3 / 2, "{made} made, {real} real");
        assert!((75..=95).contains(&(made + real)), "{made} + {real}");
    }

    #[test]
    fn adaptive_learns_the_usual_rate_and_keeps_it() {
        let mut pacer = Pacer::new();
        let mut t = Instant::now();
        // Ten seconds at 80, then frames take 20 ms (50 a second).
        for i in 0..1800 {
            pacer.frame_started(t);
            let slow = i >= 800;
            let target = (pacer.baseline * 0.95).max(30.0);
            let made = slow && pacer.decide(1.0 / target, 1);
            let ms = if made {
                2.0
            } else if slow {
                20.0
            } else {
                12.5
            };
            t += Duration::from_secs_f64(ms / 1000.0);
        }
        assert!(pacer.baseline > 65.0, "{}", pacer.baseline);
        let (made, real) = pacer.shown();
        assert!(made > 10 && made + real > 62, "{made} + {real}");
    }
}
