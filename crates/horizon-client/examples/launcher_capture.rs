//! Capture the Android launcher's actual shared native components through its
//! production HUD renderer. No HTML, image generation, game files or real accounts.
//! cargo run -p horizon-client --no-default-features --example launcher_capture -- OUTPUT
use horizon_client::{
    hud::{HudAtlas, HudBatch, HudRenderer},
    ui_theme::{self, Icon, LauncherThemeConfig, ThemePreset, Ui, launcher_visual as visual},
};
use std::{fs, path::Path};

fn texture(
    device: &wgpu::Device,
    w: u32,
    h: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Launcher capture"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn read_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &wgpu::Texture,
    [w, h]: [u32; 2],
) -> horizon_client::Result<Vec<u8>> {
    let row = (w * 4).next_multiple_of(256);
    let read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Native capture readback"),
        size: u64::from(row) * u64::from(h),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &read,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    read.slice(..)
        .map_async(wgpu::MapMode::Read, |r| r.unwrap());
    device.poll(wgpu::PollType::wait_indefinitely())?;
    let bytes = read.slice(..).get_mapped_range()?;
    let pixels = bytes
        .chunks(row as usize)
        .flat_map(|r| r[..(w * 4) as usize].iter().copied())
        .collect();
    drop(bytes);
    read.unmap();
    Ok(pixels)
}

