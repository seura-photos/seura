//! Runs the shaders with wgpu: Direct3D 12 on Windows, Metal on macOS, and
//! the software adapter on PCs without a usable GPU (technical design,
//! section 6). Photos are rendered in tiles, so any size fits within the
//! smallest texture limit. Highlights and shadows sample a luminance guide
//! built once for the whole photo beforehand, so tiles join without seams
//! (technical design, "Neighborhood operations").

use std::sync::mpsc;

use half::f16;
use seura_image::{LinearImage, Rgba8};

use crate::geometry::{Geometry, Rect};
use crate::settings::{Draw, Pass};
use crate::{SHADER, Settings};

/// The tile size. The part of the photo a tile samples must fit every
/// adapter's texture limit, 2048: straightened by up to 45°, a tile's part
/// is up to √2 times as wide, with room for the filter on each side.
const TILE: u32 = 1440;

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

/// A graphics device with the render pipelines built. Making one takes a
/// moment, so it's made once and kept.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// The photo, drawn in 8 bits.
    pipeline: wgpu::RenderPipeline,
    /// The guide's passes: 32-bit floats between them, and 16 for the guide,
    /// which is sampled bilinearly.
    build: wgpu::RenderPipeline,
    finish: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    guide_sampler: wgpu::Sampler,
    params: wgpu::Buffer,
    /// Bound where a draw reads nothing: the guide while it's being built,
    /// or without highlights and shadows, and the stage while drawing.
    empty: wgpu::TextureView,
    empty_stage: wgpu::TextureView,
    adapter: String,
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
        let texture = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: fragment,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: fragment,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
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
                texture(1, true),
                sampler(2),
                texture(3, true),
                sampler(4),
                // 32-bit floats can't be filtered everywhere; the stage is
                // only ever read pixel by pixel.
                texture(5, false),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Seura render"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline_for = |format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipeline = pipeline_for(wgpu::TextureFormat::Rgba8Unorm);
        let build = pipeline_for(wgpu::TextureFormat::Rgba32Float);
        let finish = pipeline_for(wgpu::TextureFormat::Rgba16Float);
        // Each output pixel is exactly one photo pixel, so nothing is blended.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Seura photo"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let guide_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Seura guide"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let empty_texture = |format| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Seura empty"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let empty = empty_texture(wgpu::TextureFormat::Rgba16Float);
        let empty_stage = empty_texture(wgpu::TextureFormat::Rgba32Float);
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Seura params"),
            size: size_of::<crate::settings::Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Renderer {
            device,
            queue,
            pipeline,
            build,
            finish,
            layout,
            sampler,
            guide_sampler,
            params,
            empty,
            empty_stage,
            adapter: adapter.get_info().name,
        })
    }

    /// The graphics adapter's name, for diagnostics.
    pub fn adapter(&self) -> &str {
        &self.adapter
    }

    /// Renders a photo with its settings, at its own size turned as they
    /// say, as 8-bit RGBA encoded for `output`. `guide` is the image its
    /// luminance guide is built from ([`seura_image::Linear`]), used only
    /// with highlights or shadows; it's upright, like `image`.
    pub fn render(
        &self,
        image: &LinearImage,
        guide: &LinearImage,
        settings: &Settings,
        output: Output,
    ) -> Result<Rgba8, RenderError> {
        let guide = (settings.highlights != 0.0 || settings.shadows != 0.0)
            .then(|| self.build_guide(guide, settings));
        let guide_view = guide
            .as_ref()
            .map(|guide| guide.create_view(&Default::default()));
        let geometry = Geometry::new(settings, image.width, image.height);
        let (width, height) = geometry.output_size();
        let mut pixels = vec![0; width as usize * height as usize * 4];
        for top in (0..height).step_by(TILE as usize) {
            for left in (0..width).step_by(TILE as usize) {
                let tile = Rect {
                    left,
                    top,
                    width: TILE.min(width - left),
                    height: TILE.min(height - top),
                };
                // Each tile uploads only the part of the photo it samples;
                // the shader is told where that part is, and which part of
                // the result the tile is.
                let source = geometry.source(tile);
                let fraction = |rect: Rect, (across, down): (u32, u32)| {
                    [
                        rect.left as f32 / across as f32,
                        rect.top as f32 / down as f32,
                        rect.width as f32 / across as f32,
                        rect.height as f32 / down as f32,
                    ]
                };
                let params = settings.uniforms(
                    output,
                    Pass::Draw,
                    &Draw {
                        full: (image.width, image.height),
                        part: fraction(source, (image.width, image.height)),
                        view: fraction(tile, (width, height)),
                    },
                );
                self.queue
                    .write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
                let rendered = self.render_tile(
                    image,
                    &source,
                    &tile,
                    guide_view.as_ref().unwrap_or(&self.empty),
                )?;
                let row = tile.width as usize * 4;
                for (y, line) in rendered.chunks(row).enumerate() {
                    let start = ((tile.top as usize + y) * width as usize + tile.left as usize) * 4;
                    pixels[start..start + row].copy_from_slice(line);
                }
            }
        }
        Ok(Rgba8::new(width, height, pixels))
    }

    /// Builds the luminance guide from its image, in the guide's four passes
    /// (`shaders/guide.wgsl`), each submitted on its own since each has its
    /// own parameters.
    fn build_guide(&self, image: &LinearImage, settings: &Settings) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };
        let texture = |label, format, usage| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let drawn = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let source = texture(
            "Seura guide image",
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &source,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&image.pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width * 8),
                rows_per_image: Some(image.height),
            },
            size,
        );
        let one = texture("Seura guide pass", wgpu::TextureFormat::Rgba32Float, drawn);
        let two = texture("Seura guide pass", wgpu::TextureFormat::Rgba32Float, drawn);
        let guide = texture("Seura guide", wgpu::TextureFormat::Rgba16Float, drawn);
        let view = |texture: &wgpu::Texture| texture.create_view(&Default::default());
        let (source, one, two, finished) = (view(&source), view(&one), view(&two), view(&guide));
        for (pass, from, to, pipeline) in [
            (Pass::Luminance, &source, &one, &self.build),
            (Pass::Across, &one, &two, &self.build),
            (Pass::Coefficients, &two, &one, &self.build),
            (Pass::Across, &one, &two, &self.build),
            (Pass::Down, &two, &finished, &self.finish),
        ] {
            let params = settings.uniforms(
                Output::DisplayP3,
                pass,
                &Draw::whole((image.width, image.height)),
            );
            self.queue
                .write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
            let bind_group = self.bind_group(&self.empty, &self.empty, from);
            let mut encoder = self.device.create_command_encoder(&Default::default());
            self.draw(&mut encoder, pipeline, &bind_group, to);
            self.queue.submit([encoder.finish()]);
        }
        guide
    }

    fn bind_group(
        &self,
        photo: &wgpu::TextureView,
        guide: &wgpu::TextureView,
        stage: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let view = wgpu::BindingResource::TextureView;
        let sampler = wgpu::BindingResource::Sampler;
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Seura render"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: view(photo),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: view(guide),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: sampler(&self.guide_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: view(stage),
                },
            ],
        })
    }

    /// Draws the one triangle covering `target`.
    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        bind_group: &wgpu::BindGroup,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Seura render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    /// One tile's 8-bit RGBA, rows packed, drawn from `part` of
    /// the upright photo.
    fn render_tile(
        &self,
        image: &LinearImage,
        part: &Rect,
        tile: &Rect,
        guide: &wgpu::TextureView,
    ) -> Result<Vec<u8>, RenderError> {
        let source_size = wgpu::Extent3d {
            width: part.width,
            height: part.height,
            depth_or_array_layers: 1,
        };
        let size = wgpu::Extent3d {
            width: tile.width,
            height: tile.height,
            depth_or_array_layers: 1,
        };
        let source = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Seura photo tile"),
            size: source_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut data: Vec<f16> = Vec::with_capacity(part.width as usize * part.height as usize * 4);
        for y in part.top..part.top + part.height {
            let start = (y as usize * image.width as usize + part.left as usize) * 4;
            data.extend_from_slice(&image.pixels[start..start + part.width as usize * 4]);
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
                bytes_per_row: Some(part.width * 8),
                rows_per_image: Some(part.height),
            },
            source_size,
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
        let bind_group = self.bind_group(&source_view, guide, &self.empty_stage);
        let view = target.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.draw(&mut encoder, &self.pipeline, &bind_group, &view);

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
