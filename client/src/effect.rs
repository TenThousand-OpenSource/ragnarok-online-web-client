//! The client's effect files (`.str`, "STRM"): layers of textured quads moved, turned, tinted and swapped from key
//! to key. The format and the way a layer is brought from one frame to the next are the client's own
//! (`CEZ2STRParticle` in the decompiled source): a key of kind 0 sets a layer's whole state, a key of another kind
//! is added to it every frame while that key is the current one.
//!
//! `strm.rs` reads the file, `transform.rs` is a layer at a frame; `played.rs` lists the effects played
//! from such files, `carried.rs` is how a file is carried through the air and `flights.rs` its ways of flying.

pub mod carried;
mod flights;
pub mod played;
mod strm;
mod transform;

pub use strm::*;
pub use transform::*;

#[cfg(test)]
mod tests;
