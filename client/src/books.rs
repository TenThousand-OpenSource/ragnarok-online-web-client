//! The books that are read in a window of their own (the client's `UIBookWnd`): which items are books, how the
//! text of one is cut into lines and pages, which page it opens at, how its pages are turned, and how it is
//! read aloud line by line.
//!
//! A book is a file of the client's data, `data\book\<the item's number>.txt` (`CGameMode::ReceiveBookOpen`,
//! `UIBookWnd::SendMsg` at `UM_GETBOOKCONTENTS`): a text whose first line may name the colour of its paper as
//! `%rrggbb`, and in which `^rrggbb` changes the colour of what follows. The client cuts it into lines no
//! wider than its viewer less 10 (`UIWindow::WrapPixelText`) and shows 19 of them a page.

use crate::markup::{Piece, pieces};

/// How many lines a page has (`UIBookWnd::SendMsg`: `m_lineCount / 19`).
pub const LINES: usize = 19;

/// The items that are books, out of the client's `data\bookitemnametable.txt` (`IsBookItem`): a number and a
/// `#` to a line.
pub fn listed(table: &str) -> Vec<u32> {
    table.lines().filter_map(|line| line.split('#').next()?.trim().parse().ok()).collect()
}

/// What a digit of a colour stands for (the client's `g_hexDigits`; anything that is no digit is taken here
/// as 0).
fn digit(letter: Option<&char>) -> u32 {
    letter.and_then(|letter| letter.to_digit(16)).unwrap_or(0)
}

/// The three parts of a colour written as six digits from `at` on.
fn colour(letters: &[char], at: usize) -> [u8; 3] {
    [0, 2, 4].map(|part| (digit(letters.get(at + part)) * 16 + digit(letters.get(at + part + 1))) as u8)
}

/// How many letters the next line takes of what is left, and whether it ends at the end of a line of the
/// text: one row of `UIWindow::WrapPixelText`. Letter by letter the widths are added up; the line ends before
/// the letter that makes it wider than `width`, with no regard to words. A `^rrggbb` takes no room: the
/// width of `^000000` is taken off again for a code that is a colour. What stands in square brackets is not
/// cut: a line ends before a `[` whose brackets would not fit on it but fit on a line of their own.
fn row(rest: &[char], width: f64, code: f64, wide: &mut dyn FnMut(&str) -> f64) -> (usize, bool) {
    let mut extent = 0.0;
    let mut letter = [0u8; 4];
    for (at, current) in rest.iter().enumerate() {
        // The client gives up on a line after 1000 letters and takes what it has.
        if at > 1000 {
            return (at, false);
        }
        let own = wide(current.encode_utf8(&mut letter));
        extent += own;
        match current {
            // A carriage return with its line feed, or either alone (and a line feed after a line feed, as
            // the client has it), ends the line.
            '\r' | '\n' => return (at + 1 + usize::from(rest.get(at + 1) == Some(&'\n')), true),
            '^' if colour(rest, at + 1) != [0, 0, 0]
                || rest.get(at..at + 7).is_some_and(|black| black.iter().collect::<String>() == "^000000") => extent -= code,
            '[' => {
                if let Some(close) = rest[at..].iter().position(|close| *close == ']') {
                    let bracketed = wide(&rest[at..=at + close].iter().collect::<String>());
                    if extent + bracketed - own > width && bracketed <= width {
                        return (at, false);
                    }
                }
            }
            _ => {}
        }
        if extent > width {
            return (at, false);
        }
    }
    (rest.len(), false)
}

/// A text cut into lines no wider than `width`, as `UIWindow::WrapPixelText` cuts it; `wide` tells how wide
/// a piece of text is in the font the lines are written in. An empty line of the text is a line of one
/// space. (The client's own lines keep the carriage return a line of the file ends with; here a line is kept
/// without it.) A letter wider than the whole width ends the cutting, as in the client.
pub fn wrapped(text: &str, width: f64, wide: &mut dyn FnMut(&str) -> f64) -> Vec<String> {
    let letters: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    if letters.is_empty() {
        return lines;
    }
    let code = wide("^000000");
    let mut offset = 0;
    // No more than 1000 lines, as the client has it.
    while offset < letters.len() && lines.len() <= 1000 {
        let (count, ended) = row(&letters[offset..], width, code, wide);
        if count == 0 {
            break;
        }
        let line: String = letters[offset..offset + count].iter().filter(|letter| !matches!(letter, '\r' | '\n')).collect();
        lines.push(if ended && line.is_empty() { " ".to_owned() } else { line });
        offset += count;
    }
    lines
}

