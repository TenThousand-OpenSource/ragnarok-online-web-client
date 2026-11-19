//! The lines of the chat window: the kinds the client gives them, by which the window's tabs list them or not,
//! and the colours it writes them in. A system that says something in the chat names its kind and its colour
//! from here; a colour only one line of the game has is written where that line is said.

/// Anything of no particular kind: most of what the game itself says. The kinds are the numbers the client's
/// code gives them (the last but two of what it sends with `UIM_PUSHINTOCHATHISTORY`).
pub const GENERAL: u8 = 0;
/// What someone says aloud.
pub const CHAT: u8 = 1;
pub const WHISPER: u8 = 2;
/// What the party says to itself.
pub const PARTY: u8 = 3;
/// What the guild says to itself, and what is said of the guild.
pub const GUILD: u8 = 4;
/// A thing obtained.
pub const ITEM: u8 = 6;
/// What is worn: put on, taken off, refused.
pub const EQUIPMENT: u8 = 7;
/// What a state does to the character: what Gospel gave, too much carried.
pub const STATUS: u8 = 8;
/// A skill that failed.
pub const SKILL_FAILED: u8 = 11;
/// Something worn that was damaged.
pub const DAMAGED: u8 = 13;
/// A line of the system's among the announcements (the tag `ssss`).
pub const SYSTEM: u8 = 14;
/// Experience gained or lost.
pub const EXPERIENCE: u8 = 18;
/// An announcement of the server's: every tab of the chat lists it.
pub const ANNOUNCED: u8 = 25;

/// The colours the client writes its own lines in, as the page takes them (the client's `0xBBGGRR` turned).
pub mod colour {
    /// What went wrong, or was refused.
    pub const RED: &str = "#ff0000";
    /// What the game tells of: a thing obtained, a quest, a whisper sent.
    pub const YELLOW: &str = "#ffff00";
    /// What went well: a deal made, someone asked or joined.
    pub const CYAN: &str = "#00ffff";
    pub const WHITE: &str = "#ffffff";
    /// A softer red: what cannot be done just now.
    pub const PALE_RED: &str = "#ff8484";
    /// A softer yellow: what a switch was set to, what the mail says.
    pub const PALE_YELLOW: &str = "#ffff64";
    /// A softer cyan: what a command answers, what a pet was fed.
    pub const PALE_CYAN: &str = "#64ffff";
}
