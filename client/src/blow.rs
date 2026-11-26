//! When a blow lands: the moment into the striker's swing at which the one struck flinches, stops and the
//! number rises; how long the swing itself is shown; and how the digits of that number stand. The hits a blow is
//! made of are in `landing.rs`, what each shows and sounds like in `signs.rs`, how the numbers fly in
//! `flight.rs`, what a monster shoots in `shots.rs`.
//!
//! From the client's `CGameActor::ProcessHitMessage` (`Source/Contents/Actors/RenderObject.cpp:3600`: the blow
//! lands `attackMotion × render.speed × 24` milliseconds after it is told of), `CRenderObject::SetAction` (`:506`:
//! `render.speed` is the speed the sprite gives the action, no less than 1, times the speed of the blow),
//! `CRenderObject::ProcessMotion` (`:185`: a picture of the swing shows for `render.speed × 24` milliseconds,
//! and the swing is over after its last), `CGameActor::GetAttackMotion` (a monster's `attackMotion`: the frame
//! that carries the sound event named `atk`; without one, the frame before the last) and
//! `CSession::AttackMotionLength` (`Source/Contents/Actors/AttackMotion.cpp`: a character's `attackMotion`, a
//! figure by its job and not a frame of its sprite). The speed of the blow is what the server sends with it
//! over 432 (`SetModifyFactorOfmotionSpeed`; the client takes no more than 432, but for one who holds a bow).
//!
//! rAthena's `ClientAttackMotion` of a monster is this same figure (the Poring's 288 is its frame 12 at the
//! speed 1 of its sprite, the Orc Warrior's 672 its frame 7 at the speed 4), and with `synchronize_damage` on
//! the server counts the blow at that moment: the one struck is stopped at the same time on both sides.

pub mod combos;
pub mod flight;
pub mod landing;
pub mod shots;
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

/// How fast the pictures of a swing go (the client's `render.speed`, in pictures of 24 milliseconds):
/// `delay` is the speed the sprite gives the action (its ACT's figure; the client takes no less than 1),
/// `speed` what the server sent with the blow, of which no more than 432 is taken unless the striker holds a
/// `bow` (`CPc::SetModifyFactorOfmotionSpeed`). A blow sent with no speed leaves the sprite's own
/// (`SetAction` takes only a speed above 0).
pub fn pace(delay: f64, speed: f64, bow: bool) -> f64 {
    let own = delay.max(1.0);
    let paced = own * if bow { speed.max(0.0) } else { speed.clamp(0.0, FULL_SPEED) } / FULL_SPEED;
    if paced > 0.0 { paced } else { own }
}

/// How long after a blow is told of it lands (milliseconds): `motion` is the striker's `attackMotion` (a
/// monster's `frame`, a character's `attack_motion`), `pace` how fast its swing goes (`pace`), `job` who
/// strikes: the client gives a few archers and guardians longer.
pub fn lands_after(motion: f64, pace: f64, job: u16) -> f64 {
    (motion * pace * FRAME_MS).trunc() + lands_later(job)
}

/// How long a swing of so many pictures is shown, and the striker's body is in its blow (the client's
/// `ST_ATTACK`: `ProcessMotion` ends it when the time gone, in pictures, is the number of the action's
/// pictures), in milliseconds.
pub fn swing_lasts(frames: usize, pace: f64) -> f64 {
    frames as f64 * pace * FRAME_MS
}

/// A character's `attackMotion` (the client's `CSession::AttackMotionLength`): by its job, whether it is
/// `male`, whether it strikes with its `second` attack (the action at 88, `IsJobWeaponAllowed`), and whether
/// it holds a `katar` (or, in the client, a weapon in each hand). Only the jobs of the classic game are
/// here; the client's list has those that came after the Assassin too.
pub fn attack_motion(job: u16, male: bool, katar: bool, second: bool) -> f64 {
    match (second, job) {
        // JT_MERCHANT, JT_THIEF, with their first attack.
        (false, 5) => 5.85,
        (false, 6) => 5.75,
        // JT_NOVICE, JT_SUPERNOVICE and the baby one, a male one's second attack.
        (true, 0 | 23 | 4045) if male => 5.85,
        // JT_ASSASSIN, the Assassin Cross, the baby one.
        (true, 12 | 4013 | 4035) if katar => 3.0,
        _ => 6.0,
    }
}

