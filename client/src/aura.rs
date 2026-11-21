//! The aura of the highest level: who shows which of the client's effects, as the client decides it. Nothing
//! here knows of the page: the world keeps the effects going by this rule (`web/world/aura.rs`), and the
//! effects themselves are `web/world/prims/max_level.rs`.
//!
//! Where each fact is from:
//!
//! - `CGameActor::UpdateMaxLevelAura` (`Contents/Actors/RenderObject.cpp`): nothing while the option that
//!   hides auras is on; then, by the job, the level is held against the highest level of that job's kind, and
//!   the actor is sent the message of that kind's aura. For every job of the classic game that level is
//!   `MaxLevelTable.BaseLevel` of the client's `ExternalSettings` and the message `AM_MAKE_LEVEL99_EFFECT`.
//! - `AM_MAKE_LEVEL99_EFFECT` (`CGameActor::SendMsg`, `Contents/Actors/GameObject.cpp`): nothing on one who
//!   hides, cloaks or is invisible (`StateFlag01`, `02`, `06`), nor on a monster; the two effects are 200 and
//!   976 unless the client's `Level99AuraTable` names others (for all, for the baby jobs, for the jobs over
//!   4000 and the Super Novice); the first is launched only with the full aura on (`/aura`), the second
//!   always.
//! - `CGameActor::SetStatusStates` and `ResetJobEffects`: the aura is taken away as one begins or ends
//!   hiding, cloaking or being invisible, and made anew if none of the three holds; `SetActorJob` and
//!   `CGameMode::RefreshActorJobEffects` (a level gained, an option switched) do the same. So at any moment
//!   the aura shows on exactly those this rule names, which is how the world keeps it.
//!
//! - `CGameSettingsMgr` (`Contents/Configuration/ServiceOptions.cpp`): `/aura2` switched takes every aura
//!   away and makes it anew (`ApplyServiceOptionActorRefresh`), and so does `/aura` while auras show; while
//!   they do not, `/aura` says so and does nothing more (`ApplyServiceOptionWithNotice`: `said_while_hidden`).
//!   Neither asks whether effects are on (`/effect`): with them off an aura lives on undrawn
//!   (`CEffect::UpdateVisibility`).
//!
//! What the client does for the later jobs and is not built here (they show no aura): the third jobs and
//! their mounts at `BaseLevel3rd` (the message 150: effects 1226 and 1227, or the table's `Default185LvAura`),
//! the fourth jobs at `BaseLevel4th` (`AM_MAKE_LEVEL4TH_EFFECT`: the table's `MaxLevelEffect4th` and its
//! `_sub`), the second Super Novice, the expanded jobs and the Doram at theirs (`AM_MAKE_LEVEL160_EFFECT`:
//! 1022 and 1023, or `Default160LvAura`), a homunculus at `BaseLevelHomun` (the message 147: effects 978 and
//! 979, or `HomunMaxLvAura`).

/// `EF_LEVEL99` and `EF_LEVEL99_ORB1`: the two effects the client's code launches when its table names none
/// (`int first = 200; int second = 0x3D0`).
pub const USUAL: [u32; 2] = [200, 976];

/// The bits of the effect state on which no aura is made: hiding, cloaking, being invisible (`StateFlag01`,
/// `StateFlag02`, `StateFlag06`).
const HIDDEN: u32 = 1 << 1 | 1 << 2 | 1 << 6;

/// What the client's `ExternalSettings` says of the aura: the highest base level (`MaxLevelTable.BaseLevel`)
/// and a homunculus's (`BaseLevelHomun`), and the pairs of effects of its `Level99AuraTable`, each the effect
/// and its `_sub`: for all (`Default99LvAura`), for the baby jobs (`Baby99LvAura`), for the jobs over 4000 and
/// the Super Novice (`SecondHigh99LvAura`). An effect of nothing is not named. (The table's `Homun99LvAura` is
/// read by the client and never comes to be used: a homunculus's job is over 4000 and takes the pair before
/// it.)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Table {
    pub highest: u16,
    pub homunculus: u16,
    pub plain: [u32; 2],
    pub baby: [u32; 2],
    pub high: [u32; 2],
}

impl Table {
    /// Every effect an aura may be by this table: what the world ends when an aura goes.
    pub fn effects(&self) -> Vec<u32> {
        let mut effects: Vec<u32> = USUAL.into_iter().chain([self.plain, self.baby, self.high].into_iter().flatten()).filter(|effect| *effect != 0).collect();
        effects.sort_unstable();
        effects.dedup();
        effects
    }
}

