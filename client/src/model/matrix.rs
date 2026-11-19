//! The 4 by 4 matrices the models are put together and stood on the map with.

use super::*;

/// A 4 by 4 matrix, column by column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix([f32; 16]);

impl Matrix {
    pub const IDENTITY: Matrix = Matrix([1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

    pub fn translation(by: Vector) -> Self {
        let mut matrix = Self::IDENTITY;
        matrix.0[12..15].copy_from_slice(&by);
        matrix
    }

    pub fn scale(by: Vector) -> Self {
        let mut matrix = Self::IDENTITY;
        (matrix.0[0], matrix.0[5], matrix.0[10]) = (by[0], by[1], by[2]);
        matrix
    }

    /// A turn by `angle` radians around an axis (of any length; none for an axis of no length).
    pub fn rotation(axis: Vector, angle: f32) -> Self {
        let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if length < 1e-6 {
            return Self::IDENTITY;
        }
        let [x, y, z] = axis.map(|part| part / length);
        let (sin, cos) = angle.sin_cos();
        let rest = 1.0 - cos;
        Matrix([
            cos + x * x * rest,
            y * x * rest + z * sin,
            z * x * rest - y * sin,
            0.0,
            x * y * rest - z * sin,
            cos + y * y * rest,
            z * y * rest + x * sin,
            0.0,
            x * z * rest + y * sin,
            y * z * rest - x * sin,
            cos + z * z * rest,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// The turn a quaternion (x, y, z, w) stands for.
    pub fn quaternion([x, y, z, w]: [f32; 4]) -> Self {
        Matrix([
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + z * w),
            2.0 * (x * z - y * w),
            0.0,
            2.0 * (x * y - z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + x * w),
            0.0,
            2.0 * (x * z + y * w),
            2.0 * (y * z - x * w),
            1.0 - 2.0 * (x * x + y * y),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    /// A 3 by 3 matrix, column by column, as the turn-and-stretch part.
    pub(super) fn from_3x3(parts: [f32; 9]) -> Self {
        let mut matrix = Self::IDENTITY;
        for column in 0..3 {
            matrix.0[column * 4..column * 4 + 3].copy_from_slice(&parts[column * 3..column * 3 + 3]);
        }
        matrix
    }

    /// `self` after `first`: a point goes through `first`, then through `self`.
    pub fn after(&self, first: &Matrix) -> Matrix {
        let mut product = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                product[column * 4 + row] = (0..4).map(|index| self.0[index * 4 + row] * first.0[column * 4 + index]).sum();
            }
        }
        Matrix(product)
    }

    pub fn point(&self, [x, y, z]: Vector) -> Vector {
        let matrix = &self.0;
        [
            matrix[0] * x + matrix[4] * y + matrix[8] * z + matrix[12],
            matrix[1] * x + matrix[5] * y + matrix[9] * z + matrix[13],
            matrix[2] * x + matrix[6] * y + matrix[10] * z + matrix[14],
        ]
    }

    /// A direction through the matrix: turned and stretched, not moved.
    pub fn direction(&self, [x, y, z]: Vector) -> Vector {
        let matrix = &self.0;
        [matrix[0] * x + matrix[4] * y + matrix[8] * z, matrix[1] * x + matrix[5] * y + matrix[9] * z, matrix[2] * x + matrix[6] * y + matrix[10] * z]
    }
}
