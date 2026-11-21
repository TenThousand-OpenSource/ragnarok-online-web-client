//! The drawing of the map itself: its ground, its models and its water, and what their programs are told every
//! frame (`shaders.rs`, `frame!`): the eye, the fog, the light the night leaves, the water's waves.

use super::*;

/// How many bytes the programs are told: the camera's matrix and six vectors of four after it.
pub(super) const FRAME_BYTES: u64 = (16 + 6 * 4) * 4;

impl Gpu {
    /// Tells the programs what this frame is drawn with: the camera, and for the map's own the fog as the
    /// scene has it now, the night's light and the water's waves at this moment.
    pub(super) fn frame_figures(&self, scene: &Scene, view: &View) {
        let surroundings = &scene.scenery;
        let frames = now() / scenery::CLIENT_FRAME_MS;
        let mut figures = Vec::with_capacity(40);
        figures.extend(view.matrix);
        figures.extend([view.eye[0], view.eye[1], view.eye[2], 1.0]);
        // The fog goes out with a camera beyond the client's near limit (`Fog::seen_from`).
        let fog = surroundings.fog.filter(|_| surroundings.fogged).map(|fog| fog.seen_from(scene.camera.distance));
        figures.extend(fog.map_or([0.0; 4], |fog| [fog.start, fog.end, 1.0, 0.0]));
        figures.extend(fog.map_or([0.0; 4], |fog| [fog.colour[0], fog.colour[1], fog.colour[2], 1.0]));
        figures.extend([surroundings.light[0], surroundings.light[1], surroundings.light[2], 1.0]);
        match &scene.water {
            Some(water) => {
                // The waves' turn goes on by their speed every frame of the client's, kept within half a round
                // either way (`GroundGeometry::UpdateWater`).
                let (height, speed, pitch) = water.wave;
                let turn = (f64::from(speed) * frames.floor()).rem_euclid(360.0);
                figures.extend([if turn > 180.0 { turn - 360.0 } else { turn } as f32, pitch as f32, height, 0.0]);
                figures.extend(water.colour);
            }
            None => figures.extend([0.0; 8]),
        }
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&figures));
    }

    /// The colour behind the map: its sky.
    pub(super) fn sky(scene: &Scene) -> wgpu::Color {
        let [red, green, blue] = scene.scenery.sky.map(f64::from);
        wgpu::Color { r: red, g: green, b: blue, a: 1.0 }
    }

    /// The map: its ground, its models (those mixed in by their alpha after the others) and its water.
    pub(super) fn draw_map(&self, pass: &mut wgpu::RenderPass, scene: &Scene) {
        pass.set_pipeline(&self.ground_pipeline);
        pass.set_bind_group(0, &self.camera_bind, &[]);
        pass.set_vertex_buffer(0, scene.ground.vertices.slice(..));
        for (indices, count, bind) in &scene.ground.batches {
            pass.set_bind_group(1, bind, &[]);
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..*count, 0, 0..1);
        }
        for blends in [false, true] {
            let which = usize::from(blends);
            pass.set_pipeline(&self.model_pipelines[which]);
            pass.set_bind_group(0, &self.model_camera_binds[which], &[]);
            for batch in scene.models.batches().filter(|batch| batch.blends == blends) {
                pass.set_bind_group(1, &batch.bind, &[]);
                pass.set_vertex_buffer(0, batch.vertices.slice(..));
                pass.draw(0..batch.count, 0..1);
            }
        }
        if let Some(water) = &scene.water {
            // A picture of the water every so many frames of the client's.
            let picture = (now() / scenery::CLIENT_FRAME_MS / f64::from(water.frame_ticks.max(1))) as usize % water.frames.len();
            pass.set_pipeline(&self.water_pipeline);
            pass.set_bind_group(0, &self.water_camera_bind, &[]);
            pass.set_bind_group(1, &water.frames[picture], &[]);
            pass.set_vertex_buffer(0, water.vertices.slice(..));
            pass.draw(0..water.count, 0..1);
        }
    }
}
