//! Frame generation (`frameGen` in `horizon-graphics.txt`, off by default), as FSR 3 and
//! DLSS 3 do it: frame interpolation with paced presentation.
//!
//! Every frame the game draws is real. While frames are generated, the last two real
//! frames (A, then B) are kept, and when B is done, the frames *between* A and B are made
//! from them and shown before B, evenly spaced over the time a real frame takes:
//!
//! - **Target FPS**: the player picks the frame rate shown; below it, the missing frames
//!   are made (up to three per real frame), above it nothing is.
//! - **Adaptive**: the target is the frame rate the game usually reaches here, learned as
//!   it runs (it rises at once and settles over a minute), so a drop in a heavy place is
//!   filled back up to it.
//!
//! A made frame is drawn in one full-screen pass (`frame_warp.wgsl`): each pixel is found
//! in B through B's depth for the camera between A's and B's (position, rotation, field of
//! view and bobbing interpolated), and in A where B never saw it (the strip a turn
//! uncovers, what was behind a block). What moves on its own (mobs, particles) shows where
//! B has it. The hand, the HUD and the screen effects are where the kept frame differs from
//! the world under it; per pixel they are taken as `frame = k * world + c` and redrawn over
//! the made frame's world, so the HUD stays sharp, the vignette darkens what is under it
//! and the world behind a translucent chat box still moves.
//!
//! Pacing is what makes it smooth: all the frames of a real frame are drawn as soon as it
//! is done, and the compositor holds each until its time (`VK_GOOGLE_display_timing`, as
//! FSR 4's driver pacing and Android's frame pacing library do), so made frames never
//! bunch up behind the real one, whatever the GPU or the CPU is busy with. Where present
//! timing is missing (OpenGL), VSync's queue spaces them, one per refresh, each repeated
//! as often as the target needs. The cost is half to most of a real frame's time of extra
//! latency, as with every interpolating frame generation.

use glam::{DMat4, DVec3, Mat4};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// How far (0..1 per channel) the kept frame may differ from the world under it before a
/// pixel counts as the hand's, the HUD's or an effect's: 8-bit rounding stays under it.
const UI_THRESHOLD: f32 = 2.5 / 255.0;
/// Frames shown per real frame at most: three made, then the real one (DLSS 4's 4x).
pub const MAX_OUTPUTS: u32 = 4;
/// Presents of one real frame at most, repeats included (VSync's queue without timing).
const MAX_PRESENTS: u32 = 8;
/// The camera moved further than this (blocks) between two real frames (a teleport,
/// a respawn): nothing is made between them.
const MAX_TRAVEL: f64 = 6.0;
/// Nearer than this (blocks), a made pixel is looked for in A too: a step aside shows
/// the sides of near blocks B saw at a slant.
const NEAR: f32 = 12.0;
/// The camera turned further than this (degrees) between two real frames.
const MAX_TURN: f32 = 60.0;
/// Real frames this far apart (a pause, a loading hitch) are not interpolated.
const MAX_GAP: Duration = Duration::from_millis(200);

/// `frame_warp.wgsl`'s `Interp`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InterpUniform {
    f_to_a: [[f32; 4]; 4],
    a_to_f: [[f32; 4]; 4],
    f_to_b: [[f32; 4]; 4],
    b_to_f: [[f32; 4]; 4],
    a_to_b: [[f32; 4]; 4],
    params: [f32; 4],
    size: [f32; 4],
}

/// Where a real frame's camera was, and how it saw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_degrees: f32,
    pub aspect: f32,
    pub far: f32,
    /// View bobbing and nausea spin (`Frame::bob * Frame::spin`).
    pub pose: Mat4,
}

impl Camera {
    fn view_proj(&self) -> DMat4 {
        super::view::View::screen(
            self.yaw,
            self.pitch,
            self.fov_degrees,
            self.aspect,
            self.far,
        )
        .view_proj(self.pose)
        .as_dmat4()
    }

    /// The depth (reversed, as the depth buffer holds it) of a point `distance` blocks
    /// straight ahead.
    fn depth_at(&self, distance: f32) -> f32 {
        let clip = super::world_projection(self.fov_degrees, self.aspect, self.far)
            * glam::Vec4::new(0.0, 0.0, -distance, 1.0);
        clip.z / clip.w
    }

