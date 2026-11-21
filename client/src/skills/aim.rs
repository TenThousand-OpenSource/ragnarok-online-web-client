//! A skill aimed with the pointer: which skills are of help, whom the pointer picks first while one is aimed,
//! and whether the skill goes on the one pressed.
//!
//! All of it is the client's, and all of it turns on Shift:
//!
//! - `IsSupportSkill` (`Source/Contents/Skills/SkillInfo.cpp:690`) names the skills of help;
//! - `CGameActor::SetRenderInfo` (`Source/Contents/Actors/RenderObject.cpp:869`) ranks those under the pointer:
//!   a monster before an NPC before a player, or, with a skill of help aimed and Shift up (`g_pickInvert`,
//!   set by `MM_HANG_USE_SKILL`), or Shift held (`g_pickModifierKey`) and none aimed, a player before a
//!   monster before an NPC;
//! - `CGameMode::HandleActorPick` and `ProcessActorClick` (`Source/Client/Mode/GameMode.cpp:10499, 8132`) use a
//!   skill aimed at someone on a monster with Shift up, a skill of help on a monster with Shift held, and the
//!   other way round on a player; with the option No Shift (`/noshift`, `TT_NOSHIFT_ON_OFF`) on, always. The
//!   pointer is the skill's second picture (the cursor's action 11) over one the skill would go on, and its
//!   first (10) over anyone else.

/// The skills of help, by number: the cases of the client's `IsSupportSkill`, written out of the decompiled
/// switch. (Past them the client asks the skill's own class, `CSkillActionMgr::IsSupportAction`, which only
/// the skills of the jobs after the classic game have; those are not here.)
const SUPPORT: [u16; 142] = [
    4, 28, 29, 34, 35, 53, 54, 66, 67, 68, 71, 72, 73, 108, 138, 139, 231, 234, 235, 236, 237, 255, 256, 262, 280,
    281, 282, 283, 361, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 460, 461, 462, 463,
    464, 465, 479, 494, 564, 572, 573, 574, 575, 577, 593, 621, 622, 626, 636, 637, 641, 642, 643, 645, 646, 688,
    689, 690, 691, 694, 729, 1015, 2001, 2012, 2026, 2041, 2042, 2043, 2050, 2051, 2052, 2053, 2275, 2287, 2322,
    2341, 2345, 2347, 2348, 2383, 2421, 2425, 2451, 2480, 2482, 2515, 2537, 2596, 2597, 2598, 2599, 2608, 2612,
    3025, 3026, 3027, 3031, 3032, 3033, 3034, 5035, 5038, 5039, 5040, 5041, 5064, 5268, 5269, 5271, 5272, 5275,
    5280, 5281, 5282, 5298, 5299, 5339, 5366, 5368, 8021, 8226, 8227, 8228, 8229, 8230, 8231, 8238, 8239, 8240,
];

/// Whether a skill is one of help (Heal, Blessing, Increase Agility, Cure, Kyrie Eleison…; not Decrease
/// Agility, Turn Undead or Lex Aeterna, which are aimed as an attack is).
pub fn support(skill: u16) -> bool {
    SUPPORT.binary_search(&skill).is_ok()
}

/// What stands under the pointer, as the client ranks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Under {
    Monster,
    /// An NPC, a homunculus or a mercenary's kind of creature that is no monster.
    Npc,
    Player,
    /// A pet (the client's object of the kind 7).
    Pet,
}

/// Whether players are picked before monsters: with a skill of help aimed and Shift up, or with Shift held
/// and no skill of help aimed (`g_pickInvert` and `g_pickModifierKey`).
pub fn players_first(support_aimed: bool, shift: bool) -> bool {
    support_aimed != shift
}

/// The rank of one under the pointer: of those the pointer is on, the highest is picked (and of the same
/// rank the nearest).
pub fn rank(under: Under, players_first: bool) -> u32 {
    match (under, players_first) {
        (Under::Pet, _) => 50,
        (Under::Monster, false) | (Under::Player, true) => 400,
        (Under::Npc, false) | (Under::Monster, true) => 200,
        (Under::Player, false) | (Under::Npc, true) => 100,
    }
}

