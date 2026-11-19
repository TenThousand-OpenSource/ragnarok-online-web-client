//! Which of the four ways across the screen an effect points, by which several effects pick the action of a
//! sprite that is drawn towards the place the effect aims at (`CRagEffect::TexturedClassChange` for its kinds 8
//! and 28, `CRagEffect::CHIMTO`):
//!
//! - `Get_Angle` gives the angle of a way in whole degrees (`angle_of`, in `facing.rs`);
//! - the effects hand it the effect's own place less the place it aims at, along the client's x and z, add the
//!   camera's yaw cut to a whole number, and bring the sum into 0 up to 360; the quarter that falls in says the
//!   action.

use super::angle_of;

/// The quarter an effect points in, the place it aims at being so far east and north of its own and the camera's
/// yaw being `yaw` (the client's degrees): 0 for a sum of 0 up to 90, 1 up to 180, 2 up to 270 and 3 for the rest.
pub fn quarter(yaw: f32, east: f32, north: f32) -> usize {
    let direction = (yaw as i32 + angle_of(-east, -north)).rem_euclid(360);
    (direction / 90) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_quarter_is_of_the_way_back_from_the_aim_and_the_cameras_yaw_together() {
        // Aiming west, the way back from the aim is along +x: angle 0, the first quarter with no yaw.
        assert_eq!(quarter(0.0, -4.0, 0.0), 0);
        // Aiming south the way back is along +z (90), aiming east along -x (180), aiming north along -z (270).
        assert_eq!(quarter(0.0, 0.0, -4.0), 1);
        assert_eq!(quarter(0.0, 4.0, 0.0), 2);
        assert_eq!(quarter(0.0, 0.0, 4.0), 3);
        // The camera's yaw is added, cut to a whole number, and the sum brought back into one round.
        assert_eq!(quarter(90.9, -4.0, 0.0), 1);
        assert_eq!(quarter(-45.0, -4.0, 0.0), 3);
        assert_eq!(quarter(200.0, 4.0, 0.0), 0);
    }
}
