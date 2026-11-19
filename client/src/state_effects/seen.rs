//! Someone coming into sight: what the bits of its third option begin and end on it
//! (`CGameMode::ApplyActorStatusEffects`).

use super::numbers::*;

/// What the third option of one coming into sight does to the effects on it.
#[derive(Debug, Default, PartialEq)]
pub struct Seen {
    /// The effects ended.
    pub ends: Vec<u32>,
    /// The effects begun, each unless another runs already (the client's `HasId`): the one begun, and the one
    /// asked after.
    pub begins: Vec<(u32, u32)>,
}

/// The third option of someone in sight (`CGameMode::ApplyActorStatusEffects`): `hidden` is whether its
/// state says it hides or is cloaked (the bits 2 and 4 of the effect state), `character` whether it is of a
/// player's job. A character's bits only begin effects, and each only if it does not run; a monster's begin
/// an effect or end it. The bits whose effects are not drawn are left out: the blurs (1, 2, 4), the copies
/// (0x4000), 0x10000 (`EF_UNDEADBODY`), 0x20000 (`EF_M04`), 0x40000 (`EF_CLOAKING2`), a monster's 0x8000
/// (`EF_DA_SPACE`), and the effect a monster grows back with (`EF_BABYBODY_BACK`).
pub fn seen(third: u32, hidden: bool, character: bool) -> Seen {
    let mut seen = Seen::default();
    if hidden {
        seen.ends.extend([EF_GUMGANG, EF_STEELBODY, EF_SU_STOOP, EF_DOUBLEGUMGANG]);
        return seen;
    }
    if character {
        const BITS: [(u32, &[u32]); 11] = [
            (0x8, &[EF_GUMGANG]),
            (0x10, &[EF_STEELBODY, EF_SU_STOOP]),
            (0x1000, &[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY]),
            (0x2000, &[EF_REFLECTBODY]),
            (0x8000, &[EF_ASURABODY]),
            (0x40, &[EF_AURABLADE2]),
            (0x100, &[EF_LIGHTBLADE]),
            (0x80, &[EF_REDBODY]),
            (0x200, &[EF_SPHEREWIND2]),
            (0x400, &[EF_PINKBODY]),
            (0x800, &[EF_ASSUMPTIO]),
        ];
        for (bit, effects) in BITS {
            if third & bit != 0 {
                seen.begins.extend(effects.iter().map(|effect| (*effect, effects[0])));
            }
        }
        return seen;
    }
    for (bit, effects) in [(0x2, &[EF_BABYBODY][..]), (0x4, &[EF_ASURABODY_MONSTER]), (0x10, &[EF_STEELBODY, EF_SU_STOOP]), (0x800, &[EF_ASSUMPTIO])] {
        if third & bit != 0 {
            seen.begins.extend(effects.iter().map(|effect| (*effect, effects[0])));
        } else {
            seen.ends.extend(effects);
        }
    }
    seen
}
