//! The lines of Frost Joke and Scream: the client's effects `EF_TALK_FROSTJOKE` (295) and `EF_TALK_SCREAM`
//! (296) draw nothing. At their first tick they tell the game (`CRagEffect::NotifyGameMode`, `MM_SONG_TALK`),
//! which draws a line by lot from a file of the client's data, `BA_frostjoke.txt` or `DC_scream.txt`, and says
//! it as the player would have typed it: it sends the server the line as plain talk (CZ_REQUEST_CHAT), and the
//! server has it shown over the head of the one who spoke and in the chat of everyone near. The client
//! launches these effects on the player's own character only (its handler of the skill used, for
//! `BA_FROSTJOKE` and `DC_SCREAM`).
//!
//! (Outside Korea the client turns the letters of what it reads round, `DecodeMessageRot`; the files of this
//! client are plain, as the Korean client reads them.)

/// The client's numbers of the two effects.
pub const EF_TALK_FROSTJOKE: u32 = 295;
pub const EF_TALK_SCREAM: u32 = 296;

/// The file an effect of talk draws its line from, among the client's data, and how many lines it draws
/// among (the client's `rand() % 117 + 1` and `rand() % 114 + 1`); nothing for any other effect.
pub fn table(effect: u32) -> Option<(&'static str, u32)> {
    match effect {
        EF_TALK_FROSTJOKE => Some(("data/BA_frostjoke.txt", 117)),
        EF_TALK_SCREAM => Some(("data/DC_scream.txt", 114)),
        _ => None,
    }
}

/// The line of that number in a table, as the client reads it: it counts the carriage returns, and the line
/// is what stands after that many of them and before the next, without tabs and ends of line. The numbers
/// drawn begin at 1, so what stands before the first carriage return, the file's title, is never said.
pub fn line(table: &str, number: u32) -> String {
    let (mut passed, mut line) = (0, String::new());
    for letter in table.chars() {
        if letter == '\r' {
            passed += 1;
        }
        if passed == number {
            if !matches!(letter, '\r' | '\n' | '\t') {
                line.push(letter);
            }
        } else if passed > number {
            break;
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_what_stands_after_so_many_carriage_returns() {
        let table = "FROST JOKE\r\n\tMe..me..mememe...\r\n\tOy~ It's cold.\r\n\tLast one\r\n";
        assert_eq!(line(table, 1), "Me..me..mememe...");
        assert_eq!(line(table, 2), "Oy~ It's cold.");
        assert_eq!(line(table, 3), "Last one");
        // Past the end of the file there is nothing to say.
        assert_eq!(line(table, 4), "");
        assert_eq!(line(table, 9), "");
    }

    #[test]
    fn each_effect_has_its_file() {
        assert_eq!(table(EF_TALK_FROSTJOKE), Some(("data/BA_frostjoke.txt", 117)));
        assert_eq!(table(EF_TALK_SCREAM), Some(("data/DC_scream.txt", 114)));
        assert_eq!(table(25), None);
    }
}
