//! An NPC's shop as rules, those of the client's two windows of it (`UIItemPurchaseWnd::SendMsg`,
//! `UIItemDropCntWnd::SendMsg`): how many of a thing may be bought at once, what goes into the basket, what a
//! count typed into the box that asks how many means, how wide a name is left beside the prices; and what the
//! panel beside the basket says of what is worn (`UIItemParamChangeDisplayWnd`). The packets of it are in
//! `net::shops`.

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

/// How many of a row of the shop's list are in the basket (the rows put into it, each with how many).
pub fn kept(basket: &[(usize, u16)], row: usize) -> u32 {
    basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| u32::from(*amount))
}

/// The sum under the basket: what one of each thing in it costs, times how many of it there are.
pub fn total(kept: impl Iterator<Item = (u32, u32)>) -> i64 {
    kept.map(|(price, amount)| i64::from(price) * i64::from(amount)).sum()
}

/// A thing worn, as the panel beside the basket asks of it: where it is worn now and where it can be worn
/// (the server's bits), and which item it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Worn {
    pub place: u32,
    pub places: u32,
    pub id: u32,
}

/// What the panel beside the basket of what is bought says (the client's `UIItemParamChangeDisplayWnd`): the
/// two figures of what is worn, and what they would be with the item pointed at worn in the place of what it
/// goes over. The client writes the first as ATK and the second as DEF.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Compared {
    pub before: (i32, i32),
    pub after: (i32, i32),
}

/// How a figure changes, which the panel shows between the two: a red triangle that points up, a grey bar,
/// a blue triangle that points down (`DrawParamChangeArrow`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    More,
    Same,
    Less,
}

/// `Change` of a figure from `before` to `after`.
pub fn change(before: i32, after: i32) -> Change {
    match after.cmp(&before) {
        std::cmp::Ordering::Greater => Change::More,
        std::cmp::Ordering::Equal => Change::Same,
        std::cmp::Ordering::Less => Change::Less,
    }
}

/// The panel's figures. `table` is the client's own of an item (`itemSlotTable.txt`, `itemParamTable.txt`):
/// the places the table says it is worn in, and its two figures if it has them.
///
/// Before (`SumEquippedStats`): the client goes through its first eight places of what is worn, in the
/// order of the server's bits (lower headgear, right hand, garment, an accessory, armour, left hand, shoes,
/// the other accessory: the upper and the middle headgear are the ninth and tenth and are not reached), and
/// adds the figures of each thing, leaving out one whose places by the table meet those of a thing counted
/// before it (a thing worn in two places is met twice).
///
/// After (`SendMsg`, `UM_PARAM` and `UM_REFRESH`): before, more by the figures of the item pointed at, less by
/// those of the first thing worn that can be worn in a place the table gives the item
/// (`FindEquippedItem`). With no item pointed at the two are the same: the window begins with item 512, an
/// apple, of which the table says nothing.
pub fn compared(worn: &[Worn], pointed: Option<u32>, table: impl Fn(u32) -> (u32, Option<(i32, i32)>)) -> Compared {
    let reached = || (0..8).filter_map(|bit| worn.iter().find(|thing| thing.place & (1 << bit) != 0));
    let (mut before, mut counted) = ((0, 0), 0u32);
    for thing in reached() {
        let (places, figures) = table(thing.id);
        let met = places & counted != 0;
        counted |= places;
        if let (false, Some(figures)) = (met, figures) {
            before = (before.0 + figures.0, before.1 + figures.1);
        }
    }
    let Some(pointed) = pointed else { return Compared { before, after: before } };
    let (places, figures) = table(pointed);
    let item = figures.unwrap_or_default();
    let over = reached().find(|thing| thing.places & places != 0).and_then(|thing| table(thing.id).1).unwrap_or_default();
    Compared { before, after: (item.0 + before.0 - over.0, item.1 + before.1 - over.1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A table as the client's files have it: a knife (1201), a two-handed sword (1116), a guard (2101),
    /// a cotton shirt (2301), a hat (2220, upper headgear), and a blade (1108) to point at.
    fn table(id: u32) -> (u32, Option<(i32, i32)>) {
        match id {
            1201 => (2, Some((17, 0))),
            1116 => (34, Some((60, 0))),
            2101 => (32, Some((0, 2))),
            2301 => (16, Some((0, 1))),
            2220 => (256, Some((0, 2))),
            1108 => (2, Some((53, 0))),
            _ => (0, None),
        }
    }

    #[test]
    fn the_panel_sums_what_is_worn_and_compares_the_item_pointed_at() {
        let worn = [
            Worn { place: 16, places: 16, id: 2301 },
            Worn { place: 2, places: 2, id: 1201 },
            Worn { place: 32, places: 32, id: 2101 },
            // The upper headgear is not among the places the client goes through.
            Worn { place: 256, places: 256, id: 2220 },
        ];
        assert_eq!(compared(&worn, None, table), Compared { before: (17, 3), after: (17, 3) });
        // A blade in the place of the knife.
        assert_eq!(compared(&worn, Some(1108), table), Compared { before: (17, 3), after: (53, 3) });
        // A two-handed sword goes over the first thing worn in one of its places, the knife; the guard stays.
        assert_eq!(compared(&worn, Some(1116), table), Compared { before: (17, 3), after: (60, 3) });
        // What the table does not have changes nothing.
        assert_eq!(compared(&worn, Some(501), table), Compared { before: (17, 3), after: (17, 3) });
        assert_eq!(compared(&[], Some(1108), table), Compared { before: (0, 0), after: (53, 0) });
    }

    #[test]
    fn a_thing_worn_in_two_places_is_counted_once() {
        let worn = [Worn { place: 34, places: 34, id: 1116 }, Worn { place: 16, places: 16, id: 2301 }];
        assert_eq!(compared(&worn, None, table).before, (60, 1));
        // The guard would go over the sword, which is the first thing worn that can be worn in its place.
        assert_eq!(compared(&worn, Some(2101), table).after, (0, 3));
    }

    #[test]
    fn a_figure_goes_up_stays_or_goes_down() {
        assert_eq!([change(1, 2), change(2, 2), change(3, 2)], [Change::More, Change::Same, Change::Less]);
    }

    #[test]
    fn the_basket_knows_how_many_of_a_row_it_has_and_their_sum() {
        let basket = [(2, 5), (0, 1), (2, 9)];
        // The first of a row counts, should it be there twice.
        assert_eq!([kept(&basket, 2), kept(&basket, 0), kept(&basket, 1)], [5, 1, 0]);
        assert_eq!(kept(&[], 0), 0);
        assert_eq!(total([(100, 5), (7, 1)].into_iter()), 507);
        assert_eq!(total([(u32::MAX, 30000), (0, 9)].into_iter()), 128_849_018_850_000);
        assert_eq!(total(std::iter::empty()), 0);
    }

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
        assert_eq!(
            [
                name_width(12000, true, 100), name_width(1200, true, 100), name_width(120, true, 100),
                name_width(12, true, 100), name_width(5, true, 110)
            ],
            [45, 55, 70, 80, 110]
        );
    }
}
