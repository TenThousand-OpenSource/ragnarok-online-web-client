//! Skill units: what a skill leaves standing on the ground (a cell of Fire Wall, of Sanctuary, a Safety Wall,
//! a Warp Portal). The server shows each with ZC_SKILL_ENTRY5 and takes it away with ZC_SKILL_DISAPPEAR
//! (`net/skill_units.rs`); the client makes a `CSkill` of it (`Contents/Actors/Skill.cpp`), which is told what
//! kind of unit it is (`AM_SKILLSTANDENTRY`) and from then on, every tick, launches the effect of its kind on
//! itself (`CSkill::OnProcess`, `Contents/Effects/Core/MsgEffect.cpp`). This file is that rule: which effect a
//! kind of unit shows and how often, and how each of those effects is drawn. `web/world/skill_units.rs` keeps
//! the units of the map and draws them.
//!
//! A unit shows its effect in one of three ways:
//!
//! - again and again: a count (`m_effectTimer`, far up at first) goes up by one a tick, and when it has reached
//!   the effect's lifetime less a few ticks the effect is launched and the count starts over;
//! - kept: the effect is launched once and made to last 21 ticks longer every tick, so it lives as long as
//!   the unit does;
//! - at once: launched as the unit appears and never again.
//!
//! An effect on a unit ends with the unit (the actor tells its effects `AM_MASTER_DEAD` as it goes).
//!
//! The kinds of the classic jobs are here (the first and second jobs, the transcendent ones, Gunslinger and
//! Ninja), each with the effect the client launches for it, whether that effect is drawn here yet or not (one
//! that is not shows nothing: `web/world/skill_units.rs`); `shows` says nothing for the others:
//!
//! - Graffiti (176), which comes in a packet of its own (ZC_SKILL_ENTRY2) and of which only the words are
//!   shown, over the unit (`web/world/written.rs`);
//! - the kind 134, which the server shows instead of a unit that is not to be seen and for which the client
//!   has nothing either; the songs and dances (158 to 175), Moonlit Water Mill (181) and Kaensin (186), for
//!   which `CSkill::OnProcess` has no arm; Evil Land (199), which the client hands to its newer effects
//!   (`g_effectMgr->Spawn` of `EF_EVILLAND`, told the unit's range), not ported; and every kind of the later
//!   jobs, most of which the client hands to its `CSkillActionMgr`.
//!
//! The traps (143 to 153) the client shows as models standing on their cells, not as effects: `traps.rs` has
//! which model each is, how it moves and what it launches when it springs. A Shockwave Trap has an effect
//! over it besides.
//!
//! The server also tells of a skill used on a place (ZC_NOTIFY_GROUNDSKILL): the client then plays an effect at
//! that place (`CGameActor::PlayGroundSkillEffect`); `ground_effect` has those of the classic jobs, and
//! `on_caster` the one that function launches on the one who used the skill instead.

mod pulse;
pub mod traps;

pub use pulse::Pulse;

/// How a kind of unit shows its effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shows {
    /// The effect, launched every so many ticks.
    Again(u32, i32),
    /// The effect, launched once and kept for as long as the unit stands.
    Kept(u32),
    /// The effect, launched as the unit appears.
    AtOnce(u32),
}

/// The client's numbers of the effects the units here launch.
pub const EF_FIREWALL: u32 = 25;
pub const EF_ICEWALL: u32 = 74;
pub const EF_BENEDICTIO: u32 = 91;
pub const EF_QUAGMIRE: u32 = 95;
pub const EF_FIREPILLARBOMB: u32 = 97;
pub const EF_FIREPILLARON: u32 = 138;
pub const EF_PNEUMA: u32 = 141;
pub const EF_VENOMDUST2: u32 = 171;
pub const EF_BOTTOM_VO: u32 = 239;
pub const EF_BOTTOM_DE: u32 = 240;
pub const EF_BOTTOM_VI: u32 = 241;
pub const EF_BOTTOM_LA: u32 = 242;
pub const EF_READYPORTAL2: u32 = 316;
pub const EF_PORTAL2: u32 = 317;
pub const EF_GLASSWALL2: u32 = 315;
pub const EF_BOTTOM_MAG: u32 = 318;
pub const EF_BOTTOM_SANC: u32 = 319;
pub const EF_DEMONSTRATION: u32 = 302;
pub const EF_PORTAL3: u32 = 341;
pub const EF_BOTTOM_GOSPEL: u32 = 370;
pub const EF_BOTTOM_BASILICA: u32 = 374;
pub const EF_BOTTOM_SPIDER: u32 = 404;
pub const EF_BOTTOM_FOGWALL: u32 = 405;
pub const EF_BOTTOM_HERMODE: u32 = 517;
pub const EF_GRAVITATION: u32 = 522;
pub const EF_BOTTOM_SUITON: u32 = 620;
pub const EF_TATAMI: u32 = 631;
pub const EF_KAEN: u32 = 634;
/// `EF_LIGHTNING_S`, `EF_BLIND_S`, `EF_POISON_S`, `EF_FREEZING_S`, `EF_FLARE_S` are the four after it.
pub const EF_LIGHTNING_S: u32 = 638;

