//! The effects a state going on or off begins and ends on the one in it: the client's `AM_STATUS_CHANGE` in
//! `CGameActor::SendMsg` (`Contents/Actors/GameObject.cpp`), for the states whose effects live until they are
//! told to go. The client ends every such effect that runs on the actor (`AM_DELETE_EFFECT`, while `HasId`),
//! whether the state goes on or off, and with the state on launches it anew.
//!
//! The same effects come by a second way: the bits of the third option of one coming into sight (the
//! `virtue` of the packets that show a unit, the server's `opt3`; `CGameMode::ApplyActorStatusEffects`,
//! `Client/Mode/GameModePackets.cpp`), which `seen` has. And some effects that a skill shows are launched
//! only while another runs, or does not (`begins`). And two effects a bit of the effect state keeps going,
//! launched again and again for as long as it is set (`Kept`, `state_effects/kept.rs`).
//!
//! Only the states are here whose effects are drawn (`web/world/prims/`, or a file of the client's that is
//! played). Every state of that arm that launches or ends an effect, with those left out and why:
//!
//! - the blur of the quickenings, Overthrust, Energy Coat and Fury (`EF_MAKEBLUR`, 166: copies of the actor
//!   left behind it), and the copies of `EFST_NJ_BUNSINJYUTSU`, `EFST_WUGDASH`, `EFST_STRIKING` and
//!   `EFST_HALLUCINATIONWALK` (617, 752, 925, 792): not drawn;
//! - `EFST_DEVIL1` and `EFST_HANDICAPSTATE_DEEPBLIND`, which darken the screen of the player's own character
//!   (`EF_BLACKDEVIL` and the `EF_DEVIL`s, `EF_BLIND` with `EF_STATUS_STATE`): not drawn here; the veil of
//!   blindness comes by the health state (`crate::ailments`);
//! - `EFST_PROPERTYUNDEAD` (655, 656), `EFST_SMA_READY` (546), `EFST_RG_CCONFINE_M` and `_S` (604),
//!   `EFST_WZ_SIGHTBLASTER` (601), `EFST_DA_CONTRACT` (586), `EFST_ORATIO` (755), `EFST_BANDING` (909),
//!   `EFST_VENOMIMPRESS` (788), `EFST_FROSTMISTY` (789), `EFST_COLD` (940), `EFST_SIREN` (879),
//!   `EFST_MANHOLE` (823), `EFST_SHADOWFORM` (809), `EFST_CAMOUFLAGE`, `EFST_STEALTHFIELD` and
//!   `EFST_ELEMENTAL_VEIL` (744), `EFST_GRANITIC_ARMOR`, `EFST_KYOMU` and `EFST_GENSOU` (963),
//!   `EFST_KAGEMUSYA` (1004), `EFST_ZANGETSU` (1003), `EFST_MAXPAIN` (1037), `EFST_WIDEWEB` (404),
//!   `EFST_BURNT` (1054), `EFST_MISTY_FROST` (1420), `EFST_PRISON` (1525), `EFST_LOCKON_LASER` (1601),
//!   `EFST_HELLS_PLANT_ARMOR` (1610), the states that launch an `EF_STATUS_…_UP` (2288 to 2300), and the songs
//!   whose effect is an `EF_CIRCLEPOWER` not drawn (865, 867, 871, 873, 875, 877): their effects are not drawn;
//! - `EFST_MEIKYOUSISUI`, which launches Moonlit's wind in a colour of its own (the effect's
//!   `colorChannels`, which no effect here takes);
//! - what the client does through its other effects' manager (`g_effectMgr`: the balls of `EFST_SUMMON1` to
//!   `5`, `EFST_ROLLINGCUTTER`, the stances, the marks and the hats), which is another system than the
//!   effects of `CRagEffect`.
//!
//! Left out of what the client does for the states that are here: the darkened sky of a Soul Link on the
//! player's own character, that Steel Body's effect stays on a character who wears a thing that shows it too
//! (`appearanceFlags`), the words a mercenary's states say over it (`AM_CHAT`), and the grey body held
//! still of the third option's bit 0x20 (a blade stopped).
//!
//! A third way is the player's own character alone: the client launches the effects of the states it is in
//! anew when it is made on a map it has come to (`CPlayer::ReapplySkillEffects`, which goes through the states
//! it keeps for the player, `g_activeSkillEffects`); `reapplied` has that list, as far as its effects are
//! drawn. (The client calls the same function when the player's own states are told and when the map's kind
//! is; that is not done here: only the arrival.)

mod kept;

pub use kept::Kept;

