//! Granny models (`.gr2`): the guild flags, the Emperium, the guardians and the treasure boxes of a castle —
//! the monsters and NPCs whose name in the client's table of jobs ends in `.gr2` and which the client draws
//! as models of bones and skin, not as sprites (`C3dGrannyModelRes`, `C3dGrannyGameActor`).
//!
//! The file (`granny/file.rs`: Granny 2.1's format 6) is packed by sections (`oodle.rs`), describes its own
//! contents (`types.rs`), and holds textures packed once more (`bink.rs`), a skeleton, meshes bound to its
//! bones, and one animation of curves (`scene.rs`). `pose.rs` plays the animation: where each bone is at a
//! time, and the skin moved by its bones.

mod bink;
mod coder;
mod file;
mod oodle;
mod pose;
mod scene;
pub mod shown;
mod types;

pub use pose::Pose;
pub use types::Transform;

/// A model as the client takes it from a file.
#[derive(Clone, Debug, PartialEq)]
pub struct Granny {
    pub textures: Vec<Texture>,
    /// The skeleton: a bone's parent comes before it.
    pub bones: Vec<Bone>,
    pub meshes: Vec<Mesh>,
    /// What the model does when it does nothing else (the client's action 0 is the model's own file); a
    /// file of `3dmob_bone` has another action's.
    pub animation: Option<Animation>,
}

/// A texture's pixels: red, green, blue, alpha, rows from the top. One the client would not take has none.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    /// Where it stands by its parent when nothing moves it.
    pub rest: Transform,
    /// From the model's space into the bone's own, as the model was bound: a 4 by 4 matrix, column by column.
    pub from_world: [f32; 16],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    /// Three for a triangle.
    pub indices: Vec<u32>,
    /// Which of the model's textures it shows, if any.
    pub texture: Option<usize>,
    /// The skeleton's bone for each of the bones the corners name.
    pub bones: Vec<usize>,
    /// No corner is weighed by bones: the whole mesh goes with its first bone (Granny's `MeshIsRigid`).
    pub rigid: bool,
}

/// A corner: its place and the way it looks out in the model's space, where on the texture it is, and up to
/// four of the mesh's bones with how much of 255 each moves it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub weights: [u8; 4],
    pub bones: [u8; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    /// Seconds.
    pub duration: f32,
    pub tracks: Vec<Track>,
}

/// How one bone, by its name, moves.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub bone: String,
    pub position: Curve,
    pub orientation: Curve,
    pub scale: Curve,
}

/// A curve of Granny 2.1: a B-spline of a degree, with as many knots (times) as controls of `width` numbers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curve {
    pub degree: usize,
    pub knots: Vec<f32>,
    pub controls: Vec<f32>,
    pub width: usize,
}

impl Granny {
    pub fn read(bytes: &[u8]) -> Result<Self, String> {
        scene::read(&file::Image::read(bytes)?)
    }
}

/// Whether a job's name in the client's table is a Granny model's (`CGameMode::NewActor`: the name has
/// `.gr2` in it).
pub fn is_model(name: &str) -> bool {
    name.to_ascii_lowercase().contains(".gr2")
}

#[cfg(test)]
mod tests;
