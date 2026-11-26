//! The chat window as rules: the sizes it steps through, how high it is at each, and how a line said is cut
//! into the rows of its list.

/// The chat's next size in rows, as the client steps it (`UM_TOGGLEPUSH`): one row, then three more each time
/// up to thirteen, then none.
pub fn rows_stepped(rows: usize) -> usize {
    let rows = if rows == 0 { 1 } else { rows + 3 };
    if rows > 15 { 0 } else { rows / 3 * 3 + 1 }
}

/// The chat's height with so many rows: the client's own sum (`UINewChatWnd::ResizeRows`), the input line, the
/// rows and three more, the tabs; with no rows the input line alone.
pub fn height(rows: usize) -> i32 {
    if rows == 0 { 24 } else { 24 + (rows as i32 + 3) * 14 + 16 }
}

/// How many letters a row of the chat's list holds at most (`UINewChatWnd::SendMsg`, its `UM_ADDCHAT`:
/// `WrapPlainText(.., 0x5E)`; the client counts the bytes of the server's writing, which for Latin and
/// Cyrillic letters are the letters).
pub const ROW: usize = 94;

/// How much narrower than its box the list writes a row that is too wide (`UIChatHisBox::AddTextWrapped`:
/// the room of its scroll bar).
pub const BAR: i32 = 16;

/// A line said, cut into the rows of the chat's list (`UM_ADDCHAT`): at the last blank before the
/// ninety-fourth letter, or at that letter where there is no blank (`WrapPlainText`). Each row is a line of
/// its own to the list: it is scrolled by rows. A row that is still wider than the box is cut again where
/// it is drawn, by its width (`held`). A line with a link to an item is left whole: the client cuts through
/// the link's own letters there and writes the pieces as they come, which is not followed.
pub fn rows(words: &str) -> Vec<String> {
    if words.contains("<ITEML>") {
        return vec![words.to_owned()];
    }
    let rows = crate::broadcast::wrapped(words, ROW);
    if rows.is_empty() { vec![words.to_owned()] } else { rows }
}

/// The longest thing in brackets that is kept together, in letters. (The client keeps one together while
/// it is no wider than the box, `bracketWidth <= width`; the page is not asked for widths here, and forty
/// letters fit the narrowest box a tab's own window has.)
const TOGETHER: usize = 40;

/// A row in the pieces that are cut by their width where the row is too wide for its box
/// (`UIWindow::WrapPixelText`): a row is cut between any two letters, but not inside what stands in
/// brackets (`[...]`), which goes to the next row whole. `true` with a piece that is kept together.
pub fn held(row: &str) -> Vec<(bool, &str)> {
    let (mut pieces, mut rest) = (Vec::new(), row);
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find(']').map(|close| open + close + 1) else { break };
        if rest[open..close].chars().count() > TOGETHER {
            // Too long to be kept: cut like the rest, and what follows its `[` is looked at again.
            pieces.push((false, &rest[..=open]));
            rest = &rest[open + 1..];
            continue;
        }
        if open > 0 {
            pieces.push((false, &rest[..open]));
        }
        pieces.push((true, &rest[open..close]));
        rest = &rest[close..];
    }
    if !rest.is_empty() {
        pieces.push((false, rest));
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_line_is_cut_into_rows_of_ninety_four_letters_at_blanks() {
        let said = "Currently, the Dungeon Teleport Service Provider Election is being held in all major cities. Your participation matters.";
        assert_eq!(rows(said), ["Currently, the Dungeon Teleport Service Provider Election is being held in all major cities.", "Your participation matters."]);
        assert!(rows(said).iter().all(|row| row.chars().count() <= ROW));
        // Without a blank it is cut at the letter.
        assert_eq!(rows(&"x".repeat(200)).iter().map(String::len).collect::<Vec<_>>(), [94, 94, 12]);
        // A short one, and one of nothing, stay one row.
        assert_eq!(rows("Hello : hi"), ["Hello : hi"]);
        assert_eq!(rows(""), [""]);
    }

    #[test]
    fn a_line_with_a_link_to_an_item_is_not_cut() {
        let said = format!("{} <ITEML>000021y&0g(00)00)00)00)00</ITEML> {}", "a ".repeat(40), "b ".repeat(40));
        assert_eq!(rows(&said), [said]);
    }

    #[test]
    fn what_stands_in_brackets_is_kept_together() {
        assert_eq!(held("to [ Some One ] : hello"), [(false, "to "), (true, "[ Some One ]"), (false, " : hello")]);
        assert_eq!(held("[a][b]"), [(true, "[a]"), (true, "[b]")]);
        assert_eq!(held("no bracket closes [ here"), [(false, "no bracket closes [ here")]);
        assert_eq!(held(""), []);
        // One too long to fit is cut like the rest.
        let long = format!("[{}]", "x".repeat(60));
        assert!(held(&long).iter().all(|(together, _)| !together));
        assert_eq!(held(&long).iter().map(|(_, piece)| *piece).collect::<String>(), long);
    }

    #[test]
    fn the_chat_is_as_high_as_its_rows_and_what_is_around_them() {
        assert_eq!([0, 1, 4, 7, 10, 13].map(height), [24, 96, 138, 180, 222, 264]);
    }

    #[test]
    fn the_chat_steps_through_its_sizes_and_away() {
        let mut rows = 0;
        let sizes: Vec<usize> = (0..7).map(|_| { rows = rows_stepped(rows); rows }).collect();
        assert_eq!(sizes, [1, 4, 7, 10, 13, 0, 1]);
    }
}