/// The client's numbers of the effects named here.
const EF_DECAGILITY: u32 = 38;
const EF_LEXDIVINA: u32 = 87;
const EF_GUMGANG: u32 = 203;
const EF_STEELBODY: u32 = 254;
const EF_AURABLADE: u32 = 367;
const EF_REDBODY: u32 = 368;
const EF_LKCONCENTRATION: u32 = 369;
const EF_ASSUMPTIO: u32 = 375;
const EF_LIGHTBLADE: u32 = 382;
const EF_SPHEREWIND2: u32 = 394;
const EF_LINELINK3: u32 = 395;
const EF_PINKBODY: u32 = 396;
const EF_DOUBLEGUMGANG: u32 = 418;
const EF_REFLECTBODY: u32 = 419;
const EF_BABYBODY: u32 = 420;
const EF_ASURABODY: u32 = 424;
const EF_QUAKEBODY: u32 = 426;
const EF_ASURABODY_MONSTER: u32 = 427;
const EF_RUN: u32 = 442;
const EF_STOPRUN: u32 = 443;
const EF_STOPEFFECT: u32 = 444;
const EF_NPC_STOP: u32 = 453;
const EF_GUMGANGNPC: u32 = 455;
const EF_AGIUP: u32 = 456;
const EF_AURABLADE2: u32 = 474;
const EF_REDLIGHTBODY: u32 = 544;
const EF_STASIS: u32 = 799;
const EF_LINELINK4: u32 = 819;
const EF_CHOOKGI_N: u32 = 833;
const EF_CIRCLEPOWER2: u32 = 861;
const EF_ENERVATION7: u32 = 864;
const EF_ALL_RAY_OF_PROTECTION: u32 = 1019;
const EF_GC_DARKCROW: u32 = 1040;
const EF_WL_TELEKINESIS_INTENSE: u32 = 1048;
const EF_GN_ILLUSIONDOPING: u32 = 1049;
const EF_NPC_CHILL: u32 = 1055;
const EF_AB_OFFERTORIUM_RING: u32 = 1057;
const EF_ESCAPE: u32 = 1081;
const EF_SU_STOOP: u32 = 1132;

/// What a state going on or off does to the effects on the one in it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Change {
    /// The effects ended, and then those begun.
    pub ends: &'static [u32],
    pub begins: &'static [u32],
    /// An effect begun only if another ran as the state changed: the one that ran, and the one begun.
    pub after: Option<(u32, u32)>,
    /// The sound the change starts (none: empty).
    pub sound: &'static str,
}