/// How many ticks an effect lives, for the effects launched again and again (the client's `EffectLifetime`).
fn lifetime(effect: u32) -> i32 {
    match effect {
        EF_FIREWALL => 40,
        EF_QUAGMIRE => 150,
        EF_READYPORTAL2 => 200,
        EF_KAEN | 638..=642 => 23,
        _ => 9999,
    }
}

/// What a kind of unit shows (`CSkill::OnProcess`; Pneuma in `CSkill::SendMsg`, `AM_SKILLSTANDENTRY`); nothing
/// for a kind not drawn here.
pub fn shows(unit: u32) -> Option<Shows> {
    let again = |effect: u32, early: i32| Shows::Again(effect, lifetime(effect) - early);
    Some(match unit {
        // UNT_SAFETYWALL, UNT_FIREWALL.
        126 => again(EF_GLASSWALL2, 7),
        127 => again(EF_FIREWALL, 7),
        // UNT_WARP_WAITING (the open portal), UNT_WARP_ACTIVE (the one about to open).
        128 => again(EF_PORTAL2, 8),
        129 => again(EF_READYPORTAL2, 8),
        // UNT_BENEDICTIO, UNT_SANCTUARY, UNT_MAGNUS, UNT_PNEUMA.
        130 => again(EF_BENEDICTIO, 8),
        131 => Shows::Kept(EF_BOTTOM_SANC),
        132 => Shows::Kept(EF_BOTTOM_MAG),
        133 => Shows::AtOnce(EF_PNEUMA),
        // UNT_FIREPILLAR_WAITING, UNT_FIREPILLAR_ACTIVE.
        135 => Shows::Kept(EF_FIREPILLARON),
        136 => again(EF_FIREPILLARBOMB, 8),
        // UNT_ICEWALL, UNT_QUAGMIRE, UNT_VENOMDUST.
        141 => Shows::Kept(EF_ICEWALL),
        142 => again(EF_QUAGMIRE, 20),
        146 => Shows::Kept(EF_VENOMDUST2),
        // UNT_SHOCKWAVE: its file over the trap, begun anew whenever it ends (`EF_SHOCKWAVE`'s case in
        // `CRagEffect::OnProcess`).
        traps::SHOCKWAVE => Shows::Again(traps::EF_SHOCKWAVE, traps::SHOCKWAVE_TICKS),
        // UNT_VOLCANO, UNT_DELUGE, UNT_VIOLENTGALE, UNT_LANDPROTECTOR.
        154 => Shows::Kept(EF_BOTTOM_VO),
        155 => Shows::Kept(EF_BOTTOM_DE),
        156 => Shows::Kept(EF_BOTTOM_VI),
        157 => Shows::Kept(EF_BOTTOM_LA),
        // UNT_DEMONSTRATION, UNT_CALLFAMILY, UNT_GOSPEL, UNT_BASILICA.
        177 => Shows::Kept(EF_DEMONSTRATION),
        178 => again(EF_PORTAL3, 8),
        179 => Shows::Kept(EF_BOTTOM_GOSPEL),
        180 => Shows::Kept(EF_BOTTOM_BASILICA),
        // UNT_FOGWALL, UNT_SPIDERWEB, UNT_GRAVITATION, UNT_HERMODE.
        182 => Shows::Kept(EF_BOTTOM_FOGWALL),
        183 => Shows::Kept(EF_BOTTOM_SPIDER),
        184 => Shows::Kept(EF_GRAVITATION),
        185 => Shows::Kept(EF_BOTTOM_HERMODE),
        // UNT_SUITON, UNT_TATAMIGAESHI, UNT_KAEN.
        187 => Shows::Kept(EF_BOTTOM_SUITON),
        188 => Shows::Kept(EF_TATAMI),
        189 => again(EF_KAEN, 1),
        // UNT_GROUNDDRIFT_WIND, _DARK, _POISON, _WATER, _FIRE.
        190..=194 => again(EF_LIGHTNING_S + unit - 190, 1),
        // UNT_DEATHWAVE and UNT_WATERATTACK show the ice of an Ice Wall, UNT_WINDATTACK the rings of a Fire
        // Pillar that waits.
        195 | 196 => Shows::Kept(EF_ICEWALL),
        197 => Shows::Kept(EF_FIREPILLARON),
        _ => return None,
    })
}

