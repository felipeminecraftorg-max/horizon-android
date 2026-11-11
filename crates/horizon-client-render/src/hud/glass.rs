//! Ordered, region-copied backdrop lenses for the standalone launcher.
//! No readback and no additional targets/passes when a batch has no glass.
use super::{HudBatch, HudRenderer};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct GlassSurface {
    pub vertices: Range<u32>,
    /// Geometry and clipping in the same GUI units as HudVertex::pos.
    pub rect: [f32; 4],
    pub clip: [f32; 4],
    pub radius: f32,
    pub blur: f32,
    pub refraction: f32,
    pub tint: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    frame: [f32; 4],
    rect: [f32; 4],
    clip: [f32; 4],
    tint: [f32; 4],
    optics: [f32; 4],
}
struct Targets {
    size: [u32; 2],
    canvas: wgpu::Texture,
    canvas_view: wgpu::TextureView,
    snapshot: wgpu::Texture,
    lens_group: wgpu::BindGroup,
    blit_group: wgpu::BindGroup,
}
pub(super) struct GlassRenderer {
    layout: wgpu::BindGroupLayout,
    lens: wgpu::RenderPipeline,
    blit: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    stride: usize,
    capacity: usize,
    format: wgpu::TextureFormat,
    targets: Option<Targets>,
}

/// Copy only the visible lens plus its sampling halo. Units are physical pixels.
fn copy_region(rect: [f32; 4], clip: [f32; 4], size: [u32; 2], halo: f32) -> Option<[u32; 4]> {
    let x0 = rect[0].max(clip[0]).max(0.);
    let y0 = rect[1].max(clip[1]).max(0.);
    let x1 = (rect[0] + rect[2])
        .min(clip[0] + clip[2])
        .min(size[0] as f32);
    let y1 = (rect[1] + rect[3])
        .min(clip[1] + clip[3])
        .min(size[1] as f32);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let x = (x0 - halo).floor().max(0.) as u32;
    let y = (y0 - halo).floor().max(0.) as u32;
    let right = (x1 + halo).ceil().min(size[0] as f32) as u32;
    let bottom = (y1 + halo).ceil().min(size[1] as f32) as u32;
    Some([x, y, right - x, bottom - y])
}

