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

/// Whether a thing of the inventory let go over the Equipment window is asked to be put on
/// (`UIEquipWnd::DragAndDrop`): ammunition (the client's item kinds 10, 16, 17 and 19) is readied whatever
/// else is true of it (`MM_WEAR_EQUIP_ARROW`); anything else must be identified and have a place to be worn
/// (`MM_WEAR_EQUIP`). The rest only ends the drag.
pub fn worn_by_drop(kind: u8, identified: bool, wearable: u32) -> bool {
    matches!(kind, 10 | 16 | 17 | 19) || (identified && wearable != 0)
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

/// What the Inventory window shows of what is carried under a tab, in its order (`UIItemWnd::SendMsg`,
/// `UM_REFRESHITEM`): what is worn is not there (`Item::in_bag`); what is kept under "Fav" (`favourites`: the
/// places of those things) is on that tab alone, in the order of its kinds and names while "Align items" is on
/// (`UIItemWnd::RefreshItems`), and as it is carried otherwise, as the other tabs are.
pub fn shown<'a>(
    items: &'a [Item],
    tab: Tab,
    favourites: &[u16],
    aligned: bool,
    name: impl Fn(&Item) -> String
) -> Vec<&'a Item> {
    let favourite = |item: &&Item| favourites.contains(&item.index);
    let mut shown: Vec<&Item> = items
        .iter()
        .filter(
            |item| item.in_bag() && if tab == Tab::Favourite { favourite(item) } else { item.tab() == tab && !favourite(item) }
        )
        .collect();
    if tab == Tab::Favourite && aligned {
        shown.sort_by_cached_key(|item| (sort_rank(item.kind), name(item)));
    }
    shown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tab_shows_its_things_and_fav_those_kept_under_it() {
        let item = |index: u16, kind: u8, worn: u32| Item {
            index,
            id: u32::from(index),
            kind,
            amount: 1,
            wearable: worn,
            worn,
            refine: 0,
            identified: true,
            cards: [0; 4]
        };
        // A potion, loot, a sword worn, a card, a potion and loot kept under "Fav".
        let items = [item(2, 0, 0), item(3, 3, 0), item(4, 5, 2), item(5, 6, 0), item(6, 0, 0), item(7, 3, 0)];
        let name = |item: &Item| format!("thing {}", 9 - item.index);
        let places = |tab: Tab, favourites: &[u16], aligned: bool| -> Vec<u16> {
            shown(&items, tab, favourites, aligned, name).iter().map(|item| item.index).collect()
        };
        assert_eq!(places(Tab::Usable, &[], false), [2, 6]);
        assert_eq!(places(Tab::Usable, &[6, 7], false), [2]);
        assert_eq!(places(Tab::Usable, &[6, 7], true), [2]);
        // What is worn is on no tab, kept under "Fav" or not.
        assert_eq!(places(Tab::Equipment, &[], false), [0u16; 0]);
        assert_eq!(places(Tab::Favourite, &[4], false), [0u16; 0]);
        // "Fav": as carried, or by kind and then by name; a place that carries nothing shows nothing.
        assert_eq!(places(Tab::Favourite, &[7, 6, 5, 99], false), [5, 6, 7]);
        assert_eq!(places(Tab::Favourite, &[7, 6, 5, 3, 99], true), [6, 5, 7, 3]);
        assert_eq!(places(Tab::Favourite, &[], true), [0u16; 0]);
        assert_eq!(shown(&[], Tab::Usable, &[1], true, name).len(), 0);
    }

    #[test]
    fn a_count_typed_is_a_number_of_what_there_is() {
        assert_eq!((count_typed("1", 30), count_typed(" 30 ", 30)), (Some(1), Some(30)));
        // Nothing, no number, none at all, more than there is, more than a count can be: nothing is done.
        assert_eq!(
            [
                count_typed("", 30), count_typed("ten", 30), count_typed("0", 30), count_typed("31", 30),
                count_typed("70000", u16::MAX), count_typed("-1", 30)
            ],
            [None; 6]
        );
    }

    #[test]
    fn only_what_is_used_shot_or_worn_goes_on_the_hotkey_bar() {
        let item = |kind: u8, wearable: u32, identified: bool| Item {
            index: 2,
            id: 501,
            kind,
            amount: 1,
            wearable,
            worn: 0,
            refine: 0,
            identified,
            cards: [0; 4]
        };
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
    fn a_thing_let_go_over_the_equipment_window_is_put_on_if_it_can_be() {
        // A hat that is identified; arrows; not a hat that is not, nor a potion or loot.
        assert!(worn_by_drop(4, true, 256) && worn_by_drop(10, true, 0x8000) && worn_by_drop(10, false, 0));
        assert!(!worn_by_drop(4, false, 256) && !worn_by_drop(0, true, 0) && !worn_by_drop(3, true, 0));
    }

    #[test]
    fn the_kinds_are_ranked_as_the_client_orders_them() {
        assert_eq!([0, 18, 4, 5, 10, 6, 3, 20].map(sort_rank), [0, 1, 2, 3, 4, 5, 6, 10]);
    }
}
