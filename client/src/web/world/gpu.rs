//! The GPU side: pipelines, textures and buffers, and the drawing of one frame.

use super::*;

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

impl Gpu {
    pub(super) async fn new() -> Result<Self, String> {
        let canvas = canvas()?;
        let (width, height) = pixel_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);

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

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });

        let pipeline = |shader: &str, attributes: &[wgpu::VertexAttribute], stride: u64, blend: wgpu::BlendState, depth_write: bool| {
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
                    targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(blend), write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
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

    /// Draws the map and, on it, each sprite standing on its feet's place.
    pub(super) fn draw(&mut self, scene: &Scene, view: &View, sprites: &mut Vec<(SpriteKey, Vector)>, rings: &[f32], marks: &[f32], numbers: &[(SpriteKey, Vector, f32, f32, f32)], effects: &(Vec<f32>, Vec<(Rc<[wgpu::BindGroup; 2]>, bool)>)) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&view.matrix));
        if effects.1.len() > self.effect_room {
            self.effect_room = effects.1.len().next_power_of_two();
            self.effect_vertices = effect_buffer(&self.device, self.effect_room);
        }
        if !effects.1.is_empty() {
            self.queue.write_buffer(&self.effect_vertices, 0, bytemuck::cast_slice(&effects.0));
        }

        // The farthest first, so that nearer ones cover them where their see-through edges overlap.
        sprites.sort_by(|a, b| dot(subtract(b.1, view.eye), view.forward).total_cmp(&dot(subtract(a.1, view.eye), view.forward)));
        let rings = if self.ring.is_some() { rings } else { &[] };
        let marks = if self.mark.is_some() { marks } else { &[] };
        let mark_corners = (marks.len() / 5) as u32;
        let ring_corners = (rings.len() / 5) as u32;
        let numbers: Vec<_> = numbers.iter().filter(|number| self.sprite_textures.contains_key(&number.0)).collect();
        let room = sprites.len() + rings.len() / 30 + marks.len() / 30 + numbers.len() + 1;
        if room > self.sprite_room {
            self.sprite_room = room.next_power_of_two();
            self.sprite_vertices = sprite_buffer(&self.device, self.sprite_room);
        }
        let mut corners = Vec::with_capacity(sprites.len() * 30 + rings.len() + marks.len());
        corners.extend_from_slice(marks);
        corners.extend_from_slice(rings);
        for (key, feet) in sprites.iter() {
            let texture = &self.sprite_textures[key];
            // A picture standing on the feet, facing the camera, a touch nearer than the ground it stands on.
            // It is drawn as if two cells nearer, in the same place on the screen, as the original does: a ledge
            // or a doorway over the cell does not cut through the one standing there.
            let nearer = 1.0 - 10.0 / dot(subtract(*feet, view.eye), view.forward).max(20.0);
            let (width, height) = texture.size;
            for (x, y) in [(0.0, 0.0), (width, 0.0), (0.0, height), (0.0, height), (width, 0.0), (width, height)] {
                let corner = along(along(*feet, view.right, (x - texture.feet.0) * SPRITE_SCALE), view.up, (texture.feet.1 - y) * SPRITE_SCALE);
                corners.extend(along(view.eye, subtract(corner, view.eye), nearer));
                corners.extend([x / width, y / height]);
            }
        }
        // The numbers: each picture at its place, as large and as faded as it is now, over everything.
        for (key, at, scale, alpha, across) in numbers.iter().copied() {
            let texture = &self.sprite_textures[key];
            let nearer = 1.0 - 25.0 / dot(subtract(*at, view.eye), view.forward).max(40.0);
            let (width, height) = texture.size;
            let faded = ((1.0 - alpha.clamp(0.0, 1.0)) * 16.0).floor().min(15.0) * 2.0;
            for (x, y) in [(0.0, 0.0), (width, 0.0), (0.0, height), (0.0, height), (width, 0.0), (width, height)] {
                let corner = along(along(*at, view.right, (x - texture.feet.0 + across) * SPRITE_SCALE * scale), view.up, (texture.feet.1 - y) * SPRITE_SCALE * scale);
                corners.extend(along(view.eye, subtract(corner, view.eye), nearer));
                corners.extend([x / width, y / height + faded]);
            }
        }
        self.queue.write_buffer(&self.sprite_vertices, 0, bytemuck::cast_slice(&corners));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => return,
        };
        let target = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.ground_pipeline);
            pass.set_bind_group(0, &self.camera_bind, &[]);
            pass.set_vertex_buffer(0, scene.ground.vertices.slice(..));
            for (indices, count, bind) in &scene.ground.batches {
                pass.set_bind_group(1, bind, &[]);
                pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..*count, 0, 0..1);
            }
            pass.set_pipeline(&self.model_pipeline);
            pass.set_bind_group(0, &self.model_camera_bind, &[]);
            for (vertices, count, bind) in &scene.models {
                pass.set_bind_group(1, bind, &[]);
                pass.set_vertex_buffer(0, vertices.slice(..));
                pass.draw(0..*count, 0..1);
            }
            if let Some(water) = &scene.water {
                pass.set_pipeline(&self.water_pipeline);
                pass.set_bind_group(0, &self.water_camera_bind, &[]);
                pass.set_bind_group(1, &water.frames[(now() / water.frame_ms) as usize % water.frames.len()], &[]);
                pass.set_vertex_buffer(0, water.vertices.slice(..));
                pass.draw(0..6, 0..1);
            }
            if !corners.is_empty() {
                pass.set_pipeline(&self.sprite_pipeline);
                pass.set_bind_group(0, &self.sprite_camera_bind, &[]);
                pass.set_vertex_buffer(0, self.sprite_vertices.slice(..));
                if let Some(mark) = self.mark.as_ref().filter(|_| mark_corners > 0) {
                    pass.set_bind_group(1, mark, &[]);
                    pass.draw(0..mark_corners, 0..1);
                }
                if let Some(ring) = self.ring.as_ref().filter(|_| ring_corners > 0) {
                    pass.set_bind_group(1, ring, &[]);
                    pass.draw(mark_corners..mark_corners + ring_corners, 0..1);
                }
                for (index, (key, _)) in sprites.iter().enumerate() {
                    pass.set_bind_group(1, &self.sprite_textures[key].bind, &[]);
                    let first = mark_corners + ring_corners + index as u32 * 6;
                    pass.draw(first..first + 6, 0..1);
                }
                for (index, number) in numbers.iter().enumerate() {
                    pass.set_bind_group(1, &self.sprite_textures[&number.0].bind, &[]);
                    let first = mark_corners + ring_corners + (sprites.len() + index) as u32 * 6;
                    pass.draw(first..first + 6, 0..1);
                }
            }
            if !effects.1.is_empty() {
                pass.set_vertex_buffer(0, self.effect_vertices.slice(..));
                for (index, (binds, adds)) in effects.1.iter().enumerate() {
                    let which = usize::from(*adds);
                    pass.set_pipeline(&self.effect_pipelines[which]);
                    pass.set_bind_group(0, &self.effect_camera_binds[which], &[]);
                    pass.set_bind_group(1, &binds[which], &[]);
                    let first = index as u32 * 6;
                    pass.draw(first..first + 6, 0..1);
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}
