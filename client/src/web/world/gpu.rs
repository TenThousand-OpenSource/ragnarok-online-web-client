//! The GPU side: pipelines, textures and buffers, and the drawing of one frame.

use super::*;

mod draw;
mod emitters;
mod granny;
mod land;
mod layers;
mod textures;

pub(super) use layers::{frames_of, sheet};

/// A frame of a sprite as one picture on the GPU for the effects' pipelines: the picture for each of the two, and
/// its size.
#[derive(Clone)]
pub(super) struct EffectSprite {
    pub(super) binds: Rc<[wgpu::BindGroup; 2]>,
    pub(super) size: (f32, f32),
}

/// A sprite's picture on the GPU: for the see-through pipeline, and for the one that adds its light to what is
/// behind it.
pub(super) struct SpriteTexture {
    pub(super) bind: wgpu::BindGroup,
    pub(super) adding: wgpu::BindGroup,
    pub(super) size: (f32, f32),
    pub(super) feet: (f32, f32),
}

pub(super) struct Ground {
    pub(super) vertices: wgpu::Buffer,
    /// Per texture: its triangles, how many indices they are, and the texture with the baked light.
    pub(super) batches: Vec<(wgpu::Buffer, u32, wgpu::BindGroup)>,
}

pub(super) struct Gpu {
    pub(super) canvas: HtmlCanvasElement,
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) depth: wgpu::TextureView,
    pub(super) camera_buffer: wgpu::Buffer,
    pub(super) camera_bind: wgpu::BindGroup,
    pub(super) sprite_camera_bind: wgpu::BindGroup,
    pub(super) model_camera_binds: [wgpu::BindGroup; 2],
    pub(super) water_camera_bind: wgpu::BindGroup,
    pub(super) ground_pipeline: wgpu::RenderPipeline,
    /// The models' pipelines: for pictures cut out by their colour, and for those mixed in by their alpha.
    pub(super) model_pipelines: [wgpu::RenderPipeline; 2],
    pub(super) water_pipeline: wgpu::RenderPipeline,
    pub(super) sprite_pipeline: wgpu::RenderPipeline,
    /// The sprites' pipeline that adds a picture's light to what is behind it (a body an effect has lit:
    /// `crate::body`), and the camera for it.
    pub(super) sprite_adding_pipeline: wgpu::RenderPipeline,
    pub(super) sprite_adding_camera_bind: wgpu::BindGroup,
    pub(super) sampler: wgpu::Sampler,
    /// For pictures laid side by side again and again: model textures and the water.
    pub(super) repeating: wgpu::Sampler,
    /// Sprites are pixel art: shown pixel by pixel, not smoothed.
    pub(super) pixel_sampler: wgpu::Sampler,
    /// The pictures drawn so far on this map.
    pub(super) sprite_textures: HashMap<SpriteKey, SpriteTexture>,
    /// The corners of this frame's sprites, and how many sprites the buffer holds.
    pub(super) sprite_vertices: wgpu::Buffer,
    pub(super) sprite_room: usize,
    /// The picture of a portal's rings.
    pub(super) ring: Option<wgpu::BindGroup>,
    /// The mark on the ground cell the pointer is over: green corners, as the original's.
    pub(super) mark: Option<wgpu::BindGroup>,
    /// For the effects: the see-through pipeline and the one that adds light, the camera for each, and the
    /// corners of this frame's quads.
    pub(super) effect_pipelines: [wgpu::RenderPipeline; 2],
    pub(super) effect_camera_binds: [wgpu::BindGroup; 2],
    pub(super) effect_vertices: wgpu::Buffer,
    pub(super) effect_room: usize,
    /// The pictures of the effects drawn by code (the rings of a cast, the mark on its target, an arrow), by name.
    pub(super) effect_pictures: HashMap<&'static str, Rc<[wgpu::BindGroup; 2]>>,
    /// The sprites' pictures the effects' pipelines draw, a frame to a picture (an arrow in flight), by picture.
    pub(super) effect_sprites: HashMap<SpriteKey, EffectSprite>,
    /// The pictures of the sprites of the effects made of the client's primitives, each by itself: by the
    /// sprite's path, then by whether the picture is a truecolor one and its number (`gpu/layers.rs`).
    pub(super) effect_layers: HashMap<String, HashMap<(bool, usize), Rc<[wgpu::BindGroup; 2]>>>,
    /// The Granny models' pipeline, pictures and corners (`gpu/granny.rs`), once one was drawn.
    pub(super) granny: Option<granny::Kept>,
    /// The particle emitters' pipelines, pictures and corners (`gpu/emitters.rs`), once a particle was drawn.
    pub(super) emitters: Option<emitters::Kept>,
}

pub(super) fn canvas() -> Result<HtmlCanvasElement, String> {
    crate::web::by_id("game")
        .ok_or("no #game canvas")?
        .dyn_into()
        .map_err(js_error)
}

/// The canvas's size in real pixels, for a sharp picture on dense screens.
pub(super) fn pixel_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let ratio = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio());
    (((f64::from(canvas.client_width()) * ratio) as u32).max(1), ((f64::from(canvas.client_height()) * ratio) as u32).max(1))
}

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

