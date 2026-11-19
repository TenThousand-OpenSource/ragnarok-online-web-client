//! Vectors of three, as far as the game needs them: written here once for the world, the models, the
//! primitives and the effects that fly.

/// A place or a direction: east, up, north, or whatever three figures its user keeps.
pub type Vector = [f32; 3];

pub fn add(a: Vector, b: Vector) -> Vector {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn subtract(a: Vector, b: Vector) -> Vector {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// A vector so many times as long.
pub fn scale(a: Vector, by: f32) -> Vector {
    [a[0] * by, a[1] * by, a[2] * by]
}

pub fn cross(a: Vector, b: Vector) -> Vector {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn dot(a: Vector, b: Vector) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn length(a: Vector) -> f32 {
    dot(a, a).sqrt()
}

/// The vector of length one that points the same way. One of no length stays of no length: it is divided by
/// the smallest number there is, not by nought.
pub fn normalize(a: Vector) -> Vector {
    let length = dot(a, a).sqrt().max(f32::EPSILON);
    [a[0] / length, a[1] / length, a[2] / length]
}

/// The place so far from `origin` along `direction`.
pub fn along(origin: Vector, direction: Vector, distance: f32) -> Vector {
    [origin[0] + direction[0] * distance, origin[1] + direction[1] * distance, origin[2] + direction[2] * distance]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_subtracts_and_scales_figure_by_figure() {
        assert_eq!(add([1.0, 2.0, 3.0], [10.0, 20.0, 30.0]), [11.0, 22.0, 33.0]);
        assert_eq!(subtract([1.0, 2.0, 3.0], [10.0, 20.0, 30.0]), [-9.0, -18.0, -27.0]);
        assert_eq!(scale([1.0, -2.0, 3.0], 2.0), [2.0, -4.0, 6.0]);
        assert_eq!(along([1.0, 1.0, 1.0], [0.0, 1.0, 0.0], 5.0), [1.0, 6.0, 1.0]);
    }

    #[test]
    fn the_cross_of_east_and_up_is_north() {
        assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
        assert_eq!(cross([0.0, 1.0, 0.0], [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]);
        assert_eq!(dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
    }

    #[test]
    fn a_normalized_vector_is_one_long() {
        assert_eq!(length([3.0, 0.0, 4.0]), 5.0);
        assert_eq!(normalize([3.0, 0.0, 4.0]), [0.6, 0.0, 0.8]);
        // One of no length is left so, and is no "not a number".
        assert_eq!(normalize([0.0, 0.0, 0.0]), [0.0, 0.0, 0.0]);
    }
}
