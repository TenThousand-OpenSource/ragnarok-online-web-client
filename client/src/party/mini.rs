//! The small windows of single members of the party (the client's `UIMiniPartyWnd`): the places on the screen
//! they stand in, which of them a new one takes, where one let go lands, their order, and the text they are
//! kept as between visits.
//!
//! The client lays a grid over the whole screen, of cells 177 by 55 (`g_miniPartyCellWidth`,
//! `g_miniPartyCellHeight`), numbered down each column and then across (`CMiniPartyWndAreaMgr`); a window is
//! 174 by 52 and stands at the corner of its cell. Which members have a window, and in which cell, it keeps in
//! `savedata\MiniPartyInfo.lua`, a line `{name,1,account,cell}` to a window, under the name of the character
//! whose windows they are (`CSession::SaveMiniPartyInfo`, `LoadMiniPartyInfo`).

/// A cell of the grid, and a window in one.
pub const CELL: (i32, i32) = (177, 55);
pub const SIZE: (i32, i32) = (174, 52);

/// The grid of a screen: how many columns and how many rows of cells it has (`BuildAvailableCells`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    pub columns: i32,
    pub rows: i32,
    /// Where a new window may be put, and where the windows are put in order: from this far down the screen
    /// to three cells above its bottom (`UIWindowMgr::GetMiniPartyUsableArea`).
    top: i32,
    bottom: i32,
}

impl Grid {
    /// The grid of a screen of that size. `menu` is how tall the strip of menu icons is: new windows go
    /// under that height (the client takes the strip's height alone, not where it ends; without the strip,
    /// three tenths of the screen's).
    pub fn of(screen: (i32, i32), menu: Option<i32>) -> Grid {
        Grid {
            columns: screen.0 / CELL.0,
            rows: screen.1 / CELL.1,
            top: menu.unwrap_or((screen.1 as f32 * 0.3) as i32),
            bottom: screen.1 - CELL.1 * 3,
        }
    }

    /// Where the window of a cell stands (`GetCellPosition`); nowhere for a cell the grid has not.
    pub fn place(&self, cell: i32) -> Option<(i32, i32)> {
        (self.rows > 0 && (0..self.columns * self.rows).contains(&cell)).then(|| (CELL.0 * (cell / self.rows), CELL.1 * (cell % self.rows)))
    }

    /// The cells new windows are put in and windows are sorted into, in their order: those wholly between
    /// the top and the bottom of the usable part (`CMiniPartyWndSortArea`).
    fn usable(&self) -> impl Iterator<Item = i32> + '_ {
        (0..self.columns * self.rows).filter(|cell| self.place(*cell).is_some_and(|(_, y)| self.top <= y && y + CELL.1 <= self.bottom))
    }

    /// The cell a new window takes: the first usable one that has none (`FindFreeCell`).
    pub fn free(&self, taken: &[i32]) -> Option<i32> {
        self.usable().find(|cell| !taken.contains(cell))
    }

    /// The cell a window let go with its corner there lands in: the nearest, past the middle of a cell
    /// counting for the next, within the grid (`FindCell`).
    pub fn at(&self, x: i32, y: i32) -> Option<i32> {
        let nearest = |length: i32, cell: i32, most: i32| (length.div_euclid(cell) + i32::from(f64::from(length.rem_euclid(cell)) > f64::from(cell) * 0.5)).clamp(0, most - 1);
        (self.columns > 0 && self.rows > 0).then(|| self.rows * nearest(x, CELL.0, self.columns) + nearest(y, CELL.1, self.rows))
    }
}

/// The small windows there are: whose (the member's account) and in which cell, in the order they were made.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Windows(pub Vec<(u32, i32)>);

impl Windows {
    pub fn has(&self, account: u32) -> bool {
        self.0.iter().any(|(of, _)| *of == account)
    }

    fn cells(&self) -> Vec<i32> {
        self.0.iter().map(|(_, cell)| *cell).collect()
    }

