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
/// A thing a member of the party obtained.
pub const PARTY_ITEM: u8 = 9;
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

    /// The red, green and blue of a colour kept as one number, `0xRRGGBB` (what an announcement and a script
    /// name a colour by).
    pub fn parts(colour: u32) -> [u8; 3] {
        [(colour >> 16) as u8, (colour >> 8) as u8, colour as u8]
    }

    /// A colour of three bytes as the page takes it, written out: `rgb(255,0,0)`.
    pub fn css([red, green, blue]: [u8; 3]) -> String {
        format!("rgb({red},{green},{blue})")
    }

    /// The red, green and blue of a colour as the page takes it, in each way one is written here: `#ff0000`,
    /// `#f00`, `rgb(255,0,0)`; none for what is none of these.
    pub fn read(colour: &str) -> Option<[u8; 3]> {
        let colour = colour.trim();
        if let Some(sixteens) = colour.strip_prefix('#') {
            let digit = |at: usize| sixteens.get(at..=at).and_then(|digit| u8::from_str_radix(digit, 16).ok());
            return match sixteens.len() {
                3 => Some([digit(0)? * 17, digit(1)? * 17, digit(2)? * 17]),
                6 => Some([digit(0)? * 16 + digit(1)?, digit(2)? * 16 + digit(3)?, digit(4)? * 16 + digit(5)?]),
                _ => None,
            };
        }
        let mut parts = colour.strip_prefix("rgb(")?.strip_suffix(')')?.split(',').map(|part| part.trim().parse::<u8>().ok());
        let read = [parts.next()??, parts.next()??, parts.next()??];
        parts.next().is_none().then_some(read)
    }

    /// Whether a colour is the pure red in which the client writes what went wrong (its `0x0000FF`), however
    /// it is spelt: every tab of the chat lists a line of no particular kind written in it
    /// (`crate::chat_tabs::Tab::lists`).
    pub fn pure_red(colour: &str) -> bool {
        read(colour) == Some([255, 0, 0])
    }

    /// A colour of three bytes as the page takes it, in sixteens: `#ff0000`, the way the colours above are
    /// written.
    pub fn hex([red, green, blue]: [u8; 3]) -> String {
        format!("#{red:02x}{green:02x}{blue:02x}")
    }
}

#[cfg(test)]
mod tests {
    use super::colour::*;

    #[test]
    fn a_colour_is_taken_apart_and_written_for_the_page() {
        assert_eq!(parts(0xff8000), [255, 128, 0]);
        assert_eq!(parts(0x12ff8000), [255, 128, 0]);
        assert_eq!(css([255, 128, 0]), "rgb(255,128,0)");
        assert_eq!((hex([255, 128, 0]), hex([0, 10, 255])), ("#ff8000".to_owned(), "#000aff".to_owned()));
        assert_eq!(hex(parts(0xff0000)), RED);
    }

    #[test]
    fn a_colour_is_read_however_it_is_spelt() {
        assert_eq!((read("#ff8000"), read("#F80"), read("rgb(255, 128,0)")), (Some([255, 128, 0]), Some([255, 136, 0]), Some([255, 128, 0])));
        assert_eq!((read("red"), read("#ff00"), read("#+f0000"), read("rgb(1,2)"), read("rgb(1,2,3,4)"), read("#яяя")), (None, None, None, None, None, None));
        // Red is red in every spelling, an announcement's (`css`) among them; a softer red is not.
        assert!(pure_red(RED) && pure_red("#f00") && pure_red("#FF0000") && pure_red(&css([255, 0, 0])) && pure_red(&hex([255, 0, 0])));
        assert!(!pure_red(PALE_RED) && !pure_red("#ff0001") && !pure_red(""));
    }
}