/// What a state (the client's `EFST_` number) going on or off does: the effects to end on the one in it, and
/// then those to begin.
pub fn changed(state: u32, on: bool) -> Change {
    let kept = |effects: &'static [u32]| Change { ends: effects, begins: if on { effects } else { &[] }, ..Change::default() };
    match state {
        // `EFST_APPLEIDUN`.
        73 => kept(&[EF_CIRCLEPOWER2]),
        // `EFST_DONTFORGETME`.
        75 => kept(&[EF_DECAGILITY]),
        // `EFST_ETERNALCHAOS`, `EFST_ROKISWEIL`, `EFST_SATURDAY_NIGHT_FEVER`.
        79 | 82 | 448 => kept(&[EF_ENERVATION7]),
        // `EFST_EXPLOSIONSPIRITS`.
        86 => kept(&[EF_GUMGANG]),
        // `EFST_STEELBODY`, `EFST_DBARRIER`.
        87 | 235 => kept(&[EF_STEELBODY, EF_SU_STOOP]),
        // `EFST_STOP`: the effect is begun by the skill (`NPC_STOP`), and only ended here.
        95 => Change { ends: &[EF_NPC_STOP], ..Change::default() },
        // `EFST_POWERUP`.
        98 => kept(&[EF_GUMGANGNPC]),
        // `EFST_AURABLADE`: the walls round the one in it, and the light of its weapon.
        103 => kept(&[EF_AURABLADE, EF_AURABLADE2]),
        // `EFST_LKCONCENTRATION`.
        105 => kept(&[EF_LKCONCENTRATION]),
        // `EFST_BERSERK`, `EFST_BLOOD_SUCKER`.
        107 | 464 => kept(&[EF_REDBODY]),
        // `EFST_ASSUMPTIO`, `EFST_ASSUMPTIO2`, `EFST_ASSUMPTIO_BUFF`.
        110 | 473 | 1121 => kept(&[EF_ASSUMPTIO]),
        // `EFST_MAGICPOWER`: the weapon drawn twice.
        113 => kept(&[EF_LIGHTBLADE]),
        // `EFST_MARIONETTE_MASTER`: the strings to the puppet go too.
        121 => Change { ends: &[EF_LINELINK3, EF_PINKBODY], begins: if on { &[EF_PINKBODY] } else { &[] }, ..Change::default() },
        // `EFST_MARIONETTE`, `EFST_MAGIC_POISON`.
        122 | 1142 => kept(&[EF_PINKBODY]),
        // `EFST_MOON`.
        123 => kept(&[EF_SPHEREWIND2]),
        // `EFST_RUN`, going on or off: the footprints end, and if there were any `EF_STOPRUN` begins, which
        // draws nothing and for as long as it lives has the skill show the blades of one who stops.
        133 => Change { ends: &[EF_RUN], after: Some((EF_RUN, EF_STOPRUN)), ..Change::default() },
        // `EFST_TING`, going on or off: one who ran into a wall is shaken, with a sound.
        134 => Change { ends: &[EF_RUN], begins: &[EF_QUAKEBODY], sound: "effect/t_벽튕김.wav", ..Change::default() },
        // `EFST_SOULLINK` and the soul states after it: the skill begins the effect; the state's end ends it.
        149 | 1056..=1059 if !on => Change { ends: &[EF_ASURABODY], ..Change::default() },
        // `EFST_KAITE`.
        153 => kept(&[EF_REFLECTBODY]),
        // `EFST_SG_SUN_WARM`, `EFST_SG_MOON_WARM`, `EFST_SG_STAR_WARM`, `EFST_DAURA`.
        165..=167 | 232 => kept(&[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY]),
        // `EFST_MER_FLEE`, `_ATK`, `_HP`, `_SP`, `_HIT`: a mercenary's state, going on or off.
        277..=281 => Change { begins: &[EF_AGIUP], ..Change::default() },
        // `EFST_DUPLELIGHT`: the two lights that go round the one in it.
        350 => kept(&[EF_CHOOKGI_N]),
        // `EFST_SHADOWFORM`: the tube to the one it is used on, which the skill begins (`SC_SHADOWFORM`), goes
        // with the state. (The client ends it beside the state's own effect, `EF_CLOAKING3`, which is not
        // drawn, and only while that one runs: so not as the state comes on.)
        394 if !on => Change { ends: &[EF_LINELINK4], ..Change::default() },
        // `EFST_NETHERWORLD`.
        433 => kept(&[EF_NPC_STOP]),
        // `EFST_MELODYOFSINK`.
        452 => kept(&[EF_LEXDIVINA]),
        // `EFST_BEYOND_OF_WARCRY`.
        453 => kept(&[EF_STASIS]),
        // `EFST_AKAITSUKI`.
        660 => kept(&[EF_REDLIGHTBODY]),
        // `EFST_RAY_OF_PROTECTION`.
        676 => kept(&[EF_ALL_RAY_OF_PROTECTION]),
        // `EFST_OFFERTORIUM`.
        716 => kept(&[EF_AB_OFFERTORIUM_RING]),
        // `EFST_TELEKINESIS_INTENSE`.
        717 => kept(&[EF_WL_TELEKINESIS_INTENSE]),
        // `EFST_DARKCROW`: the client launches its effect three times.
        730 => Change { ends: &[EF_GC_DARKCROW], begins: if on { &[EF_GC_DARKCROW; 3] } else { &[] }, ..Change::default() },
        // `EFST_ILLUSIONDOPING`.
        734 => kept(&[EF_GN_ILLUSIONDOPING]),
        // `EFST_CHILL`.
        736 => kept(&[EF_NPC_CHILL]),
        _ => Change::default(),
    }
}