    /// The camera `t` of the way from `self` to `to` (0: `self`, 1: `to`).
    fn lerp(&self, to: &Camera, t: f32) -> Camera {
        let turn = wrap_degrees(to.yaw - self.yaw);
        let pose = Mat4::from_cols_array(&std::array::from_fn(|i| {
            let (a, b) = (self.pose.to_cols_array()[i], to.pose.to_cols_array()[i]);
            a + (b - a) * t
        }));
        Camera {
            position: self.position.lerp(to.position, f64::from(t)),
            yaw: self.yaw + turn * t,
            pitch: self.pitch + (to.pitch - self.pitch) * t,
            fov_degrees: self.fov_degrees + (to.fov_degrees - self.fov_degrees) * t,
            aspect: to.aspect,
            far: to.far,
            pose,
        }
    }
}

fn wrap_degrees(d: f32) -> f32 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// One real frame, kept: the frame as shown (`frame`, which the game draws into), the
/// world under the hand and the HUD (`world`), its depth and its camera.
struct Slot {
    size: [u32; 2],
    depth_size: [u32; 2],
    frame: wgpu::Texture,
    world: wgpu::Texture,
    depth: wgpu::Texture,
    /// Presents `frame` on the screen (`world_post::Post`).
    show: wgpu::BindGroup,
    camera: Option<Camera>,
    /// When the frame was done.
    done: Option<Instant>,
}

/// What one present of a real frame shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Output {
    /// Where between the last real frame (0) and this one (1) the picture is; 0 and 1
    /// are those real frames themselves.
    pub fraction: f32,
    /// When it should reach the screen; `None`: as soon as it can.
    pub at: Option<Instant>,
}

impl Output {
    pub fn real(&self) -> bool {
        self.fraction >= 1.0 || self.fraction <= 0.0
    }
}

pub struct FrameWarp {
    layout: wgpu::BindGroupLayout,
    pipeline: Option<wgpu::RenderPipeline>,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    format: wgpu::TextureFormat,
    show: super::world_post::Post,
    depth_copy: DepthCopy,
    slots: [Option<Slot>; 2],
    /// The slot the frame being drawn goes to (B); the other one holds A.
    current: usize,
    /// Interpolation bindings, by the slot that holds B; made with the slots.
    pairs: Option<[wgpu::BindGroup; 2]>,
    /// The setting, and Target FPS's frame rate (0: the display's refresh rate).
    pub mode: crate::graphics::FrameGen,
    pub target_fps: u32,
    /// This frame goes through frame generation (`begin_frame`), and it was kept (`keep`).
    active: bool,
    kept: bool,
    /// Why no frame is made now, for the performance overlay.
    why: Why,
    pacer: Pacer,
    /// Present timing: on while the compositor honours it (`present_time`).
    timing: bool,
    next_id: u32,
    /// Presents waiting for the compositor's report: id, desired time, first of its real frame.
    pending: VecDeque<(u32, u64, bool)>,
    /// Frames the screen really showed over the last second (the compositor's report),
    /// and how far their spacing strayed from its mean (ms, mean absolute deviation).
    shown: Option<(u32, f32)>,
    gaps: Vec<f64>,
    gaps_since: Instant,
    last_actual: Option<u64>,
}

