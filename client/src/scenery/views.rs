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

/// Something of the camera eased towards what is asked by a share of the way: its tilt, its distance, and
/// each figure of the place it looks at (its turn goes the shorter way round: `eased_turn`).
pub fn eased(now: f32, asked: f32, share: f32) -> f32 {
    now + (asked - now) * share
}

/// The place the camera looks at, moved towards the character's feet by a share of the way.
pub fn followed(aim: [f32; 3], feet: [f32; 3], share: f32) -> [f32; 3] {
    [0, 1, 2].map(|axis| eased(aim[axis], feet[axis], share))
}

/// How far away the camera is, and how it is tilted, when it was never moved (degrees above the ground).
pub const DISTANCE: f32 = 300.0;
pub const PITCH: f32 = 45.0;
/// Between which distances and which tilts the camera is kept at all.
pub const DISTANCE_RANGE: [f32; 2] = [150.0, 730.0];
pub const PITCH_RANGE: [f32; 2] = [20.0, 70.0];
/// How far the camera goes outdoors without the client's Zoom Out option (with it: `DISTANCE_RANGE`'s end).
pub const DISTANCE_NEAR_MOST: f32 = 400.0;

/// How far the camera may tilt and how far away it may be on a map the table does not know, outdoors or
/// inside (the original's limits: 20 degrees either way of 45 outdoors and 10 inside, and inside no farther
/// than where it starts).
pub fn limits(indoor: bool, zoom_out: bool) -> ([f32; 2], [f32; 2]) {
    // Outdoors as far as the client's Zoom Out option lets it: 730 with it, 400 without.
    let far = if zoom_out { DISTANCE_RANGE[1] } else { DISTANCE_NEAR_MOST };
    if indoor { ([35.0, 55.0], [DISTANCE_RANGE[0], DISTANCE]) } else { (PITCH_RANGE, [DISTANCE_RANGE[0], far]) }
}

/// The camera as it was written down to be kept ("turn,tilt,distance"), within what a camera may ever be:
/// its turn, tilt and distance. None when what is kept is not three numbers (what is no number among them
/// is passed over).
pub fn kept(text: &str) -> Option<[f32; 3]> {
    let numbers: Vec<f32> = text
        .split(',')
        .filter_map(|number| number.parse().ok())
        .filter(|number: &f32| number.is_finite())
        .collect();
    match numbers[..] {
        [yaw, pitch, distance] => Some([
            yaw.rem_euclid(360.0),
            pitch.clamp(PITCH_RANGE[0], PITCH_RANGE[1]),
            distance.clamp(DISTANCE_RANGE[0], DISTANCE_RANGE[1]),
        ]),
        _ => None,
    }
}

/// A tilt asked of the camera, held between the two a map allows (given either way round).
pub fn held_tilt(asked: f32, pitches: [f32; 2]) -> f32 {
    asked.clamp(pitches[0].min(pitches[1]), pitches[1].max(pitches[0]))
}

/// A distance asked of the camera, held between the two a map allows (the farther no nearer than the
/// nearer).
pub fn held_distance(asked: f32, distances: [f32; 2]) -> f32 {
    asked.clamp(distances[0], distances[1].max(distances[0]))
}

