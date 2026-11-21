//! Ways on the ground and the camera's turn, as the client measures them, and how a sprite that points a way in
//! the world is turned on the screen (`Prim3DParticle`, `pattern & 0x20`).
//!
//! Where each fact is from:
//!
//! - `CRenderObject::Face` and `Heading` (`Contents/Actors/ActorGeometry.cpp`): the way from one place to
//!   another is `atan2(dx, -dz)` in degrees, brought into 0 up to 360. Here the client's x is east and its z
//!   north, so 0 is to the south and 90 to the east.
//! - `CView::Refresh` (`Contents/Camera/CameraFrame.cpp`): the eye is the point `(0, 0, -distance)` tipped by
//!   `MakeXRotation(pitch)` and turned by `AppendYRotation(yaw)`, added to the place looked at. With the values
//!   `matrix::MakeYRotation` writes (`Utility/Math/AffineMatrix.cpp`: a point's z goes to `(sin, 0, cos)`), the
//!   eye stands `distance * cos(pitch)` times `(sin yaw, cos yaw)` back from that place along x and z. So with a
//!   yaw of 0 the camera looks along +z, north, and with 90 along +x, east.
//! - This client's `Camera::view` (`web/world/camera.rs`) puts its eye by the same formula along east and
//!   north, so its `yaw` is the client's `camera.yaw`; `camera_yaw` gets it back from the view's right.
//! - `Prim3DParticle` (`EffectPrimitiveMotion.cpp`): `roll = render.angle - camera.yaw`, every tick.
//! - `CRenderObject::DrawQuad` (`Contents/Actors/GameObject.cpp`): a corner `(x, y)` from the quad's middle goes
//!   to `(x cos - y sin, y cos + x sin)`, the screen's y pointing down: a roll turns the picture clockwise.

use super::*;

/// `CRenderObject::Heading`: the way from a place to one so far east and north of it, in the client's degrees:
/// from 0 up to 360, 0 to the south and 90 to the east.
pub fn heading(east: f32, north: f32) -> f32 {
    east.atan2(-north).to_degrees().rem_euclid(360.0)
}

/// `CRenderObject::Face`: the angle of one that turns to a place so far east and north of its own; told its
/// own place it keeps the angle it has, which for a primitive just launched is nothing.
pub fn faced(east: f32, north: f32) -> f32 {
    if east == 0.0 && north == 0.0 { 0.0 } else { heading(east, north) }
}

/// The `render.angle` the effects give a sprite that flies to a place so far east and north (`FireBall`,
/// `SpearBmr`): 180 less the angle it faces, brought back over nothing.
pub fn pointing(east: f32, north: f32) -> f32 {
    (180.0 - faced(east, north)).rem_euclid(360.0)
}

/// The camera's yaw, the client's degrees, from the way the view's right points in the world (east, up, north):
/// a camera that looks along `(sin yaw, cos yaw)` on the ground has its right along `(cos yaw, -sin yaw)`.
pub fn camera_yaw(right: [f32; 3]) -> f32 {
    (-right[2]).atan2(right[0]).to_degrees()
}

impl Prim {
    /// Tells it the camera's yaw, the client's degrees, before a tick of its life: what the kinds that look at
    /// the camera as they live make of it (`PrimGhost`, `PrimPulsingColumns` for `PP_HEAL_LOW`). Nothing for a
    /// primitive of any other kind.
    pub fn watched(&mut self, yaw: f32) {
        match self.kind {
            Shape::Ghost => self.ghost_watched(yaw),
            Shape::CastingLow => self.low_walls_watched(yaw),
            _ => {}
        }
    }

    /// How far its sprite is turned on the screen, clockwise, the camera's yaw being `yaw`: its own roll, or for
    /// one that points a way in the world that way less the camera's yaw (`Prim3DParticle`, `pattern & 0x20`).
    pub fn rolled(&self, yaw: f32) -> f32 {
        self.points.map_or(self.roll, |points| points - yaw)
    }
}

