//! What the client's handler of a skill that strikes does by itself for the combos of a monk, beside what its
//! tables give every skill (`crate::effect_tables::{skill_launch, skill_follow, skill_struck}`).
//!
//! From `CGameActor::ProcessSkillMessage` (`Contents/Actors/RenderObject.cpp`), the arms of its switch over
//! the skill that follows the hits: each launches effects on the striker or on the one struck
//! (`LaunchEffect`), some shake the screen (`MM_QUAKE` with 0) and two start a sound
//! (`g_soundMgr->PlayWave`), all as the skill's packet is read. (The poses the arms give the striker,
//! `SetFrameSequence`, are not here.)

/// What a combo's arm does.
#[derive(Debug, PartialEq)]
pub struct Combo {
    /// Whether the screen shakes (`MM_QUAKE` with 0: `crate::quake`).
    pub quake: bool,
    /// The sound it starts, among the client's sounds (none: empty).
    pub sound: &'static str,
    /// The effects it launches, by the client's numbers, each with whether it is on the one struck (or else
    /// on the striker). `EF_CHEMICAL2DASH` (512) alone is told a place, that of the one struck (`AM_SETPOS`).
    pub effects: &'static [(bool, u32)],
}

/// What the client's handler does by itself for a skill that strikes; nothing for a skill that is none of the
/// five combos.
pub fn of_skill(skill: u16) -> Option<Combo> {
    // `EF_GUMGANG3`, which the striker of four of the five shows.
    const AURA: (bool, u32) = (false, 263);
    Some(match skill {
        // `MO_CHAINCOMBO`: `EF_TEIHIT1` and `EF_CHAINCOMBO` on the one struck.
        272 => Combo { quake: false, sound: "", effects: &[(true, 262), AURA, (true, 273)] },
        // `MO_COMBOFINISH`: `EF_HITLINE` on the striker.
        273 => Combo { quake: true, sound: "effect/mon_맹룡과강.wav", effects: &[AURA, (false, 330)] },
        // `CH_PALMSTRIKE`: `EF_HITLINE2` on the one struck.
        370 => Combo { quake: true, sound: "effect/맹호경파산.wav", effects: &[(true, 376)] },
        // `CH_TIGERFIST`: `EF_BASH3D2` on the striker (which starts its own sound).
        371 => Combo { quake: true, sound: "", effects: &[(false, 377), AURA] },
        // `CH_CHAINCRUSH`: `EF_CHEMICAL2DASH` on the striker, towards the one struck.
        372 => Combo { quake: false, sound: "", effects: &[AURA, (false, 512)] },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_combo_launches_what_its_arm_of_the_client_does() {
        let effects = |skill| of_skill(skill).map(|combo| combo.effects.to_vec());
        assert_eq!(effects(272), Some(vec![(true, 262), (false, 263), (true, 273)]));
        assert_eq!(effects(273), Some(vec![(false, 263), (false, 330)]));
        assert_eq!(effects(370), Some(vec![(true, 376)]));
        assert_eq!(effects(371), Some(vec![(false, 377), (false, 263)]));
        assert_eq!(effects(372), Some(vec![(false, 263), (false, 512)]));
        // Triple Attack and Asura Strike have no effects in those arms, nor has a skill that is no combo.
        assert!([263, 271, 5, 0].into_iter().all(|skill| of_skill(skill).is_none()));
    }

    #[test]
    fn three_combos_shake_the_screen_and_two_of_them_sound() {
        let shakes: Vec<u16> = [272, 273, 370, 371, 372].into_iter().filter(|skill| of_skill(*skill).is_some_and(|combo| combo.quake)).collect();
        assert_eq!(shakes, [273, 370, 371]);
        let sound = |skill| of_skill(skill).map(|combo| combo.sound);
        assert_eq!(sound(273), Some("effect/mon_맹룡과강.wav"));
        assert_eq!(sound(370), Some("effect/맹호경파산.wav"));
        assert_eq!((sound(272), sound(371), sound(372)), (Some(""), Some(""), Some("")));
    }
}
