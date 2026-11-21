//! The effects' quads of one frame, as the renderer takes them.

use super::*;

/// What the effects of a frame are drawn with: the corners of their quads (a place, where of the picture, a
/// colour: six corners to a quad), and for each quad its picture and whether its light is added to what is behind.
/// `under` names the quads, by their number among `pictures`, that are drawn ahead of everyone standing, who
/// then cover them (`Quads::picture_under`); every other is drawn after them.
#[derive(Default)]
pub(super) struct Quads {
    pub(super) corners: Vec<f32>,
    pub(super) pictures: Vec<(Rc<[wgpu::BindGroup; 2]>, bool)>,
    pub(super) under: BTreeSet<usize>,
}

impl Quads {
    /// One corner of a quad: at a place in the world, drawn `nearer` to the eye than it is (1 is where it is), so
    /// that the ground and those standing on it do not cut through it.
    pub(super) fn corner(&mut self, view: &View, nearer: f32, at: Vector, uv: [f32; 2], colour: [f32; 4]) {
        self.corners.extend(along(view.eye, subtract(at, view.eye), nearer));
        self.corners.extend(uv);
        self.corners.extend(colour);
    }

    /// The picture of the quad whose six corners were just given.
    pub(super) fn picture(&mut self, binds: &Rc<[wgpu::BindGroup; 2]>, adds: bool) {
        self.pictures.push((binds.clone(), adds));
    }

    /// The same, of a quad drawn ahead of the shadows and the bodies of those standing: the client's renderer
    /// draws such a particle right after the ground and the models (`CRendererDX9::DrawQueues`, its
    /// `queue_0210`), and everyone after it.
    pub(super) fn picture_under(&mut self, binds: &Rc<[wgpu::BindGroup; 2]>, adds: bool) {
        self.under.insert(self.pictures.len());
        self.picture(binds, adds);
    }
}

/// How much nearer to the eye than `at` something is drawn so as to stand `by` in front of it.
pub(super) fn nearer(view: &View, at: Vector, by: f32) -> f32 {
    1.0 - by / dot(subtract(at, view.eye), view.forward).max(by * 1.6)
}