/// `Get_Angle`: the whole degrees of the way (x, z) on the ground, such that it is along (cos, sin) of them:
/// the first degree from nothing up whose cosine is not over the way's, turned over for a way of z under
/// nothing; 180 for no way at all.
pub fn angle_of(x: f32, z: f32) -> i32 {
    let length = (x * x + z * z).sqrt();
    let (across, along) = (x / length, z / length);
    (0..=180).find(|degree| across >= cosine(*degree as f32)).map_or(180, |degree| if along < 0.0 { 360 - degree } else { degree })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ball as `FireBall` launches it towards a place so far east and north.
    fn ball(east: f32, north: f32) -> Prim {
        let mut ball = Prim::new(Shape::Particle3d, "");
        ball.points = Some(pointing(east, north));
        ball
    }

    /// The right of this client's camera at a yaw: it looks along (sin, cos) on the ground, and its right is
    /// the upright crossed with that (`Camera::view`).
    fn right_at(yaw: f32) -> [f32; 3] {
        let (sin, cos) = yaw.to_radians().sin_cos();
        // (0, 1, 0) x (sin, 0, cos)
        [cos, 0.0, -sin]
    }

    #[test]
    fn the_way_to_a_place_is_measured_from_the_south_round_by_the_east() {
        // atan2(dx, -dz): south 0, east 90, north 180, west 270.
        assert_eq!(heading(0.0, -3.0), 0.0);
        assert_eq!(heading(3.0, 0.0), 90.0);
        assert_eq!(heading(0.0, 3.0), 180.0);
        assert_eq!(heading(-3.0, 0.0), 270.0);
        // One told its own place keeps its angle of nothing.
        assert_eq!(faced(0.0, 0.0), 0.0);
        assert_eq!(pointing(0.0, 0.0), 180.0);
    }

    #[test]
    fn the_cameras_yaw_is_read_back_from_its_right() {
        for yaw in [0.0f32, 45.0, 90.0, 135.0, -60.0] {
            assert!((camera_yaw(right_at(yaw)) - yaw).abs() < 0.001, "{yaw}");
        }
    }

    /// The client's formula, written out: `Face` gives atan2(dx, -dz), the effect makes the angle 180 less
    /// that, and the roll is that angle less the camera's yaw. The picture of the ball points up the screen
    /// when not turned, and a roll turns it clockwise; so at a yaw of 0 (looking north) a ball flying north
    /// points up, one flying east to the right, and at a yaw of 90 (looking east) the one flying east points up.
    #[test]
    fn a_ball_is_turned_the_way_it_flies_less_the_cameras_yaw() {
        // Flying east, north, west, south: the angle it faces is 90, 180, 270, 0; its own 90, 0, 270, 180.
        let flights = [((5.0, 0.0), 90.0), ((0.0, 5.0), 0.0), ((-5.0, 0.0), 270.0), ((0.0, -5.0), 180.0)];
        for yaw in [0.0f32, 90.0] {
            let seen = camera_yaw(right_at(yaw));
            for ((east, north), angle) in flights {
                let ball = ball(east, north);
                assert_eq!(ball.points, Some(angle));
                assert!((ball.rolled(seen) - (angle - yaw)).abs() < 0.001, "east {east} north {north} yaw {yaw}");
            }
        }
        // Looking north: east is a quarter turn clockwise, to the right; west three quarters, to the left.
        assert!((ball(5.0, 0.0).rolled(0.0) - 90.0).abs() < 0.001);
        assert!((ball(-5.0, 0.0).rolled(0.0) - 270.0).abs() < 0.001);
        // Looking east: the ball flying east points up the screen, the one flying north to the left (-90).
        assert!(ball(5.0, 0.0).rolled(90.0).abs() < 0.001);
        assert!((ball(0.0, 5.0).rolled(90.0) + 90.0).abs() < 0.001);
    }

    #[test]
    fn a_sprite_that_points_no_way_keeps_its_own_roll() {
        let mut spark = Prim::new(Shape::Particle3d, "");
        spark.roll = 33.0;
        assert_eq!(spark.rolled(120.0), 33.0);
    }

    #[test]
    fn the_angle_of_a_way_is_in_whole_degrees_round_from_x_to_z() {
        assert_eq!([angle_of(1.0, 0.0), angle_of(0.0, 1.0), angle_of(-1.0, 0.0), angle_of(0.0, -1.0), angle_of(1.0, -1.0)], [0, 90, 180, 270, 315]);
    }
}
