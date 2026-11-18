//! The GPU side: pipelines, textures and buffers, and the drawing of one frame.

use super::*;

mod draw;

pub(super) fn depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// A buffer for the corners of `room` quads of effects: six corners each, of three coordinates, two texture ones
/// and four of colour.
pub(super) fn effect_buffer(device: &wgpu::Device, room: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (room * 6 * 36) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

/// A buffer for the corners of `room` sprites: six corners each, of three coordinates and two texture ones.
pub(super) fn sprite_buffer(device: &wgpu::Device, room: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (room * 6 * 20) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

/// The canvas's surface, the device that draws on it and its queue, and how the surface is set up for a canvas
/// of this size.
async fn connect(canvas: &HtmlCanvasElement, (width, height): (u32, u32)) -> Result<(wgpu::Surface<'static>, wgpu::Device, wgpu::Queue, wgpu::SurfaceConfiguration), String> {
    // WebGPU where the browser has it, WebGL2 otherwise.
    let instance = wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor::new_without_display_handle()).await;
    let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone())).map_err(|error| error.to_string())?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() })
        .await
        .map_err(|error| format!("no GPU adapter: {error}"))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        })
        .await
        .map_err(|error| error.to_string())?;
    let mut config = surface.get_default_config(&adapter, width, height).ok_or("the canvas surface is not supported")?;
    // Colours are mixed as the original mixes them: as stored, without converting to linear light.
    config.format = config.format.remove_srgb_suffix();
    surface.configure(&device, &config);
    Ok((surface, device, queue, config))
}

/// A pipeline drawing corners of these attributes with this shader onto a surface of this format.
fn pipeline(device: &wgpu::Device, format: wgpu::TextureFormat, shader: &str, attributes: &[wgpu::VertexAttribute], stride: u64, blend: wgpu::BlendState, depth_write: bool) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(shader.into()) });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout { array_stride: stride, step_mode: wgpu::VertexStepMode::Vertex, attributes }],
        },
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: Some(depth_write),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState { format, blend: Some(blend), write_mask: wgpu::ColorWrites::ALL })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