/// A buffer for the corners of `room` quads, of effects or of sprites alike: six corners each, of three
/// coordinates, two texture ones and four of colour.
pub(super) fn quad_buffer(device: &wgpu::Device, room: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (room * 6 * 36) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false
    })
}

/// The canvas's surface, the device that draws on it and its queue, and how the surface is set up for a canvas
/// of this size.
async fn connect(
    canvas: &HtmlCanvasElement,
    (width, height): (u32, u32)
) -> Result<(wgpu::Surface<'static>, wgpu::Device, wgpu::Queue, wgpu::SurfaceConfiguration), String> {
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
fn pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shader: &str,
    attributes: &[wgpu::VertexAttribute],
    stride: u64,
    blend: wgpu::BlendState,
    depth_write: bool
) -> wgpu::RenderPipeline {
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

/// The three samplers: the usual one, smooth; the same that repeats its picture (the water's); and the one
/// that takes the nearest pixel.
fn samplers(device: &wgpu::Device) -> (wgpu::Sampler, wgpu::Sampler, wgpu::Sampler) {
    let smooth = wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() };
    let repeating = wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    };
    (device.create_sampler(&smooth), device.create_sampler(&repeating), device.create_sampler(&wgpu::SamplerDescriptor::default()))
}

/// The bind group through which a pipeline reads the camera.
fn camera_bind(device: &wgpu::Device, camera: &wgpu::Buffer, pipeline: &wgpu::RenderPipeline) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera.as_entire_binding() }],
    })
}

impl Gpu {
    pub(super) async fn new() -> Result<Self, String> {
        let canvas = canvas()?;
        let (width, height) = pixel_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);
        let (surface, device, queue, config) = connect(&canvas, (width, height)).await?;

        // The camera's matrix and, after it, what the programs of the map are told besides (`gpu/land.rs`).
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: land::FRAME_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let (sampler, repeating, pixel_sampler) = samplers(&device);

        let pipeline =
            |shader: &str, attributes: &[wgpu::VertexAttribute], stride: u64, blend: wgpu::BlendState, depth_write: bool| pipeline(
                &device,
                config.format,
                shader,
                attributes,
                stride,
                blend,
                depth_write
            );
        let ground_pipeline = pipeline(
            GROUND_SHADER,
            &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x2, 3 => Float32x3],
            40,
            wgpu::BlendState::REPLACE,
            true
        );
        // The models: those whose pictures are cut out by their colour, and those mixed in by their alpha.
        let model_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x3];
        let model_pipelines = [
            pipeline(MODEL_SHADER, &model_attributes, 44, wgpu::BlendState::REPLACE, true),
            pipeline(MODEL_BLENDING_SHADER, &model_attributes, 44, wgpu::BlendState::ALPHA_BLENDING, false)
        ];
        let water_pipeline = pipeline(
            WATER_SHADER,
            &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32],
            24,
            wgpu::BlendState::ALPHA_BLENDING,
            false
        );
        let effect_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];
        let adding = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
        let sprite_pipeline = pipeline(SPRITE_SHADER, &effect_attributes, 36, wgpu::BlendState::ALPHA_BLENDING, false);
        let sprite_adding_pipeline = pipeline(SPRITE_SHADER, &effect_attributes, 36, wgpu::BlendState { color: adding, alpha: adding }, false);
        let effect_pipelines = [
            pipeline(EFFECT_SHADER, &effect_attributes, 36, wgpu::BlendState::ALPHA_BLENDING, false),
            pipeline(EFFECT_SHADER, &effect_attributes, 36, wgpu::BlendState { color: adding, alpha: adding }, false),
        ];

        // The pipelines read the same camera, each through a bind group of its own layout.
        let camera = |pipeline: &wgpu::RenderPipeline| camera_bind(&device, &camera_buffer, pipeline);
        let (camera_bind, sprite_camera_bind) = (camera(&ground_pipeline), camera(&sprite_pipeline));
        let (model_camera_binds, water_camera_bind) = ([camera(&model_pipelines[0]), camera(&model_pipelines[1])], camera(&water_pipeline));

        let effect_camera_binds = [camera(&effect_pipelines[0]), camera(&effect_pipelines[1])];
        let sprite_adding_camera_bind = camera(&sprite_adding_pipeline);
        let sprite_room = 64;
        let (sprite_vertices, effect_vertices) = (quad_buffer(&device, sprite_room), quad_buffer(&device, 64));
        Ok(Self {
            canvas,
            depth: depth_view(&device, &config),
            surface,
            device,
            queue,
            config,
            camera_buffer,
            camera_bind,
            sprite_camera_bind,
            model_camera_binds,
            water_camera_bind,
            ground_pipeline,
            model_pipelines,
            water_pipeline,
            sprite_pipeline,
            sprite_adding_pipeline,
            sprite_adding_camera_bind,
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
            effect_sprites: HashMap::new(),
            effect_layers: HashMap::new(),
            granny: None, emitters: None,
        })
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
}
