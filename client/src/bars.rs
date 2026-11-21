//! The bars laid over the world, as figures: how much of each is filled for the life, the spirit or the time
//! of a cast it shows, when one is red, whose life has a bar at all, and where a bar or a name sits by the
//! feet it belongs to. (The page's side, which finds the feet on the screen and writes the bars:
//! `web/world/overlays/bars.rs`.)

use crate::scenery::views::DISTANCE;

/// How many pixels of a screen 480 high the bar of someone's life sits under the feet (the client's place
/// for a monster's gauge and a player's; the name stands at 20), and the bar of a cast over them (the
/// client's `UIRechargeGage`).
pub const LIFE_BELOW: f32 = 12.0;
pub const CAST_OVER: f32 = 81.0;

/// How far under the feet, on a page so high, something stands that the client puts so many pixels of a
/// screen 480 high under them (over them, if less than nothing): nearer when the camera is.
pub fn below(pixels: f32, height: f32, distance: f32) -> f32 {
    height * pixels / 480.0 * DISTANCE / distance
}

/// How far over the feet, on a page so high, something stands that the client puts so many pixels of a
/// screen 480 high over them: nearer when the camera is.
pub fn over(pixels: f32, height: f32, distance: f32) -> f32 {
    pixels * DISTANCE / distance * height / 480.0
}

/// Where a bar 60 wide begins on a page so wide, for feet at such a share of the way across.
pub fn left(under: f32, width: f32) -> f32 {
    under * width - 30.0
}

/// Whether the life told of someone (what is left, and the most) is shown as a bar: a member of the
/// player's party always, a monster with the option for it; nobody whose most is nothing. A monster's bar
/// stays once its life was told, whole again or not: the client makes the gauge at the first telling and takes
/// it away only when told -1 for either figure (`CGameActor::SendMsg`, `AM_UPDATE_MONSTER_GAGE`,
/// `Source/Contents/Actors/GameObject.cpp:4400`), so one healed back to all its life keeps its bar. (The
/// server tells nothing of a monster nobody has hurt, and -1 with one that comes into sight unhurt.)
pub fn shown((left, most): (i32, i32), member: bool, monster_life: bool) -> bool {
    most > 0 && left >= 0 && (member || monster_life)
}

/// How many of its 58 pixels the bar of a monster's life has filled. (`most` is over nothing: `shown`.)
pub fn monster(left: i32, most: i32) -> f32 {
    (left.max(0) as f32 / most as f32 * 58.0).round()
}

/// How many of their 58 pixels the bars under the player's own character have filled, of life or of
/// spirit; none when the most is nothing.
pub fn own(left: i64, most: i64) -> f32 {
    if most > 0 { (left.clamp(0, most) as f32 / most as f32 * 58.0).round() } else { 0.0 }
}

/// Whether the player's own life is shown as low: under a quarter of the most.
pub fn own_low(left: i64, most: i64) -> bool {
    left * 4 < most
}

/// The bar of life under a member of the player's party (the client's `UIPcGage`): how many pixels of it
/// are filled, which is the whole hundredths of the life of 60, less the rim; and whether it is red, under
/// a quarter. `most` is not nothing (it is divided by).
pub fn member(left: i32, most: i32) -> (i64, bool) {
    let percent = i64::from(left.max(0)) * 100 / i64::from(most);
    ((60 * percent / 100 - 2).clamp(0, 58), percent < 25)
}

/// How many pixels the bar of a cast has filled, so many milliseconds into a cast that lasts so long (the
/// client's `UIRechargeGage`: 60 wide, less the rim).
pub fn cast(passed: f64, lasts: f64) -> f32 {
    ((60.0 * (passed / lasts.max(1.0)).clamp(0.0, 1.0) as f32) - 2.0).max(0.0).floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bar_sits_by_the_feet_nearer_when_the_camera_is() {
        assert_eq!((below(LIFE_BELOW, 480.0, 300.0), below(LIFE_BELOW, 960.0, 150.0), below(20.0, 720.0, 730.0)), (12.0, 48.0, 12.328767));
        assert_eq!((below(-100.0, 480.0, 300.0), below(LIFE_BELOW, 1080.0, 400.0)), (-100.0, 20.25));
        assert_eq!((over(CAST_OVER, 480.0, 300.0), over(CAST_OVER, 960.0, 150.0), over(CAST_OVER, 720.0, 730.0)), (81.0, 324.0, 49.931507));
        assert_eq!((left(0.5, 1280.0), left(0.0, 1280.0), left(0.333, 1000.0)), (610.0, -30.0, 303.0));
    }

    #[test]
    fn whose_life_has_a_bar() {
        // A member of the party whether whole or not; a monster with the option, once hurt.
        assert!(shown((100, 100), true, false) && shown((0, 100), true, false) && shown((99, 100), false, true));
        // A monster healed back to all its life keeps the bar it was given; told -1, it has none.
        assert!(shown((100, 100), false, true) && !shown((50, 100), false, false));
        assert!(!shown((-1, -1), false, true) && !shown((-1, 100), false, true));
        assert!(!shown((0, 0), true, true) && !shown((5, -1), true, true));
    }

    #[test]
    fn how_much_of_a_bar_is_filled() {
        assert_eq!((monster(100, 100), monster(50, 100), monster(1, 1000), monster(-5, 100), monster(0, 100)), (58.0, 29.0, 0.0, 0.0, 0.0));
        // A monster with more than its most is not held to it; a half rounds away from nothing.
        assert_eq!((monster(150, 100), monster(1, 116), monster(3, 116)), (87.0, 1.0, 2.0));
        assert_eq!((own(100, 100), own(50, 100), own(150, 100), own(-5, 100), own(5, 0), own(5, -3)), (58.0, 29.0, 58.0, 0.0, 0.0, 0.0));
        assert!(own_low(24, 100) && !own_low(25, 100) && own_low(0, 1) && !own_low(0, 0) && own_low(-1, -3));
    }

    #[test]
    fn a_members_bar_counts_whole_hundredths() {
        assert_eq!((member(100, 100), member(50, 100), member(25, 100), member(24, 100)), ((58, false), (28, false), (13, false), (12, true)));
        assert_eq!((member(0, 100), member(-9, 100), member(3, 100), member(4, 100), member(5, 100)), ((0, true), (0, true), (0, true), (0, true), (1, true)));
        // More than the most fills no more than the bar; the hundredths are cut, not rounded.
        assert_eq!((member(500, 100), member(999, 1000), member(i32::MAX, 1)), ((58, false), (57, false), (58, false)));
    }

    #[test]
    fn a_casts_bar_fills_over_its_time() {
        assert_eq!((cast(0.0, 1000.0), cast(500.0, 1000.0), cast(1000.0, 1000.0), cast(5000.0, 1000.0)), (0.0, 28.0, 58.0, 58.0));
        assert_eq!((cast(-100.0, 1000.0), cast(40.0, 1000.0), cast(50.0, 1000.0), cast(999.0, 1000.0)), (0.0, 0.0, 1.0, 57.0));
        // A cast of no time is taken as of a millisecond; what is no number fills nothing.
        assert_eq!((cast(0.5, 0.0), cast(1.0, -5.0), cast(f64::NAN, 1000.0), cast(1.0, f64::NAN)), (28.0, 58.0, 0.0, 58.0));
    }
}
