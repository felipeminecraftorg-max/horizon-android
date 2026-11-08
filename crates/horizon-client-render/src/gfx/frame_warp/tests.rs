use super::*;

fn ms(v: f64) -> Duration {
    Duration::from_secs_f64(v / 1000.0)
}

/// Real frames `real_ms` apart for `frames` frames: every frame's plan.
fn run(real_ms: f64, target: f64, refresh: f64, timed: bool, frames: usize) -> Vec<Vec<Output>> {
    let mut pacer = Pacer::new();
    pacer.refresh = refresh;
    let mut t = Instant::now();
    (0..frames)
        .map(|_| {
            t += ms(real_ms);
            pacer.plan(t, target, true, timed)
        })
        .collect()
}

fn fractions(outputs: &[Output]) -> Vec<f32> {
    outputs.iter().map(|o| o.fraction).collect()
}

#[test]
fn nothing_is_made_while_real_frames_reach_the_target() {
    for plan in run(14.0, 60.0, 120.0, true, 60) {
        assert_eq!(fractions(&plan), [1.0]);
        assert_eq!(
            plan[0].at, None,
            "a frame above the target goes out at once"
        );
    }
}

/// Every output's time on the screen, in order, from `plans`.
fn due(plans: &[Vec<Output>]) -> Vec<Instant> {
    plans.iter().flatten().filter_map(|o| o.at).collect()
}

/// The gaps (ms) between consecutive times.
fn gaps(times: &[Instant]) -> Vec<f64> {
    times
        .windows(2)
        .map(|w| (w[1] - w[0]).as_secs_f64() * 1000.0)
        .collect()
}

#[test]
fn thirty_real_frames_make_sixty_against_a_target_of_sixty() {
    let plans = run(1000.0 / 30.0, 60.0, 120.0, true, 120);
    // About one made frame per real one, and the real frame itself.
    let settled = &plans[10..];
    let made: usize = settled
        .iter()
        .map(|p| p.iter().filter(|o| !o.real()).count())
        .sum();
    let shown: usize = settled.iter().map(Vec::len).sum();
    assert!(
        (settled.len() * 9 / 10..=settled.len() * 11 / 10).contains(&made),
        "{made}"
    );
    assert!(
        (settled.len() * 19 / 10..=settled.len() * 21 / 10).contains(&shown),
        "{shown}"
    );
    for gap in gaps(&due(settled)) {
        assert!((gap - 16.667).abs() < 0.01, "{gap}");
    }
    // Within a real frame the moments shown move forward.
    for plan in settled {
        assert!(
            plan.windows(2).all(|w| w[0].fraction < w[1].fraction),
            "{plan:?}"
        );
    }
    // The real frames land on the grid: each is shown as it is (at the end of its own
    // frames, or first among the next real frame's), with one made between.
    let real: usize = settled
        .iter()
        .map(|p| p.iter().filter(|o| o.real()).count())
        .sum();
    assert!(real * 10 >= settled.len() * 9, "{real} real frames shown");
}

#[test]
fn up_to_three_frames_are_made_per_real_one() {
    let plans = run(1000.0 / 30.0, 120.0, 120.0, true, 120);
    let shown: usize = plans[10..].iter().map(Vec::len).sum();
    assert!((shown as f64 / 110.0 - 4.0).abs() < 0.15, "{shown}");
    // 15 real frames could take seven made ones each for 120; three is the most.
    let plans = run(1000.0 / 15.0, 120.0, 120.0, true, 60);
    assert!(
        plans[10..]
            .iter()
            .all(|p| p.len() <= MAX_OUTPUTS as usize && p.len() >= 3)
    );
}

