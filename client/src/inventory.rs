//! The inventory as rules: its tabs and which a thing is under, what the window shows of what is carried, what
//! can be used or set on the hotkey bar, what a count typed means, the order of the "Fav" tab, and (in
//! `weight`) the lines the chat says of the weight carried. The packets of it are in `net::inventory`, whose
//! `Item` is the thing these rules are about.

use crate::net::inventory::Item;

pub mod weight;

/// The tabs of the inventory window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Usable,
    Equipment,
    Other,
    /// "Fav": what the player keeps under it, whatever kind it is of (`net::inventory::favourites`). No thing is
    /// under it by its kind: `Item::tab` never says it.
    Favourite,
}

/// Where arrows and the like are worn (the client's wear location `0x8000`).
pub const WORN_AS_AMMO: u32 = 0x8000;

impl Item {
    /// Whether the inventory window shows it: what is not worn, and the arrows that are. (The client takes what
    /// is worn out of its list of what is carried, but leaves the arrows in it and only remembers which are
    /// readied: `CSession::equippedArrowIndex`, looked up among `m_inventoryItems`.)
    pub fn in_bag(&self) -> bool {
        self.worn == 0 || self.worn == WORN_AS_AMMO
    }

    /// Which tab the item is under: what is used up, what is worn, and everything else.
    pub fn tab(&self) -> Tab {
        match self.kind {
            0 | 2 | 11 | 18 => Tab::Usable,
            4 | 5 | 8 | 12 => Tab::Equipment,
            _ if self.wearable != 0 && self.kind != 10 => Tab::Equipment,
            _ => Tab::Other,
        }
    }

    pub fn usable(&self) -> bool {
        self.tab() == Tab::Usable
    }

    /// Whether it may be put on the hotkey bar (the client's shortcut window, where a drag ends on it): one that
    /// is identified and is used up (the kinds 0, 1 and 2), shot or thrown (10, 16, 17, 19), or worn (4, 5, 8, 9,
    /// 11 to 15) with a place to wear it.
    pub fn fits_hotkeys(&self) -> bool {
        self.identified && (matches!(self.kind, 0 | 1 | 2 | 10 | 16 | 17 | 19) || (matches!(self.kind, 4 | 5 | 8 | 9 | 11..=15) && self.wearable != 0))
    }
}

/// The number typed into the box that asks how many of a thing carried (`UIItemDropCntWnd`), of which there
/// are `most`: as the client's box has it, what is no number, nothing, or more than there is counts for
/// nothing, and the box goes away without a word.
///
/// The same box in a shop, the storage and the cart is read by another rule, `crate::shops::count_asked`, and
/// the two are kept apart on purpose: this one takes the whole text for a number ("12a" is nothing) and has
/// nothing for more than there is, that one reads the digits a text begins with ("12a" is 12) and takes the
/// most there is for more.
pub fn count_typed(typed: &str, most: u16) -> Option<u16> {
    typed.trim().parse::<u32>().ok().filter(|count| (1..=u32::from(most)).contains(count)).map(|count| count as u16)
}

/// The order the things under the tab are put in while its "Align items" button is on (`ItemTypeNameLess`):
/// by this rank of their kind (`ItemTypeSortRank`), then by name.
pub fn sort_rank(kind: u8) -> u8 {
    match kind {
        0..=2 => 0,
        18 => 1,
        4 | 11..=14 => 2,
        5 | 8 | 9 | 15 => 3,
        10 | 16 | 17 | 19 => 4,
        6 => 5,
        3 | 7 => 6,
        _ => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_typed_is_a_number_of_what_there_is() {
        assert_eq!((count_typed("1", 30), count_typed(" 30 ", 30)), (Some(1), Some(30)));
        // Nothing, no number, none at all, more than there is, more than a count can be: nothing is done.
        assert_eq!([count_typed("", 30), count_typed("ten", 30), count_typed("0", 30), count_typed("31", 30), count_typed("70000", u16::MAX), count_typed("-1", 30)], [None; 6]);
    }

    #[test]
    fn only_what_is_used_shot_or_worn_goes_on_the_hotkey_bar() {
        let item = |kind: u8, wearable: u32, identified: bool| Item { index: 2, id: 501, kind, amount: 1, wearable, worn: 0, refine: 0, identified, cards: [0; 4] };
        // A potion and an arrow do; loot does not.
        assert!(item(0, 0, true).fits_hotkeys());
        assert!(item(10, 0, true).fits_hotkeys());
        assert!(!item(3, 0, true).fits_hotkeys());
        // A weapon does if it has a place to be worn, and nothing does until it is identified.
        assert!(item(5, 2, true).fits_hotkeys());
        assert!(!item(5, 0, true).fits_hotkeys());
        assert!(!item(5, 2, false).fits_hotkeys());
    }

    #[test]
    fn the_kinds_are_ranked_as_the_client_orders_them() {
        assert_eq!([0, 18, 4, 5, 10, 6, 3, 20].map(sort_rank), [0, 1, 2, 3, 4, 5, 6, 10]);
    }
}
