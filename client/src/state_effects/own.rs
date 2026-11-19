//! The player's own character arriving on a map: the effects of the states it is in, launched anew
//! (`CPlayer::ReapplySkillEffects`).

use super::numbers::*;

/// The effects launched anew on the player's own character for a state it is in, as it arrives on a map
/// (`CPlayer::ReapplySkillEffects`): not always what the state began with (Aura Blade's walls are not launched
/// again, only the light of the weapon). The states whose effects are not drawn are left out, with the
/// darkened sky of a Soul Link; `EFST_DUPLELIGHT`'s lights the client tells a size of 2 there (`AM_SETPOS`),
/// which is not told here.
pub fn reapplied(state: u32) -> &'static [u32] {
    match state {
        // The quickenings, Overthrust and Energy Coat: the blur (`blur`).
        2 | 25 | 31 | 68 | 161 | 188 => &[EF_MAKEBLUR],
        // `EFST_EXPLOSIONSPIRITS`, `EFST_STEELBODY`.
        86 => &[EF_GUMGANG],
        87 => &[EF_STEELBODY, EF_SU_STOOP],
        // `EFST_AURABLADE`, `EFST_LKCONCENTRATION`, `EFST_BERSERK` and `EFST_BLOOD_SUCKER`.
        103 => &[EF_AURABLADE2],
        105 => &[EF_LKCONCENTRATION],
        107 | 464 => &[EF_REDBODY],
        // `EFST_ASSUMPTIO`, `EFST_ASSUMPTIO2`, `EFST_ASSUMPTIO_BUFF`.
        110 | 473 | 1121 => &[EF_ASSUMPTIO],
        // `EFST_MAGICPOWER`, `EFST_MOON`.
        113 => &[EF_LIGHTBLADE],
        123 => &[EF_SPHEREWIND2],
        // `EFST_SOULLINK` and the soul states after it.
        149 | 1056..=1059 => &[EF_ASURABODY],
        // `EFST_KAITE`.
        153 => &[EF_REFLECTBODY],
        // `EFST_SG_SUN_WARM`, `EFST_SG_MOON_WARM`, `EFST_SG_STAR_WARM`, `EFST_DAURA`.
        165..=167 | 232 => &[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY],
        // `EFST_DUPLELIGHT`.
        350 => &[EF_CHOOKGI_N],
        // `EFST_SATURDAY_NIGHT_FEVER`, `EFST_MELODYOFSINK`, `EFST_BEYOND_OF_WARCRY`.
        448 => &[EF_ENERVATION7],
        452 => &[EF_LEXDIVINA],
        453 => &[EF_STASIS],
        // `EFST_OFFERTORIUM`, `EFST_TELEKINESIS_INTENSE`.
        716 => &[EF_AB_OFFERTORIUM_RING],
        717 => &[EF_WL_TELEKINESIS_INTENSE],
        // `EFST_DARKCROW`, three times as the state begins it; `EFST_CHILL`.
        730 => &[EF_GC_DARKCROW; 3],
        736 => &[EF_NPC_CHILL],
        _ => &[],
    }
}