impl FrameWarp {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let format = format.remove_srgb_suffix();
        let texture = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        // Depth is read as a plain float texture: OpenGL's GLSL backend cannot load
        // from depth textures, and nothing here filters or compares it.
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Horizon frame generation bindings"),
            entries: &[
                texture(0, true),
                texture(1, true),
                texture(2, true),
                texture(3, true),
                texture(4, false),
                texture(5, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
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
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Horizon frame generation"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../frame_warp.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Horizon frame generation"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Horizon frame generation"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_interp"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_interp"),
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
                log::warn!("Horizon renderer: frame generation unavailable ({e})");
                None
            }
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Horizon frame generation"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Horizon frame generation"),
            size: std::mem::size_of::<InterpUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            layout,
            pipeline,
            sampler,
            uniform,
            format,
            show: super::world_post::Post::new(device, format),
            depth_copy: DepthCopy::new(device),
            slots: [None, None],
            current: 0,
            pairs: None,
            mode: crate::graphics::FrameGen::Off,
            target_fps: 0,
            active: false,
            kept: false,
            why: Why::Off,
            pacer: Pacer::new(),
            timing: horizon_hdr::display_timing::supported(device),
            next_id: 1,
            pending: VecDeque::new(),
            shown: None,
            gaps: Vec::new(),
            gaps_since: Instant::now(),
            last_actual: None,
        }
    }

    /// The device can make frames at all.
    pub fn supported(&self) -> bool {
        self.pipeline.is_some()
    }

    /// The settings; any change makes Adaptive learn the usual frame rate again.
    pub fn set(&mut self, mode: crate::graphics::FrameGen, target_fps: u32) {
        if (mode, target_fps) != (self.mode, self.target_fps) {
            self.pacer.baseline = 0.0;
        }
        self.mode = mode;
        self.target_fps = target_fps;
        if !self.enabled() {
            self.slots = [None, None];
            self.pairs = None;
            self.active = false;
        }
    }

    pub fn enabled(&self) -> bool {
        self.mode != crate::graphics::FrameGen::Off
    }

    /// The world needs a target of its own (`world_post`) while this is on, so that its
    /// depth can be kept.
    pub fn wants_world_target(&self) -> bool {
        self.enabled() && self.supported()
    }

    /// Present timing is in use (frames are spaced by the compositor, not by VSync alone).
    pub fn timed(&self) -> bool {
        self.timing
    }

    /// Called once at the start of every frame the game draws: whether it goes through
    /// frame generation. `allowed`: a world is shown with nothing over it a made frame
    /// would get wrong (a menu, the title screen, a shader pack's own passes).
    pub fn begin_frame(&mut self, allowed: bool, refresh_hz: Option<u32>) -> bool {
        let refresh = refresh_hz.filter(|&hz| hz >= 30).unwrap_or(60);
        self.pacer.refresh = f64::from(refresh);
        self.why = if !self.enabled() {
            Why::Off
        } else if !self.supported() {
            Why::Unsupported
        } else if !allowed {
            Why::Paused
        } else {
            Why::Ready
        };
        self.active = self.why == Why::Ready;
        if !self.active {
            // What was kept is older than what the screen shows next.
            self.invalidate();
        }
        self.kept = false;
        self.active
    }

    /// The frame being drawn goes through frame generation (`begin_frame`).
    pub fn active(&self) -> bool {
        self.active
    }

    /// The history is unusable (a menu came between, a shader pack drew the frame).
    pub fn invalidate(&mut self) {
        for slot in self.slots.iter_mut().flatten() {
            slot.camera = None;
            slot.done = None;
        }
    }

    /// The frame shown on the screen while this real frame is the last: `target`, the
    /// texture the game draws this frame into.
    pub fn frame_target(&mut self, device: &wgpu::Device, size: [u32; 2]) -> wgpu::Texture {
        if self.slots[self.current]
            .as_ref()
            .is_none_or(|s| s.size != size)
        {
            self.slots[self.current] = Some(self.make_slot(device, size, [1, 1]));
            self.pairs = None;
        }
        self.slots[self.current]
            .as_ref()
            .expect("frame generation slot")
            .frame
            .clone()
    }

    fn make_slot(&self, device: &wgpu::Device, size: [u32; 2], depth_size: [u32; 2]) -> Slot {
        let make = |label, size: [u32; 2], format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: size[0].max(1),
                    height: size[1].max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let frame = make(
            "Frame generation frame",
            size,
            self.format,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let world = make(
            "Frame generation world",
            size,
            self.format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        // A copy of the depth, its float's bits in the four bytes of a colour (`DepthCopy`).
        let depth = make(
            "Frame generation depth",
            depth_size,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let show = self
            .show
            .bind(device, &frame.create_view(&Default::default()));
        Slot {
            size,
            depth_size,
            frame,
            world,
            depth,
            show,
            camera: None,
            done: None,
        }
    }

    /// Keeps this real frame's world as the screen shows it (after the world's post pass,
    /// before the hand), its depth and its camera. `frame` is `frame_target`'s texture.
    pub fn keep(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::Texture,
        depth: &wgpu::Texture,
        camera: Camera,
    ) {
        if !self.active {
            return;
        }
        let depth_size = [depth.width(), depth.height()];
        let Some(slot) = &self.slots[self.current] else {
            return;
        };
        if slot.frame != *frame || depth.format() != wgpu::TextureFormat::Depth32Float {
            self.invalidate();
            return;
        }
        if slot.depth_size != depth_size {
            let size = slot.size;
            let mut fresh = self.make_slot(device, size, depth_size);
            // The frame being drawn is in the old slot's texture: keep drawing there.
            let old = self.slots[self.current].take().expect("slot");
            fresh.frame = old.frame;
            fresh.show = old.show;
            self.slots[self.current] = Some(fresh);
            self.pairs = None;
        }
        let slot = self.slots[self.current].as_mut().expect("slot");
        encoder.copy_texture_to_texture(
            frame.as_image_copy(),
            slot.world.as_image_copy(),
            frame.size(),
        );
        self.depth_copy.copy(
            device,
            encoder,
            depth,
            &slot.depth.create_view(&Default::default()),
        );
        slot.camera = Some(camera);
        self.kept = true;
    }

    /// What this real frame shows, made frames first: called once the frame is encoded.
    pub fn plan(&mut self, now: Instant) -> Vec<Output> {
        let b = self.slots[self.current].as_ref();
        let a = self.slots[1 - self.current].as_ref();
        let history = match (a, b) {
            (Some(a), Some(b)) if self.kept => {
                let close = a
                    .done
                    .is_some_and(|done| now.duration_since(done) < MAX_GAP);
                let same = a.size == b.size && a.depth_size == b.depth_size;
                let near = match (a.camera, b.camera) {
                    (Some(ca), Some(cb)) => {
                        ca.position.distance(cb.position) < MAX_TRAVEL
                            && wrap_degrees(ca.yaw - cb.yaw).abs() < MAX_TURN
                            && (ca.pitch - cb.pitch).abs() < MAX_TURN
                    }
                    _ => false,
                };
                close && same && near
            }
            _ => false,
        };
        let target = self.target();
        let outputs = self.pacer.plan(now, target, history, self.timing);
        if self.pacer.generating {
            self.why = Why::Making;
        } else if self.why == Why::Making || self.why == Why::Ready {
            self.why = Why::AboveTarget;
        }
        outputs
    }

    /// The frame rate to show: the player's, or the one the game usually reaches.
    fn target(&self) -> f64 {
        use crate::graphics::FrameGen;
        let refresh = self.pacer.refresh;
        match self.mode {
            FrameGen::Off => refresh,
            FrameGen::Target => match self.target_fps {
                0 => refresh,
                fps => f64::from(fps).clamp(30.0, refresh.max(30.0)),
            },
            FrameGen::Adaptive => self.pacer.baseline.clamp(30.0, refresh.max(30.0)),
        }
    }

    fn ensure_pairs(&mut self, device: &wgpu::Device) -> bool {
        if self.pairs.is_some() {
            return true;
        }
        let (Some(s0), Some(s1)) = (&self.slots[0], &self.slots[1]) else {
            return false;
        };
        let view = |t: &wgpu::Texture| t.create_view(&Default::default());
        let pair = |a: &Slot, b: &Slot| {
            let views = [
                view(&a.frame),
                view(&b.frame),
                view(&a.world),
                view(&b.world),
                view(&a.depth),
                view(&b.depth),
            ];
            let mut entries: Vec<wgpu::BindGroupEntry> = views
                .iter()
                .enumerate()
                .map(|(i, v)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: wgpu::BindingResource::TextureView(v),
                })
                .collect();
            entries.push(wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 7,
                resource: self.uniform.as_entire_binding(),
            });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Horizon frame generation"),
                layout: &self.layout,
                entries: &entries,
            })
        };
        // pairs[i]: B in slot i, A in the other.
        self.pairs = Some([pair(s1, s0), pair(s0, s1)]);
        true
    }

    /// Draws `output` into `view` (the screen's, `rect` of it on Android's scaled
    /// presentation). False when there was nothing to draw it from.
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        output: &Output,
        view: &wgpu::TextureView,
        rect: Option<[u32; 4]>,
    ) -> bool {
        let b = self.current;
        if output.real() {
            let shown = if output.fraction <= 0.0 { 1 - b } else { b };
            let Some(slot) = &self.slots[shown] else {
                return false;
            };
            self.show.finish_in_rect(encoder, view, &slot.show, rect);
            return true;
        }
        if !self.ensure_pairs(device) {
            return false;
        }
        let (Some(pipeline), Some(pairs), Some(sa), Some(sb)) = (
            &self.pipeline,
            &self.pairs,
            &self.slots[1 - b],
            &self.slots[b],
        ) else {
            return false;
        };
        let (Some(ca), Some(cb)) = (sa.camera, sb.camera) else {
            return false;
        };
        let cf = ca.lerp(&cb, output.fraction);
        let (vp_a, vp_b, vp_f) = (ca.view_proj(), cb.view_proj(), cf.view_proj());
        let between = |to: DMat4, to_camera: DVec3, from: DMat4, from_camera: DVec3| {
            (to * DMat4::from_translation(from_camera - to_camera) * from.inverse())
                .as_mat4()
                .to_cols_array_2d()
        };
        let output_size = rect.map_or(sb.size, |r| [r[2], r[3]]);
        let uniform = InterpUniform {
            f_to_a: between(vp_a, ca.position, vp_f, cf.position),
            a_to_f: between(vp_f, cf.position, vp_a, ca.position),
            f_to_b: between(vp_b, cb.position, vp_f, cf.position),
            b_to_f: between(vp_f, cf.position, vp_b, cb.position),
            a_to_b: between(vp_b, cb.position, vp_a, ca.position),
            params: [output.fraction, UI_THRESHOLD, 0.0, cf.depth_at(NEAR)],
            size: [
                output_size[0] as f32,
                output_size[1] as f32,
                sb.size[0] as f32,
                sb.size[1] as f32,
            ],
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
            label: Some("Horizon frame generation"),
            color_attachments: &attachments,
            ..Default::default()
        });
        if let Some([x, y, w, h]) = rect {
            pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
            pass.set_scissor_rect(x, y, w, h);
        }
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &pairs[b], &[]);
        pass.draw(0..3, 0..1);
        true
    }

    /// The real frame's presents are done: it becomes A for the next one.
    pub fn finish(&mut self, now: Instant) {
        if let Some(slot) = &mut self.slots[self.current] {
            if self.kept {
                slot.done = Some(now);
            } else {
                slot.camera = None;
                slot.done = None;
            }
        }
        if self.active {
            self.current = 1 - self.current;
        }
        self.kept = false;
    }

    /// An id for the next present and the time to ask the compositor for
    /// (`CLOCK_MONOTONIC` nanoseconds, 0: as soon as possible), while timing is on.
    pub fn present_time(&mut self, output: &Output, first: bool) -> Option<(u32, u64)> {
        if !self.timing {
            return None;
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let desired = output.at.map_or(0, monotonic_ns_at);
        self.pending.push_back((id, desired, first));
        while self.pending.len() > 64 {
            self.pending.pop_front();
        }
        Some((id, desired))
    }

    /// Present timing failed on this surface: frames are spaced by VSync from now on.
    pub fn timing_failed(&mut self) {
        if self.timing {
            log::warn!(
                "Horizon renderer: present timing unavailable, frame generation paces by VSync"
            );
        }
        self.timing = false;
    }

    /// The compositor's report on past presents (`display_timing::past_presents`): how late
    /// the first frame of each real frame was teaches the pacer how long the GPU takes.
    pub fn reported(&mut self, past: &[horizon_hdr::display_timing::Past]) {
        let vsync = 1.0 / self.pacer.refresh;
        for p in past {
            while let Some(&(id, desired, first)) = self.pending.front() {
                if id_before(id, p.id) {
                    self.pending.pop_front();
                    continue;
                }
                if id == p.id {
                    self.pending.pop_front();
                    if first && desired > 0 && p.actual_ns > 0 {
                        let late = (p.actual_ns as f64 - desired as f64) * 1e-9;
                        self.pacer.learn_lead(late, vsync);
                    }
                }
                break;
            }
            if p.actual_ns > 0 {
                if let Some(last) = self.last_actual
                    && p.actual_ns > last
                {
                    self.gaps.push((p.actual_ns - last) as f64 * 1e-9);
                }
                self.last_actual = Some(p.actual_ns);
            }
        }
        if self.gaps_since.elapsed() >= Duration::from_secs(1) {
            self.shown = spread(&self.gaps)
                .map(|(gap, deviation)| ((1.0 / gap).round() as u32, (deviation * 1000.0) as f32));
            self.gaps.clear();
            self.gaps_since = Instant::now();
        }
    }

    /// A line for the performance overlay while the setting is on.
    pub fn status(&self) -> Option<String> {
        use crate::graphics::FrameGen;
        let mode = match self.mode {
            FrameGen::Off => return None,
            FrameGen::Target => "Target",
            FrameGen::Adaptive => "Adaptive",
        };
        let target = self.target().round() as u32;
        let [real, made] = self.pacer.shown;
        let pacing = if self.timing { "paced" } else { "vsync" };
        let screen = match self.shown {
            Some((count, jitter)) => format!(", screen {count}/s ±{jitter:.1} ms"),
            None => String::new(),
        };
        Some(match self.why {
            Why::Off => return None,
            Why::Unsupported => "FG: not supported on this GPU".to_owned(),
            Why::Paused => format!("FG {mode}: paused"),
            Why::Ready | Why::AboveTarget => {
                format!("FG {mode} {target}: {real} real, none needed{screen}")
            }
            Why::Making => format!(
                "FG {mode} {target}: {real} real + {made} gen = {} ({pacing}){screen}",
                real + made
            ),
        })
    }

    /// Real and made frames shown over the last second.
    pub fn stats(&self) -> (u32, u32) {
        (self.pacer.shown[0], self.pacer.shown[1])
    }

    /// Counts a present of `output` (`plan`'s) that reached the swapchain; a repeat of the
    /// one before (VSync's spacing) is not a frame of its own.
    pub fn presented(&mut self, output: &Output, repeat: bool) {
        if !repeat {
            self.pacer.count(!output.real());
        }
    }
}

