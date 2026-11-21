//! The Granny models on the GPU: their pipeline, their pictures (a model's own and the guilds' emblems on
//! the flags) and this frame's corners, as `world/granny.rs` worked them out.

use super::*;
use crate::web::world::granny::{self as models, Corners, Picture};

/// What the GPU keeps for them; made when the first model is drawn.
pub(in crate::web::world) struct Kept {
    pipeline: wgpu::RenderPipeline,
    camera: wgpu::BindGroup,
    pictures: HashMap<Picture, wgpu::BindGroup>,
    vertices: wgpu::Buffer,
    /// How many corners the buffer holds.
    room: usize,
    batches: Vec<(Picture, std::ops::Range<u32>)>,
}

/// A corner is a place, where of the picture it is, and a colour: nine numbers.
const CORNER: u64 = 36;

impl Gpu {
    fn granny_kept(&self, room: usize) -> Kept {
        // Drawn into the depth, unlike the sprites: a model's own parts hide one another.
        let attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];
        let pipeline = pipeline(&self.device, self.config.format, GRANNY_SHADER, &attributes, CORNER, wgpu::BlendState::ALPHA_BLENDING, true);
        Kept {
            camera: camera_bind(&self.device, &self.camera_buffer, &pipeline),
            pipeline,
            pictures: HashMap::new(),
            vertices: self.granny_buffer(room),
            room,
            batches: Vec::new(),
        }
    }

    fn granny_buffer(&self, room: usize) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: room as u64 * CORNER,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// This frame's corners are put on the GPU, with the pictures they need that are not there yet.
    pub(in crate::web::world) fn granny_loaded(&mut self, read: &models::Models, guilds: &guild::Guilds, corners: Corners) {
        let count = corners.numbers.len() / 9;
        if count == 0 && self.granny.is_none() {
            return;
        }
        let mut kept = self.granny.take().unwrap_or_else(|| self.granny_kept(count.next_power_of_two().max(1024)));
        if count > kept.room {
            kept.room = count.next_power_of_two();
            kept.vertices = self.granny_buffer(kept.room);
        }
        let layout = kept.pipeline.get_bind_group_layout(1);
        for (picture, _) in &corners.batches {
            if kept.pictures.contains_key(picture) {
                continue;
            }
            let pixels = match picture {
                Picture::Own(name, index) => read.model(name).and_then(|model| model.granny.textures.get(*index)).map(
                    |texture| (texture.width as u32, texture.height as u32, &texture.pixels[..])
                ),
                Picture::Emblem(guild, version) => guilds.emblem((*guild, *version)).map(|image| (image.width, image.height, &image.rgba[..])),
            };
            if let Some((width, height, rgba)) = pixels.filter(|(width, height, rgba)| *width > 0 && rgba.len() == (*width * *height * 4) as usize) {
                let bind = self.textured(&layout, &self.texture(width, height, rgba), &self.sampler);
                kept.pictures.insert(picture.clone(), bind);
            }
        }
        if count > 0 {
            self.queue.write_buffer(&kept.vertices, 0, bytemuck::cast_slice(&corners.numbers));
        }
        kept.batches = corners.batches;
        self.granny = Some(kept);
    }

    /// The models of this frame, each mesh with its picture.
    pub(super) fn draw_granny(&self, pass: &mut wgpu::RenderPass) {
        let Some(kept) = self.granny.as_ref().filter(|kept| !kept.batches.is_empty()) else { return };
        pass.set_pipeline(&kept.pipeline);
        pass.set_bind_group(0, &kept.camera, &[]);
        pass.set_vertex_buffer(0, kept.vertices.slice(..));
        for (picture, corners) in &kept.batches {
            if let Some(bind) = kept.pictures.get(picture) {
                pass.set_bind_group(1, bind, &[]);
                pass.draw(corners.clone(), 0..1);
            }
        }
    }
}
