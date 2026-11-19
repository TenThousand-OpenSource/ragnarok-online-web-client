//! The marks of the minimap that the server tells of, as the client takes them: where the boss monster of a
//! map is marked and what the chat says of it (`CGameMode::ReceiveBossMonsterInfo`). The packets are
//! `net/marks.rs`.

use crate::net::marks::Boss;

impl Boss {
    /// Where the minimap marks it: only one that lives has a place (`CGameMode::ReceiveBossMonsterInfo` sets
    /// the place to none first, whatever is told).
    pub fn marked(&self) -> Option<(u16, u16)> {
        matches!(self.kind, 1 | 2).then_some((u16::try_from(self.cell.0).ok()?, u16::try_from(self.cell.1).ok()?))
    }

    /// What the chat says of it, in white: the line of the message table and what is written into it. None
    /// here (1223); its place will be shown (1225, said for kind 2 only); it comes within a minute (1261,
    /// when the time told is 0 hours and 1 minute) or in so many hours and minutes (1224).
    pub fn says(&self) -> Option<(usize, Vec<String>)> {
        Some(match self.kind {
            0 => (1223, Vec::new()),
            2 => (1225, vec![self.name.clone()]),
            3 if self.hours == 0 && self.minutes == 1 => (1261, vec![self.name.clone()]),
            3 => (1224, vec![self.name.clone(), self.hours.to_string(), self.minutes.to_string()]),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn told(kind: u8, hours: u16, minutes: u16) -> Boss {
        Boss { kind, cell: (120, 85), hours, minutes, name: "Baphome".to_owned() }
    }

    #[test]
    fn a_boss_that_lives_is_marked() {
        let boss = told(1, 0, 0);
        assert_eq!((boss.marked(), boss.says()), (Some((120, 85)), None));
        assert_eq!(told(2, 0, 0).says(), Some((1225, vec!["Baphome".to_owned()])));
        // A cell that is no cell of a map has no place (decided by the width of the number alone).
        assert_eq!(Boss { cell: (70_000, 85), ..told(1, 0, 0) }.marked(), None);
        assert_eq!(told(2, 0, 0).marked(), Some((120, 85)));
    }

    #[test]
    fn a_dead_boss_tells_when_it_comes_again() {
        let dead = told(3, 2, 30);
        assert_eq!(
            (dead.marked(), dead.says()),
            (None, Some((1224, vec!["Baphome".to_owned(), "2".to_owned(), "30".to_owned()])))
        );
        assert_eq!(told(3, 0, 1).says().map(|said| said.0), Some(1261));
        assert_eq!(told(0, 0, 0).says(), Some((1223, Vec::new())));
        assert_eq!((told(0, 0, 0).marked(), told(4, 0, 0).says()), (None, None));
    }
}