/// The mean of `gaps` and their mean absolute deviation from it.
fn spread(gaps: &[f64]) -> Option<(f64, f64)> {
    if gaps.is_empty() {
        return None;
    }
    let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
    let deviation = gaps.iter().map(|g| (g - mean).abs()).sum::<f64>() / gaps.len() as f64;
    Some((mean, deviation))
}

/// Copies a `Depth32Float` texture into an `Rgba8Unorm` one with a draw, each depth's
/// bits in the four bytes: copies of depth textures come out empty on some backends
/// (OpenGL), where float colour targets may not be drawable either.
struct DepthCopy {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    /// The bindings of the last depth texture copied (the world's, remade on a resize).
    source: Option<(wgpu::Texture, wgpu::BindGroup)>,
}

const DEPTH_COPY: &str = r#"
@group(0) @binding(0) var source: texture_2d<f32>;
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
@fragment fn fs(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    let bits = bitcast<u32>(textureLoad(source, vec2<i32>(at.xy), 0).r);
    let bytes = vec4(bits, bits >> 8u, bits >> 16u, bits >> 24u) & vec4(255u);
    return vec4<f32>(bytes) / 255.0;
}
"#;

impl DepthCopy {
    fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Horizon frame generation depth copy"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Horizon frame generation depth copy"),
            source: wgpu::ShaderSource::Wgsl(DEPTH_COPY.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Horizon frame generation depth copy"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Horizon frame generation depth copy"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            layout,
            pipeline,
            source: None,
        }
    }

    fn copy(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        depth: &wgpu::Texture,
        into: &wgpu::TextureView,
    ) {
        if self.source.as_ref().is_none_or(|(t, _)| t != depth) {
            let view = depth.create_view(&wgpu::TextureViewDescriptor {
                aspect: wgpu::TextureAspect::DepthOnly,
                ..Default::default()
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Horizon frame generation depth copy"),
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                }],
            });
            self.source = Some((depth.clone(), group));
        }
        let Some((_, group)) = &self.source else {
            return;
        };
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view: into,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Horizon frame generation depth copy"),
            color_attachments: &attachments,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// `a` came before `b` (ids wrap).