/// The player's two options of the aura as the client's code reads them: whether auras show at all (the
/// Game Settings' "Aura Display", `/aura2`: the client's `TT_HIDE_AURA_ON_OFF` at nothing) and whether the
/// aura is the full one (`/aura`: `TT_FULL_AURA_ON_OFF` not at nothing, which is the row "Aura Intensity"
/// unticked: `CGameSettingsMgr` keeps these two the other way round from their rows, `IsServiceOption`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    pub shown: bool,
    pub full: bool,
}

/// The client's names of the two options (`GameSettings.lub`): the row "Aura Display" (`/aura2`), ticked
/// while auras show, and the row "Aura Intensity" (`/aura`), ticked while the aura is the simple one.
pub const SHOWN_OPTION: &str = "TT_HIDE_AURA_ON_OFF";
pub const SIMPLE_OPTION: &str = "TT_FULL_AURA_ON_OFF";

/// The message the client says, and says alone, when the option of the full aura is switched while auras do
/// not show (`shown`: the row "Aura Display" as it is ticked), by a command or by its row: the option is
/// switched, nothing is made anew and the option's own message is not said (`ApplyServiceOptionWithNotice`,
/// `Contents/Configuration/ServiceOptions.cpp`: the line 2996 of the message table, in `0x64FFFF`). Nothing
/// for any other switch.
pub fn said_while_hidden(option: &str, shown: bool) -> Option<&'static str> {
    (option == SIMPLE_OPTION && !shown).then_some("MSI_ALREADY_AURA_EFFECT_OFF")
}

/// `CSession::IsMonsterClass`.
fn monster(job: u16) -> bool {
    (job > 1000 && job < 3999) || (20021..=31999).contains(&job)
}

/// `CSession::IsMercenaryJob`: the homunculi (6001 to 6016, and the later ones 6048 to 6052, which are
/// `IsNewMercenaryJob`).
fn homunculus(job: u16) -> bool {
    matches!(job, 6001..=6016 | 6048..=6052)
}

/// `CSession::IsBabyJob`, as far as the classic game has baby jobs: 4023 to 4045 (from the Baby Novice to the
/// Super Baby). The later baby jobs are among `later`.
fn baby(job: u16) -> bool {
    (4023..=4045).contains(&job)
}

/// The jobs the classic game does not have, from the Rune Knight (4054) up to the homunculi: the third and
/// fourth jobs, the expanded ones, the Doram, the jobs on the mounts of a later time and the later baby jobs.
/// For most of them `UpdateMaxLevelAura` holds the level against another highest level and sends another
/// message (the module's comment lists them); none of that is built, and none of them shows an aura here.
fn later(job: u16) -> bool {
    (4054..6001).contains(&job)
}

