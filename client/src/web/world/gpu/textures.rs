//! Pictures put on the GPU: a texture of a picture's bytes, the bind groups a pipeline draws it through, the
//! ground's mesh with its textures, and the sprites' pictures made the first time each is asked for.

use super::*;

impl Gpu {
    pub(in crate::web::world) fn texture(&self, width: u32, height: u32, rgba: &[u8]) -> wgpu::TextureView {
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
    pub(in crate::web::world) fn textured(&self, layout: &wgpu::BindGroupLayout, picture: &wgpu::TextureView, sampler: &wgpu::Sampler) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(picture) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) }
            ],
        })
    }

    /// A picture for the sprites' pipeline, shown pixel by pixel.
    pub(in crate::web::world) fn sprite_bind(&self, width: u32, height: u32, rgba: &[u8]) -> wgpu::BindGroup {
        let view = self.texture(width, height, rgba);
        self.textured(&self.sprite_pipeline.get_bind_group_layout(1), &view, &self.pixel_sampler)
    }

    /// A picture for the effects' two pipelines (the see-through one and the one that adds light), sampled so.
    pub(in crate::web::world) fn effect_binds(&self, width: u32, height: u32, rgba: &[u8], sampler: &wgpu::Sampler) -> Rc<[wgpu::BindGroup; 2]> {
        let view = self.texture(width, height, rgba);
        Rc::new([0, 1].map(|which| self.textured(&self.effect_pipelines[which].get_bind_group_layout(1), &view, sampler)))
    }

    /// Puts the ground's mesh and textures on the GPU. A texture that could not be read is drawn plain grey.
    pub(in crate::web::world) fn ground(&self, mesh: &crate::map::Mesh, pictures: &[Option<bmp::Image>]) -> Ground {
        let mut floats = Vec::with_capacity(mesh.vertices.len() * 10);
        for vertex in &mesh.vertices {
            floats.extend(vertex.position);
            floats.extend(vertex.uv);
            floats.extend(vertex.light_uv);
            floats.extend(vertex.colour);
        }
        let vertices = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&floats), usage: wgpu::BufferUsages::VERTEX }
        );
        let light = self.texture(
            mesh.light_atlas_size.0.max(1),
            mesh.light_atlas_size.1.max(1),
            if mesh.light_atlas.is_empty() { &[0, 0, 0, 255] } else { &mesh.light_atlas }
        );
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
            let indices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(triangles),
                usage: wgpu::BufferUsages::INDEX
            });
            batches.push((indices, triangles.len() as u32, bind));
        }
        Ground { vertices, batches }
    }

    /// Whether a sprite's picture is on the GPU: put there now if it is ready.
    pub(in crate::web::world) fn has_sprite(&mut self, key: &SpriteKey) -> bool {
        if self.sprite_textures.contains_key(key) {
            return true;
        }
        let Some(picture) = figure::picture(key) else { return false };
        let canvas = &picture.canvas;
        let view = self.texture(canvas.width as u32, canvas.height as u32, &canvas.rgba);
        let bind = self.textured(&self.sprite_pipeline.get_bind_group_layout(1), &view, &self.pixel_sampler);
        let adding = self.textured(&self.sprite_adding_pipeline.get_bind_group_layout(1), &view, &self.pixel_sampler);
        self.sprite_textures.insert(key.clone(), SpriteTexture { bind, adding, size: (canvas.width as f32, canvas.height as f32), feet: picture.feet });
        true
    }

    /// A frame of a sprite as one picture for the effects' pipelines (an arrow in flight): put on the GPU now if
    /// it is ready, nothing while it is not. (The sprites of the effects made of the client's primitives are
    /// drawn a layer at a time: `gpu/layers.rs`.)
    pub(in crate::web::world) fn effect_sprite(&mut self, key: &SpriteKey) -> Option<EffectSprite> {
        if let Some(known) = self.effect_sprites.get(key) {
            return Some(known.clone());
        }
        let picture = figure::picture(key)?;
        let canvas = &picture.canvas;
        let made = EffectSprite {
            binds: self.effect_binds(canvas.width as u32, canvas.height as u32, &canvas.rgba, &self.pixel_sampler),
            size: (canvas.width as f32, canvas.height as f32)
        };
        self.effect_sprites.insert(key.clone(), made.clone());
        Some(made)
    }
}