/// How much later the blow of a character who shoots an arrow lands (milliseconds): the 8 pictures of 24
/// milliseconds its arrow flies (`CPc::SetState`: `attackMotion = 8 / render.speed + attackMotion`).
pub const ARROW_LATER: f64 = 8.0 * FRAME_MS;

/// How much later than its frame of the swing the blow of a job lands (milliseconds): the client gives a few
/// archers and guardians longer.
pub fn lands_later(job: u16) -> f64 {
    match job {
        // JT_ARCHER_SKELETON, JT_G_ARCHER_SKELETON.
        1016 | 1420 => ARROW_LATER,
        // JT_ARCHER_GUARDIAN, JT_BOW_GUARDIAN.
        1285 | 1830 => 912.0,
        // JT_KNIGHT_GUARDIAN, JT_SWORD_GUARDIAN, JT_SOLDIER_GUARDIAN.
        1286 | 1829 | 1287 => 408.0,
        _ => 0.0,
    }
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
        // The server's own figure for it (rAthena's `ClientAttackMotion` of the Poring): its frame 12, its
        // sprite's attack at the speed 1.
        assert_eq!(lands_after(12.0, pace(1.0, 432.0, false), 1002), 288.0);
        // A swing sent slower than the sprite's is not shown slower.
        assert_eq!(lands_after(12.0, pace(1.0, 900.0, false), 1002), 288.0);
        assert_eq!(lands_after(12.0, pace(1.0, 216.0, false), 1002), 144.0);
    }

    #[test]
    fn an_orc_warrior_lands_its_blow_672_milliseconds_into_its_swing() {
        // Its frame 7, its sprite's attack at the speed 4 (`ork_warrior.act`): rAthena's `ClientAttackMotion`
        // of it is 672, and its swing of 9 pictures is the server's `AttackMotion`, 864.
        let pace = pace(4.0, 432.0, false);
        assert_eq!(lands_after(7.0, pace, 1023), 672.0);
        assert_eq!(swing_lasts(9, pace), 864.0);
    }

    #[test]
    fn a_sprite_slower_than_one_is_taken_as_one_and_a_blow_of_no_speed_as_the_sprites_own() {
        assert_eq!(pace(0.5, 432.0, false), 1.0);
        assert_eq!(pace(0.0, 432.0, false), 1.0);
        assert_eq!(pace(3.0, 0.0, false), 3.0);
        // Only one who holds a bow swings slower than its sprite.
        assert_eq!(pace(2.0, 864.0, true), 4.0);
        assert_eq!(pace(2.0, 864.0, false), 2.0);
    }

    #[test]
    fn a_character_lands_its_blow_by_its_job() {
        assert_eq!(attack_motion(7, true, false, false), 6.0);
        assert_eq!([attack_motion(5, true, false, false), attack_motion(6, false, false, false)], [5.85, 5.75]);
        assert_eq!([attack_motion(0, true, false, true), attack_motion(0, false, false, true)], [5.85, 6.0]);
        assert_eq!([attack_motion(12, true, true, true), attack_motion(12, true, false, false)], [3.0, 6.0]);
        // A Knight at the server's 432 whose sprite gives its attack the speed 3: 432 milliseconds.
        assert_eq!(lands_after(attack_motion(7, true, false, false), pace(3.0, 432.0, false), 7), 432.0);
    }

    #[test]
    fn the_archers_and_guardians_land_later() {
        assert_eq!(lands_after(0.0, 1.0, 1016), 192.0);
        assert_eq!(lands_after(0.0, 1.0, 1285), 912.0);
        assert_eq!(lands_after(0.0, 1.0, 1287), 408.0);
    }
}
