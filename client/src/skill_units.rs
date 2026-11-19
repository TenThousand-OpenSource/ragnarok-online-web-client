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
//! The kinds of the first and second jobs are here as far as their effects are drawn; `shows` says nothing for
//! the others, which then show no effect:
//!
//! - Graffiti (176), which comes in a packet of its own (ZC_SKILL_ENTRY2) and of which only the words are
//!   shown, over the unit (`web/world/written.rs`);
//! - the kind 134, which the server shows instead of a unit that is not to be seen and for which the client
//!   has nothing either; and every kind of the later jobs, most of which the client hands to its
//!   `CSkillActionMgr`.
//!
//! The traps (143 to 153) the client shows as models standing on their cells, not as effects: `traps.rs` has
//! which model each is, how it moves and what it launches when it springs. A Shockwave Trap has an effect
//! over it besides.
//!
//! The server also tells of a skill used on a place (ZC_NOTIFY_GROUNDSKILL): the client then plays an effect at
//! that place (`CGameActor::PlayGroundSkillEffect`); `ground_effect` has those of the first and second jobs.

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

/// How many ticks an effect lives, for the effects launched again and again (the client's `EffectLifetime`).
fn lifetime(effect: u32) -> i32 {
    match effect {
        EF_FIREWALL => 40,
        EF_QUAGMIRE => 150,
        EF_READYPORTAL2 => 200,
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
        _ => return None,
    })
}

/// The sound a unit of a kind appears with (`AM_SKILLSTANDENTRY`): Fire Wall's.
pub fn appears_with(unit: u32) -> Option<&'static str> {
    (unit == 127).then_some("effect/ef_firewall.wav")
}

/// The file an effect plays when its function picks one of several by lot (`SelectRagEffectResource`): `lot`
/// is any number, as the client's `rand()`. Nothing for an effect with one file or none. (Fire Wall's two
/// files are picked where every effect of that sort is, whoever launches it: `crate::effect::played`; this
/// one is only launched by its unit.)
pub fn picked_file(effect: u32, lot: u32) -> Option<String> {
    match effect {
        EF_PNEUMA => Some(format!("pneuma{}.str", lot % 3 + 1)),
        _ => None,
    }
}

/// The sound an effect played from a file starts with (the effect's case in `CRagEffect::OnProcess`). Quagmire
/// sounds one time in twelve: `lot` is any number, as the client's `rand()`.
pub fn starts_with(effect: u32, lot: u32) -> Option<&'static str> {
    match effect {
        EF_BENEDICTIO => Some("effect/priest_benedictio.wav"),
        EF_FIREPILLARBOMB => Some("effect/wizard_fire_pillar_b.wav"),
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

/// The effect the client plays at the place a skill was used on (`CGameActor::PlayGroundSkillEffect`, the
/// skills of the first and second jobs; with the client's option for fewer effects off), and for how many
/// ticks at most: its lifetime (`EffectLifetime`) and ten more (`LaunchDummyEffect`). An effect played from a
/// file ends with the file.
pub fn ground_effect(skill: u16) -> Option<(u32, i32)> {
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
        // BS_HAMMERFALL, BS_ADRENALINE, HT_SKIDTRAP, HT_DETECTING, AS_VENOMDUST.
        110 => 102,
        111 => 98,
        115 => 69,
        130 => 119,
        140 => 124,
        // SA_VOLCANO, SA_DELUGE, SA_VIOLENTGALE, SA_LANDPROTECTOR.
        285 => 225,
        286 => 236,
        287 => 237,
        288 => 238,
        _ => return None,
    };
    let lifetime = match effect {
        30 => 104,
        74 | 98 | 119 | 142 => 250,
        95 => 150,
        225 => 300,
        83 | 236..=238 => 999,
        _ => 9999,
    };
    Some((effect, lifetime + 10))
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
        assert_eq!((appears_with(127), appears_with(126)), (Some("effect/ef_firewall.wav"), None));
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
    fn files_and_sounds_are_picked_by_lot() {
        assert_eq!(picked_file(EF_FIREWALL, 5), None);
        assert_eq!(picked_file(EF_PNEUMA, 3).as_deref(), Some("pneuma1.str"));
        assert_eq!(picked_file(EF_QUAGMIRE, 0), None);
        assert_eq!((starts_with(EF_QUAGMIRE, 24), starts_with(EF_QUAGMIRE, 25)), (Some("effect/wizard_quagmire.wav"), None));
        assert_eq!((ground_effect(70), ground_effect(92), ground_effect(87), ground_effect(1)), (Some((83, 1009)), Some((95, 160)), Some((74, 260)), None));
    }
}
