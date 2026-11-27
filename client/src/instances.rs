//! The instances as rules: a copy of a dungeon made for one party (the client's "memorial dungeon":
//! `UIMemorialDunWnd` in `Source/UI/Shortcuts/UIShortCutWnd.cpp`, `CGameMode::ReceiveMemorialDungeonInfo`,
//! `ReceiveMemorialDungeonNotify`, `ReceiveMemorialDungeonSubscriptionInfo` and
//! `ReceiveMemorialDungeonSubscriptionNotify` in `Source/Client/Mode/GameModePackets.cpp`). What the window
//! says of one, when it comes back by itself, and how the client writes the time one ends at.

/// The window: 200 by 150, at the left edge 310 down until it is dragged (`UIWindowMgr::EnsureWindow`,
/// `WID_MEMORIALDUNWND`).
pub const SIZE: (i32, i32) = (200, 150);
pub const PLACE: (i32, i32) = (0, 310);

/// The line of the message table that is the window's title, "Memorial Dungeon".
pub const TITLE: usize = 1336;
/// "Your Standby Priority: %d", "The requested dungeon will be removed if you do not enter within %s.",
/// "Dungeon Mission Time Limit:", and how the client writes a time there ("%m/%d %H:%M", `MSI_TIME_TYPE2`).
pub const PRIORITY: usize = 1342;
pub const ENTER_WITHIN: usize = 1343;
pub const TIME_LIMIT: usize = 1344;
pub const TIME: usize = 1260;
/// "The Memorial Dungeon reservation has been canceled.", said when the place in the queue is given up.
pub const CANCELLED: usize = 1345;
/// The words on the leader's button, "Destroy instance" (the header's name for the line is another line's).
pub const DESTROY_WORDS: usize = 3270;
/// What CZ_MEMORIALDUNGEON_COMMAND carries to destroy the party's instance (`MM_MEMORIALDUNGEON_COMMAND`,
/// rAthena's `COMMAND_MEMORIALDUNGEON_DESTROY_FORCE`).
pub const DESTROY: u32 = 3;
/// The colour of what the chat says of an instance (the client's 0x64FFFF, blue first).
pub const SAID: &str = "#ffff64";

/// Where an instance is (the client's `memorialDungeonState`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Waiting for a free copy of the map: the place in the queue is told.
    Waiting = 0,
    /// Made, and nobody in it: it goes if nobody enters by a time.
    Ready = 1,
    /// Someone is in it: it lasts until a time.
    Running = 2,
}

/// The instance the party has, as the server last told of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    pub name: String,
    /// The place in the queue, while it waits.
    pub queue: i16,
    /// When it ends at the latest (the client's `memorialCreationTime`, which the server fills with that time),
    /// in seconds since 1970.
    pub limit: u32,
    /// When it goes if nobody has entered it by then (`memorialExpirationTime`); nothing while someone is in it.
    pub idle: u32,
    pub state: State,
}

/// Where an instance is by the time it goes with nobody in it (`ReceiveMemorialDungeonInfo`, `…Notify`): such
/// a time means it stands empty.
pub fn state_of(idle: u32) -> State {
    if idle != 0 { State::Ready } else { State::Running }
}

/// The line of the message table over the window for an instance in that state: "%s in Standby",
/// "%s Available", "%s in Progress" (`UIMemorialDunWnd::Paint`: the state added to line 1337).
pub fn headline(state: State) -> usize {
    1337 + state as usize
}

/// Whether the window comes back by itself when the server tells of the instance again: always, but for one
/// that runs as it ran before and whose window the player has closed (`ReceiveMemorialDungeonInfo`: a state
/// that changed, or one that is not the third, or `memorialWindowOpen`). One the client knows nothing of yet
/// counts as waiting, as the client's zero does.
pub fn shown_again(before: Option<State>, now: State, wanted: bool) -> bool {
    before.unwrap_or(State::Waiting) != now || now != State::Running || wanted
}

/// What the chat says as an instance ends, by the kind ZC_MEMORIALDUNGEON_NOTIFY tells
/// (`ReceiveMemorialDungeonNotify`): its time ran out, nobody entered it in time, it was removed. Any other
/// kind but the first closes the window and says nothing.
pub fn ended_line(kind: u32) -> Option<usize> {
    match kind {
        1 => Some(1346),
        2 => Some(1347),
        3 => Some(1348),
        _ => None,
    }
}

/// Whether the window has the button that destroys the instance: for the party's leader, while the instance
/// stands empty (`UIMemorialDunWnd::ArrangeButtons`).
pub fn destroy_offered(state: State, leads: bool) -> bool {
    leads && state == State::Ready
}

/// A time as the client writes it in the window: the pattern of the message table (`strftime`'s `%m`, `%d`,
/// `%H`, `%M`, each of two digits) filled with a month, a day, an hour and a minute of the player's own clock.
pub fn time_written(pattern: &str, (month, day): (u32, u32), (hour, minute): (u32, u32)) -> String {
    [("%m", month), ("%d", day), ("%H", hour), ("%M", minute)]
        .into_iter()
        .fold(pattern.to_owned(), |written, (mark, value)| written.replace(mark, &format!("{value:02}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instance_with_a_time_to_enter_by_stands_empty() {
        assert_eq!(state_of(1_795_800_000), State::Ready);
        assert_eq!(state_of(0), State::Running);
    }

    #[test]
    fn each_state_has_its_line_over_the_window() {
        assert_eq!([State::Waiting, State::Ready, State::Running].map(headline), [1337, 1338, 1339]);
    }

    /// `ReceiveMemorialDungeonInfo`: `previous != state || state != 2 || memorialWindowOpen`.
    #[test]
    fn a_window_closed_while_the_instance_runs_stays_closed() {
        assert!(!shown_again(Some(State::Running), State::Running, false));
        assert!(shown_again(Some(State::Running), State::Running, true));
        assert!(shown_again(Some(State::Ready), State::Running, false));
        assert!(shown_again(Some(State::Ready), State::Ready, false));
        assert!(shown_again(None, State::Running, false));
        assert!(shown_again(None, State::Waiting, false));
    }

    #[test]
    fn the_chat_says_why_an_instance_ended() {
        assert_eq!([0, 1, 2, 3, 4].map(ended_line), [None, Some(1346), Some(1347), Some(1348), None]);
    }

    #[test]
    fn only_the_leader_destroys_and_only_an_empty_instance() {
        assert!(destroy_offered(State::Ready, true));
        assert!(!destroy_offered(State::Ready, false));
        assert!(!destroy_offered(State::Running, true));
        assert!(!destroy_offered(State::Waiting, true));
    }

    #[test]
    fn a_time_is_written_by_the_table_s_pattern() {
        assert_eq!(time_written("%m/%d %H:%M", (3, 7), (9, 5)), "03/07 09:05");
        assert_eq!(time_written("%m/%d %H:%M", (11, 27), (20, 15)), "11/27 20:15");
    }
}
