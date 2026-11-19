//! Vectors of three, as far as the world needs them, and the lots the world draws.

use super::*;

pub(super) fn subtract(a: Vector, b: Vector) -> Vector {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn cross(a: Vector, b: Vector) -> Vector {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub(super) fn dot(a: Vector, b: Vector) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn normalize(a: Vector) -> Vector {
    let length = dot(a, a).sqrt().max(f32::EPSILON);
    [a[0] / length, a[1] / length, a[2] / length]
}

/// A whole number from 0 up to, not reaching, `under`, drawn by lot: the client's `rand() % under`.
pub(super) fn lot(under: u32) -> u32 {
    (js_sys::Math::random() * f64::from(under)) as u32
}

pub(super) fn along(origin: Vector, direction: Vector, distance: f32) -> Vector {
    [origin[0] + direction[0] * distance, origin[1] + direction[1] * distance, origin[2] + direction[2] * distance]
}
