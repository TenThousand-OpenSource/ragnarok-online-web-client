//! The action a monster is shown in as it uses a skill, where the client's table has one for it
//! (`data\MonsterSkillInfo.xml`, the gateway's `tables/monster_skills.rs`).
//!
//! The client asks the table as a skill of a monster lands (`CGameMode::ReceiveActorSkillAction` for
//! ZC_NOTIFY_SKILL, `ReceiveNotifySkill` for ZC_NOTIFY_SKILL2, rodecomp `Source/Client/Mode/GameModePackets.cpp`;
//! the one whose skill it is has been told to strike by then, `AM_SKILL`), by the name of the monster's job
//! in small letters and the skill's number (`CSession::GetMonsterSkillInfo`,
//! `Source/Contents/Monster/MonsterXmlActions.cpp`), and for what it finds calls
//! `SetAction(action * 8 - 8, 1, loopType == 0)`: the swing is replaced by the table's action, played once
//! through or round and round. A skill used on a place (ZC_NOTIFY_GROUNDSKILL) asks the table too and does
//! nothing with the answer; a skill that strikes nobody (ZC_USE_SKILL) does not ask.

use crate::pose::Run;

/// An action a monster is put in: the first of its eight directions, and how it goes on at its last
/// picture: `Run::Once` holds until the motion's time is up (`MT_ONETIME`), `Run::Loops` goes round until
/// the monster does something else (`MT_LOOP`, whose motion never finishes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    pub action: usize,
    pub run: Run,
}

/// What a monster of a job is shown in for a skill of its that landed. `own` is whether the one who used the
/// skill is the player's own character (the client then has no other actor to pose and leaves it), `listed`
/// the table's `action` and `loopType` for the monster's name and the skill. None for one that is no
/// monster (`CSession::IsMonsterActor`, `crate::jobs::monster`), for a skill the table does not have, and for an action under 1 (the table counts from 1; the
/// client would ask the sprite for an action that is not there).
pub fn shown(job: u16, own: bool, listed: Option<(i32, i32)>) -> Option<Shown> {
    let (action, loops) = listed.filter(|_| crate::jobs::monster(job) && !own)?;
    let action = usize::try_from(action).ok()?.checked_sub(1)? * 8;
    Some(Shown { action, run: if loops == 0 { Run::Once } else { Run::Loops } })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_monster_is_shown_in_the_tables_action_for_a_skill() {
        // Baphomet Jr. (1101) and skill 661: action 6, once. An Orc Warrior smoking (195): action 9, round.
        assert_eq!(shown(1101, false, Some((6, 0))), Some(Shown { action: 40, run: Run::Once }));
        assert_eq!(shown(1023, false, Some((9, 1))), Some(Shown { action: 64, run: Run::Loops }));
        // Most of the table: action 1, the monster stands instead of swinging.
        assert_eq!(shown(1275, false, Some((1, 0))), Some(Shown { action: 0, run: Run::Once }));
    }

    #[test]
    fn nothing_for_what_the_table_lacks_or_who_is_no_monster() {
        assert_eq!(shown(1101, false, None), None);
        assert_eq!(shown(1101, true, Some((6, 0))), None);
        // A player, a homunculus and a mercenary are no monsters.
        assert_eq!([12, 6001, 6017].map(|job| shown(job, false, Some((6, 0)))), [None, None, None]);
        assert_eq!(shown(1101, false, Some((0, 0))), None);
        assert_eq!(shown(1101, false, Some((-1, 0))), None);
    }
}
