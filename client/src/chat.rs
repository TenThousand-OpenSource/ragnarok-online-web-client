//! The chat window as rules: the sizes it steps through.

/// The chat's next size in rows, as the client steps it (`UM_TOGGLEPUSH`): one row, then three more each time
/// up to thirteen, then none.
pub fn rows_stepped(rows: usize) -> usize {
    let rows = if rows == 0 { 1 } else { rows + 3 };
    if rows > 15 { 0 } else { rows / 3 * 3 + 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chat_steps_through_its_sizes_and_away() {
        let mut rows = 0;
        let sizes: Vec<usize> = (0..7).map(|_| { rows = rows_stepped(rows); rows }).collect();
        assert_eq!(sizes, [1, 4, 7, 10, 13, 0, 1]);
    }
}
