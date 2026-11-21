//! A Granny model moved: where an animation's curves are at a time, the bones stood one on another, and the
//! skin carried by them.
//!
//! The DLL was not read for this; it follows what Granny's manual says of its curves and poses, and the
//! usual B-spline (de Boor): a curve's knot is the time by which its control has come, and a stretch
//! between two knots is shaped by the `degree` controls before it. The client's files have curves of
//! degree 0 (one value) and 2. Where the DLL may differ: the ends of a curve are held here, not wrapped
//! round for an animation that loops.

use super::*;
use crate::model::Matrix;

/// Where every bone has carried the model's space to: for each, the matrix that takes a place of the model
/// as it was bound to where the bone has it now (Granny's "composite" of a world pose).
#[derive(Clone, Debug, PartialEq)]
pub struct Pose {
    pub bones: Vec<Matrix>,
}

impl Curve {
    /// The curve's value at a time; none for a curve of nothing (which leaves what it moves as it is).
    pub fn at(&self, time: f32) -> Option<Vec<f32>> {
        let count = self.knots.len();
        let control = |index: isize| {
            let index = index.clamp(0, count as isize - 1) as usize;
            &self.controls[index * self.width..(index + 1) * self.width]
        };
        match count {
            0 => return None,
            1 => return Some(control(0).to_vec()),
            _ => {}
        }
        // The first knot later than the time: the stretch that ends there holds the time.
        let end = self.knots.iter().position(|&knot| knot > time).unwrap_or(count - 1) as isize;
        let degree = self.degree as isize;
        let knot = |index: isize| self.knots[(end - degree + index).clamp(0, count as isize - 1) as usize];
        let mut points: Vec<Vec<f32>> = (0..=degree).map(|index| control(end - degree + index).to_vec()).collect();
        for round in 1..=degree {
            for index in (round..=degree).rev() {
                let (from, to) = (knot(index - 1), knot(index + degree - round));
                let along = if to > from { ((time - from) / (to - from)).clamp(0.0, 1.0) } else { 0.0 };
                let before = points[index as usize - 1].clone();
                for (value, before) in points[index as usize].iter_mut().zip(before) {
                    *value = before + (*value - before) * along;
                }
            }
        }
        points.pop()
    }
}

impl Track {
    /// The bone's place by its parent at a time.
    fn at(&self, time: f32) -> Transform {
        let mut transform = Transform::IDENTITY;
        if let Some(position) = self.position.at(time) {
            transform.position.copy_from_slice(&position);
        }
        if let Some(orientation) = self.orientation.at(time) {
            // A blend of quaternions is made one long again.
            let length = orientation.iter().map(|part| part * part).sum::<f32>().sqrt().max(f32::EPSILON);
            transform.orientation = [0, 1, 2, 3].map(|part| orientation[part] / length);
        }
        if let Some(scale) = self.scale.at(time) {
            transform.scale = [0, 1, 2].map(|row| [scale[row * 3], scale[row * 3 + 1], scale[row * 3 + 2]]);
        }
        transform
    }
}

impl Transform {
    /// As a matrix: stretched, turned, then moved (Granny's `BuildCompositeTransform4x4`).
    pub fn matrix(&self) -> Matrix {
        let scale = &self.scale;
        let stretch = Matrix::from_3x3([scale[0][0], scale[1][0], scale[2][0], scale[0][1], scale[1][1], scale[2][1], scale[0][2], scale[1][2], scale[2][2]]);
        Matrix::translation(self.position).after(&Matrix::quaternion(self.orientation)).after(&stretch)
    }
}

impl Animation {
    /// The time of the animation `seconds` after it began: over and over, or once and held at its end.
    pub fn time(&self, seconds: f32, looped: bool) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else if looped {
            seconds.rem_euclid(self.duration)
        } else {
            seconds.clamp(0.0, self.duration)
        }
    }
}

impl Granny {
    /// The model's bones at a time of an animation; a bone the animation has no track for, and every bone
    /// without an animation, stands as it rests.
    pub fn pose(&self, animation: Option<(&Animation, f32)>) -> Pose {
        let mut world: Vec<Matrix> = Vec::with_capacity(self.bones.len());
        for bone in &self.bones {
            let track = animation.and_then(|(animation, time)| Some((animation.tracks.iter().find(|track| track.bone == bone.name)?, time)));
            let local = track.map_or(bone.rest, |(track, time)| track.at(time)).matrix();
            world.push(match bone.parent.and_then(|parent| world.get(parent)) {
                Some(parent) => parent.after(&local),
                None => local,
            });
        }
        Pose { bones: world.iter().zip(&self.bones).map(|(world, bone)| world.after(&Matrix::columns(bone.from_world))).collect() }
    }

    /// A mesh's corners as a pose has them: each its place and the way it looks out, in the model's space.
    ///
    /// A rigid mesh goes whole with its first bone and keeps its normals as they were bound (the client's
    /// `GetMeshRenderData` hands out the bound corners and the bone's matrix, and only the places go through
    /// it); a skinned one has each corner, place and normal, carried by its bones by their weights
    /// (Granny's `DeformVertices`).
    pub fn skin(&self, mesh: &Mesh, pose: &Pose) -> Vec<([f32; 3], [f32; 3])> {
        let bone = |index: u8| mesh.bones.get(index as usize).and_then(|&bone| pose.bones.get(bone)).copied().unwrap_or(Matrix::IDENTITY);
        if mesh.rigid {
            let whole = bone(0);
            return mesh.vertices.iter().map(|vertex| (whole.point(vertex.position), vertex.normal)).collect();
        }
        let carried = |vertex: &Vertex| {
            let (mut position, mut normal) = ([0.0; 3], [0.0; 3]);
            for (&index, &weight) in vertex.bones.iter().zip(&vertex.weights).filter(|(_, weight)| **weight != 0) {
                let (matrix, weight) = (bone(index), weight as f32 / 255.0);
                position = crate::vector::add(position, crate::vector::scale(matrix.point(vertex.position), weight));
                normal = crate::vector::add(normal, crate::vector::scale(matrix.direction(vertex.normal), weight));
            }
            (position, normal)
        };
        mesh.vertices.iter().map(carried).collect()
    }
}