impl Gpu {
    pub(super) async fn new() -> Result<Self, String> {
        let canvas = canvas()?;
        let (width, height) = pixel_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);
        let (surface, device, queue, config) = connect(&canvas, (width, height)).await?;

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });

        let pipeline = |shader: &str, attributes: &[wgpu::VertexAttribute], stride: u64, blend: wgpu::BlendState, depth_write: bool| pipeline(&device, config.format, shader, attributes, stride, blend, depth_write);
        let ground_pipeline = pipeline(GROUND_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x2, 3 => Float32x3], 40, wgpu::BlendState::REPLACE, true);
        let model_pipeline = pipeline(MODEL_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3], 32, wgpu::BlendState::REPLACE, true);
        let water_pipeline = pipeline(WATER_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2], 20, wgpu::BlendState::ALPHA_BLENDING, false);
        let sprite_pipeline = pipeline(SPRITE_SHADER, &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2], 20, wgpu::BlendState::ALPHA_BLENDING, false);
        let effect_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];
        let adding = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
        let effect_pipelines = [
            pipeline(EFFECT_SHADER, &effect_attributes, 36, wgpu::BlendState::ALPHA_BLENDING, false),
            pipeline(EFFECT_SHADER, &effect_attributes, 36, wgpu::BlendState { color: adding, alpha: adding }, false),
        ];
        let repeating = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Both pipelines read the same camera, each through a bind group of its own layout.
        let camera = |pipeline: &wgpu::RenderPipeline| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() }],
            })
        };
        let (camera_bind, sprite_camera_bind) = (camera(&ground_pipeline), camera(&sprite_pipeline));
        let (model_camera_bind, water_camera_bind) = (camera(&model_pipeline), camera(&water_pipeline));

        let effect_camera_binds = [camera(&effect_pipelines[0]), camera(&effect_pipelines[1])];
        let pixel_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let sprite_room = 64;
        let sprite_vertices = sprite_buffer(&device, sprite_room);
        let depth = depth_view(&device, &config);
        let effect_vertices = effect_buffer(&device, 64);
        Ok(Self {
            canvas,
            surface,
            device,
            queue,
            config,
            depth,
            camera_buffer,
            camera_bind,
            sprite_camera_bind,
            model_camera_bind,
            water_camera_bind,
            ground_pipeline,
            model_pipeline,
            water_pipeline,
            sprite_pipeline,
            sampler,
            repeating,
            pixel_sampler,
            sprite_textures: HashMap::new(),
            sprite_vertices,
            sprite_room,
            ring: None,
            mark: None,
            effect_pipelines,
            effect_camera_binds,
            effect_vertices,
            effect_room: 64,
            effect_pictures: HashMap::new(),
            arrow: None,
        })
    }

    pub(super) fn texture(&self, width: u32, height: u32, rgba: &[u8]) -> wgpu::TextureView {
        self.device
            .create_texture_with_data(
                &self.queue,
                &wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                rgba,
            )
            .create_view(&Default::default())
    }

    /// A bind group of one picture and how it is sampled.
    pub(super) fn textured(&self, layout: &wgpu::BindGroupLayout, picture: &wgpu::TextureView, sampler: &wgpu::Sampler) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(picture) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) }],
        })
    }

    /// Puts the ground's mesh and textures on the GPU. A texture that could not be read is drawn plain grey.
    pub(super) fn ground(&self, mesh: &crate::map::Mesh, pictures: &[Option<bmp::Image>]) -> Ground {
        let mut floats = Vec::with_capacity(mesh.vertices.len() * 10);
        for vertex in &mesh.vertices {
            floats.extend(vertex.position);
            floats.extend(vertex.uv);
            floats.extend(vertex.light_uv);
            floats.extend(vertex.colour);
        }
        let vertices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&floats), usage: wgpu::BufferUsages::VERTEX });
        let light = self.texture(mesh.light_atlas_size.0.max(1), mesh.light_atlas_size.1.max(1), if mesh.light_atlas.is_empty() { &[0, 0, 0, 255] } else { &mesh.light_atlas });
        let layout = self.ground_pipeline.get_bind_group_layout(1);
        let mut batches = Vec::new();
        for (index, triangles) in mesh.by_texture.iter().enumerate() {
            if triangles.is_empty() {
                continue;
            }
            let picture = match pictures.get(index).and_then(Option::as_ref) {
                Some(picture) => self.texture(picture.width, picture.height, &picture.rgba),
                None => self.texture(1, 1, &[128, 128, 128, 255]),
            };
            let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&picture) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&light) },
                ],
            });
            let indices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(triangles), usage: wgpu::BufferUsages::INDEX });
            batches.push((indices, triangles.len() as u32, bind));
        }
        Ground { vertices, batches }
    }

    /// Follows the canvas when the window changes size.
    pub(super) fn fit(&mut self) {
        let (width, height) = pixel_size(&self.canvas);
        if (width, height) != (self.config.width, self.config.height) {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.depth = depth_view(&self.device, &self.config);
        }
    }

    /// Whether a sprite's picture is on the GPU: put there now if it is ready.
    pub(super) fn has_sprite(&mut self, key: &SpriteKey) -> bool {
        if self.sprite_textures.contains_key(key) {
            return true;
        }
        let Some(picture) = figure::picture(key) else { return false };
        let canvas = &picture.canvas;
        let view = self.texture(canvas.width as u32, canvas.height as u32, &canvas.rgba);
        let bind = self.textured(&self.sprite_pipeline.get_bind_group_layout(1), &view, &self.pixel_sampler);
        self.sprite_textures.insert(key.clone(), SpriteTexture { bind, size: (canvas.width as f32, canvas.height as f32), feet: picture.feet });
        true
    }
}
