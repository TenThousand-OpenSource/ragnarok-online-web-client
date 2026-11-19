//! The chat window as rules: the sizes it steps through, and how high it is at each.

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

#[cfg(test)]
mod tests {
    use super::*;

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