#[test]
fn frames_reach_the_screen_evenly_across_real_frames() {
    // Real frames 28 to 36 ms apart against 60: a real frame is worth 1.7 to 2.2 frames,
    // and yet every frame is due 16.7 ms after the last, and each shows the moment that
    // belongs to its time (a steady motion stays steady).
    let mut pacer = Pacer::new();
    pacer.refresh = 120.0;
    let mut t = Instant::now();
    let start = t;
    let mut shown: Vec<(Instant, f64)> = Vec::new();
    let mut real_times: Vec<Instant> = vec![t];
    for i in 0..400u32 {
        t += ms(28.0 + f64::from((i * 5) % 9));
        let previous = *real_times.last().unwrap();
        real_times.push(t);
        for o in pacer.plan(t, 60.0, true, true) {
            // The moment shown, on the real frames' own timeline.
            let moment = previous + (t - previous).mul_f64(f64::from(o.fraction));
            if let Some(at) = o.at {
                shown.push((at, (moment - start).as_secs_f64()));
            }
        }
    }
    let shown = &shown[40..];
    let times: Vec<Instant> = shown.iter().map(|s| s.0).collect();
    for gap in gaps(&times) {
        assert!((gap - 16.667).abs() < 0.01, "{gap}");
    }
    // The moments shown advance by about the same step each frame (16.7 ms of motion
    // per 16.7 ms on the screen), within a few ms where the latency follows the real
    // frames.
    let steps: Vec<f64> = shown
        .windows(2)
        .map(|w| (w[1].1 - w[0].1) * 1000.0)
        .collect();
    let off = steps.iter().filter(|s| (**s - 16.667).abs() > 4.0).count();
    assert!(
        off * 20 <= steps.len(),
        "{off} of {}: {steps:?}",
        steps.len()
    );
}

#[test]
fn without_present_timing_vsync_repeats_space_the_frames() {
    // 120 Hz, target 60, 30 real frames: each of the two frames shown for two refreshes.
    let plans = run(1000.0 / 30.0, 60.0, 120.0, false, 60);
    for plan in &plans[10..] {
        assert_eq!(fractions(plan), [0.5, 0.5, 1.0, 1.0]);
        assert!(plan.iter().all(|o| o.at.is_none()));
    }
}

#[test]
fn nothing_is_made_without_a_history() {
    let mut pacer = Pacer::new();
    pacer.refresh = 120.0;
    let mut t = Instant::now();
    for _ in 0..20 {
        t += ms(33.0);
        assert_eq!(fractions(&pacer.plan(t, 60.0, false, true)), [1.0]);
    }
}

#[test]
fn adaptive_learns_the_usual_rate_and_fills_a_drop_back_to_it() {
    let mut pacer = Pacer::new();
    pacer.refresh = 120.0;
    let mut t = Instant::now();
    // Ten seconds at 80, then 40 for three seconds.
    for _ in 0..800 {
        t += ms(12.5);
        pacer.plan(t, pacer.baseline.clamp(30.0, 120.0), true, true);
    }
    assert!(
        (75.0..=81.0).contains(&pacer.baseline),
        "{}",
        pacer.baseline
    );
    let mut made = 0;
    for _ in 0..120 {
        t += ms(25.0);
        let target = pacer.baseline.clamp(30.0, 120.0);
        made += pacer
            .plan(t, target, true, true)
            .iter()
            .filter(|o| !o.real())
            .count();
    }
    assert!(pacer.baseline > 70.0, "{}", pacer.baseline);
    // About one made frame per real one: 40 real frames shown as about 80 (the target
    // follows the usual rate down slowly, so the ratio is a little under two).
    assert!((90..=170).contains(&made), "{made}");
}

#[test]
fn a_late_frame_teaches_the_pacer_the_gpus_time() {
    let mut pacer = Pacer::new();
    let vsync = 1.0 / 120.0;
    let before = pacer.lead;
    pacer.learn_lead(0.030, vsync);
    assert!(pacer.lead > before + 0.015, "{}", pacer.lead);
    // On time (within a refresh): it shrinks slowly back.
    let grown = pacer.lead;
    for _ in 0..10 {
        pacer.learn_lead(0.004, vsync);
    }
    assert!(pacer.lead < grown && pacer.lead > grown - 0.005);
}

#[test]
fn camera_halfway_turns_the_short_way_round() {
    let a = Camera {
        position: DVec3::ZERO,
        yaw: 170.0,
        pitch: 0.0,
        fov_degrees: 70.0,
        aspect: 1.5,
        far: 256.0,
        pose: Mat4::IDENTITY,
    };
    let b = Camera {
        yaw: -170.0,
        position: DVec3::new(2.0, 0.0, 0.0),
        ..a
    };
    let half = a.lerp(&b, 0.5);
    assert!(
        (wrap_degrees(half.yaw) - 180.0).abs() < 1e-3
            || (wrap_degrees(half.yaw) + 180.0).abs() < 1e-3
    );
    assert_eq!(half.position, DVec3::new(1.0, 0.0, 0.0));
}

// ---- The interpolation pass on a real GPU (the test machine's adapter) ----

const W: u32 = 192;
const H: u32 = 112;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn gpu() -> Option<Gpu> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).ok()?;
    Some(Gpu { device, queue })
}