/// The sound a unit of a kind appears with (`AM_SKILLSTANDENTRY`): Fire Wall's always, Kaensin's fire (`UNT_KAEN`)
/// one time in eight; `lot` is any number, as the client's `rand()`.
pub fn appears_with(unit: u32, lot: u32) -> Option<&'static str> {
    match unit {
        127 => Some("effect/ef_firewall.wav"),
        189 if lot % 8 == 0 => Some("effect/화염진.wav"),
        _ => None,
    }
}

/// The sound an effect played from a file starts with by lot (the effect's case in `CRagEffect::OnProcess`):
/// Quagmire sounds one time in twelve; `lot` is any number, as the client's `rand()`. (A sound an effect
/// always starts is in the generated table, `crate::effect_tables::effect_sounds`.)
pub fn starts_with(effect: u32, lot: u32) -> Option<&'static str> {
    match effect {
        EF_QUAGMIRE if lot % 12 == 0 => Some("effect/wizard_quagmire.wav"),
        _ => None,
    }
}

/// The count of a unit that launches its effect again and again, as it appears (`CSkill::CSkill`).
pub const COUNT_AT_FIRST: i32 = 36000;

/// One tick of such a unit, its count at `count` and its effect due every `every` ticks: the count after the
/// tick, and whether the effect is launched at it.
pub fn tick(count: i32, every: i32) -> (i32, bool) {
    if count >= every { (1, true) } else { (count + 1, false) }
}

/// The effect the client plays at the place a skill of that level was used on
/// (`CGameActor::PlayGroundSkillEffect`, the skills of the classic jobs; with the client's option for fewer
/// effects off), and for how many ticks at most: its lifetime (`EffectLifetime`) and ten more
/// (`LaunchDummyEffect`), but for Gravitation Field's, which is told 260. An effect played from a file ends
/// with the file. (The newer skills, which the client's `skilleffectinfo` script names an effect for
/// (`CSession::PlaySkillGroundEffect`), are not here.)
pub fn ground_effect(skill: u16, level: i16) -> Option<(u32, i32)> {
    let effect = match skill {
        // MG_THUNDERSTORM, PR_BENEDICTIO, PR_SANCTUARY, PR_MAGNUS.
        21 => 30,
        69 => 91,
        70 => 83,
        79 => 113,
        // WZ_METEOR, WZ_VERMILION, WZ_ICEWALL, WZ_STORMGUST, WZ_HEAVENDRIVE, WZ_QUAGMIRE.
        83 => 92,
        85 => 90,
        87 => 74,
        89 => 89,
        91 => 142,
        92 => 95,
        // BS_HAMMERFALL, BS_ADRENALINE and BS_ADRENALINE2, HT_SKIDTRAP, HT_DETECTING, AS_VENOMDUST.
        110 => 102,
        111 | 459 => 98,
        115 => 69,
        130 => 119,
        140 => 124,
        // SA_VOLCANO, SA_DELUGE, SA_VIOLENTGALE, SA_LANDPROTECTOR.
        285 => 225,
        286 => 236,
        287 => 237,
        288 => 238,
        // CR_SLIMPITCHER: `EF_SLIM`, `EF_SLIM2` or `EF_SLIM3`, by its level.
        478 => match level {
            ..6 => 497,
            6..10 => 498,
            _ => 499,
        },
        // HW_GANBANTEIN, HW_GRAVITATION.
        483 => 223,
        484 => EF_GRAVITATION,
        _ => return None,
    };
    let lifetime = match effect {
        30 => 104,
        74 | 98 | 119 | 142 | 497..=499 => 250,
        95 => 150,
        223 | 225 => 300,
        83 | 236..=238 => 999,
        // (Not its lifetime, 2000: the 260 ticks the arm of HW_GRAVITATION writes out, less the ten.)
        EF_GRAVITATION => 250,
        _ => 9999,
    };
    Some((effect, lifetime + 10))
}