/// A turn asked of the camera, held between the two marks a map of the table has; one it lets go all the
/// way round, from -360 to 360, turns as a map without marks does: any way, counted from 0 up to 360.
pub fn held_turn(asked: f32, yaws: Option<[f32; 2]>) -> f32 {
    match yaws {
        Some([from, to]) if to - from < 720.0 => asked.clamp(from, to),
        _ => asked.rem_euclid(360.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_of_the_table_has_its_view() {
        let table = "//ex_map02.rsw#\t250#\t250#\t500#\t\t0#\t\t90#\t\t45#\t\t-40#\t\t-65#\t\t-65#\n\neclage.rsw#     \
            250#    250#    500#             10#             80#            45#             -25#            \
            -60#            -35#\n";
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

    #[test]
    fn the_camera_eases_by_a_share_of_what_is_left() {
        // The figures as they come out now, to the last bit: the sum is not rounded, nor put to its end.
        assert_eq!((eased(45.0, 55.0, 0.1), eased(300.0, 150.0, 0.271), eased(20.0, 20.0, 0.3)), (46.0, 259.35, 20.0));
        assert_eq!((eased(70.0, 20.0, 0.051316738), eased(1.0, 2.0, 0.0), eased(1.0, 2.0, 1.0)), (67.434166, 1.0, 2.0));
        // Not the shorter way round, as a turn is: plainly from one to the other.
        assert_eq!((eased(0.1, 0.3, 1.0), eased(359.0, 1.0, 0.5)), (0.3, 180.0));
        assert_eq!(followed([10.0, -2.5, 300.0], [12.0, 0.0, 290.0], 0.1), [10.2, -2.25, 299.0]);
        // A whole share does not always land on the place itself.
        assert_eq!(followed([0.1, 0.2, 0.3], [0.3, 0.2, 0.1], 1.0), [0.3, 0.2, 0.099999994]);
    }

    #[test]
    fn the_limits_of_a_map_the_table_does_not_know() {
        assert_eq!(limits(false, false), ([20.0, 70.0], [150.0, 400.0]));
        assert_eq!(limits(false, true), ([20.0, 70.0], [150.0, 730.0]));
        assert_eq!((limits(true, false), limits(true, true)), (([35.0, 55.0], [150.0, 300.0]), ([35.0, 55.0], [150.0, 300.0])));
    }

    #[test]
    fn a_kept_camera_is_three_numbers_within_what_a_camera_may_be() {
        assert_eq!(kept("90,45,300"), Some([90.0, 45.0, 300.0]));
        assert_eq!(kept("-90,5,9000"), Some([270.0, 20.0, 730.0]));
        assert_eq!(kept("725.5,99,1"), Some([5.5, 70.0, 150.0]));
        assert_eq!((kept(""), kept("1,2"), kept("1,2,3,4"), kept("1,inf,3"), kept("1,NaN,3"), kept("1, 2,3")), (None, None, None, None, None, None));
        // What is no number is passed over, wherever it stands; a turn a hair under nothing comes out as 360.
        assert_eq!((kept("x,10,45,300"), kept("10,,45,300,")), (Some([10.0, 45.0, 300.0]), Some([10.0, 45.0, 300.0])));
        assert_eq!(kept("-0.00000001,45,300"), Some([360.0, 45.0, 300.0]));
    }

    #[test]
    fn what_is_asked_of_the_camera_is_held_within_the_maps_limits() {
        assert_eq!((held_tilt(10.0, [20.0, 70.0]), held_tilt(80.0, [20.0, 70.0]), held_tilt(45.0, [20.0, 70.0])), (20.0, 70.0, 45.0));
        // The table's two tilts either way round.
        assert_eq!((held_tilt(10.0, [60.0, 25.0]), held_tilt(80.0, [60.0, 25.0])), (25.0, 60.0));
        assert_eq!((held_distance(100.0, [150.0, 400.0]), held_distance(500.0, [150.0, 400.0])), (150.0, 400.0));
        // A farthest nearer than the nearest: the nearest alone.
        assert_eq!((held_distance(100.0, [250.0, 200.0]), held_distance(500.0, [250.0, 200.0])), (250.0, 250.0));
        assert_eq!((held_turn(100.0, Some([10.0, 80.0])), held_turn(-5.0, Some([10.0, 80.0]))), (80.0, 10.0));
        assert_eq!((held_turn(400.0, Some([-360.0, 360.0])), held_turn(-90.0, None), held_turn(725.0, None)), (40.0, 270.0, 5.0));
        // Marks a little under all the way round hold the turn as it is counted, under nothing too.
        assert_eq!((held_turn(-200.0, Some([-360.0, 359.0])), held_turn(-1e-8, None)), (-200.0, 360.0));
    }
}
