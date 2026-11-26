//! The client's message box as rules (`UIMessageBox::Configure`, which `UIWindowMgr::Confirm` makes 280 by
//! 120 for every question of the windows): how its words are cut into lines and how many of them show.
//!
//! The box never grows with its words. They stand in a list of lines (`UITextViewer`, a `UIListBox`) 10 in
//! and 22 down, 20 narrower than the box and 55 lower: 260 by 65. A line is 18 high and begins 3 in and 4
//! down (`UITextViewer::UITextViewer`, `Paint`), so three show; with more the list has the client's bar at
//! its right edge (`UIListBox::Refresh`) and the wheel moves it three lines (`UIListBox::OnWheel`). The
//! words are cut by letters, not by their width: at the box's width over 7.5 (`UM_SETTEXT` with
//! `width / 7.5`, 37 for 280; `WrapRichText`).

/// How high a line of the box is, and how many of them it shows (65 / 18).
pub const LINE: i32 = 18;
pub const SHOWN: i32 = 3;

/// How many letters a line of a box so wide holds (`static_cast<int>(width / 7.5)`).
pub fn letters(width: i32) -> usize {
    (f64::from(width) / 7.5) as usize
}

/// Whether a colour of the client's markup (`^RRGGBB`) begins at `at` (`IsHexColorToken`).
fn colour_at(letters: &[char], at: usize) -> bool {
    letters.get(at) == Some(&'^') && letters.len() >= at + 7 && letters[at + 1..at + 7].iter().all(char::is_ascii_hexdigit)
}

/// Words cut into lines of `width` letters as the box cuts them (`WrapRichText`, without its carriage
/// returns, which a message box does not ask for): a line ends after the last blank or `]` before its end,
/// or at its end when it has neither; a colour (`^RRGGBB`) is not counted and a line may end after one; words
/// in square brackets go to the next line whole when they do not fit and are shorter than a line; a new line
/// in the words ends one. The lines keep their colours and blanks as they stand; the client counts bytes of
/// its code page where letters are counted here, which is the same for the Latin and the Cyrillic it writes.
/// (An item named by `^nItemID^` in the words is not looked for: no question of this client has one.)
pub fn lines(words: &str, width: usize) -> Vec<String> {
    let text: Vec<char> = words.chars().collect();
    let end = text.len();
    let is = |at: usize, letter: char| text.get(at) == Some(&letter);
    let (mut lines, mut begin, mut current, mut break_at, mut limit) = (Vec::new(), 0usize, 0usize, 0usize, width as isize);
    let mut push = |from: usize, to: usize| lines.push(text[from..to.min(end)].iter().filter(|letter| **letter != '\n').collect::<String>());
    while current < end {
        if colour_at(&text, current) {
            current += 7;
            limit += 7;
            break_at = current;
        } else {
            current += 1;
        }
        for letter in [' ', ']'] {
            while is(current, letter) {
                current += 1;
                break_at = current;
            }
        }
        while is(current, '[') {
            // How long the words in brackets are, the brackets with them; with no end they are taken to fit.
            let extent = text[current..].iter().position(|letter| *letter == ']').map(|length| length as isize + 1);
            match extent {
                Some(extent) if current as isize + extent >= limit => {
                    if extent >= width as isize || current == 0 {
                        break;
                    }
                    limit -= extent;
                    current -= 1;
                }
                _ => current += 1,
            }
        }
        if current as isize >= limit || is(current, '\n') {
            let split = if is(current, '\n') {
                current += 1;
                current
            } else {
                break_at
            };
            break_at = split;
            let cut = if split <= begin { current } else { split };
            push(begin, cut);
            begin = cut;
            limit = (cut + width) as isize;
        }
    }
    if begin < current {
        push(begin, current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_of_280_holds_37_letters_a_line() {
        assert_eq!(letters(280), 37);
    }

    #[test]
    fn words_are_cut_after_the_last_blank_before_the_end() {
        assert_eq!(lines("one two three four", 9), vec!["one two ", "three ", "four"]);
        // No blank: cut at the end.
        assert_eq!(lines("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
        assert_eq!(lines("one\ntwo", 37), vec!["one", "two"]);
        assert_eq!(lines("", 37), Vec::<String>::new());
    }

    #[test]
    fn the_question_of_deleting_a_homunculus_is_six_lines() {
        let words = "You have decided to delete this Homunculus. When deleted, the homunculus and its history will be \
                     deleted and they cannot be restored in the future. Would you like to continue?";
        let cut = lines(words, letters(280));
        assert_eq!(cut.len(), 6);
        assert!(cut.iter().all(|line| line.chars().count() <= 37), "{cut:?}");
        assert_eq!(cut.concat(), words);
        // Three more than the box shows: it gets its bar.
        assert!(cut.len() as i32 > SHOWN);
    }

    #[test]
    fn a_colour_is_not_counted() {
        // Nine letters and two colours: one line of nine.
        assert_eq!(lines("^ff0000one^000000 two th", 9), vec!["^ff0000one^000000 two ", "th"]);
        // A caret that is no colour is a letter.
        assert_eq!(lines("^zz one", 4), vec!["^zz ", "one"]);
    }

    #[test]
    fn words_in_brackets_stay_together() {
        // "[two three]" does not fit after "one " in a line of 12 and is shorter than a line: it goes down whole.
        // (A blank after the bracket begins the next line: the client looks for blanks before brackets.)
        assert_eq!(lines("one [two three] x", 12), vec!["one ", "[two three]", " x"]);
        // Longer than a line, they are cut as any words.
        assert_eq!(lines("[abcdefgh ijkl]", 6).concat(), "[abcdefgh ijkl]");
    }
}
