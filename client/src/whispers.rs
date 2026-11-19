//! The rules of a talk with one player in a window of its own (the client's `UIWhisperWnd`): what the window is
//! called, where a new one stands, how a line is written in it and in the chat, and what a command typed into
//! it does. Its drawing is `web/hud/whispers.rs`.

/// How many such windows may be open at once. The client has no such limit (it makes a window for every name);
/// here each window is an entry of the list of windows, which is as long as it is written.
pub const WINDOWS: usize = 8;

/// The window's size, and where the client first opens it (`UIM_MAKE_WHISPER_WINDOW`, `WhisperWndInfo::Reset`).
pub const SIZE: (i32, i32) = (280, 120);
const FIRST: (i32, i32) = (0, 347);

/// The colours of a talk's lines (`UM_ADDCHAT`): the player's own in yellow, the other's pale blue.
pub const OWN: &str = crate::said::colour::YELLOW;
pub const THEIRS: &str = "#b6dde8";

/// What the server answers to a whisper (the client's `ACKWHISPER_*`; read by `crate::net::whispers::whisper_answered`).
pub mod answer {
    pub const ARRIVED: u8 = 0;
    /// Nobody of that name is in the game.
    pub const ABSENT: u8 = 1;
    /// That player turns the player's whispers away, or everyone's.
    pub const REFUSED: u8 = 2;
    pub const REFUSES_ALL: u8 = 3;
}

/// The line of the client's message table said after the name, in brackets, of one a whisper did not reach
/// (`CGameMode::ReceiveWhisperResult`).
pub fn failure(result: u8) -> usize {
    match result {
        answer::REFUSED => 149,
        answer::REFUSES_ALL => 150,
        _ => 148,
    }
}

/// What the window is called (`UIWhisperWnd::Paint` and `UIM_MAKE_WHISPER_WINDOW`): `With  name`, and for a
/// friend or one of the guild what they are to the player after it.
pub fn title(name: &str, what: Option<&str>) -> String {
    match what {
        Some(what) => format!("With  {name}  ({what})  *^_^*"),
        None => format!("With  {name}"),
    }
}

/// Where a new window stands on a screen of that size when so many are open already
/// (`UIM_MAKE_WHISPER_WINDOW`): 17 right and down of the first for each, while it fits; else as far from the
/// corner of the screen, while that fits; else where the first stands.
pub fn place(open: usize, view: (i32, i32)) -> (i32, i32) {
    let offset = open as i32 * 17;
    let fits = |x: i32, y: i32| x > 0 && SIZE.0 + x < view.0 - 17 && y > 0 && SIZE.1 + y < view.1 - 17;
    if open != 0 && fits(FIRST.0 + offset, FIRST.1 + offset) {
        (FIRST.0 + offset, FIRST.1 + offset)
    } else if fits(offset, offset) {
        (offset, offset)
    } else {
        FIRST
    }
}

/// A line of a talk as its window shows it: who said it, and what (`ProcessWhisper`, `ReceiveWhisperResult`).
pub fn line(who: &str, words: &str) -> String {
    format!("{who} : {words}")
}

/// A whisper sent, as the chat shows it when no window takes it (`ReceiveWhisperResult`): in square brackets to
/// a friend or one of the guild. The client writes a code of the other's character after the name; its making
/// was not found, and it is left out.
pub fn sent(to: &str, close: bool, words: &str) -> String {
    if close { format!("[ To {to} ] : {words}") } else { format!("( To {to} ) : {words}") }
}

/// A whisper heard, as the chat shows it when no window takes it (`ProcessWhisper`): `from` is the word From
/// in the player's language. The same code is left out here.
pub fn heard(name: &str, friend: bool, guild: bool, from: &str, words: &str) -> String {
    if friend {
        format!("[ Friend {name} ] : {words}")
    } else if guild {
        format!("[ Member {name} ] : {words}")
    } else {
        format!("( {from} {name} ) : {words}")
    }
}

/// What a line typed into a talk's window is (`UIWhisperWnd::SendMsg`, `ID_OK`): one that begins with a slash
/// is a command, of which `/q` closes the window (the client's `TT_EX_EXIT_CHAT_ROOM`); anything else is said
/// to the other.
#[derive(Debug, PartialEq)]
pub enum Typed<'a> {
    Nothing,
    Close,
    Command(&'a str),
    Said(&'a str),
}

pub fn typed(words: &str) -> Typed<'_> {
    let words = words.trim();
    if words.is_empty() {
        Typed::Nothing
    } else if !words.starts_with('/') {
        Typed::Said(words)
    } else if crate::commands::split(words).and_then(|(command, _)| crate::commands::asks(command)) == Some("TT_EX_EXIT_CHAT_ROOM") {
        Typed::Close
    } else {
        Typed::Command(words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_is_called_by_whom_it_is_with() {
        assert_eq!(title("Somebody", None), "With  Somebody");
        assert_eq!(title("Somebody", Some("Friend")), "With  Somebody  (Friend)  *^_^*");
    }

    #[test]
    fn windows_stand_a_step_from_each_other() {
        assert_eq!(place(0, (1024, 768)), (0, 347));
        assert_eq!(place(1, (1024, 768)), (17, 364));
        assert_eq!(place(3, (1024, 768)), (51, 398));
        // Too far down the screen: as far from its corner instead.
        assert_eq!(place(17, (1024, 768)), (289, 289));
        // Nowhere it fits: where the first stands.
        assert_eq!(place(40, (1024, 768)), (0, 347));
    }

    #[test]
    fn lines_are_written_as_the_client_writes_them() {
        assert_eq!(line("Me", "hello"), "Me : hello");
        assert_eq!(sent("Other", false, "hello"), "( To Other ) : hello");
        assert_eq!(sent("Other", true, "hello"), "[ To Other ] : hello");
        assert_eq!(heard("Other", false, false, "From", "hi"), "( From Other ) : hi");
        assert_eq!(heard("Other", true, true, "From", "hi"), "[ Friend Other ] : hi");
        assert_eq!(heard("Other", false, true, "From", "hi"), "[ Member Other ] : hi");
    }

    #[test]
    fn an_answer_says_why_a_whisper_did_not_arrive() {
        assert_eq!((failure(answer::ABSENT), failure(answer::REFUSED), failure(answer::REFUSES_ALL), failure(9)), (148, 149, 150, 148));
    }

    #[test]
    fn a_line_typed_is_said_or_done() {
        assert_eq!(typed("  "), Typed::Nothing);
        assert_eq!(typed(" hello there "), Typed::Said("hello there"));
        assert_eq!(typed("/q"), Typed::Close);
        assert_eq!(typed("/Q"), Typed::Close);
        assert_eq!(typed("/who"), Typed::Command("/who"));
    }
}
