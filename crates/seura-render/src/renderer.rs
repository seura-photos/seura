//! Runs the shaders with wgpu: Direct3D 12 on Windows, Metal on macOS, and
//! the software adapter on PCs without a usable GPU (technical design,
//! section 6). Photos are rendered in tiles, so any size fits within the
//! smallest texture limit.

use std::sync::mpsc;

use bytemuck::{Pod, Zeroable};
use half::f16;
use seura_image::{LinearImage, Rgba8};

use crate::{SHADER, Settings};

/// The tile size: within every adapter's texture limit.
const TILE: u32 = 2048;

/// The color space a render is encoded for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// For the preview, which the web view maps to the display.
    DisplayP3,
    /// For export (EXP-5).
    Srgb,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("no graphics adapter: {0}")]
    NoAdapter(String),
    #[error("couldn't start the graphics device: {0}")]
    Device(String),
    #[error("couldn't read the rendered pixels: {0}")]
    Readback(String),
}

/// The shader's `Params`, with the same layout.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    exposure_ev: f32,
    output_srgb: u32,
    _pad: [u32; 2],
}

/// A graphics device with the render pipeline built. Making one takes a
/// moment, so it's made once and kept.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params: wgpu::Buffer,
    adapter: String,
}

struct Tile {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new() -> Result<Renderer, RenderError> {
        pollster::block_on(Renderer::create())
    }

    async fn create() -> Result<Renderer, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let options = |fallback| wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: fallback,
            ..Default::default()
        };
        let adapter = match instance.request_adapter(&options(false)).await {
            Ok(adapter) => adapter,
            Err(_) => instance
                .request_adapter(&options(true))
                .await
                .map_err(|error| RenderError::NoAdapter(error.to_string()))?,
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Seura render"),
                required_limits: wgpu::Limits::downlevel_defaults(),
                ..Default::default()
            })
            .await
            .map_err(|error| RenderError::Device(error.to_string()))?;

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Seura shaders"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Seura render"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: fragment,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: fragment,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: fragment,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Seura render"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Seura render"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        // Each output pixel is exactly one photo pixel, so nothing is blended.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Seura photo"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Seura params"),
            size: size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Renderer {
            device,
            queue,
            pipeline,
            layout,
            sampler,
            params,
            adapter: adapter.get_info().name,
        })
    }

    /// The graphics adapter's name, for diagnostics.
    pub fn adapter(&self) -> &str {
        &self.adapter
    }

    /// Renders a photo with its settings, at its own size, as 8-bit RGBA
    /// encoded for `output`.
    pub fn render(
        &self,
        image: &LinearImage,
        settings: &Settings,
        output: Output,
    ) -> Result<Rgba8, RenderError> {
        let params = Params {
            exposure_ev: settings.exposure_ev,
            output_srgb: u32::from(output == Output::Srgb),
            _pad: [0; 2],
        };
        self.queue
            .write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
        let (width, height) = (image.width, image.height);
        let mut pixels = vec![0; width as usize * height as usize * 4];
        for top in (0..height).step_by(TILE as usize) {
            for left in (0..width).step_by(TILE as usize) {
                let tile = Tile {
                    left,
                    top,
                    width: TILE.min(width - left),
                    height: TILE.min(height - top),
                };
                let rendered = self.render_tile(image, &tile)?;
                let row = tile.width as usize * 4;
                for (y, line) in rendered.chunks(row).enumerate() {
                    let start = ((tile.top as usize + y) * width as usize + tile.left as usize) * 4;
                    pixels[start..start + row].copy_from_slice(line);
                }
            }
        }
        Ok(Rgba8::new(width, height, pixels))
    }

    /// One tile's 8-bit RGBA, rows packed.
    fn render_tile(&self, image: &LinearImage, tile: &Tile) -> Result<Vec<u8>, RenderError> {
        let size = wgpu::Extent3d {
            width: tile.width,
            height: tile.height,
            depth_or_array_layers: 1,
        };
        let source = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Seura photo tile"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut data: Vec<f16> = Vec::with_capacity(tile.width as usize * tile.height as usize * 4);
        for y in tile.top..tile.top + tile.height {
            let start = (y as usize * image.width as usize + tile.left as usize) * 4;
            data.extend_from_slice(&image.pixels[start..start + tile.width as usize * 4]);
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &source,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(tile.width * 8),
                rows_per_image: Some(tile.height),
            },
            size,
        );

        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Seura rendered tile"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let source_view = source.create_view(&Default::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Seura render"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let view = target.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Seura render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        // Rows in a copy to a buffer are padded to a fixed alignment.
        let row = tile.width * 4;
        let padded =
            row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Seura readback"),
            size: u64::from(padded) * u64::from(tile.height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(tile.height),
                },
            },
            size,
        );
        self.queue.submit([encoder.finish()]);

        let (sender, receiver) = mpsc::channel();
        readback.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = sender.send(result);
        });
        let failed = |error: &dyn std::fmt::Display| RenderError::Readback(error.to_string());
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| failed(&error))?;
        receiver
            .recv()
            .map_err(|error| failed(&error))?
            .map_err(|error| failed(&error))?;
        let mut pixels = Vec::with_capacity(row as usize * tile.height as usize);
        {
            let mapped = readback
                .get_mapped_range(..)
                .map_err(|error| failed(&error))?;
            for line in mapped.chunks(padded as usize) {
                pixels.extend_from_slice(&line[..row as usize]);
            }
        }
        readback.unmap();
        Ok(pixels)
    }
}