    /// A member is given a window, in the first free cell (the menu's "Add small party window",
    /// `OpenMiniPartyWnd(FindMiniPartyCell(), …)`): not one who has one, and not with no cell free (the
    /// client makes a window even then, in a cell of no place, which is never seen).
    pub fn add(&mut self, account: u32, grid: &Grid) -> bool {
        let free = grid.free(&self.cells()).filter(|_| !self.has(account));
        self.0.extend(free.map(|cell| (account, cell)));
        free.is_some()
    }

    /// A member's window is taken away (`CloseMiniPartyWindow`).
    pub fn remove(&mut self, account: u32) -> bool {
        let before = self.0.len();
        self.0.retain(|(of, _)| *of != account);
        self.0.len() != before
    }

    /// Only the windows of those still in the party stay (`RebuildMiniPartyWindows`).
    pub fn keep(&mut self, members: &[u32]) -> bool {
        let before = self.0.len();
        self.0.retain(|(of, _)| members.contains(of));
        self.0.len() != before
    }

    /// A member's window was let go with its corner at that place (`UIMiniPartyWnd::OnLBtnUp`): it takes the
    /// nearest cell, and a window that is there takes the cell it came from.
    pub fn dropped(&mut self, account: u32, at: (i32, i32), grid: &Grid) {
        let (Some(to), Some(from)) = (grid.at(at.0, at.1), self.0.iter().find(|(of, _)| *of == account).map(|(_, cell)| *cell)) else { return };
        for (of, cell) in &mut self.0 {
            if *of == account {
                *cell = to;
            } else if *cell == to {
                *cell = from;
            }
        }
    }

    /// The windows are put in order (the button "Sort Mini Party Window", `RearrangeMiniPartyWindows`): each,
    /// in the order they were made, into the next usable cell; those there is no cell for stay.
    pub fn sort(&mut self, grid: &Grid) {
        for ((_, cell), usable) in self.0.iter_mut().zip(grid.usable()) {
            *cell = usable;
        }
    }

    /// The windows as they are kept (`SaveMiniPartyInfo`): a line `{name,1,account,cell}` to each, `name` the
    /// character's whose they are; nothing when there are none.
    pub fn written(&self, name: &str) -> Option<String> {
        (!self.0.is_empty()).then(|| self.0.iter().map(|(account, cell)| format!("{{{name},1,{account},{cell}}}\n")).collect())
    }

    /// The windows of that character read back from what is kept (`LoadMiniPartyInfo`); the lines of another
    /// character, and what cannot be read, are passed by.
    pub fn read(kept: &str, name: &str) -> Windows {
        let line = |line: &str| {
            let mut fields = line.trim().strip_prefix('{')?.strip_suffix('}')?.rsplitn(4, ',');
            let (cell, account, active, of) = (fields.next()?, fields.next()?, fields.next()?, fields.next()?);
            (of == name && active == "1").then_some((account.parse().ok()?, cell.parse().ok()?))
        };
        let mut windows = Windows::default();
        for (account, cell) in kept.lines().filter_map(line) {
            if !windows.has(account) {
                windows.0.push((account, cell));
            }
        }
        windows
    }
}

/// What a small window writes of its member (`UIMiniPartyInfoWnd::Paint`: `"Lv.%d %s(%s)"`), `map` being what
/// the client calls the map the member is on.
pub fn label(level: u16, name: &str, map: &str) -> String {
    format!("Lv.{level} {name}({map})")
}

