//! How a primitive is turned in the world and where its middle is: the client's `transform`, made of its
//! latitude and its longitude (`MakeXRotation`, `AppendYRotation`), and its sines and cosines of whole degrees.

use super::*;

impl Prim {
    /// A point of the primitive's own space turned as the primitive is, in the world's directions (east, up,
    /// north). The client tips it about its x by the latitude and then turns it about its y by the longitude
    /// (`MakeXRotation`, `AppendYRotation`, on rows), or the other way round for one turned first; its y points
    /// down, the world's up.
    pub(super) fn turned(&self, point: [f32; 3]) -> [f32; 3] {
        if self.yaw_first { turn_yaw_first(self.latitude, self.longitude, point) } else { turn(self.latitude, self.longitude, point) }
    }

    /// Where its middle is in the world, around `place`: where it was put (the client's y points down) and as
    /// far as it has gone since.
    pub(super) fn middle(&self, place: [f32; 3]) -> [f32; 3] {
        [place[0] + self.put.0 + self.moved[0] + self.aside[0], place[1] - self.put.1 + self.moved[1] + self.aside[1], place[2] + self.deep + self.moved[2] + self.aside[2]]
    }
}

/// A point tipped about the client's x by a latitude and then turned about its y by a longitude, in the world's
/// directions (east, up, north).
pub(super) fn turn(latitude: f32, longitude: f32, [x, y, z]: [f32; 3]) -> [f32; 3] {
    let (tip_sin, tip_cos) = (sine(latitude), cosine(latitude));
    let (y, z) = (y * tip_cos - z * tip_sin, y * tip_sin + z * tip_cos);
    let (turn_sin, turn_cos) = (sine(longitude), cosine(longitude));
    [x * turn_cos + z * turn_sin, -y, -x * turn_sin + z * turn_cos]
}

/// A point turned about the client's y by a longitude and then tipped about its x by a latitude
/// (`MakeYRotation`, `AppendXRotation`), in the world's directions (east, up, north).
fn turn_yaw_first(latitude: f32, longitude: f32, [x, y, z]: [f32; 3]) -> [f32; 3] {
    let (turn_sin, turn_cos) = (sine(longitude), cosine(longitude));
    let (x, z) = (x * turn_cos + z * turn_sin, -x * turn_sin + z * turn_cos);
    let (tip_sin, tip_cos) = (sine(latitude), cosine(latitude));
    [x, -(y * tip_cos - z * tip_sin), y * tip_sin + z * tip_cos]
}

/// The client's sines and cosines are of whole degrees.
pub fn sine(degrees: f32) -> f32 {
    degrees.trunc().to_radians().sin()
}

pub fn cosine(degrees: f32) -> f32 {
    degrees.trunc().to_radians().cos()
}