/// A test world: a floor and blocks with fine checkers, drawn like the game's world
/// (camera-relative positions, reversed depth).
struct Scene {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    count: u32,
    uniform: wgpu::Buffer,
    group: wgpu::BindGroup,
    hud: wgpu::RenderPipeline,
}

const SCENE: &str = r#"
struct U { view_proj: mat4x4<f32>, camera: vec4<f32> };
@group(0) @binding(0) var<uniform> u: U;
struct Out { @builtin(position) clip: vec4<f32>, @location(0) world: vec3<f32>, @location(1) shade: f32 };
@vertex fn vs(@location(0) pos: vec3<f32>, @location(1) shade: f32) -> Out {
    var out: Out;
    out.clip = u.view_proj * vec4(pos - u.camera.xyz, 1.0);
    out.world = pos;
    out.shade = shade;
    return out;
}
@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    // A quarter cell off: no face of a block lies on a cell edge.
    let c = floor(in.world * 2.0 + 0.25);
    let checker = abs(c.x + c.y + c.z) % 2.0;
    let base = mix(vec3(0.15, 0.45, 0.2), vec3(0.85, 0.7, 0.35), checker);
    return vec4(base * in.shade, 1.0);
}
// The HUD: an opaque bar, and a vignette darkening the left of the screen by half.
struct HudOut { @builtin(position) clip: vec4<f32>, @location(0) colour: vec4<f32> };
@vertex fn vs_hud(@builtin(vertex_index) i: u32) -> HudOut {
    var quads = array<vec4<f32>, 2>(vec4(-0.6, -0.95, 0.6, -0.75), vec4(-1.0, -1.0, -0.4, 1.0));
    let q = quads[i / 6u];
    var corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0));
    let c = corners[i % 6u];
    var out: HudOut;
    out.clip = vec4(mix(q.xy, q.zw, c), 0.0, 1.0);
    out.colour = select(vec4(0.0, 0.0, 0.0, 0.5), vec4(0.95, 0.95, 0.95, 1.0), i < 6u);
    return out;
}
@fragment fn fs_hud(in: HudOut) -> @location(0) vec4<f32> { return in.colour; }
"#;

fn push_box(v: &mut Vec<f32>, min: [f32; 3], max: [f32; 3]) {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;
    let faces: [([[f32; 3]; 4], f32); 6] = [
        (
            [[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]],
            1.0,
        ),
        (
            [[x0, y0, z0], [x0, y0, z1], [x1, y0, z1], [x1, y0, z0]],
            0.5,
        ),
        (
            [[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]],
            0.8,
        ),
        (
            [[x0, y0, z1], [x0, y1, z1], [x1, y1, z1], [x1, y0, z1]],
            0.8,
        ),
        (
            [[x0, y0, z0], [x0, y1, z0], [x0, y1, z1], [x0, y0, z1]],
            0.65,
        ),
        (
            [[x1, y0, z0], [x1, y0, z1], [x1, y1, z1], [x1, y1, z0]],
            0.65,
        ),
    ];
    for (q, shade) in faces {
        for i in [0, 1, 2, 0, 2, 3] {
            v.extend_from_slice(&q[i]);
            v.push(shade);
        }
    }
}

impl Scene {
    fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SCENE.into()),
        });
        let mut v = Vec::new();
        push_box(&mut v, [-40.0, -1.0, -40.0], [40.0, 0.0, 40.0]);
        for (x, z, h) in [
            (3.0, -6.0, 2.0),
            (-4.0, -5.0, 3.0),
            (6.0, 2.0, 1.5),
            (-6.0, 3.0, 2.5),
            (0.0, 7.0, 4.0),
            (1.5, -3.0, 1.0),
            (-2.0, 5.0, 1.0),
            (7.0, -2.0, 3.0),
        ] {
            push_box(&mut v, [x, 0.0, z], [x + 1.0, h, z + 1.0]);
        }
        let vertices = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32],
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(FORMAT.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let empty = device.create_pipeline_layout(&Default::default());
        let hud = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&empty),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_hud"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_hud"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            vertices,
            count: (v.len() / 4) as u32,
            uniform,
            group,
            hud,
        }
    }

    /// Draws the world as `camera` sees it into `color` and `depth`.
    fn draw(&self, gpu: &Gpu, camera: &Camera, color: &wgpu::Texture, depth: &wgpu::Texture) {
        let vp = camera.view_proj().as_mat4();
        let mut data = vp.to_cols_array().to_vec();
        // The world is near the origin: camera-relative in f32 is exact enough.
        data.extend([
            camera.position.x as f32,
            camera.position.y as f32,
            camera.position.z as f32,
            0.0,
        ]);
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::cast_slice(&data));
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        {
            let color_view = color.create_view(&Default::default());
            let depth_view = depth.create_view(&Default::default());
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.4,
                            g: 0.6,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.draw(0..self.count, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
    }

    fn draw_hud(&self, gpu: &Gpu, color: &wgpu::Texture) {
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        {
            let view = color.create_view(&Default::default());
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.hud);
            pass.draw(0..12, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
    }
}

fn texture(gpu: &Gpu, format: wgpu::TextureFormat) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn read(gpu: &Gpu, texture: &wgpu::Texture) -> Vec<[u8; 4]> {
    let row = (W * 4).div_ceil(256) * 256;
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(row * H),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(H),
            },
        },
        texture.size(),
    );
    gpu.queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    let data = buffer.get_mapped_range(..).unwrap();
    (0..H)
        .flat_map(|y| {
            let data = &data;
            (0..W).map(move |x| {
                let i = (y * row + x * 4) as usize;
                [data[i], data[i + 1], data[i + 2], data[i + 3]]
            })
        })
        .collect()
}