/// The effects launched anew on the player's own character for a state it is in, as it arrives on a map
/// (`CPlayer::ReapplySkillEffects`): not always what the state began with (Aura Blade's walls are not launched
/// again, only the light of the weapon). The states whose effects are not drawn are left out, with the
/// darkened sky of a Soul Link; `EFST_DUPLELIGHT`'s lights the client tells a size of 2 there (`AM_SETPOS`),
/// which is not told here.
pub fn reapplied(state: u32) -> &'static [u32] {
    match state {
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

/// Whether an effect that a skill shows is launched, `running` saying which effects run on the one it is for:
/// some the client launches only if they do not run already (`HasId` in its handler of the skill), and the
/// two of `TK_RUN` by whether `EF_STOPRUN` runs: the blades of a stop (`EF_STOPEFFECT`) if it does, the
/// footprints (`EF_RUN`) if it does not.
pub fn begins(effect: u32, running: impl Fn(u32) -> bool) -> bool {
    match effect {
        EF_RUN => !running(EF_STOPRUN) && !running(EF_RUN),
        EF_STOPEFFECT => running(EF_STOPRUN),
        EF_CHOOKGI_N | EF_ESCAPE => !running(effect),
        _ => true,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_going_on_ends_its_effect_and_begins_it_anew() {
        assert_eq!(changed(107, true), Change { ends: &[EF_REDBODY], begins: &[EF_REDBODY], ..Change::default() });
        assert_eq!(changed(107, false), Change { ends: &[EF_REDBODY], ..Change::default() });
        assert_eq!(changed(166, true).begins, &[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY][..]);
        // Aura Blade's walls and the light of the weapon; Mystical Amplification's weapon; the two lights.
        assert_eq!(changed(103, true).begins, &[EF_AURABLADE, EF_AURABLADE2][..]);
        assert_eq!((changed(113, true).begins, changed(113, false).ends), (&[EF_LIGHTBLADE][..], &[EF_LIGHTBLADE][..]));
        assert_eq!(changed(350, true).begins, &[EF_CHOOKGI_N][..]);
        assert_eq!(changed(730, true).begins.len(), 3);
    }

    #[test]
    fn some_states_only_end_what_a_skill_began() {
        assert_eq!(changed(95, true), Change { ends: &[EF_NPC_STOP], ..Change::default() });
        assert_eq!(changed(149, true), Change::default());
        assert_eq!(changed(149, false).ends, &[EF_ASURABODY][..]);
        assert_eq!(changed(121, false).ends, &[EF_LINELINK3, EF_PINKBODY][..]);
    }

    #[test]
    fn running_ends_with_a_stop_and_a_wall_shakes_the_runner() {
        // Whichever way the state goes: the footprints end, and what follows them begins if there were any.
        for on in [true, false] {
            assert_eq!(changed(133, on), Change { ends: &[EF_RUN], after: Some((EF_RUN, EF_STOPRUN)), ..Change::default() });
            assert_eq!((changed(134, on).begins, changed(134, on).sound), (&[EF_QUAKEBODY][..], "effect/t_벽튕김.wav"));
        }
        // The skill shows the footprints, or the blades of a stop while what follows the footprints runs.
        assert!(begins(EF_RUN, |_| false) && !begins(EF_STOPEFFECT, |_| false));
        assert!(!begins(EF_RUN, |effect| effect == EF_STOPRUN) && begins(EF_STOPEFFECT, |effect| effect == EF_STOPRUN));
        assert!(!begins(EF_RUN, |effect| effect == EF_RUN));
        assert!(begins(EF_CHOOKGI_N, |_| false) && !begins(EF_CHOOKGI_N, |effect| effect == EF_CHOOKGI_N));
        assert!(begins(EF_REDBODY, |_| true));
    }

    #[test]
    fn any_other_state_has_none() {
        assert_eq!(changed(10, true), Change::default());
    }

    #[test]
    fn one_who_comes_into_sight_shows_the_states_of_its_third_option() {
        // A player with Fury and Steel Body on: each effect unless it runs already, the sound of Steel Body
        // going by its body.
        let player = seen(0x18, false, true);
        assert_eq!(player.begins, [(EF_GUMGANG, EF_GUMGANG), (EF_STEELBODY, EF_STEELBODY), (EF_SU_STOOP, EF_STEELBODY)]);
        assert!(player.ends.is_empty());
        assert_eq!(seen(0x140, false, true).begins, [(EF_AURABLADE2, EF_AURABLADE2), (EF_LIGHTBLADE, EF_LIGHTBLADE)]);
        assert_eq!(seen(0x800, false, true).begins, [(EF_ASSUMPTIO, EF_ASSUMPTIO)]);
        // One who hides shows none of them.
        assert_eq!(seen(0x18, true, true), Seen { ends: vec![EF_GUMGANG, EF_STEELBODY, EF_SU_STOOP, EF_DOUBLEGUMGANG], begins: Vec::new() });
        // A monster's bits are others, and one not set ends its effect.
        let monster = seen(0x6, false, false);
        assert_eq!(monster.begins, [(EF_BABYBODY, EF_BABYBODY), (EF_ASURABODY_MONSTER, EF_ASURABODY_MONSTER)]);
        assert_eq!(monster.ends, [EF_STEELBODY, EF_SU_STOOP, EF_ASSUMPTIO]);
    }

    #[test]
    fn the_players_own_states_launch_their_effects_anew_on_arrival() {
        assert_eq!(reapplied(86), &[EF_GUMGANG][..]);
        assert_eq!(reapplied(87), &[EF_STEELBODY, EF_SU_STOOP][..]);
        // Aura Blade: the light of the weapon alone, not the walls the state began with.
        assert_eq!((reapplied(103), changed(103, true).begins), (&[EF_AURABLADE2][..], &[EF_AURABLADE, EF_AURABLADE2][..]));
        assert_eq!((reapplied(166), reapplied(232)), (&[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY][..], &[EF_DOUBLEGUMGANG, EF_REDLIGHTBODY][..]));
        // A Soul Link's body, which the state itself does not begin; Dark Crow three times over.
        assert_eq!((reapplied(149), changed(149, true).begins), (&[EF_ASURABODY][..], &[][..]));
        assert_eq!(reapplied(730).len(), 3);
        // A state that is only ended as it changes, or one not drawn, launches nothing.
        assert_eq!((reapplied(95), reapplied(133), reapplied(10)), (&[][..], &[][..], &[][..]));
    }
}