/// How much of a gauge so wide is filled for that much life left of that most (`UIPcGage::Paint`: the whole
/// hundredths of the life left, of the width, less 2), and whether in the colour of little life (under a
/// quarter). Nothing of it is filled with no life at all.
pub fn gauge(width: i32, left: i32, most: i32) -> (i32, bool) {
    if most <= 0 {
        return (0, false);
    }
    let percent = (i64::from(left.max(0)) * 100 / i64::from(most)) as i32;
    (if percent == 0 { 0 } else { (width * percent / 100 - 2).clamp(0, width - 2) }, percent < 25)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The client's own screen, 1024 by 768, with a strip of menu icons 100 tall.
    fn grid() -> Grid {
        Grid::of((1024, 768), Some(100))
    }

    #[test]
    fn the_grid_is_numbered_down_each_column() {
        let grid = grid();
        assert_eq!((grid.columns, grid.rows), (5, 13));
        assert_eq!(grid.place(0), Some((0, 0)));
        assert_eq!(grid.place(2), Some((0, 110)));
        assert_eq!(grid.place(13), Some((177, 0)));
        assert_eq!(grid.place(65), None);
        assert_eq!(grid.place(-1), None);
        // Without the strip of icons new windows go under three tenths of the screen.
        assert_eq!(Grid::of((1024, 768), None).free(&[]), Some(5));
    }

    #[test]
    fn a_new_window_takes_the_first_free_cell_under_the_menu() {
        let grid = grid();
        // Rows 2 to 9 of each column are usable: from 110 down to 603 at most.
        assert_eq!(grid.free(&[]), Some(2));
        assert_eq!(grid.free(&[2, 3]), Some(4));
        assert_eq!(grid.free(&[2, 3, 4, 5, 6, 7, 8, 9]), Some(15));
        let mut windows = Windows::default();
        assert!(windows.add(7, &grid) && windows.add(8, &grid));
        assert!(!windows.add(7, &grid));
        assert_eq!(windows.0, [(7, 2), (8, 3)]);
        assert!(windows.remove(7) && !windows.remove(7));
        assert!(windows.add(9, &grid));
        assert_eq!(windows.0, [(8, 3), (9, 2)]);
        assert!(windows.keep(&[9]) && !windows.keep(&[9]));
        assert_eq!(windows.0, [(9, 2)]);
        // A screen with no usable cell gives no window.
        assert!(!Windows::default().add(1, &Grid::of((170, 100), Some(0))));
    }

    #[test]
    fn a_window_let_go_lands_in_the_nearest_cell_and_changes_places_with_one_there() {
        let grid = grid();
        assert_eq!(grid.at(0, 0), Some(0));
        assert_eq!(grid.at(88, 27), Some(0));
        assert_eq!(grid.at(89, 28), Some(14));
        assert_eq!(grid.at(-40, 5000), Some(12));
        assert_eq!(grid.at(5000, -40), Some(52));
        let mut windows = Windows(vec![(7, 2), (8, 3)]);
        windows.dropped(7, (180, 60), &grid);
        assert_eq!(windows.0, [(7, 14), (8, 3)]);
        windows.dropped(8, (170, 50), &grid);
        assert_eq!(windows.0, [(7, 3), (8, 14)]);
        windows.dropped(5, (0, 0), &grid);
        assert_eq!(windows.0, [(7, 3), (8, 14)]);
    }

    #[test]
    fn sorting_puts_the_windows_in_the_usable_cells_in_the_order_they_were_made() {
        let mut windows = Windows(vec![(7, 40), (8, 0), (9, 14)]);
        windows.sort(&grid());
        assert_eq!(windows.0, [(7, 2), (8, 3), (9, 4)]);
    }

    #[test]
    fn the_windows_are_read_back_from_what_they_are_kept_as() {
        let windows = Windows(vec![(2000001, 2), (2000007, 15)]);
        let kept = windows.written("Hunt Three").unwrap();
        assert_eq!(kept, "{Hunt Three,1,2000001,2}\n{Hunt Three,1,2000007,15}\n");
        assert_eq!(Windows::read(&kept, "Hunt Three"), windows);
        assert_eq!(Windows::read(&kept, "Other"), Windows::default());
        assert_eq!(Windows::read("{a,b,1,2,3}\n{x}\nrubbish\n{a,b,0,5,6}\n{a,b,1,2,9}", "a,b"), Windows(vec![(2, 3)]));
        assert_eq!(Windows::default().written("x"), None);
    }

    #[test]
    fn what_a_window_says_and_how_full_its_gauge_is() {
        assert_eq!(label(99, "HuntThree", "Morroc"), "Lv.99 HuntThree(Morroc)");
        assert_eq!(gauge(66, 9572, 9572), (64, false));
        assert_eq!(gauge(66, 50, 100), (31, false));
        assert_eq!(gauge(66, 24, 100), (13, true));
        assert_eq!(gauge(66, 1, 1000), (0, true));
        assert_eq!(gauge(66, 0, 0), (0, false));
        assert_eq!(gauge(60, 3, 100), (0, true));
    }
}
