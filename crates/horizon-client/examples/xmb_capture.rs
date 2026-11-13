//! Capture the production Horizon XMB HUD renderer for visual regression.
//! This captures Rust HUD layout + the actual wgpu WGSL ribbons, not HTML,
//! a synthetic mockup, or a screenshot taken on an Android phone.
//! cargo run -p horizon-client --no-default-features --example xmb_capture -- ./xmb-preview
use horizon_client::{
    hud::{HudAtlas, HudBatch, HudRenderer},
    ui_theme::{
        self, Icon, LauncherThemeConfig, ThemePreset, Ui,
        launcher_visual as visual, launcher_xmb as xmb,
    },
};
use std::{fs, path::Path};

fn tex(
    device: &wgpu::Device,
    w: u32, h: u32,
    format: wgpu::TextureFormat, usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Horizon XMB native capture"),
        size: wgpu::Extent3d { width:w, height:h, depth_or_array_layers:1 },
        mip_level_count:1, sample_count:1, dimension:wgpu::TextureDimension::D2,
        format, usage, view_formats:&[],
    })
}

fn read_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &wgpu::Texture,
    size: [u32;2],
) -> horizon_client::Result<Vec<u8>> {
    let [w,h] = size;
    let stride = (w*4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label:Some("XMB capture readback"),
        size: u64::from(stride)*u64::from(h),
        usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,
        mapped_at_creation:false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer:&buffer,
            layout:wgpu::TexelCopyBufferLayout {
                offset:0, bytes_per_row:Some(stride), rows_per_image:Some(h),
            },
        },
        wgpu::Extent3d {width:w,height:h,depth_or_array_layers:1},
    );
    queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    device.poll(wgpu::PollType::wait_indefinitely())?;
    let bytes=buffer.slice(..).get_mapped_range()?;
    let pixels=bytes.chunks(stride as usize)
        .flat_map(|row|row[..(w*4) as usize].iter().copied()).collect();
    drop(bytes);
    buffer.unmap();
    Ok(pixels)
}

fn scene(
    atlas:&HudAtlas, size:[u32;2],
    home_focus:usize, phase:f32, quality:f32,
) -> HudBatch {
    let items=[
        ("Play",Icon::Play), ("Mods",Icon::Mods), ("Packs",Icon::Packs),
        ("Shaders",Icon::Shaders), ("Installed",Icon::Installed),
        ("Settings",Icon::Settings), ("Skins",Icon::Person),
    ];
    let mut batch=HudBatch::new(size);
    let mut ui=Ui::mobile(atlas,&mut batch,(0.,0.));
    let (w,h)=(ui.w,ui.h);
    xmb::background(&mut ui,phase,quality);
    let layout=visual::shell(&mut ui,&items,0,"Horizon Player",true,"Android / Preview",|_,_|{});
    let top=130.;
    visual::home_with_focus(&mut ui,
        [layout.x, top, layout.width,
            (layout.bottom-top-if layout.portrait {60.}else{78.}).max(0.)],
        &visual::HomeView {
            installed:true, launching:false, profile:0, renderer:"Vulkan",
        },home_focus,|_,_|{});
    // Replicate the actual native Home status footer as text only, not a card.
    ui.text(ui_theme::Face::Regular,10.,layout.x+5.,h-33.,
        ui_theme::color::hex(0xC3D4E9,160),"STATUS DO SISTEMA");
    let _=w;
    ui.finish();
    batch
}

fn main()->horizon_client::Result<()> {
    let output=std::env::args().nth(1).unwrap_or_else(||"xmb-preview".into());
    fs::create_dir_all(&output)?;
    let instance=wgpu::Instance::new(
        wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter=pollster::block_on(instance.request_adapter(&Default::default()))?;
    eprintln!("XMB preview GPU: {}",adapter.get_info().name);
    let (device,queue)=pollster::block_on(
        adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
    let resources=Path::new(&output).join("empty-resources");
    fs::create_dir_all(&resources)?;
    let atlas=HudAtlas::build(&resources)?;
    let atlas_tex=tex(&device,atlas.width,atlas.height,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_DST);
    queue.write_texture(
        atlas_tex.as_image_copy(),&atlas.pixels,
        wgpu::TexelCopyBufferLayout {
            offset:0,bytes_per_row:Some(atlas.width*4),rows_per_image:Some(atlas.height),
        },
        wgpu::Extent3d {width:atlas.width,height:atlas.height,depth_or_array_layers:1},
    );
    let glint=tex(&device,1,1,wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_DST);
    let sampler=device.create_sampler(&Default::default());
    let mut renderer=HudRenderer::new(
        &device,wgpu::TextureFormat::Rgba8Unorm,
        &atlas_tex.create_view(&Default::default()),
        &glint.create_view(&Default::default()),&sampler,
    )?;
    renderer.atlas_texture=Some(atlas_tex);
    let mut cfg=LauncherThemeConfig::default();
    cfg.apply_preset(ThemePreset::OldSchool);
    ui_theme::set_active_theme(cfg.resolve());
    for (name,size,focus,phase,quality) in [
        ("XMB_Home_Landscape_1600x900", [1600,900],0,20.,1.),
        ("XMB_Home_Compact_900x480", [900,480],0,20.,1.),
        ("XMB_Home_Portrait_900x1600", [900,1600],0,20.,1.),
        ("XMB_Context_Shader_1600x900", [1600,900],4,20.,1.),
        ("XMB_Wave_Second_Phase_1600x900", [1600,900],0,100.,1.),
        ("XMB_Energy_Save_900x480", [900,480],0,0.,0.38),
    ] {
        ui_theme::reset_atlas();
        let batch=scene(&atlas,size,focus,phase,quality);
        let target=tex(&device,size[0],size[1],wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC);
        let depth=tex(&device,size[0],size[1],wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT);
        renderer.render_ui_frame(&device,&queue,
            &target.create_view(&Default::default()),
            &depth.create_view(&Default::default()),&batch,size);
        let rgba=read_pixels(&device,&queue,&target,size)?;
        let png_path=Path::new(&output).join(format!("{name}.png"));
        let mut png=png::Encoder::new(fs::File::create(&png_path)?,size[0],size[1]);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        png.write_header()?.write_image_data(&rgba)?;
        eprintln!("Native XMB: {} · {} vertices",png_path.display(),batch.vertices.len());
    }
    Ok(())
}