/// Numerical checks on the production compositor, with scene changes behind a fixed lens.
fn check_optics(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    hud: &mut HudRenderer,
    atlas: &HudAtlas,
) -> horizon_client::Result<()> {
    let size = [640, 360];
    let render = |hud: &mut HudRenderer,
                  blur,
                  refraction,
                  phase,
                  clipped,
                  cursor|
     -> horizon_client::Result<Vec<u8>> {
        let mut config = LauncherThemeConfig::default();
        config.apply_preset(ThemePreset::LiquidGlass);
        config.glass = visual::GlassTuning {
            opacity: 0.14,
            blur,
            refraction,
        };
        ui_theme::set_active_theme(config.resolve());
        let mut batch = HudBatch::new(size);
        let mut ui = Ui::new(atlas, &mut batch, cursor);
        for x in 0..320 {
            let rgb = if (x + phase) % 2 == 0 {
                0x385A90
            } else {
                0xC0D0E0
            };
            ui.fill((x * 2) as f32, 0., 2., 360., ui_theme::color::hex(rgb, 255));
        }
        if clipped {
            ui.set_clip(Some([100., 110., 420., 140.]));
        }
        visual::panel(&mut ui, [100., 80., 420., 200.], 36., false);
        ui.set_clip(None);
        ui.fill(220., 160., 20., 20., u32::MAX);
        ui.finish();
        assert_eq!(batch.glass.len(), 1);
        let target = texture(
            device,
            640,
            360,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = texture(
            device,
            640,
            360,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        hud.render_ui_frame(
            device,
            queue,
            &target.create_view(&Default::default()),
            &depth.create_view(&Default::default()),
            &batch,
            size,
        );
        read_pixels(device, queue, &target, size)
    };
    let clear = render(hud, 0., 0., 0, false, (0., 0.))?;
    let refracted = render(hud, 0., 14., 0, false, (0., 0.))?;
    let blurred = render(hud, 4., 0., 0, false, (0., 0.))?;
    let moved = render(hud, 0., 14., 1, false, (0., 0.))?;
    let clipped = render(hud, 0., 14., 0, true, (0., 0.))?;
    let relit = render(hud, 0., 14., 0, false, (640., 360.))?;
    assert_ne!(refracted, relit, "reflected light did not respond to input");
    let pixel = |image: &[u8], x: usize, y: usize| -> [u8; 3] {
        image[(y * 640 + x) * 4..(y * 640 + x) * 4 + 3]
            .try_into()
            .unwrap()
    };
    let mut edge_changed = 0;
    let mut clear_contrast = 0u64;
    let mut blur_contrast = 0u64;
    for y in 130..150 {
        for x in 270..340 {
            let a = pixel(&clear, x, y)[0] as i32;
            let b = pixel(&blurred, x, y)[0] as i32;
            clear_contrast += (a - pixel(&clear, x + 1, y)[0] as i32).unsigned_abs() as u64;
            blur_contrast += (b - pixel(&blurred, x + 1, y)[0] as i32).unsigned_abs() as u64;
        }
        for x in 103..125 {
            if pixel(&clear, x, y) != pixel(&refracted, x, y) {
                edge_changed += 1;
            }
        }
    }
    assert!(
        edge_changed > 200,
        "refraction did not bend the background: {edge_changed}"
    );
    assert!(
        blur_contrast * 10 < clear_contrast * 8,
        "backdrop blur did not reduce contrast: {blur_contrast}/{clear_contrast}"
    );
    assert_ne!(
        pixel(&refracted, 110, 140),
        pixel(&moved, 110, 140),
        "lens froze the prior frame's background"
    );
    assert_eq!(
        pixel(&refracted, 225, 165),
        [255; 3],
        "foreground UI was blurred"
    );
    // The centre stays substantially transparent, rather than becoming the old opaque grey plate.
    let base = [192i32, 208, 224];
    let center = pixel(&clear, 302, 140);
    assert!(
        center
            .into_iter()
            .zip(base)
            .all(|(a, b)| (a as i32 - b).abs() < 38),
        "glass is too opaque: {center:?}"
    );
    assert_eq!(
        pixel(&clipped, 300, 95),
        pixel(&clear, 20, 95),
        "lens escaped the scroll clip"
    );
    eprintln!(
        "Native optics checks passed: {edge_changed} bent edge pixels; blur contrast {blur_contrast}/{clear_contrast}; live background, sharp foreground, transparency and clipping verified."
    );
    Ok(())
}

fn main() -> horizon_client::Result<()> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "launcher-captures".into());
    fs::create_dir_all(&output)?;
    let mut instance_descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    if std::env::var("HORIZON_CAPTURE_BACKEND").as_deref() == Ok("gl") {
        instance_descriptor.backends = wgpu::Backends::GL;
    }
    let instance = wgpu::Instance::new(instance_descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))?;
    eprintln!(
        "Native launcher capture adapter: {} ({:?})",
        adapter.get_info().name,
        adapter.get_info().backend
    );
    let mut descriptor = wgpu::DeviceDescriptor::default();
    if let Ok(alignment) = std::env::var("HORIZON_CAPTURE_UNIFORM_ALIGNMENT") {
        let alignment: u32 = alignment.parse()?;
        assert!(alignment.is_power_of_two());
        assert!(alignment >= adapter.limits().min_uniform_buffer_offset_alignment);
        descriptor
            .required_limits
            .min_uniform_buffer_offset_alignment = alignment;
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&descriptor))?;
    eprintln!(
        "Capture uniform alignment: {}",
        device.limits().min_uniform_buffer_offset_alignment
    );
    let empty = Path::new(&output).join("empty-resources");
    fs::create_dir_all(&empty)?;
    let atlas = HudAtlas::build(&empty)?;
    let atlas_tex = texture(
        &device,
        atlas.width,
        atlas.height,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    queue.write_texture(
        atlas_tex.as_image_copy(),
        &atlas.pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(atlas.width * 4),
            rows_per_image: Some(atlas.height),
        },
        wgpu::Extent3d {
            width: atlas.width,
            height: atlas.height,
            depth_or_array_layers: 1,
        },
    );
    let glint = texture(
        &device,
        1,
        1,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let sampler = device.create_sampler(&Default::default());
    let mut hud = HudRenderer::new(
        &device,
        wgpu::TextureFormat::Rgba8Unorm,
        &atlas_tex.create_view(&Default::default()),
        &glint.create_view(&Default::default()),
        &sampler,
    )?;
    hud.atlas_texture = Some(atlas_tex);
    // The capture workflow renders actual UI; optical invariants remain
    // available separately when explicitly exercising this example.
    if std::env::var_os("HORIZON_CAPTURE_SKIP_OPTICS").is_none() {
        check_optics(&device, &queue, &mut hud, &atlas)?;
    }
    let items = [
        ("Play", Icon::Play),
        ("Mods", Icon::Mods),
        ("Packs", Icon::Packs),
        ("Shaders", Icon::Shaders),
        ("Installed", Icon::Installed),
        ("Settings", Icon::Settings),
        ("Skins", Icon::Person),
    ];
    for (stem, preset) in [
        ("old-school", ThemePreset::OldSchool),
        ("liquid-glass", ThemePreset::LiquidGlass),
    ] {
        let mut config = LauncherThemeConfig::default();
        config.apply_preset(preset);
        ui_theme::set_active_theme(config.resolve());
        ui_theme::reset_atlas();
        let mut last_page = None;
        for (page, [w, h]) in [
            ("home", [1600, 900]),
            ("home", [1600, 900]),
            ("home-compact", [1280, 720]),
            ("home-portrait", [900, 1600]),
            ("appearance", [1080, 1920]),
            ("appearance", [1080, 1920]),
        ] {
            let mut batch = HudBatch::new([w, h]);
            let mut ui = Ui::mobile(&atlas, &mut batch, (0., 0.));
            let (uw, uh) = (ui.w, ui.h);
            if preset == ThemePreset::OldSchool
                && config.wallpaper == visual::Wallpaper::Graphite
            {
                visual::console_background(&mut ui, [0., 0., uw, uh]);
            } else {
                ui.draw_theme_background(0., 0., uw, uh, &config.resolve());
            }
            let layout = visual::shell(
                &mut ui,
                &items,
                if page.starts_with("home") { 0 } else { 5 },
                "felipeORG",
                false,
                "Android 0.7.8",
                |_, _| {},
            );
            if page.starts_with("home") {
                visual::home(
                    &mut ui,
                    [layout.x, if layout.portrait { 74. } else { 132. },
                        layout.width,
                        (layout.bottom - if layout.portrait { 152. } else { 186. }).max(180.)],
                    &visual::HomeView {
                        installed: true,
                        launching: false,
                        profile: 0,
                        renderer: "Vulkan / compatibility",
                    },
                    |_, _| {},
                );
            } else {
                visual::appearance_screen(
                    &mut ui,
                    layout.x,
                    layout.width,
                    layout.bottom,
                    0.,
                    &config,
                    false,
                    if preset == ThemePreset::LiquidGlass {
                        210.
                    } else {
                        0.
                    },
                    |_, _| {},
                );
            }
            if page.starts_with("home") {
                visual::button(
                    &mut ui,
                    [uw - 118., layout.bottom - 38., 92., 28.],
                    "GitHub",
                    None,
                    false,
                );
            }
            ui.finish();
            if last_page == Some(page) {
                assert!(
                    batch.patches.is_empty(),
                    "steady {stem}/{page} reuploaded cached UI textures"
                );
            }
            last_page = Some(page);
            let target = texture(
                &device,
                w,
                h,
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            let depth = texture(
                &device,
                w,
                h,
                wgpu::TextureFormat::Depth32Float,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            hud.render_ui_frame(
                &device,
                &queue,
                &target.create_view(&Default::default()),
                &depth.create_view(&Default::default()),
                &batch,
                [w, h],
            );
            let pixels = read_pixels(&device, &queue, &target, [w, h])?;
            let path = Path::new(&output).join(format!("{stem}-{page}.png"));
            let mut png = png::Encoder::new(fs::File::create(&path)?, w, h);
            png.set_color(png::ColorType::Rgba);
            png.set_depth(png::BitDepth::Eight);
            png.write_header()?.write_image_data(&pixels)?;
            println!("{} · {} vertices", path.display(), batch.vertices.len());
        }
    }
    Ok(())
}
