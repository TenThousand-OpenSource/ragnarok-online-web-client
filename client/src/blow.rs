//! When a blow lands: the moment into the striker's swing at which the one struck flinches, stops and the
//! number rises; and how the digits of that number stand. The hits a blow is made of are in `landing.rs`, what
//! each shows and sounds like in `signs.rs`, how the numbers fly in `flight.rs`.
//!
//! From the client's `CGameActor::ProcessHitMessage` and `CGameActor::GetAttackMotion`: the blow lands as the
//! swing reaches the frame that carries the sound event named `atk` (without one, the frame before the last),
//! each frame taking 24 milliseconds at full speed. The speed is what the server sends with the blow (432 is
//! full speed; the client takes no more than that).

pub mod flight;
pub mod landing;
pub mod signs;

/// The speed of a swing shown as the sprite has it (the client's `g_maxMoveSpeed`).
const FULL_SPEED: f64 = 432.0;

/// The milliseconds a frame of a swing takes at full speed.
const FRAME_MS: f64 = 24.0;

/// The frame of a swing at which its blow lands, from the name of the sound each frame of the action starts
/// (the client's `GetAttackMotion`).
pub fn frame<'a>(sounds: impl ExactSizeIterator<Item = Option<&'a str>>) -> usize {
    let count = sounds.len();
    let mut sounds = sounds;
    sounds.position(|sound| sound == Some("atk")).unwrap_or(count.saturating_sub(if count == 1 { 1 } else { 2 }))
}

/// How long after a blow is told of it lands (milliseconds): `frame` is where it lands in the swing, `speed`
/// what the server sent with it, `job` who strikes — the client gives a few archers and guardians longer.
pub fn lands_after(frame: usize, speed: f64, job: u16) -> f64 {
    let extra = match job {
        // JT_ARCHER_SKELETON, JT_G_ARCHER_SKELETON.
        1016 | 1420 => 192.0,
        // JT_ARCHER_GUARDIAN, JT_BOW_GUARDIAN.
        1285 | 1830 => 912.0,
        // JT_KNIGHT_GUARDIAN, JT_SWORD_GUARDIAN, JT_SOLDIER_GUARDIAN.
        1286 | 1829 | 1287 => 408.0,
        _ => 0.0,
    };
    (frame as f64 * (speed.clamp(0.0, FULL_SPEED) / FULL_SPEED) * FRAME_MS).trunc() + extra
}

/// How far across from a number's place one of its digits stands, in pixels of the digits' sprite: `from_last`
/// is how many places before the last digit it is, `count` how many digits the number has. The client's
/// `CGameActor::MakeNumber` puts each digit 8 from the next, the same for every digit: a digit's own width has
/// no part in it.
pub fn digit_across(count: usize, from_last: usize) -> f32 {
    (count as f32 - from_last as f32 * 2.0) * 4.0 - 4.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digits_of_a_number_stand_eight_apart_about_its_middle() {
        // 102: the 2 is last, the 0 one before it, the 1 two before it.
        assert_eq!([digit_across(3, 2), digit_across(3, 1), digit_across(3, 0)], [-8.0, 0.0, 8.0]);
        assert_eq!([digit_across(2, 1), digit_across(2, 0)], [-4.0, 4.0]);
        assert_eq!(digit_across(1, 0), 0.0);
    }

    #[test]
    fn the_blow_lands_on_the_frame_named_atk() {
        let swing = [None, Some("poring_attack.wav"), None, Some("atk"), None, None];
        assert_eq!(frame(swing.into_iter()), 3);
    }

    #[test]
    fn a_swing_that_names_no_frame_lands_on_the_one_before_its_last() {
        assert_eq!(frame([None; 6].into_iter()), 4);
        assert_eq!(frame([None; 1].into_iter()), 0);
        assert_eq!(frame([None; 0].into_iter()), 0);
    }

    #[test]
    fn a_poring_lands_its_blow_288_milliseconds_into_its_swing() {
        // The server's own figure for it (rAthena's `ClientAttackMotion` of the Poring).
        assert_eq!(lands_after(12, 432.0, 1002), 288.0);
        // A swing sent slower than the sprite's is not shown slower.
        assert_eq!(lands_after(12, 900.0, 1002), 288.0);
        assert_eq!(lands_after(12, 216.0, 1002), 144.0);
    }

    #[test]
    fn the_archers_and_guardians_land_later() {
        assert_eq!(lands_after(0, 432.0, 1016), 192.0);
        assert_eq!(lands_after(0, 432.0, 1285), 912.0);
        assert_eq!(lands_after(0, 432.0, 1287), 408.0);
    }
}
