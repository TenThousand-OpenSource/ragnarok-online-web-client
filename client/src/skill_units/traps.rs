//! The traps a Hunter lays: the units the client shows as a model standing on their cell (`CSkill::ShowModel`,
//! `CSkill::GetTrapModelName`), not as an effect.
//!
//! A trap stands still, at the fourth frame of its model's animation (`ShowModel`: `Animate(3.0)` and no
//! playback). It springs when it strikes (`AM_SKILL`, the skill's damage coming from the unit) or when the
//! server makes another kind of it (`AM_SETJOB`, ZC_SPRITE_CHANGE of its base: `UNT_USED_TRAPS` for most, the
//! same kind again for an Ankle Snare that has caught someone): its animation then plays once, at the speed
//! the client gives it (`SetSpeed(1.0)`), and stays at its end. A trap made a used one also launches the effect
//! its kind remembered as it appeared (`m_effectId`).
//!
//! Only the traps of the second jobs are here. Those of the later jobs (the Ranger's, the Windhawk's) have
//! models of their own in the same function; they wait with their jobs.

use crate::model::Light;

/// `UNT_USED_TRAPS`: what the server makes of a trap that has sprung.
pub const USED: u32 = 140;
/// `UNT_SHOCKWAVE`, and the effect that plays over it for as long as it is seen (`EF_SHOCKWAVE`,
/// `ShockWave.str`: its function begins the file anew whenever it ends).
pub const SHOCKWAVE: u32 = 148;
pub const EF_SHOCKWAVE: u32 = 145;
/// How many ticks `ShockWave.str` takes: its last frame is 74.
pub const SHOCKWAVE_TICKS: i32 = 75;

/// The frame of its animation a trap stands at until it springs.
const STANDS_AT: i32 = 3;

/// The model of a kind of trap, by its path under `data\model\` (`CSkill::GetTrapModelName`); nothing for a
/// kind that is no trap of the second jobs.
pub fn model(unit: u32) -> Option<&'static str> {
    Some(match unit {
        // UNT_BLASTMINE, UNT_SKIDTRAP, UNT_ANKLESNARE.
        143 => "외부소품\\트랩03_3.rsm",
        144 => "외부소품\\트랩02.rsm",
        145 => "외부소품\\트랩01.rsm",
        // UNT_LANDMINE, UNT_SHOCKWAVE, UNT_SANDMAN, UNT_FLASHER, UNT_FREEZINGTRAP.
        147 => "외부소품\\트랩03.rsm",
        148 => "외부소품\\트랩03_6.rsm",
        149 => "외부소품\\트랩03_4.rsm",
        150 => "외부소품\\트랩03_5.rsm",
        151 => "외부소품\\트랩03_2.rsm",
        // UNT_CLAYMORETRAP, UNT_TALKIEBOX.
        152 => "외부소품\\트랩04.rsm",
        153 => "외부소품\\트랩05.rsm",
        _ => return None,
    })
}

/// The effect a trap of a kind launches on itself when the server makes a used trap of it (`m_effectId`, set in
/// `AM_SKILLSTANDENTRY` and launched in `AM_SETJOB`): `EF_BLASTMINEBOMB`, `EF_SANDMAN`, `EF_FLASHER`,
/// `EF_FREEZING`, `EF_CLAYMORE`. The others launch none: what they do shows on the one caught.
pub fn sprung_with(unit: u32) -> Option<u32> {
    Some(match unit {
        143 => 106,
        149 => 139,
        150 => 99,
        151 => 108,
        152 => 107,
        _ => return None,
    })
}

/// The light a trap's model is lit by, whatever the map's (`ShowModel`'s three vectors, handed to
/// `C3dActorV1::Prepare` as the way to the light, its colour and the light all around): the way is the
/// client's (0.5, 0.7, 0.5) of the files' space, whose y points down, and is not made one long.
pub fn light() -> Light {
    Light { direction: [0.5, -0.7, 0.5], diffuse: [1.0; 3], ambient: [0.5; 3] }
}

/// The frame of its animation a trap's model shows: where it stands until it springs, and from then on what
/// `C3dActorV1::Update` makes of so many of the client's frames (`sprung`), playing once.
pub fn frame(length: i32, sprung: Option<u64>) -> i32 {
    match sprung {
        Some(frames) => crate::model::frame_at(frames, 1.0, length, 1),
        None => STANDS_AT.min((length - 1).max(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_trap_has_its_model() {
        assert_eq!(model(145), Some("외부소품\\트랩01.rsm"));
        assert_eq!(model(147), Some("외부소품\\트랩03.rsm"));
        assert_eq!(model(144), Some("외부소품\\트랩02.rsm"));
        assert_eq!(model(152), Some("외부소품\\트랩04.rsm"));
        // Every trap of the Hunter's has one; a used trap, a Venom Dust and a Fire Wall have none.
        assert!([143, 144, 145, 147, 148, 149, 150, 151, 152, 153].into_iter().all(|unit| model(unit).is_some()));
        assert_eq!((model(USED), model(146), model(127), model(138)), (None, None, None, None));
    }

    #[test]
    fn a_trap_made_a_used_one_launches_its_effect() {
        assert_eq!(
            (sprung_with(143), sprung_with(149), sprung_with(150), sprung_with(151), sprung_with(152)),
            (Some(106), Some(139), Some(99), Some(108), Some(107))
        );
        // An Ankle Snare, a Land Mine, a Skid Trap, a Shockwave Trap and a Talkie Box launch none.
        assert!([144, 145, 147, 148, 153].into_iter().all(|unit| sprung_with(unit).is_none()));
    }

    #[test]
    fn a_trap_stands_still_until_it_springs_and_then_plays_once() {
        // 트랩01.rsm is 16000 frames long; a frame of the client's moves it on by 33.
        assert_eq!(frame(16000, None), 3);
        assert_eq!((frame(16000, Some(0)), frame(16000, Some(1)), frame(16000, Some(10))), (0, 33, 330));
        assert_eq!(frame(16000, Some(100_000)), 15999);
        assert_eq!(frame(0, None), 0);
    }
}