/// Mean absolute difference per channel (0..255) over the pixels `keep` lets through.
fn error(a: &[[u8; 4]], b: &[[u8; 4]], keep: impl Fn(u32, u32) -> bool) -> f64 {
    let (mut sum, mut n) = (0.0, 0usize);
    for y in 0..H {
        for x in 0..W {
            if !keep(x, y) {
                continue;
            }
            let i = (y * W + x) as usize;
            for c in 0..3 {
                sum += (f64::from(a[i][c]) - f64::from(b[i][c])).abs();
            }
            n += 3;
        }
    }
    sum / n.max(1) as f64
}

struct Made {
    made: Vec<[u8; 4]>,
    truth: Vec<[u8; 4]>,
    b: Vec<[u8; 4]>,
}

/// With `HORIZON_FRAME_GEN_DUMP` set to a folder: the frames as PPM files, to look at.
fn dump(name: &str, pixels: &[[u8; 4]]) {
    let Ok(dir) = std::env::var("HORIZON_FRAME_GEN_DUMP") else {
        return;
    };
    let mut data = format!("P6 {W} {H} 255\n").into_bytes();
    for p in pixels {
        data.extend_from_slice(&p[..3]);
    }
    let _ = std::fs::write(std::path::Path::new(&dir).join(format!("{name}.ppm")), data);
}

/// Draws A and B through `FrameWarp` as the renderer does, makes the frame at `fraction`,
/// and draws the truth at the camera between them.
fn make(gpu: &Gpu, a: Camera, b: Camera, fraction: f32, hud: bool) -> Made {
    let scene = Scene::new(gpu);
    let mut warp = FrameWarp::new(&gpu.device, FORMAT);
    assert!(warp.supported(), "the interpolation pipeline builds");
    warp.set(crate::graphics::FrameGen::Target, 60);
    let depth = texture(gpu, wgpu::TextureFormat::Depth32Float);
    let mut b_frame = None;
    for camera in [a, b] {
        assert!(warp.begin_frame(true, Some(60)));
        let frame = warp.frame_target(&gpu.device, [W, H]);
        scene.draw(gpu, &camera, &frame, &depth);
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        warp.keep(&gpu.device, &mut encoder, &frame, &depth, camera);
        gpu.queue.submit([encoder.finish()]);
        if hud {
            scene.draw_hud(gpu, &frame);
        }
        if camera == b {
            b_frame = Some(frame);
            break;
        }
        warp.finish(Instant::now());
    }
    let out = texture(gpu, FORMAT);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    let output = Output { fraction, at: None };
    assert!(warp.encode(
        &gpu.device,
        &gpu.queue,
        &mut encoder,
        &output,
        &out.create_view(&Default::default()),
        None,
    ));
    gpu.queue.submit([encoder.finish()]);
    let truth = texture(gpu, FORMAT);
    scene.draw(gpu, &a.lerp(&b, fraction), &truth, &depth);
    if hud {
        scene.draw_hud(gpu, &truth);
    }
    let made = Made {
        made: read(gpu, &out),
        truth: read(gpu, &truth),
        b: read(gpu, &b_frame.unwrap()),
    };
    let name = format!("{fraction}{}", if hud { "_hud" } else { "" });
    dump(&format!("made_{name}"), &made.made);
    dump(&format!("truth_{name}"), &made.truth);
    dump(&format!("b_{name}"), &made.b);
    made
}