impl GlassRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Launcher lens bindings"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(80),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Launcher lens layout"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Launcher backdrop optics"),
            source: wgpu::ShaderSource::Wgsl(include_str!("glass.wgsl").into()),
        });
        let pipeline = |label, vertex, fragment| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vertex),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fragment),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let stride = (device.limits().min_uniform_buffer_offset_alignment as usize)
            .max(80)
            .next_multiple_of(16);
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Launcher lens uniforms"),
            size: stride as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            lens: pipeline("Launcher refraction", "vs_lens", "fs_lens"),
            blit: pipeline("Launcher composite", "vs_blit", "fs_blit"),
            layout,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Launcher backdrop sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            uniforms,
            stride,
            capacity: 1,
            format,
            targets: None,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        uploads: &mut crate::gfx::upload::Uploads,
        encoder: &mut wgpu::CommandEncoder,
        batch: &HudBatch,
        size: [u32; 2],
        zoom: f32,
    ) {
        if size.contains(&0) {
            return;
        }
        if batch.glass.len() > self.capacity {
            self.capacity = batch.glass.len().next_power_of_two();
            self.uniforms = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Launcher lens uniforms"),
                size: (self.stride * self.capacity) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.targets = None;
        }
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            let texture = |label| {
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
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                })
            };
            let canvas = texture("Launcher glass canvas");
            let snapshot = texture("Launcher backdrop snapshot");
            let canvas_view = canvas.create_view(&Default::default());
            let snapshot_view = snapshot.create_view(&Default::default());
            let group = |view: &wgpu::TextureView| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Launcher backdrop"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.uniforms,
                                offset: 0,
                                size: wgpu::BufferSize::new(80),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                })
            };
            self.targets = Some(Targets {
                size,
                lens_group: group(&snapshot_view),
                blit_group: group(&canvas_view),
                canvas,
                canvas_view,
                snapshot,
            });
        }
        let scale = batch.scale * zoom;
        let mut bytes = vec![0; self.stride * batch.glass.len()];
        for (i, lens) in batch.glass.iter().enumerate() {
            let uniform = Uniform {
                frame: [size[0] as f32, size[1] as f32, scale, 0.],
                rect: lens.rect.map(|v| v * scale),
                clip: lens.clip.map(|v| v * scale),
                tint: [0, 8, 16, 24].map(|shift| ((lens.tint >> shift) & 255) as f32 / 255.),
                optics: [
                    lens.radius * scale,
                    lens.blur * scale,
                    lens.refraction * scale,
                    scale,
                ],
            };
            bytes[i * self.stride..i * self.stride + 80]
                .copy_from_slice(bytemuck::bytes_of(&uniform));
        }
        uploads.buffer(encoder, &self.uniforms, 0, &bytes);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        hud: &HudRenderer,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        batch: &HudBatch,
        size: [u32; 2],
        vertices: Range<u32>,
    ) {
        if vertices.is_empty() || size.contains(&0) {
            return;
        }
        let Some(t) = &self.targets else {
            return;
        };
        let attachments = |view, load| {
            [Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })]
        };
        // Launcher frames paint an opaque, full-viewport background first.
        drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Launcher canvas clear"),
            color_attachments: &attachments(
                &t.canvas_view,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ),
            ..Default::default()
        }));
        let scale = batch.scale * hud.zoom;
        let mut at = vertices.start;
        for (i, lens) in batch.glass.iter().enumerate() {
            if lens.vertices.end <= vertices.start || lens.vertices.start >= vertices.end {
                continue;
            }
            hud.draw_plain_range(
                encoder,
                &t.canvas_view,
                depth,
                batch,
                size,
                at..lens.vertices.start,
            );
            let rect = lens.rect.map(|v| v * scale);
            let clip = lens.clip.map(|v| v * scale);
            // Nine-tap blur and chromatic edge samples never read beyond this halo.
            let halo = (lens.refraction + lens.blur * 1.5 + 3.) * scale;
            if let Some([x, y, w, h]) = copy_region(rect, clip, size, halo) {
                let origin = wgpu::Origin3d { x, y, z: 0 };
                let info = |texture| wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin,
                    aspect: wgpu::TextureAspect::All,
                };
                encoder.copy_texture_to_texture(
                    info(&t.canvas),
                    info(&t.snapshot),
                    wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                );
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Launcher lens"),
                    color_attachments: &attachments(&t.canvas_view, wgpu::LoadOp::Load),
                    ..Default::default()
                });
                pass.set_pipeline(&self.lens);
                pass.set_bind_group(0, &t.lens_group, &[(i * self.stride) as u32]);
                pass.draw(0..6, 0..1);
            }
            at = lens.vertices.end;
        }
        hud.draw_plain_range(
            encoder,
            &t.canvas_view,
            depth,
            batch,
            size,
            at..vertices.end,
        );
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Launcher glass composite"),
            color_attachments: &attachments(target, wgpu::LoadOp::Load),
            ..Default::default()
        });
        pass.set_pipeline(&self.blit);
        pass.set_bind_group(0, &t.blit_group, &[0]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipped_lens_copy_includes_sampling_halo_without_leaving_frame() {
        assert_eq!(
            copy_region(
                [-10., 25., 100., 80.],
                [0., 40., 200., 60.],
                [200, 120],
                12.
            ),
            Some([0, 28, 102, 84])
        );
        assert_eq!(
            copy_region([30., 30., 20., 20.], [0., 0., 20., 20.], [200, 120], 12.),
            None
        );
        assert_eq!(
            copy_region(
                [190., 110., 30., 30.],
                [0., 0., 200., 120.],
                [200, 120],
                12.
            ),
            Some([178, 98, 22, 22])
        );
    }
}
