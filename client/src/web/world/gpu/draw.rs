//! The drawing of one frame: the map, on it the sprites and the numbers, and over them the effects.

use super::*;

/// A number flying over someone: its picture, where it is, how large and how faded it is now, and how far across
/// from its place.
type Number = (SpriteKey, Vector, f32, f32, f32);

/// A picture to stand at a place: which, where its feet are, how much of it is left to see, how large it is
/// and how far across from its place, how much nearer than its place it is drawn (by so much, at least from
/// so far), and how it is drawn (`crate::body::Quad`).
struct Standing<'a> {
    key: &'a SpriteKey,
    at: Vector,
    left: f32,
    scale: f32,
    across: f32,
    nearer: (f32, f32),
    quad: crate::body::Quad,
}

impl Gpu {
    /// Draws the map and, on it, each sprite standing on its feet's place.
    pub(in crate::web::world) fn draw(
        &mut self,
        scene: &Scene,
        view: &View,
        sprites: &mut Vec<Stood>,
        rings: &[f32],
        marks: &[f32],
        numbers: &[Number],
        effects: &Quads
    ) {
        self.frame_figures(scene, view);
        self.load_effects(effects);

        // The farthest first, so that nearer ones cover them where their see-through edges overlap; and the
        // shadows before every body (`Quad::under`).
        sprites.sort_by(|a, b| b.3.under.cmp(&a.3.under).then(
            dot(subtract(b.1, view.eye), view.forward).total_cmp(&dot(subtract(a.1, view.eye), view.forward))
        ));
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
            let mut pass = self.pass(&mut encoder, &target, Self::sky(scene));
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
            self.effect_vertices = quad_buffer(&self.device, self.effect_room);
        }
        if !effects.pictures.is_empty() {
            self.queue.write_buffer(&self.effect_vertices, 0, bytemuck::cast_slice(&effects.corners));
        }
    }

    /// Puts on the GPU, in a buffer made larger when they do not fit, the corners of what the sprite pipeline
    /// draws, in the order it draws them: the marks, the rings, the sprites, the numbers. Gives those corners back.
    fn load_sprites(&mut self, view: &View, sprites: &[Stood], rings: &[f32], marks: &[f32], numbers: &[&Number]) -> Vec<f32> {
        let room = sprites.len() + rings.len() / 30 + marks.len() / 30 + numbers.len() + 1;
        if room > self.sprite_room {
            self.sprite_room = room.next_power_of_two();
            self.sprite_vertices = quad_buffer(&self.device, self.sprite_room);
        }
        let mut corners = Vec::with_capacity((sprites.len() * 6 + (rings.len() + marks.len()) / 5) * 9);
        // The marks and the rings come as a place and where of the picture: each corner is given white.
        for corner in marks.chunks_exact(5).chain(rings.chunks_exact(5)) {
            corners.extend_from_slice(corner);
            corners.extend([1.0; 4]);
        }
        // Everyone standing: each as if two cells nearer, in the same place on the screen, as the original does
        // (a ledge or a doorway over the cell does not cut through the one standing there).
        for (key, feet, left, quad) in sprites.iter() {
            self.stood(&mut corners, view, &Standing { key, at: *feet, left: *left, scale: 1.0, across: 0.0, nearer: (10.0, 20.0), quad: *quad });
        }
        // The numbers: each picture at its place, as large and as faded as it is now, over everything.
        for (key, at, scale, alpha, across) in numbers.iter().copied() {
            self.stood(&mut corners, view, &Standing {
                key,
                at: *at,
                left: *alpha,
                scale: *scale,
                across: *across,
                nearer: (25.0, 40.0),
                quad: crate::body::Quad::PLAIN
            });
        }
        self.queue.write_buffer(&self.sprite_vertices, 0, bytemuck::cast_slice(&corners));
        corners
    }

    /// The six corners of a picture standing at a place, facing the camera, added to `corners`: each where it
    /// is in the world, where of the picture it shows and its colour. The picture stands on its feet and is
    /// then changed as its `Quad` says, in that order: made larger or smaller about its feet, its edges moved,
    /// turned about its middle. (Which way round the client turns it was not made out: clockwise here.)
    fn stood(&self, corners: &mut Vec<f32>, view: &View, standing: &Standing) {
        let texture = &self.sprite_textures[standing.key];
        // A touch nearer than the ground it stands on, by so much, at least from so far.
        let nearer = 1.0 - standing.nearer.0 / dot(subtract(standing.at, view.eye), view.forward).max(standing.nearer.1);
        let quad = &standing.quad;
        // Its colour, and how much of it is left to see.
        let colour = [
            f32::from(quad.colour[0]) / 255.0,
            f32::from(quad.colour[1]) / 255.0,
            f32::from(quad.colour[2]) / 255.0,
            f32::from(quad.colour[3]) / 255.0 * standing.left.clamp(0.0, 1.0)
        ];
        let (width, height) = texture.size;
        let size = SPRITE_SCALE * standing.scale;
        // Where a corner of the picture is from its feet, in its pixels, to the right and down.
        let placed = |x: f32, y: f32| {
            let across = (x - texture.feet.0 + standing.across) * quad.scale + if x == 0.0 { quad.edges[0] } else { quad.edges[2] };
            let down = (y - texture.feet.1) * quad.scale + if y == 0.0 {
                quad.edges[1] + quad.of_height[0] * height * quad.scale
            } else {
                quad.edges[3] + quad.of_height[1] * height * quad.scale
            };
            (across, down)
        };
        let (first, last) = (placed(0.0, 0.0), placed(width, height));
        let middle = ((first.0 + last.0) / 2.0, (first.1 + last.1) / 2.0);
        let (sine, cosine) = quad.turn.to_radians().sin_cos();
        for (x, y) in [(0.0, 0.0), (width, 0.0), (0.0, height), (0.0, height), (width, 0.0), (width, height)] {
            let (across, down) = placed(x, y);
            let (across, down) = (across - middle.0, down - middle.1);
            let (across, down) = (middle.0 + across * cosine - down * sine, middle.1 + across * sine + down * cosine);
            let corner = along(along(standing.at, view.right, across * size), view.up, -down * size);
            corners.extend(along(view.eye, subtract(corner, view.eye), nearer));
            corners.extend([x / width, y / height]);
            corners.extend(colour);
        }
    }

    /// The frame's one render pass: onto `target`, from the colour of the map's sky and from the farthest depth.
    fn pass<'encoder>(&self, encoder: &'encoder mut wgpu::CommandEncoder, target: &wgpu::TextureView, sky: wgpu::Color) -> wgpu::RenderPass<'encoder> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(sky), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                stencil_ops: None,
            }),
            ..Default::default()
        })
    }

    /// What `load_sprites` put on the GPU, in its order: the marks and the rings, of so many corners each, then
    /// the sprites and the numbers, each with its own picture.
    fn draw_sprites(&self, pass: &mut wgpu::RenderPass, (mark_corners, ring_corners): (u32, u32), sprites: &[Stood], numbers: &[&Number]) {
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
        // A picture whose light is added to what is behind it goes through the pipeline that does so.
        let mut adding = false;
        for (index, (key, _, _, quad)) in sprites.iter().enumerate() {
            let texture = &self.sprite_textures[key];
            if std::mem::replace(&mut adding, quad.adds) != quad.adds {
                pass.set_pipeline(if adding { &self.sprite_adding_pipeline } else { &self.sprite_pipeline });
                pass.set_bind_group(0, if adding { &self.sprite_adding_camera_bind } else { &self.sprite_camera_bind }, &[]);
            }
            pass.set_bind_group(1, if adding { &texture.adding } else { &texture.bind }, &[]);
            let first = mark_corners + ring_corners + index as u32 * 6;
            pass.draw(first..first + 6, 0..1);
        }
        if adding {
            pass.set_pipeline(&self.sprite_pipeline);
            pass.set_bind_group(0, &self.sprite_camera_bind, &[]);
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