/// The effects of the aura that show on one of that job and level in that effect state: the first of the
/// pair (with the full aura only) and its `_sub`. Nothing while the table is not known (`highest` of nothing).
pub fn shown(job: u16, level: u16, state: u32, table: &Table, options: Options) -> [Option<u32>; 2] {
    const NONE: [Option<u32>; 2] = [None, None];
    // `UpdateMaxLevelAura`.
    if !options.shown || table.highest == 0 || later(job) {
        return NONE;
    }
    if homunculus(job) {
        // (At a homunculus's own highest level the client sends the message 147, a later aura: not built.)
        if level == table.homunculus || level != table.highest || job >= 6048 {
            return NONE;
        }
    } else if level != table.highest {
        return NONE;
    }
    // `AM_MAKE_LEVEL99_EFFECT`. (It also names the jobs 1324 to 1363, 1938 to 1955 and 1732, which are
    // monsters' already.)
    if state & HIDDEN != 0 || monster(job) {
        return NONE;
    }
    let or = |pair: [u32; 2], named: [u32; 2]| [if named[0] > 0 { named[0] } else { pair[0] }, if named[1] > 0 { named[1] } else { pair[1] }];
    let mut pair = or(USUAL, table.plain);
    if baby(job) {
        pair = or(pair, table.baby);
    } else if job > 4000 || job == 23 {
        pair = or(pair, table.high);
    } else if job >= 1638 {
        return NONE;
    }
    [options.full.then_some(pair[0]), Some(pair[1])]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table of the client of 2026: 99 the highest, the baby jobs with the second kind of lights.
    const TABLE: Table = Table { highest: 99, homunculus: 275, plain: [200, 976], baby: [200, 977], high: [0, 0] };
    const SIMPLE: Options = Options { shown: true, full: false };
    const FULL: Options = Options { shown: true, full: true };

    #[test]
    fn one_of_the_highest_level_shows_the_lights_and_with_the_full_aura_the_rings() {
        // A Knight of 99: the lights alone as the client starts, both with `/aura`.
        assert_eq!(shown(7, 99, 0, &TABLE, SIMPLE), [None, Some(976)]);
        assert_eq!(shown(7, 99, 0, &TABLE, FULL), [Some(200), Some(976)]);
        // Not at 98, and not with the auras hidden.
        assert_eq!(shown(7, 98, 0, &TABLE, FULL), [None, None]);
        assert_eq!(shown(7, 99, 0, &TABLE, Options { shown: false, full: true }), [None, None]);
        // A Lord Knight and a Super Novice take the table's pair for the high jobs, which names none.
        assert_eq!(shown(4008, 99, 0, &TABLE, FULL), [Some(200), Some(976)]);
        assert_eq!(shown(23, 99, 0, &TABLE, FULL), [Some(200), Some(976)]);
        assert_eq!(shown(4008, 99, 0, &Table { high: [0, 977], ..TABLE }, FULL), [Some(200), Some(977)]);
    }

    #[test]
    fn a_baby_job_shows_its_own_lights() {
        assert_eq!(shown(4023, 99, 0, &TABLE, SIMPLE), [None, Some(977)]);
        assert_eq!(shown(4045, 99, 0, &TABLE, FULL), [Some(200), Some(977)]);
    }

    #[test]
    fn one_who_hides_cloaks_or_is_invisible_shows_none() {
        for bit in [1, 2, 6] {
            assert_eq!(shown(12, 99, 1 << bit, &TABLE, FULL), [None, None]);
        }
        // Any other state leaves it: a cart, a falcon, a mount.
        assert_eq!(shown(12, 99, 1 << 3 | 1 << 4 | 1 << 5, &TABLE, SIMPLE), [None, Some(976)]);
    }

    #[test]
    fn monsters_and_the_later_jobs_show_none() {
        // A Poring of level 99, a monster of the later numbers, the jobs between 1638 and 4000.
        assert_eq!(shown(1002, 99, 0, &TABLE, FULL), [None, None]);
        assert_eq!(shown(20100, 99, 0, &TABLE, FULL), [None, None]);
        assert_eq!(shown(3999, 99, 0, &TABLE, FULL), [None, None]);
        // A Rune Knight at 99 and at its own highest level: the later auras are not built.
        assert_eq!(shown(4054, 99, 0, &TABLE, FULL), [None, None]);
        assert_eq!(shown(4054, 200, 0, &TABLE, FULL), [None, None]);
    }

    #[test]
    fn a_homunculus_of_the_highest_level_shows_it_too() {
        assert_eq!(shown(6001, 99, 0, &TABLE, SIMPLE), [None, Some(976)]);
        // Not one of the later homunculi, nor one at a homunculus's own highest level.
        assert_eq!(shown(6048, 99, 0, &TABLE, SIMPLE), [None, None]);
        assert_eq!(shown(6001, 99, 0, &Table { homunculus: 99, ..TABLE }, SIMPLE), [None, None]);
    }

    #[test]
    fn the_full_aura_switched_while_auras_are_hidden_says_so_and_nothing_else_does() {
        assert_eq!(said_while_hidden(SIMPLE_OPTION, false), Some("MSI_ALREADY_AURA_EFFECT_OFF"));
        // The line the client's code names by its number, 2996 (`commands.txt` has the client's names).
        assert_eq!(crate::commands::message("MSI_ALREADY_AURA_EFFECT_OFF"), Some(2996));
        assert_eq!(said_while_hidden(SIMPLE_OPTION, true), None);
        assert_eq!((said_while_hidden(SHOWN_OPTION, false), said_while_hidden("TT_EFFECT_ON_OFF", false)), (None, None));
    }

    #[test]
    fn nothing_shows_until_the_table_is_known() {
        assert_eq!(shown(0, 0, 0, &Table::default(), FULL), [None, None]);
        // Its effects, each once: those the code names and those the table adds.
        assert_eq!(TABLE.effects(), vec![200, 976, 977]);
        assert_eq!(Table::default().effects(), vec![200, 976]);
    }
}
