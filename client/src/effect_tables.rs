//! The tables of skills and effects the original client has compiled into it, written out of its decompiled code
//! (`tools/client-tables/skill_effects.py` says from where, and writes the files in `effect_tables/`).
//!
//! An effect is a number of the client's (`EF_…`). Most play a `.str` file; the others the client draws by code,
//! and only those of them this client has drawn too are shown (`effect_tables/effects_missing.txt` lists the
//! rest, with the skills that use them).

/// The entry of a table of lines `number<TAB>value` whose number is `wanted`.
fn found(table: &'static str, wanted: u32) -> Option<&'static str> {
    table.lines().filter_map(|line| line.split_once('\t')).find(|(number, _)| number.parse() == Ok(wanted)).map(|(_, value)| value)
}

/// The scope of a skill cast on a place, which the circle on the ground is sized by (the client's
/// `CGroundScope::SkillScopeType`); none for a skill that table does not name.
pub fn ground_scope(skill: u16) -> Option<i32> {
    found(include_str!("effect_tables/ground_scopes.txt"), u32::from(skill))?.parse().ok()
}

/// The `.str` file an effect plays (the client's `SelectRagEffectResource` and `CEZ2STRScreenEffect::
/// InitParticle`), in the client's folder of effects.
pub fn effect_file(effect: u32) -> Option<&'static str> {
    found(include_str!("effect_tables/effect_files.txt"), effect)
}

/// The sound an effect starts with (the first the client's `CRagEffect` plays for it), among the client's sounds.
pub fn effect_sound(effect: u32) -> Option<&'static str> {
    found(include_str!("effect_tables/effect_sounds.txt"), effect)
}

/// The effects of a skill that strikes nobody (the client's `CGameMode::ReceiveSkillUse`): each with whether it
/// is shown on the skill's target (or else on the one who used it).
pub fn skill_effects(skill: u16) -> Vec<(bool, u32)> {
    of_skill(include_str!("effect_tables/skill_effects.txt"), skill)
}

/// The effects of a skill that strikes somebody (the client's `CSession::PlaySkillEffect`): each with whether its
/// place is that of the one struck (or else of the striker).
pub fn skill_hits(skill: u16) -> Vec<(bool, u32)> {
    of_skill(include_str!("effect_tables/skill_hits.txt"), skill)
}

/// The effect the striker launches as a skill of its strikes, towards the one struck (the client's table
/// `skillEffectIds`, used by `CGameActor::ProcessSkillMessage`).
pub fn skill_launch(skill: u16) -> Option<u32> {
    found(include_str!("effect_tables/skill_launch.txt"), u32::from(skill))?.parse().ok()
}

/// The effect launched on the one struck by a skill when the blow lands (the client's table
/// `skillFollowEffects`): for most skills the flare, `EF_HIT2`.
pub fn skill_follow(skill: u16) -> Option<u32> {
    found(include_str!("effect_tables/skill_follow.txt"), u32::from(skill))?.parse().ok()
}

/// The effect shown on the one struck at each hit of a skill that hurts (the `hitEffect` the client's
/// `CGameActor::ProcessSkillMessage` picks by the skill): the ring of a plain blow, `EF_HIT1` (0), for most of
/// the older skills, the hit of an element for some, none for the rest.
pub fn skill_struck(skill: u16) -> Option<u32> {
    found(include_str!("effect_tables/skill_struck.txt"), u32::from(skill))?.parse().ok()
}

/// The lines `skill<TAB>s or t<TAB>effect` of a table that are of that skill.
fn of_skill(table: &'static str, skill: u16) -> Vec<(bool, u32)> {
    table
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let (id, who, effect) = (fields.next()?, fields.next()?, fields.next()?);
            (id.parse() == Ok(skill)).then_some((who == "t", effect.parse().ok()?))
        })
        .collect()
}

/// The state a skill puts the body of its user in (the client's `CSession::GetSkillActionInfo2`; its
/// `ACTORSTATEID`): 7, the motion of using a skill, unless the table says otherwise (2 and 9 the blow of an
/// attack, 11 the stance of one, 12 a throw…).
pub fn skill_state(skill: u16) -> u8 {
    found(include_str!("effect_tables/skill_motions.txt"), u32::from(skill)).and_then(|state| state.parse().ok()).unwrap_or(7)
}

/// The effect an item shows on the one who uses it, by its number among the client's effects (the client's
/// table of `CGameMode::PlayItemUseEffect`). It is launched as any effect is (`world::burst`), which plays its
/// file or draws it and starts its sounds: an item's use has no sound but its effect's.
pub fn item_effect(item: u32) -> Option<u32> {
    found(include_str!("effect_tables/item_effects.txt"), item)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_what_the_client_shows_for_a_skill() {
        // First Aid: its own effect on the one who uses it. Resurrection: two on the one raised, the first a file.
        assert_eq!(skill_effects(142), [(false, 309)]);
        assert_eq!(skill_effects(54), [(true, 77), (true, 140)]);
        assert!(effect_file(77).is_some_and(|file| file.to_lowercase().ends_with(".str")));
        // A level gained: the angel.
        assert_eq!(effect_file(371), Some("angel.str"));
        // No skill 0, and the usual state for a skill the table does not name.
        assert!(skill_effects(0).is_empty());
        assert_eq!(skill_state(142), 7);
        // Sonic Blow strikes: one effect at the striker, a file, and one the client draws at the one struck.
        assert_eq!(skill_hits(136), [(false, 143), (true, 121)]);
        assert_eq!(effect_file(143), Some("SonicBlow.str"));
        // Bash launches nothing and is followed by the flare; Magnum Break launches `EF_BASH` and is followed
        // by its own burst.
        assert_eq!((skill_launch(5), skill_follow(5)), (None, Some(1)));
        assert_eq!((skill_launch(7), skill_follow(7)), (Some(16), Some(17)));
        // Each hit of Bash shows the ring of a plain blow, of Fire Bolt the hit of fire, of Cold Bolt that of
        // cold, of Lightning Bolt that of lightning; Pierce shows none, nor does a skill of the later jobs.
        assert_eq!([5, 19, 14, 20, 56, 2001].map(skill_struck), [Some(0), Some(49), Some(51), Some(52), None, None]);
    }

    #[test]
    fn potions_keep_their_effects() {
        // A Red Potion: `EF_POTION1`, a file of the client's with the sound of healing.
        assert_eq!(item_effect(501), Some(204));
        assert!(effect_file(204).is_some_and(|file| file.ends_with(".str")));
    }
}