fn id_before(a: u32, b: u32) -> bool {
    b.wrapping_sub(a) as i32 > 0
}

/// `at` on the compositor's clock, `CLOCK_MONOTONIC` (the clock `Instant` reads too).
fn monotonic_ns_at(at: Instant) -> u64 {
    let now = Instant::now();
    let base = monotonic_ns();
    if at > now {
        base + (at - now).as_nanos() as u64
    } else {
        base.saturating_sub((now - at).as_nanos() as u64)
    }
}

#[cfg(unix)]
fn monotonic_ns() -> u64 {
    let t = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
    (t.tv_sec as u64) * 1_000_000_000 + t.tv_nsec as u64
}
#[cfg(not(unix))]
fn monotonic_ns() -> u64 {
    0
}

/// Why frames are or are not made, for the overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Why {
    Off,
    Unsupported,
    /// A menu, the title screen, a shader pack's passes, HDR or a screenshot.
    Paused,
    /// On, and nothing decided yet this frame.
    Ready,
    /// The real frames reach the target.
    AboveTarget,
    Making,
}

/// Decides what each real frame shows, and when. Like FSR 3's pacing thread, it keeps the
/// real frames' average spacing; the frames of one real frame are spread over it.
struct Pacer {
    /// The display's refresh rate.
    refresh: f64,
    last: Option<Instant>,
    /// The last real frames' spacing (seconds), newest last.
    intervals: VecDeque<f64>,
    /// The real frame rate the game usually reaches (Adaptive's target): it follows a rise
    /// within a second and a drop over most of a minute.
    baseline: f64,
    /// When the last planned frame is due on the screen.
    last_due: Option<Instant>,
    /// While frames are made with present timing: the next free time on the output's
    /// grid (one frame every 1/target seconds), and how long after a real frame is done
    /// the frame showing that moment is due (smoothed).
    next_slot: Option<Instant>,
    latency: f64,
    /// From planning a real frame's frames to the first one being on the screen: the time
    /// the GPU still needs for it. Learnt from the compositor's reports (`learn_lead`).
    lead: f64,
    generating: bool,
    counted: [u32; 2],
    shown: [u32; 2],
    since: Instant,
}