fn cameras() -> (Camera, Camera) {
    let a = Camera {
        position: DVec3::new(0.3, 1.7, 0.2),
        yaw: 150.0,
        pitch: 14.0,
        fov_degrees: 70.0,
        aspect: W as f32 / H as f32,
        far: 256.0,
        pose: Mat4::IDENTITY,
    };
    // A turn of 8 degrees and a step to the side and forward: what one real frame at
    // 30 a second sees while the player looks around running.
    let b = Camera {
        position: DVec3::new(0.65, 1.7, -0.1),
        yaw: 158.0,
        pitch: 17.0,
        fov_degrees: 71.0,
        ..a
    };
    (a, b)
}

#[test]
fn the_made_frame_is_the_world_between_the_two_real_ones() {
    let Some(gpu) = gpu() else {
        eprintln!("no GPU adapter: skipped");
        return;
    };
    let (a, b) = cameras();
    let made = make(&gpu, a, b, 0.5, false);
    let all = |_, _| true;
    let made_error = error(&made.made, &made.truth, all);
    let b_error = error(&made.b, &made.truth, all);
    eprintln!("made frame {made_error:.2}, B shown again {b_error:.2} (0..255 per channel)");
    // Showing B again is what a frame generation that makes nothing looks like.
    assert!(b_error > 15.0, "the scene moves enough to tell: {b_error}");
    // What is left is resampling: the kept frames' pixels fall between the made frame's
    // (the made frame at A's own camera is as far from A's real frame).
    assert!(
        made_error < b_error * 0.35,
        "made {made_error:.2} against B {b_error:.2}"
    );
    // Away from the edges (where the fine checkers' edges resample) almost exact.
    let mostly: Vec<f64> = made
        .made
        .iter()
        .zip(&made.truth)
        .map(|(m, t)| {
            (0..3)
                .map(|c| (f64::from(m[c]) - f64::from(t[c])).abs())
                .sum::<f64>()
                / 3.0
        })
        .collect();
    let close = mostly.iter().filter(|e| **e < 24.0).count();
    assert!(
        close * 100 >= mostly.len() * 80,
        "{close} of {} close",
        mostly.len()
    );
}

#[test]
fn every_fraction_follows_the_camera() {
    let Some(gpu) = gpu() else {
        return;
    };
    let (a, b) = cameras();
    for fraction in [0.25, 0.75] {
        let made = make(&gpu, a, b, fraction, false);
        let made_error = error(&made.made, &made.truth, |_, _| true);
        let b_error = error(&made.b, &made.truth, |_, _| true);
        eprintln!("{fraction}: made {made_error:.2}, B {b_error:.2}");
        assert!(
            made_error < b_error * 0.4,
            "{fraction}: {made_error} against {b_error}"
        );
    }
}

#[test]
fn the_hud_stays_sharp_and_the_vignette_darkens_the_moving_world() {
    let Some(gpu) = gpu() else {
        return;
    };
    let (a, b) = cameras();
    let made = make(&gpu, a, b, 0.5, true);
    // The opaque bar: exactly B's (the HUD of the newest frame), never warped.
    let in_bar = |x: u32, y: u32| {
        let (nx, ny) = (
            (x as f32 + 0.5) / W as f32 * 2.0 - 1.0,
            1.0 - (y as f32 + 0.5) / H as f32 * 2.0,
        );
        (-0.58..0.58).contains(&nx) && (-0.93..-0.77).contains(&ny)
    };
    let bar = error(&made.made, &made.b, in_bar);
    assert!(bar < 1.0, "the bar is B's: {bar}");
    // Under the vignette the world moves with the camera, at half brightness.
    let in_vignette = |x: u32, y: u32| {
        let nx = (x as f32 + 0.5) / W as f32 * 2.0 - 1.0;
        let ny = 1.0 - (y as f32 + 0.5) / H as f32 * 2.0;
        nx < -0.45 && !(-0.97..-0.73).contains(&ny)
    };
    let made_error = error(&made.made, &made.truth, in_vignette);
    let b_error = error(&made.b, &made.truth, in_vignette);
    eprintln!("vignette: made {made_error:.2}, B {b_error:.2}");
    assert!(
        made_error < b_error * 0.55,
        "under the vignette: made {made_error} against {b_error}"
    );
}
