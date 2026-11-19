//! What stands on a map's ground: RSW (the map's water, its sunlight, where each model stands, and the effects
//! and sounds it places: `model/rsw.rs`) and RSM (the models: buildings, trees, lamp posts, with the turning of
//! their parts: `model/rsm.rs`), and the triangles they come to in the world, lit as the client lights them.
//!
//! Format layouts and the way a model's parts are put together follow korangar (`ragnarok-formats`, and its
//! `loaders/model`; MIT).
//!
//! The files' own space has y pointing down and positions measured from the map's middle. Everything is worked
//! out there and turned into the world's space (y up, measured from the map's south-west corner) at the end.

type Vector = [f32; 3];

mod matrix;
mod rsm;
mod rsw;

pub use matrix::Matrix;
pub use rsm::{Rsm, Triangle, frame_at};
pub use rsw::{Light, MapEffect, MapSound, Placement, Rsw, Water};

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    /// The file's version, as major times 256 plus minor.
    version: u16,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let slice = self.bytes.get(self.at..self.at.checked_add(count).ok_or("a size too large")?).ok_or("the file ends too early")?;
        self.at += count;
        Ok(slice)
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn vector(&mut self) -> Result<Vector, String> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    /// A count that the rest of the file could hold.
    fn count(&mut self, each: usize) -> Result<usize, String> {
        let count = usize::try_from(self.i32()?).map_err(|_| "a negative count")?;
        if count.saturating_mul(each) > self.bytes.len() - self.at {
            return Err("a count larger than the file".to_owned());
        }
        Ok(count)
    }

    /// A zero-terminated name in a field of `size` bytes, as its bytes.
    fn name(&mut self, size: usize) -> Result<&'a [u8], String> {
        let field = self.take(size)?;
        Ok(&field[..field.iter().position(|&byte| byte == 0).unwrap_or(size)])
    }
}

/// One corner of a model's triangle in the world: its place, texture coordinates, the light on it, and the way
/// its face looks out of the model (nothing for a face that shows from both sides: the client's `twoSided`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vector,
    pub uv: [f32; 2],
    pub colour: Vector,
    pub outward: Vector,
}

/// A model's triangles (`Rsm::triangles`, or `Rsm::at` a frame of its animation) as it stands at a placement, in
/// the world's space, lit by the map's sun as the model's kind of shading says (`shade`). `map_size` is the
/// ground's size in world units. Gives each triangle's three corners with the model's texture index.
///
/// The client's `C3dActor::UpdateMatrix` and `C3dNode::Prepare`: the normals go through the same matrices as
/// the corners and are made one long again.
pub fn place(triangles: &[Triangle], shade: i32, placement: &Placement, map_size: (f32, f32), light: &Light) -> Vec<(usize, [Vertex; 3])> {
    let turn = |axis: Vector, degrees: f32| Matrix::rotation(axis, degrees.to_radians());
    let to_map = Matrix::translation([placement.position[0] + map_size.0 / 2.0, placement.position[1], placement.position[2] + map_size.1 / 2.0])
        .after(&turn([0.0, 0.0, 1.0], placement.rotation[2]))
        .after(&turn([1.0, 0.0, 0.0], placement.rotation[0]))
        .after(&turn([0.0, 1.0, 0.0], placement.rotation[1]))
        .after(&Matrix::scale(placement.scale));
    // The file's space has y down; the world's has it up.
    let up = |[x, y, z]: Vector| [x, -y, z];
    triangles
        .iter()
        .map(|triangle| {
            let positions = triangle.positions.map(|position| up(to_map.point(position)));
            let outward = if triangle.two_sided { [0.0; 3] } else { up(unit(to_map.direction(triangle.outward))) };
            // A model shaded flat is lit by its faces, one shaded smooth by the corners of each.
            let normal = |corner: usize| up(unit(to_map.direction(if shade == 2 { triangle.normals[corner] } else { triangle.outward })));
            let corners = [0, 1, 2].map(|corner| Vertex { position: positions[corner], uv: triangle.uvs[corner], colour: lit(shade, normal(corner), light), outward });
            (triangle.texture, corners)
        })
        .collect()
}

/// A vector made one long; one of no length stays so.
pub(crate) fn unit(vector: Vector) -> Vector {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length < 1e-9 { [0.0; 3] } else { vector.map(|part| part / length) }
}

/// The light on a corner whose face looks out along `normal` (the client's `Light` in `C3dNode::Prepare`): the
/// map's ambient light and its sunlight by how squarely the face meets the sun, nothing of the sun from behind;
/// each no more than 1. A model of shade 0 is not lit at all: it shows its texture as it is.
///
/// (The client halves this for a corner whose texture vertex carries a number over nothing: the models looked
/// at carry nothing there, and only those of version 1.1 are given one; it is not halved here. The client
/// then adds to each corner the colour of the ground's baked light under it, `C3dNode::SampleGroundColors`:
/// not done here.)
fn lit(shade: i32, normal: Vector, light: &Light) -> Vector {
    if shade == 0 {
        return [1.0; 3];
    }
    let facing = (normal[0] * light.direction[0] + normal[1] * light.direction[1] + normal[2] * light.direction[2]).max(0.0);
    [0, 1, 2].map(|channel| (light.ambient[channel] + light.diffuse[channel] * facing).min(1.0))
}

#[cfg(test)]
mod tests;