/// Whether the skill aimed at someone goes on a monster pressed: an attack with Shift up, a skill of help
/// with Shift held; either with No Shift on.
pub fn goes_on_monster(support: bool, shift: bool, no_shift: bool) -> bool {
    no_shift || support == shift
}

/// Whether it goes on another player pressed: a skill of help with Shift up, an attack with Shift held;
/// either with No Shift on, or on a player who is fought on this map.
pub fn goes_on_player(support: bool, shift: bool, no_shift: bool, fought: bool) -> bool {
    no_shift || fought || support != shift
}

/// Whether a skill that waits to be used is asked of the server now: not while the character is in the motion
/// of a blow it deals. The client holds the skill it was told to use (`AM_PROCEEDSKILL`, the pending action 3,
/// which takes the place of a pending attack) from frame to frame until its body stands, walks or flinches
/// (`CPlayer::UpdatePendingAction`, `PT_SKILL` and `PT_GROUNDSKILL`, `Source/Contents/Actors/Player.cpp:2973,
/// 3019`: `state != ST_MOVE && state != ST_STAND && state != ST_DAMAGE && state < 16` returns), and only then
/// looks whether it is near enough. A server refuses a skill asked for during the blow (rAthena's
/// `canact_tick`, "Cannot use the skill due to the delay"). `now` and `striking_until` are moments of one
/// clock, in milliseconds: the blow's motion lasts until the second.
///
/// Not the client's: `AFTER_BLOW`. The client's attack state ends when its picture of the blow has run out,
/// which is not the server's `amotion` to the millisecond; here the motion is taken as `amotion` whole, and
/// rAthena lets a player act again `amotion` and half the fastest blow's motion after a blow
/// (`unit_set_attackdelay`: `status_get_amotion + pc_maxaspd / AMOTION_DIVIDER_PC`, 50 ms with `max_aspd`
/// 190), so a skill asked for as the motion ends was refused every time. The wait is that and a frame more.
pub fn asked_now(now: f64, striking_until: f64) -> bool {
    now >= striking_until + AFTER_BLOW
}

/// See `asked_now`.
const AFTER_BLOW: f64 = 70.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_skills_of_help_are_the_clients() {
        assert!(SUPPORT.is_sorted());
        // Heal, Increase Agility, Blessing, Cure, Resurrection, Kyrie Eleison, Potion Pitcher.
        assert!([28, 29, 34, 35, 54, 73, 231].into_iter().all(support));
        // Decrease Agility, Turn Undead, Lex Aeterna, Lex Divina, Holy Light, Bash: aimed as attacks.
        assert!(![30, 77, 78, 76, 156, 5].into_iter().any(support));
    }

    #[test]
    fn a_skill_waits_for_the_blow_under_way_to_end() {
        assert!(!asked_now(1000.0, 1300.0) && !asked_now(1300.0, 1300.0));
        assert!(asked_now(1370.0, 1300.0) && asked_now(1000.0, 0.0));
    }

    #[test]
    fn heal_goes_on_a_monster_with_shift_held_or_no_shift_on() {
        assert!(!goes_on_monster(true, false, false));
        assert!(goes_on_monster(true, true, false));
        assert!(goes_on_monster(true, false, true));
        // An attack is the other way round.
        assert!(goes_on_monster(false, false, false) && !goes_on_monster(false, true, false) && goes_on_monster(false, true, true));
        // And a player the other way round from a monster.
        assert!(goes_on_player(true, false, false, false) && !goes_on_player(true, true, false, false));
        assert!(!goes_on_player(false, false, false, false) && goes_on_player(false, true, false, false));
        assert!(goes_on_player(false, false, false, true) && goes_on_player(false, false, true, false));
    }

    #[test]
    fn a_skill_of_help_picks_a_player_before_a_monster_until_shift_is_held() {
        let first = |support, shift| {
            let friends = players_first(support, shift);
            [Under::Monster, Under::Npc, Under::Player, Under::Pet].into_iter().max_by_key(|under| rank(*under, friends)).unwrap()
        };
        assert_eq!(first(false, false), Under::Monster);
        assert_eq!(first(true, false), Under::Player);
        assert_eq!(first(true, true), Under::Monster);
        assert_eq!(first(false, true), Under::Player);
        assert_eq!(rank(Under::Monster, true), 200);
        assert_eq!(rank(Under::Npc, true), 100);
    }
}
