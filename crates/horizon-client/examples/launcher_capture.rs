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

fn main() -> horizon_client::Result<()> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "launcher-captures".into());
    fs::create_dir_all(&output)?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))?;
    eprintln!(
        "Native launcher capture adapter: {}",
        adapter.get_info().name
    );
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
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
        ("liquid-glass", ThemePreset::LiquidGlass),
        ("dark-aero", ThemePreset::DarkAero),
    ] {
        let mut config = LauncherThemeConfig::default();
        config.apply_preset(preset);
        ui_theme::set_active_theme(config.resolve());
        ui_theme::reset_atlas();
        let mut last_page = None;
        for (page, [w, h]) in [
            ("home", [1600, 900]),
            ("home", [1600, 900]),
            ("appearance", [1080, 1920]),
            ("appearance", [1080, 1920]),
        ] {
            let mut batch = HudBatch::new([w, h]);
            let mut ui = Ui::mobile(&atlas, &mut batch, (0., 0.));
            let (uw, uh) = (ui.w, ui.h);
            ui.draw_theme_background(0., 0., uw, uh, &config.resolve());
            let layout = visual::shell(
                &mut ui,
                &items,
                if page == "home" { 0 } else { 5 },
                "felipeORG",
                false,
                "Android 0.7.8",
                |_, _| {},
            );
            if page == "home" {
                visual::home(
                    &mut ui,
                    [layout.x, 76., layout.width, layout.bottom - 116.],
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
                    0.,
                    |_, _| {},
                );
            }
            if page == "home" {
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
            let row = (w * 4).next_multiple_of(256);
            let read = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
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
            let pixels: Vec<_> = bytes
                .chunks(row as usize)
                .flat_map(|r| r[..(w * 4) as usize].iter().copied())
                .collect();
            let path = Path::new(&output).join(format!("{stem}-{page}.png"));
            let mut png = png::Encoder::new(fs::File::create(&path)?, w, h);
            png.set_color(png::ColorType::Rgba);
            png.set_depth(png::BitDepth::Eight);
            png.write_header()?.write_image_data(&pixels)?;
            println!("{} · {} vertices", path.display(), batch.vertices.len());
            drop(bytes);
            read.unmap();
        }
    }
    Ok(())
}