impl Pacer {
    fn new() -> Self {
        Self {
            refresh: 60.0,
            last: None,
            intervals: VecDeque::new(),
            baseline: 0.0,
            last_due: None,
            next_slot: None,
            latency: 0.0,
            lead: 0.012,
            generating: false,
            counted: [0; 2],
            shown: [0; 2],
            since: Instant::now(),
        }
    }

    /// The real frames' spacing: the mean of the last eight, the slowest left out so
    /// that one hitch does not make frames for the next few.
    fn interval(&self) -> Option<f64> {
        if self.intervals.len() < 3 {
            return None;
        }
        let max = self.intervals.iter().copied().fold(0.0, f64::max);
        let sum: f64 = self.intervals.iter().sum::<f64>() - max;
        Some(sum / (self.intervals.len() - 1) as f64)
    }

    /// A real frame was done at `now`: what it shows. `target`: frames a second to show;
    /// `history`: the last real frame can be interpolated from; `timed`: presents can ask
    /// for a time.
    fn plan(&mut self, now: Instant, target: f64, history: bool, timed: bool) -> Vec<Output> {
        let previous = self.last;
        if let Some(last) = previous {
            let gap = now.duration_since(last);
            if gap > MAX_GAP {
                self.intervals.clear();
            } else {
                self.intervals.push_back(gap.as_secs_f64());
                if self.intervals.len() > 8 {
                    self.intervals.pop_front();
                }
                let fps = 1.0 / gap.as_secs_f64().max(1e-4);
                let rate = if fps > self.baseline { 0.05 } else { 0.0004 };
                self.baseline += (fps - self.baseline) * rate;
                self.baseline = self.baseline.min(self.refresh);
            }
        }
        self.last = Some(now);
        let spacing = 1.0 / target.max(1.0);
        let interval = self.interval();
        // Hysteresis: start below 92% of the target, stop above 98%.
        let needed =
            interval.is_some_and(|t| t > spacing * if self.generating { 1.02 } else { 1.08 });
        let (Some(previous), Some(interval), true, true) = (previous, interval, needed, history)
        else {
            self.generating = false;
            self.next_slot = None;
            self.last_due = Some(now);
            return vec![Output {
                fraction: 1.0,
                at: None,
            }];
        };
        let started = !self.generating;
        self.generating = true;
        if !timed {
            // VSync's queue shows one frame per refresh: each output is repeated for as
            // many refreshes as the target spacing covers.
            let vsync = 1.0 / self.refresh.max(1.0);
            let repeats = (spacing / vsync).round().max(1.0) as u32;
            let count = ((interval / (vsync * f64::from(repeats))).round() as u32)
                .clamp(1, MAX_OUTPUTS)
                .min(MAX_PRESENTS / repeats)
                .max(1);
            return (0..count)
                .flat_map(|j| {
                    let fraction = (j + 1) as f32 / count as f32;
                    std::iter::repeat_n(Output { fraction, at: None }, repeats as usize)
                })
                .collect();
        }
        // With present timing the frames leave on a fixed grid, one every `spacing`, and
        // each shows the moment `latency` before its time, interpolated between the real
        // frames on either side of it: even on the screen however the real frames fall
        // (a TV's motion interpolation does the same). The latency covers a real frame
        // and the GPU's time, plus up to a step that the pacer moves (`lock`) until real
        // frames land on the grid and are shown themselves.
        let least = interval + self.lead;
        if started || self.latency < least {
            self.latency = least + spacing * 0.5;
        }
        let earliest = now + Duration::from_secs_f64(self.lead);
        let step = Duration::from_secs_f64(spacing);
        let mut slot = match self.next_slot {
            Some(slot) if !started => slot,
            _ => self
                .last_due
                .map_or(earliest, |due| (due + step).max(earliest)),
        };
        // Slots the GPU can no longer make in time are dropped (the frame on the screen
        // stays a step longer), and the latency grows by as much.
        while slot < earliest {
            slot += step;
            self.latency += spacing;
        }
        let gap = now.duration_since(previous).as_secs_f64().max(1e-4);
        let latency = Duration::from_secs_f64(self.latency);
        let mut outputs = Vec::new();
        // The slots whose moment is up to this real frame's.
        while slot <= now + latency {
            if outputs.len() == MAX_OUTPUTS as usize {
                // More than the most made per real frame: the rest are skipped.
                while slot <= now + latency {
                    slot += step;
                }
                break;
            }
            let moment = (slot - latency)
                .saturating_duration_since(previous)
                .as_secs_f64();
            let fraction = (moment / gap).clamp(0.0, 1.0) as f32;
            // Within a few percent of a real frame: the real frame itself.
            let fraction = if fraction > 0.92 {
                1.0
            } else if fraction < 0.08 {
                0.0
            } else {
                fraction
            };
            outputs.push(Output {
                fraction,
                at: Some(slot),
            });
            slot += step;
        }
        self.next_slot = Some(slot);
        if let Some(last) = outputs.last().and_then(|o| o.at) {
            self.last_due = Some(last);
            self.lock(last.duration_since(now).as_secs_f64(), least, spacing);
        }
        outputs
    }

