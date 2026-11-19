//! The camera's view of a map: the figures `data\viewpointtable.txt` has for some maps
//! (`CSession::GetMapViewParameters`, used by `CGameMode::InitializeCamera`, `ProcessRightButtonDrag` and
//! `ProcessMouseWheel`), and how the camera eases to what is asked of it (`CameraState::Smooth`).
//!
//! The client measures the camera's tilt downwards from level, as a negative number (-45 looks down at 45
//! degrees); here it is the degrees above the ground, positive.

use super::{records, table_name};

/// What the table says of a map's view (its nine figures, in the table's order: range, scope, range_IN,
/// rotationFrom, rotationTo, rotation_IN, altitudeFrom, altitudeTo, altitude_IN).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapView {
    /// How near the camera may come and how far it may go.
    pub distances: [f32; 2],
    /// How far it is, how it is turned and how it is tilted as the map is entered.
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    /// Between which turns and which tilts it may be moved.
    pub yaws: [f32; 2],
    pub pitches: [f32; 2],
}

impl MapView {
    /// The view the table has for a map, if it has one. The client keeps each figure as a whole number.
    pub fn of(table: &str, map: &str) -> Option<Self> {
        let name = table_name(map);
        let (_, figures) = records(table, 9).into_iter().find(|(known, _)| *known == name)?;
        let figure = |index: usize| (figures[index] as i16) as f32;
        Some(Self {
            distances: [figure(0), figure(0) + figure(1)],
            distance: figure(2),
            yaws: [figure(3), figure(4)],
            yaw: figure(5),
            // The client holds its tilt from `altitudeTo` (the steeper) up to `altitudeFrom`.
            pitches: [-figure(6), -figure(7)],
            pitch: -figure(8),
        })
    }
}

/// How many degrees a notch of the wheel tilts the camera with Shift held (`ProcessMouseWheel`).
pub const WHEEL_TILT: f32 = 5.0;
/// How far a notch of the wheel moves the camera in or out (`ProcessMouseWheel`).
pub const WHEEL_STEP: f32 = 24.3;

/// Between which tilts the wheel with Shift held leaves the camera on a map the table does not know, outdoors
/// or inside (`ProcessMouseWheel`: from 25 over to 20 under the client's 45 outdoors, 10 either way less
/// inside; with Ctrl held too, and the camera no farther than it may go, as flat as 25 or 35).
pub fn wheel_tilts(indoor: bool, ctrl: bool) -> [f32; 2] {
    let range = if indoor { 10.0 } else { 20.0 };
    [if ctrl { 45.0 - range } else { 70.0 - range }, 45.0 + range]
}

/// The share of the way to what is asked that the camera goes in so many of the client's frames: a tenth of
/// what is left, each frame (`CameraState::Smooth`).
pub fn share(frames: f64) -> f32 {
    (1.0 - 0.9f64.powf(frames.clamp(0.0, 600.0))) as f32
}

/// A turn eased towards another by a share of the way, the shorter way round; both in degrees, the answer
/// from 0 up to 360.
pub fn eased_turn(now: f32, asked: f32, share: f32) -> f32 {
    let left = (asked - now + 180.0).rem_euclid(360.0) - 180.0;
    (now + left * share).rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_of_the_table_has_its_view() {
        let table = "//ex_map02.rsw#\t250#\t250#\t500#\t\t0#\t\t90#\t\t45#\t\t-40#\t\t-65#\t\t-65#\n\neclage.rsw#     250#    250#    500#             10#             80#            45#             -25#            -60#            -35#\n";
        let view = MapView::of(table, "eclage").unwrap();
        assert_eq!(view, MapView { distances: [250.0, 500.0], distance: 500.0, yaws: [10.0, 80.0], yaw: 45.0, pitches: [25.0, 60.0], pitch: 35.0 });
        assert_eq!(MapView::of(table, "prontera"), None);
    }

    #[test]
    fn the_camera_goes_a_tenth_of_the_way_a_frame() {
        assert!((share(1.0) - 0.1).abs() < 1e-6);
        assert!((share(2.0) - 0.19).abs() < 1e-6);
        assert_eq!(share(0.0), 0.0);
        // The shorter way round: from 350 to 10 is 20 on, through nothing.
        assert!((eased_turn(350.0, 10.0, 0.5) - 0.0).abs() < 1e-4);
        assert!((eased_turn(10.0, 350.0, 0.25) - 5.0).abs() < 1e-4);
        assert_eq!(wheel_tilts(false, false), [50.0, 65.0]);
        assert_eq!(wheel_tilts(true, true), [35.0, 55.0]);
    }
}
