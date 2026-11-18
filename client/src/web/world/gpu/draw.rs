//! The drawing of one frame: the map, on it the sprites and the numbers, and over them the effects.

use super::*;

/// A number flying over someone: its picture, where it is, how large and how faded it is now, and how far across
/// from its place.
type Number = (SpriteKey, Vector, f32, f32, f32);

/// A picture to stand at a place: which, where its feet are, how much of it is left to see, how large it is
/// and how far across from its place, and how much nearer than its place it is drawn (by so much, at least from
/// so far).
struct Standing<'a> {
    key: &'a SpriteKey,
    at: Vector,
    left: f32,
    scale: f32,
    across: f32,
    nearer: (f32, f32),
}

impl Gpu {
    /// Draws the map and, on it, each sprite standing on its feet's place.
    pub(in crate::web::world) fn draw(&mut self, scene: &Scene, view: &View, sprites: &mut Vec<(SpriteKey, Vector, f32)>, rings: &[f32], marks: &[f32], numbers: &[Number], effects: &Quads) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&view.matrix));
        self.load_effects(effects);

        // The farthest first, so that nearer ones cover them where their see-through edges overlap.
        sprites.sort_by(|a, b| dot(subtract(b.1, view.eye), view.forward).total_cmp(&dot(subtract(a.1, view.eye), view.forward)));
        let rings = if self.ring.is_some() { rings } else { &[] };
        let marks = if self.mark.is_some() { marks } else { &[] };
        let numbers: Vec<_> = numbers.iter().filter(|number| self.sprite_textures.contains_key(&number.0)).collect();
        let corners = self.load_sprites(view, sprites, rings, marks, &numbers);

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            _ => return,
        };
        let target = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = self.pass(&mut encoder, &target);
            self.draw_map(&mut pass, scene);
            if !corners.is_empty() {
                self.draw_sprites(&mut pass, ((marks.len() / 5) as u32, (rings.len() / 5) as u32), sprites, &numbers);
            }
            self.draw_effects(&mut pass, effects);
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
    }

    /// Puts the corners of this frame's effects on the GPU, in a buffer made larger when they do not fit.
    fn load_effects(&mut self, effects: &Quads) {
        if effects.pictures.len() > self.effect_room {
            self.effect_room = effects.pictures.len().next_power_of_two();
            self.effect_vertices = effect_buffer(&self.device, self.effect_room);
        }
        if !effects.pictures.is_empty() {
            self.queue.write_buffer(&self.effect_vertices, 0, bytemuck::cast_slice(&effects.corners));
        }
    }

    /// Puts on the GPU, in a buffer made larger when they do not fit, the corners of what the sprite pipeline
    /// draws, in the order it draws them: the marks, the rings, the sprites, the numbers. Gives those corners back.
    fn load_sprites(&mut self, view: &View, sprites: &[(SpriteKey, Vector, f32)], rings: &[f32], marks: &[f32], numbers: &[&Number]) -> Vec<f32> {
        let room = sprites.len() + rings.len() / 30 + marks.len() / 30 + numbers.len() + 1;
        if room > self.sprite_room {
            self.sprite_room = room.next_power_of_two();
            self.sprite_vertices = sprite_buffer(&self.device, self.sprite_room);
        }
        let mut corners = Vec::with_capacity(sprites.len() * 30 + rings.len() + marks.len());
        corners.extend_from_slice(marks);
        corners.extend_from_slice(rings);
        // Everyone standing: each as if two cells nearer, in the same place on the screen, as the original does
        // (a ledge or a doorway over the cell does not cut through the one standing there).
        for (key, feet, left) in sprites.iter() {
            self.stood(&mut corners, view, &Standing { key, at: *feet, left: *left, scale: 1.0, across: 0.0, nearer: (10.0, 20.0) });
        }
        // The numbers: each picture at its place, as large and as faded as it is now, over everything.
        for (key, at, scale, alpha, across) in numbers.iter().copied() {
            self.stood(&mut corners, view, &Standing { key, at: *at, left: *alpha, scale: *scale, across: *across, nearer: (25.0, 40.0) });
        }
        self.queue.write_buffer(&self.sprite_vertices, 0, bytemuck::cast_slice(&corners));
        corners
    }

    /// The six corners of a picture standing at a place, facing the camera, added to `corners`: each where it
    /// is in the world and where of the picture it shows.
    fn stood(&self, corners: &mut Vec<f32>, view: &View, standing: &Standing) {
        let texture = &self.sprite_textures[standing.key];
        // A touch nearer than the ground it stands on, by so much, at least from so far.
        let nearer = 1.0 - standing.nearer.0 / dot(subtract(standing.at, view.eye), view.forward).max(standing.nearer.1);
        // How much of it is left to see, in the shader's sixteenths.
        let faded = ((1.0 - standing.left.clamp(0.0, 1.0)) * 16.0).floor().min(15.0) * 2.0;
        let (width, height) = texture.size;
        let size = SPRITE_SCALE * standing.scale;
        for (x, y) in [(0.0, 0.0), (width, 0.0), (0.0, height), (0.0, height), (width, 0.0), (width, height)] {
            let corner = along(along(standing.at, view.right, (x - texture.feet.0 + standing.across) * size), view.up, (texture.feet.1 - y) * size);
            corners.extend(along(view.eye, subtract(corner, view.eye), nearer));
            corners.extend([x / width, y / height + faded]);
        }
    }

    /// The frame's one render pass: onto `target`, from black and from the farthest depth.
    fn pass<'encoder>(&self, encoder: &'encoder mut wgpu::CommandEncoder, target: &wgpu::TextureView) -> wgpu::RenderPass<'encoder> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
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
        })
    }

    /// The map: its ground, its models and its water.
    fn draw_map(&self, pass: &mut wgpu::RenderPass, scene: &Scene) {
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
    }

    /// What `load_sprites` put on the GPU, in its order: the marks and the rings, of so many corners each, then
    /// the sprites and the numbers, each with its own picture.
    fn draw_sprites(&self, pass: &mut wgpu::RenderPass, (mark_corners, ring_corners): (u32, u32), sprites: &[(SpriteKey, Vector, f32)], numbers: &[&Number]) {
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
        for (index, (key, ..)) in sprites.iter().enumerate() {
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

    /// The effects' quads, each through the see-through pipeline or the one that adds light.
    fn draw_effects(&self, pass: &mut wgpu::RenderPass, effects: &Quads) {
        if effects.pictures.is_empty() {
            return;
        }
        pass.set_vertex_buffer(0, self.effect_vertices.slice(..));
        for (index, (binds, adds)) in effects.pictures.iter().enumerate() {
            let which = usize::from(*adds);
            pass.set_pipeline(&self.effect_pipelines[which]);
            pass.set_bind_group(0, &self.effect_camera_binds[which], &[]);
            pass.set_bind_group(1, &binds[which], &[]);
            let first = index as u32 * 6;
            pass.draw(first..first + 6, 0..1);
        }
    }
}