/// A book as its window keeps it: its lines, and the colour of its paper where its text names one.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Book {
    /// The colour the text's first line names (`%rrggbb`); a book without one has the skin's own
    /// (`CSkinMgr::PaletteChannels(2, 2)`).
    pub paper: Option<[u8; 3]>,
    pub lines: Vec<String>,
}

impl Book {
    /// The text of a book's file cut into lines (`UM_GETBOOKCONTENTS`): a text that begins with `%` names its
    /// paper's colour there, and that first line is not one of the book's.
    pub fn bound(text: &str, width: f64, wide: &mut dyn FnMut(&str) -> f64) -> Book {
        let mut lines = wrapped(text, width, wide);
        let paper = text.starts_with('%').then(|| colour(&text.chars().take(7).collect::<Vec<char>>(), 1));
        if paper.is_some() && !lines.is_empty() {
            lines.remove(0);
        }
        Book { paper, lines }
    }

    /// How many pages it has: 19 lines to a page, and one more for what is left.
    pub fn pages(&self) -> usize {
        self.lines.len().div_ceil(LINES)
    }

    /// The lines of a page, counted from 1 (`UM_READBOOK`). Page 0, which the client is left at when it is
    /// asked for a page the book does not have, has none here (the client reads before its first line there).
    pub fn page(&self, page: usize) -> &[String] {
        let from = page.saturating_sub(1) * LINES;
        if page == 0 || from >= self.lines.len() { &[] } else { &self.lines[from..(from + LINES).min(self.lines.len())] }
    }

    /// The page a book is opened at (`UM_READBOOK`, `UM_AUTOREAD`): the one asked for, for 0 the one the
    /// bookmark was left at (the first where there is none: `UIBookWnd::LoadBookmark`), and 0 for a page past
    /// the last.
    pub fn opened_at(&self, asked: usize, marked: Option<usize>) -> usize {
        let page = if asked == 0 { marked.unwrap_or(1) } else { asked };
        if page > self.pages() { 0 } else { page }
    }

    /// The page before (`ID_PREVIEW`: the first stays the first) or after (`ID_NEXTVIEW`: the last stays the
    /// last). From page 0 the client's arithmetic goes to a page before the first; here the first is shown.
    pub fn turned(&self, page: usize, forward: bool) -> usize {
        let page = if forward {
            if page == self.pages() { page } else { page + 1 }
        } else {
            page.saturating_sub(1).max(1)
        };
        if page > self.pages() { 0 } else { page }
    }

    /// The line reading aloud begins at for a page (`UM_AUTOREAD`: `page * 19 - 19`, and the first line when
    /// that is past the last). For page 0 the client's figure is before the first line; here it is the first.
    pub fn aloud_from(&self, page: usize) -> usize {
        let line = page.saturating_sub(1) * LINES;
        if line > self.lines.len() { 0 } else { line }
    }

    /// The line said next when reading aloud has come to `line`, and the line it comes to after
    /// (`UIBookWnd::OnProcess`): an empty line that is not the last is passed over, one of them at a time;
    /// nothing once the lines are at an end, when the reading is over.
    pub fn aloud(&self, line: usize) -> Option<(&str, usize)> {
        let empty = |line: &String| line.is_empty() || line == " ";
        let said = line + usize::from(self.lines.get(line).is_some_and(empty) && line + 1 < self.lines.len());
        Some((self.lines.get(said)?.as_str(), said + 1))
    }
}

