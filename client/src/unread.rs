//! The packets of the map server that this client does not read yet, counted: a tool of this project's own for
//! whoever builds it (the original client has nothing of the kind), shown by the chat command `/packets`.
//! What rAthena calls each is `net/unread.rs`.

/// One kind of packet that came and was not read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unread {
    pub id: u16,
    /// How many of it came.
    pub count: u32,
    /// How long the last of them was, in bytes.
    pub length: usize,
}

/// The packets that were not read, a row to each number: the one that came last is first.
#[derive(Debug, Default)]
pub struct Tally {
    rows: Vec<Unread>,
}

impl Tally {
    /// One more packet of that number and length was not read; `true` if it is the first of its number.
    pub fn count(&mut self, id: u16, length: usize) -> bool {
        let known = self.rows.iter().position(|row| row.id == id).map(|at| self.rows.remove(at));
        self.rows.insert(0, Unread { id, count: known.map_or(0, |row| row.count) + 1, length });
        known.is_none()
    }

    /// The rows, the one that came last first.
    pub fn rows(&self) -> &[Unread] {
        &self.rows
    }

    /// How many packets were not read, of every number together.
    pub fn total(&self) -> u32 {
        self.rows.iter().map(|row| row.count).sum()
    }

    /// Everything counted is forgotten.
    pub fn clear(&mut self) {
        self.rows.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tally_counts_by_number_and_keeps_the_newest_first() {
        let mut tally = Tally::default();
        assert!(tally.count(0x0984, 30));
        assert!(tally.count(0x0b1d, 2));
        assert!(!tally.count(0x0984, 42));
        assert_eq!(tally.rows(), &[Unread { id: 0x0984, count: 2, length: 42 }, Unread { id: 0x0b1d, count: 1, length: 2 }]);
        assert_eq!(tally.total(), 3);
        tally.clear();
        assert_eq!(tally.total(), 0);
    }
}
