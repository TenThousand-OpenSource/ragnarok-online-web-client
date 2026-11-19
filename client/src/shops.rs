//! An NPC's shop as rules, those of the client's two windows of it (`UIItemPurchaseWnd::SendMsg`,
//! `UIItemDropCntWnd::SendMsg`): how many of a thing may be bought at once, what goes into the basket, what a
//! count typed into the box that asks how many means, how wide a name is left beside the prices. The packets of
//! it are in `net::trade`.

/// The most of one item that is bought at a time (`UIItemPurchaseWnd::SendMsg`, `UM_DROP`: 30000).
pub const MOST_BOUGHT: u32 = 30000;

/// Whether a thing is of the kinds the client lets into the basket once and without asking how many: what is
/// worn (`TYPE_ARMOR`, `TYPE_WEAPON`, `TYPE_BOW`, `TYPE_BOTHHAND`, `TYPE_ARMORTM`…`TYPE_ARMORTMB`, `TYPE_GUN`).
pub fn bought_singly(kind: u8) -> bool {
    matches!(kind, 4 | 5 | 8 | 9 | 11..=15)
}

/// How many of `asked` more go into the basket that has `kept` of the item already, and whether the client
/// says its line 1704 (no more than 30000 of one item at a time).
pub fn into_basket(kept: u32, asked: u32) -> (u32, bool) {
    if kept + asked > MOST_BOUGHT { (MOST_BOUGHT.saturating_sub(kept), true) } else { (asked, false) }
}

/// What the box that asks how many is asked for (`UIItemDropCntWnd`'s states).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// How many to buy (state 1): nothing for a number under one, and no more than 30000.
    Buying,
    /// How many of what there is to move: to sell, out of a basket, out of the storage (states 2, 3, 4, 6):
    /// nothing for a number under one, and no more than there are.
    Moving,
    /// How many to put into the storage (state 7): a number under one counts as one, and no more than there are.
    Storing,
}

/// The number typed into the box, as the client reads it (`atoi`: the digits a text begins with, nought for
/// anything else), by what was asked and how many there are.
///
/// The same box over the inventory (how many to drop) is read by another rule, `crate::inventory::count_typed`,
/// and the two are kept apart on purpose: this one reads the digits a text begins with ("12a" is 12) and
/// takes the most there is for more, that one takes the whole text for a number ("12a" is nothing) and has
/// nothing for more than there is.
pub fn count_asked(typed: &str, most: u32, asked: Asked) -> Option<u32> {
    let digits: String = typed.trim().chars().take_while(char::is_ascii_digit).take(9).collect();
    let count = digits.parse::<u32>().unwrap_or(0);
    match asked {
        Asked::Buying => (count > 0).then(|| count.min(MOST_BOUGHT)),
        Asked::Moving => (count > 0 && most > 0).then(|| count.min(most)),
        Asked::Storing => (most > 0).then(|| count.clamp(1, most)),
    }
}

/// How wide the client leaves an item's name in a row of a shop's windows: `usual` (100 in the list and among
/// what is sold, 110 among what is bought), less when two prices are written and the first has more figures.
pub fn name_width(price: u32, differs: bool, usual: i32) -> i32 {
    match price {
        _ if !differs => usual,
        10000.. => 45,
        1000.. => 55,
        100.. => 70,
        10.. => 80,
        _ => usual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_basket_takes_what_the_client_lets_into_it() {
        assert_eq!(into_basket(0, 5), (5, false));
        assert_eq!(into_basket(29990, 20), (10, true));
        assert_eq!(into_basket(30000, 1), (0, true));
        assert!(bought_singly(5) && bought_singly(4) && bought_singly(15) && !bought_singly(0) && !bought_singly(10) && !bought_singly(3));
    }

    #[test]
    fn the_number_typed_is_read_as_each_state_of_the_box_reads_it() {
        assert_eq!(count_asked("12", 0, Asked::Buying), Some(12));
        assert_eq!(count_asked("99999", 0, Asked::Buying), Some(30000));
        assert_eq!(count_asked("0", 0, Asked::Buying), None);
        assert_eq!(count_asked("x", 0, Asked::Buying), None);
        assert_eq!(count_asked("7", 5, Asked::Moving), Some(5));
        assert_eq!(count_asked(" 3 ", 5, Asked::Moving), Some(3));
        assert_eq!(count_asked("", 5, Asked::Moving), None);
        assert_eq!(count_asked("0", 5, Asked::Storing), Some(1));
        assert_eq!(count_asked("12a", 50, Asked::Storing), Some(12));
        assert_eq!(count_asked("60", 50, Asked::Storing), Some(50));
    }

    #[test]
    fn a_name_is_left_less_room_beside_two_prices() {
        assert_eq!(name_width(12000, false, 100), 100);
        assert_eq!([name_width(12000, true, 100), name_width(1200, true, 100), name_width(120, true, 100), name_width(12, true, 100), name_width(5, true, 110)], [45, 55, 70, 80, 110]);
    }
}
