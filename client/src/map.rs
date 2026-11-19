//! A map's ground: GAT (where one can walk, and how high the ground is there) and GND (the ground's shape,
//! textures and baked light), the mesh drawn from them, and the paths walked on them.
//!
//! Format layouts follow korangar's `ragnarok-formats` (MIT). The path search follows rAthena's `path.cpp`, as the
//! server decides where a character really goes.
//!
//! World space: x grows east and z north, 5 units per cell, y up (the files keep heights pointing down).
//!
//! `gat.rs` reads the GAT, `paths.rs` finds the ways over it, `gnd.rs` reads the GND and makes its mesh.

mod gat;
mod gnd;
mod paths;

pub use gat::*;
pub use gnd::*;
pub use paths::*;

/// World units per GAT cell.
pub const CELL: f32 = 5.0;

#[cfg(test)]
mod tests;
