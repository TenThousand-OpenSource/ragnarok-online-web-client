//! Vectors of three, as far as the world needs them (`crate::vector`), and the lots the world draws.

pub(super) use crate::vector::{along, cross, dot, normalize, subtract};

/// A whole number from 0 up to, not reaching, `under`, drawn by lot: the client's `rand() % under`.
pub(super) fn lot(under: u32) -> u32 {
    (js_sys::Math::random() * f64::from(under)) as u32
}