/// The lines of a page, each beginning with the colour the lines before it left the text in: the client's
/// viewer writes a page from black and carries a `^rrggbb` over the ends of its lines (`UITextViewer::Paint`).
pub fn carried(lines: &[String]) -> Vec<String> {
    let mut colour: Option<String> = None;
    lines
        .iter()
        .map(|line| {
            let written = format!("{}{line}", colour.as_deref().unwrap_or_default());
            // The last colour the line names is the one it leaves the text in.
            let named = pieces(line, &[]).into_iter().rev().find_map(|piece| if let Piece::Colour(code) = piece { Some(code) } else { None });
            if let Some(code) = named {
                colour = Some(format!("^{code}"));
            }
            written
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every letter 10 wide.
    fn tens(text: &str) -> f64 {
        text.chars().count() as f64 * 10.0
    }

    fn cut(text: &str, width: f64) -> Vec<String> {
        wrapped(text, width, &mut tens)
    }

    #[test]
    fn the_table_of_books_is_a_number_to_a_line() {
        assert_eq!(listed("11000#\r\n11001#\r\n\r\n7127#\r\n"), [11000, 11001, 7127]);
        assert!(listed("").is_empty());
    }

    #[test]
    fn a_text_is_cut_at_its_line_ends_and_an_empty_line_is_a_space() {
        assert_eq!(cut("one\r\n\r\ntwo\nthree", 1000.0), ["one", " ", "two", "three"]);
        // A line feed after a line feed goes with it, as the client has it.
        assert_eq!(cut("a\n\nb", 1000.0), ["a", "b"]);
        assert!(cut("", 100.0).is_empty());
    }

    #[test]
    fn a_line_too_wide_is_cut_before_the_letter_that_does_not_fit_whatever_the_word() {
        assert_eq!(cut("abcdefg", 30.0), ["abc", "def", "g"]);
        assert_eq!(cut("ab cd", 30.0), ["ab ", "cd"]);
        // A letter wider than the whole width ends the cutting.
        assert!(cut("abc", 5.0).is_empty());
    }

    #[test]
    fn a_colour_takes_no_room_and_brackets_are_not_cut() {
        assert_eq!(cut("^ff0000abc^000000d", 40.0), ["^ff0000abc^000000d"]);
        // What is no colour is as wide as it is written.
        assert_eq!(cut("^zzzzzzab", 40.0), ["^zzz", "zzza", "b"]);
        assert_eq!(cut("ab [cd] e", 60.0), ["ab ", "[cd] e"]);
        // Brackets wider than a line are cut as anything else.
        assert_eq!(cut("[abcdefgh]", 60.0), ["[abcde", "fgh]"]);
    }

    #[test]
    fn a_book_names_its_paper_in_its_first_line() {
        let book = Book::bound("%dcdcdc \r\n^000088Title^000000\r\n\r\ntext", 1000.0, &mut tens);
        assert_eq!(book.paper, Some([0xdc, 0xdc, 0xdc]));
        assert_eq!(book.lines, ["^000088Title^000000", " ", "text"]);
        let plain = Book::bound("text", 1000.0, &mut tens);
        assert_eq!((plain.paper, plain.lines.len()), (None, 1));
    }

    fn of(lines: usize) -> Book {
        Book { paper: None, lines: (0..lines).map(|line| line.to_string()).collect() }
    }

    #[test]
    fn nineteen_lines_make_a_page() {
        assert_eq!([of(0).pages(), of(1).pages(), of(19).pages(), of(20).pages(), of(259).pages()], [0, 1, 1, 2, 14]);
        let book = of(40);
        assert_eq!(book.page(1).len(), 19);
        assert_eq!(book.page(2)[0], "19");
        assert_eq!(book.page(3), ["38", "39"]);
        assert!(book.page(0).is_empty() && book.page(4).is_empty());
    }

    #[test]
    fn a_book_opens_at_the_page_asked_for_or_at_its_bookmark() {
        let book = of(40);
        assert_eq!(book.opened_at(2, Some(3)), 2);
        assert_eq!(book.opened_at(0, Some(3)), 3);
        assert_eq!(book.opened_at(0, None), 1);
        // A page the book does not have is page 0.
        assert_eq!(book.opened_at(4, None), 0);
        assert_eq!(book.opened_at(0, Some(9)), 0);
        assert_eq!(of(0).opened_at(1, None), 0);
    }

    #[test]
    fn the_first_page_and_the_last_stay_when_turned_past() {
        let book = of(40);
        assert_eq!([book.turned(1, false), book.turned(2, false), book.turned(2, true), book.turned(3, true)], [1, 1, 3, 3]);
        assert_eq!(book.turned(0, true), 1);
        assert_eq!(of(0).turned(0, true), 0);
    }

    #[test]
    fn reading_aloud_passes_one_empty_line_and_ends_with_the_lines() {
        let book = Book { paper: None, lines: ["a", " ", " ", "b", " "].map(str::to_owned).to_vec() };
        assert_eq!(book.aloud(0), Some(("a", 1)));
        assert_eq!(book.aloud(1), Some((" ", 3)));
        assert_eq!(book.aloud(3), Some(("b", 4)));
        // The last line is said though it is empty.
        assert_eq!(book.aloud(4), Some((" ", 5)));
        assert_eq!(book.aloud(5), None);
        assert_eq!([of(40).aloud_from(1), of(40).aloud_from(2), of(40).aloud_from(0), of(10).aloud_from(3)], [0, 19, 0, 0]);
    }

    #[test]
    fn a_colour_is_carried_over_the_end_of_a_line() {
        let lines = ["plain", "^ff0000red", "still red^000000 black", "black"].map(str::to_owned);
        assert_eq!(carried(&lines), ["plain", "^ff0000red", "^ff0000still red^000000 black", "^000000black"]);
    }
}