/// The effect `PlayGroundSkillEffect` launches on the one who used a skill on a place, not on the place:
/// `EF_BEGINSPELL6` for CR_CULTIVATION.
pub fn on_caster(skill: u16) -> Option<u32> {
    (skill == 491).then_some(58)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_shows_its_effect() {
        assert_eq!(shows(127), Some(Shows::Again(EF_FIREWALL, 33)));
        assert_eq!(shows(126), Some(Shows::Again(EF_GLASSWALL2, 9992)));
        assert_eq!(shows(142), Some(Shows::Again(EF_QUAGMIRE, 130)));
        assert_eq!(shows(131), Some(Shows::Kept(EF_BOTTOM_SANC)));
        assert_eq!(shows(133), Some(Shows::AtOnce(EF_PNEUMA)));
        // An open Warp Portal launches its effect anew 8 ticks before it would end, as one about to open does.
        assert_eq!((shows(128), shows(129)), (Some(Shows::Again(EF_PORTAL2, 9991)), Some(Shows::Again(EF_READYPORTAL2, 192))));
        assert_eq!((shows(154), shows(157)), (Some(Shows::Kept(EF_BOTTOM_VO)), Some(Shows::Kept(EF_BOTTOM_LA))));
        // A trap is a model, not an effect (`traps.rs`), but for the file over a Shockwave Trap; the kind 134
        // shows nothing in the client either.
        assert_eq!((shows(145), shows(134), shows(148)), (None, None, Some(Shows::Again(145, 75))));
        assert_eq!((appears_with(127, 3), appears_with(126, 0)), (Some("effect/ef_firewall.wav"), None));
        assert_eq!((appears_with(189, 16), appears_with(189, 17)), (Some("effect/화염진.wav"), None));
    }

    #[test]
    fn the_later_classic_kinds_show_what_the_client_launches() {
        // `CSkill::OnProcess`: kept for as long as the unit stands.
        let kept = [(177, 302), (179, 370), (180, 374), (182, 405), (183, 404), (184, 522), (185, 517), (187, 620), (188, 631)];
        for (unit, effect) in kept {
            assert_eq!(shows(unit), Some(Shows::Kept(effect)), "{unit}");
        }
        assert_eq!((shows(195), shows(196), shows(197)), (Some(Shows::Kept(74)), Some(Shows::Kept(74)), Some(Shows::Kept(138))));
        // Launched anew a tick before it would end (23 ticks), or 8 before.
        assert_eq!((shows(189), shows(178)), (Some(Shows::Again(634, 22)), Some(Shows::Again(341, 9991))));
        for (unit, effect) in [(190, 638), (191, 639), (192, 640), (193, 641), (194, 642)] {
            assert_eq!(shows(unit), Some(Shows::Again(effect, 22)), "{unit}");
        }
        // The songs and dances, Graffiti, Moonlit Water Mill, Kaensin and Evil Land have no arm there.
        for unit in (158..=176).chain([181, 186, 198, 199]) {
            assert_eq!(shows(unit), None, "{unit}");
        }
    }

    #[test]
    fn a_skill_used_on_a_place_plays_what_the_client_has_for_it() {
        // Every classic skill (to 544, and 1001 to 1019) with an arm of `PlayGroundSkillEffect` that launches
        // an effect on the place: the skill, its effect and the ticks `LaunchDummyEffect` is told.
        const ROWS: [(u16, u32, i32); 23] = [
            (21, 30, 114), (69, 91, 10009), (70, 83, 1009), (79, 113, 10009), (83, 92, 10009), (85, 90, 10009),
            (87, 74, 260), (89, 89, 10009), (91, 142, 260), (92, 95, 160), (110, 102, 10009), (111, 98, 260),
            (459, 98, 260), (115, 69, 10009), (130, 119, 260), (140, 124, 10009), (285, 225, 310), (286, 236, 1009),
            (287, 237, 1009), (288, 238, 1009), (478, 497, 260), (483, 223, 310), (484, 522, 260)
        ];
        for skill in (1..=544).chain(1001..=1019) {
            let row = ROWS.iter().find(|row| row.0 == skill).map(|row| (row.1, row.2));
            assert_eq!(ground_effect(skill, 1), row, "{skill}");
        }
        // Slim Potion Pitcher shows the potion of its level: red to level 5, yellow to 9, white at 10.
        assert_eq!([1, 5, 6, 9, 10].map(|level| ground_effect(478, level).unwrap().0), [497, 497, 498, 498, 499]);
        // Plant Cultivation shows its rings on the one who plants, and nothing at the place; Ground Drift and
        // Suiton only sound (`crate::effect_tables::skill_sounds`).
        assert_eq!((on_caster(491), ground_effect(491, 1), on_caster(83)), (Some(58), None, None));
        assert_eq!((ground_effect(518, 1), ground_effect(538, 1)), (None, None));
    }

    #[test]
    fn an_effect_is_launched_at_once_and_then_every_so_often() {
        let (mut count, mut launched) = (COUNT_AT_FIRST, Vec::new());
        for at in 0..70 {
            let (next, launch) = tick(count, 33);
            count = next;
            if launch {
                launched.push(at);
            }
        }
        // At once, the count being far up at first; then whenever it has gone up to that many again.
        assert_eq!(launched, vec![0, 33, 66]);
    }

    #[test]
    fn sounds_are_picked_by_lot() {
        assert_eq!((starts_with(EF_QUAGMIRE, 24), starts_with(EF_QUAGMIRE, 25)), (Some("effect/wizard_quagmire.wav"), None));
        assert_eq!(
            (ground_effect(70, 1), ground_effect(92, 1), ground_effect(87, 1), ground_effect(1, 1)),
            (Some((83, 1009)), Some((95, 160)), Some((74, 260)), None)
        );
    }
}