    /// Moves the latency a little towards the one that puts this real frame's moment on
    /// a slot (its last is `due` after it): with real frames a whole number of steps apart
    /// every real frame is then shown as it is, with the made ones between. Never past the
    /// range the latency may take: where the real frames drift along the grid (a
    /// fractional ratio), the moments shown stay steady instead of jumping back a step.
    fn lock(&mut self, due: f64, least: f64, spacing: f64) {
        let steps = ((self.latency - due) / spacing).round();
        let target = due + steps * spacing;
        if (least..=least + spacing).contains(&target) {
            self.latency += (target - self.latency) * 0.3;
        }
    }

    /// The first frame of a real frame reached the screen `late` seconds after it asked
    /// to: under a refresh late is on time (the compositor waits for the next refresh),
    /// more means the GPU was still drawing; the lead follows a rise at once and shrinks
    /// slowly back, so that a frame that happens to be late now and then costs little.
    fn learn_lead(&mut self, late: f64, vsync: f64) {
        if late > vsync * 1.25 {
            self.lead = (self.lead + (late - vsync * 0.5) * 0.7).min(0.15);
        } else {
            self.lead = (self.lead - 0.0002).max(0.002);
        }
    }

    fn count(&mut self, made: bool) {
        self.counted[usize::from(made)] += 1;
        if self.since.elapsed() >= Duration::from_secs(1) {
            self.shown = self.counted;
            self.counted = [0; 2];
            self.since = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests;
