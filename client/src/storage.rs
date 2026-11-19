//! The storage as rules, those of the client's window of it (`UIItemStoreWnd`): its tabs, which tab a thing
//! lies under, the orders its lists are put in. The packets of it are in `net::storage`.

use crate::net::inventory::Item;

/// How many tabs the storage's window has: things used, of the cash shop, armour, weapons, costumes,
/// ammunition, cards, and the rest.
pub const TABS: usize = 8;

/// The line of the client's message table each tab is called by (what its button says when pointed at, and
/// the title of the window that shows the tab alone: `UIItemStoreWnd::OnCreate`, `UIWindowMgr`), with the
/// English of it.
pub const TAB_LINES: [(usize, &str); TABS] =
    [(1465, "Use"), (1466, "Cash"), (1467, "Armors"), (1468, "Weapons"), (3151, "Costumes"), (1469, "Ammo"), (1470, "Card"), (1471, "Etc")];

/// The places a costume is worn in (the server's `EQP_COSTUME_HEAD_TOP`, `_MID`, `_LOW` and `_GARMENT`).
pub const COSTUME_PLACES: u32 = 0x400 | 0x800 | 0x1000 | 0x2000;

/// Which tab a thing is under (`UIItemStoreWnd::SendMsg`, `UM_REFRESHITEM`), by its type and whether it is a
/// costume: `None` for a type the window lists nowhere. The client asks its item table whether a thing is a
/// costume; here it is one that is worn in a costume's place, which the server tells with the thing.
pub fn tab_of(item: &Item) -> Option<usize> {
    if item.wearable & COSTUME_PLACES != 0 {
        return Some(4);
    }
    match item.kind {
        0..=2 => Some(0),
        18 => Some(1),
        4 | 11..=14 => Some(2),
        5 | 8 | 9 | 15 => Some(3),
        10 | 16 | 17 | 19 => Some(5),
        6 => Some(6),
        3 | 7 => Some(7),
        _ => None,
    }
}

/// How the storage's lists are ordered (the window's box of three choices): as the server told them, or by
/// name, forwards or backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sorted {
    #[default]
    AsTold,
    Ascending,
    Descending,
}

impl Sorted {
    pub const ALL: [Sorted; 3] = [Sorted::AsTold, Sorted::Ascending, Sorted::Descending];

    /// The line of the client's message table the choice is called by, with the English the table has.
    pub fn line(self) -> (usize, &'static str) {
        match self {
            Sorted::AsTold => (3276, "Base"),
            Sorted::Ascending => (3274, "Upgrade"),
            Sorted::Descending => (3275, "Downgrade"),
        }
    }

    /// Puts names (each with what it names) into this order.
    pub fn order<Thing>(self, things: &mut [(String, Thing)]) {
        match self {
            Sorted::AsTold => {}
            Sorted::Ascending => things.sort_by(|one, other| one.0.cmp(&other.0)),
            Sorted::Descending => things.sort_by(|one, other| other.0.cmp(&one.0)),
        }
    }
}

/// The things of the storage a list shows, each with its name, in the order chosen: those under a tab, or all of
/// them, and of those the ones whose name has what was searched for in it (`UIItemStoreWnd::FilterByName`: as
/// it is written, capitals and all).
pub fn listed<'a>(
    items: &'a [Item],
    tab: Option<usize>,
    sought: Option<&str>,
    sorted: Sorted,
    name: impl Fn(&Item) -> String
) -> Vec<(String, &'a Item)> {
    let mut things: Vec<(String, &Item)> = items
        .iter()
        .filter(|item| tab.is_none_or(|tab| tab_of(item) == Some(tab)))
        .map(|item| (name(item), item))
        .filter(|(name, _)| sought.is_none_or(|sought| name.contains(sought)))
        .collect();
    sorted.order(&mut things);
    things
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thing(kind: u8, wearable: u32) -> Item {
        Item { index: 1, id: 501, kind, amount: 1, wearable, worn: 0, refine: 0, identified: true, cards: [0; 4] }
    }

    #[test]
    fn a_thing_is_under_the_tab_of_its_kind() {
        let tabs: Vec<Option<usize>> = [0, 2, 18, 4, 12, 5, 8, 15, 10, 16, 6, 3, 7, 20].iter().map(|kind| tab_of(&thing(*kind, 0))).collect();
        assert_eq!(tabs, [Some(0), Some(0), Some(1), Some(2), Some(2), Some(3), Some(3), Some(3), Some(5), Some(5), Some(6), Some(7), Some(7), None]);
        // A costume is under its own tab, whatever it is worn as.
        assert_eq!(tab_of(&thing(4, 0x400)), Some(4));
        assert_eq!(tab_of(&thing(4, 0x100)), Some(2));
    }

    #[test]
    fn the_lists_are_ordered_by_name() {
        let mut things = vec![("Jellopy".to_owned(), 1), ("Apple".to_owned(), 2), ("Red Potion".to_owned(), 3)];
        Sorted::AsTold.order(&mut things);
        assert_eq!(things[0].1, 1);
        Sorted::Ascending.order(&mut things);
        assert_eq!([things[0].1, things[1].1, things[2].1], [2, 1, 3]);
        Sorted::Descending.order(&mut things);
        assert_eq!([things[0].1, things[1].1, things[2].1], [3, 1, 2]);
    }

    #[test]
    fn a_list_has_a_tabs_things_or_those_a_search_found() {
        let stored = |index: u16, kind: u8| Item { index, ..thing(kind, 0) };
        // A potion, a card, another potion, and a thing of a kind that is under no tab.
        let items = [stored(1, 0), stored(2, 6), stored(3, 0), stored(4, 20)];
        let names = ["", "Red Potion", "Poring Card", "Apple", "red herring"];
        let name = |item: &Item| names[usize::from(item.index)].to_owned();
        let places = |tab: Option<usize>, sought: Option<&str>, sorted: Sorted| -> Vec<u16> {
            listed(&items, tab, sought, sorted, name).iter().map(|(_, item)| item.index).collect()
        };
        assert_eq!(places(Some(0), None, Sorted::AsTold), [1, 3]);
        assert_eq!(places(Some(0), None, Sorted::Ascending), [3, 1]);
        assert_eq!(places(Some(6), None, Sorted::Descending), [2]);
        assert_eq!(places(Some(4), None, Sorted::AsTold), [0u16; 0]);
        // Without a tab all of them, the one under no tab too; a search is by the letters as they are written.
        assert_eq!(places(None, None, Sorted::Descending), [4, 1, 2, 3]);
        assert_eq!(places(None, Some("Red"), Sorted::AsTold), [1]);
        assert_eq!(places(None, Some("red"), Sorted::AsTold), [4]);
        assert_eq!(places(None, Some(""), Sorted::AsTold), [1, 2, 3, 4]);
        assert_eq!(places(Some(0), Some("P"), Sorted::AsTold), [1]);
        assert_eq!(listed(&items, Some(0), None, Sorted::AsTold, name)[0].0, "Red Potion");
        assert!(listed(&[], None, None, Sorted::Ascending, name).is_empty());
    }
}
