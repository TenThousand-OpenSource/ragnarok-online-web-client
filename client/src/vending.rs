//! A player's shop as rules: what the client lets a shop have and a thing cost, what the chat says of a
//! purchase that failed, a price typed, and what is left of a price after the commission.
//!
//! The rules are the client's (`UI/Shop/PriceColor.cpp`, `PriceCommission.cpp`, `UIMerchantShopMakeWnd`,
//! `UIMerchantItemPurchaseWnd`). The packets of a shop are in `net/vending.rs`.

/// The most the client lets a thing cost, the most of one thing a shop sells or a basket takes, and the
/// longest title it lets be typed (`UIMerchantShopMakeWnd`).
pub const MOST_PRICE: u32 = 1_000_000_000;
pub const MOST_AMOUNT: u32 = 30_000;
pub const TITLE: usize = 36;
/// How few and how many kinds of things the client lets a shop have, whatever the server says (the handler
/// of ZC_OPENSTORE).
pub const SLOTS: std::ops::RangeInclusive<u16> = 3..=13;

/// What the chat says of a purchase that failed (the client's handler of 0x0135), for a thing called `name`
/// of which `amount` were asked for; `line` reads a line of the client's message table. Nothing for one that
/// went through, nor for a result the client says nothing of.
pub fn purchase_said(result: u8, name: &str, amount: u16, line: impl Fn(usize, &str) -> String) -> Option<String> {
    let failed = |why: usize, otherwise: &str| line(229, "%s Purchase Failed %s").replacen("%s", name, 1).replacen("%s", &line(why, otherwise), 1);
    Some(match result {
        1 => failed(55, "You do not have enough zeny."),
        2 => failed(56, "You are over your Weight Limit."),
        4 => line(240, "Buying %s has failed. It's out of stock. Current Stock %d.").replacen("%s", name, 1).replacen("%d", &amount.to_string(), 1),
        5 => line(281, "You cannot use an NPC shop while in a trade."),
        6 => line(1695, "Item purchase failed due to incorrect shop information."),
        7 => line(1797, "No sales information."),
        _ => return None,
    })
}

/// What a vendor is left of a price once the server has kept its share, as the window of a shop being made
/// writes it under the price (`PriceCommission`: a twentieth of a price over ten million).
pub fn after_commission(price: u32) -> u32 {
    if price > 10_000_000 { price - (u64::from(price) * 5 / 100) as u32 } else { price }
}

/// A price as it was typed into the window of a shop being made: its figures with the commas between them
/// left out; none for anything that is not a number (the client's line 602), and never more than the most.
pub fn price_typed(typed: &str) -> Option<u32> {
    crate::numbers::typed(typed).map(|price| price.min(u64::from(MOST_PRICE)) as u32)
}

/// How high the window of a shop being made is with so many things put in: 260, and from four things on 42
/// for each and 92 more (`UIMerchantShopMakeWnd::UpdateLayout`).
pub fn stall_height(things: usize) -> i32 {
    let rows = things as i32;
    if rows >= 4 { rows * 42 + 92 } else { 260 }
}

/// How many rows the list of a vendor that sells so many things shows: one to a thing, four at the least
/// (`UM_REFRESHITEM`), and here eight at the most, past which it scrolls (the client lets the window be drawn
/// out to 336 high).
pub fn rows_shown(things: usize) -> i32 {
    things.clamp(4, 8) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numbers::price_colour;

    #[test]
    fn a_purchase_that_failed_is_told_of() {
        let line = |_: usize, otherwise: &str| otherwise.to_owned();
        assert_eq!(purchase_said(0, "Jellopy", 3, line), None);
        assert_eq!(purchase_said(1, "Jellopy", 3, line).as_deref(), Some("Jellopy Purchase Failed You do not have enough zeny."));
        assert_eq!(purchase_said(4, "Jellopy", 3, line).as_deref(), Some("Buying Jellopy has failed. It's out of stock. Current Stock 3."));
    }

    #[test]
    fn the_windows_grow_with_what_is_sold() {
        assert_eq!([0, 3, 4, 5, 12].map(stall_height), [260, 260, 260, 302, 596]);
        assert_eq!([0, 1, 4, 5, 8, 9, 100].map(rows_shown), [4, 4, 4, 5, 8, 8, 8]);
    }

    #[test]
    fn prices_have_their_colours_their_share_and_their_reading() {
        assert_eq!(price_colour(5).0, "#00ffff");
        assert_eq!(price_colour(10_000_000), ("#ff0000", None));
        assert_eq!(price_colour(999), ("#00ffff", Some("#0000ff")));
        assert_eq!((after_commission(10_000_000), after_commission(20_000_000)), (10_000_000, 19_000_000));
        assert_eq!(
            [price_typed("1,500"), price_typed(" 0 "), price_typed("12a"), price_typed(""), price_typed("99999999999")],
            [Some(1500), Some(0), None, None, Some(MOST_PRICE)]
        );
    }
}
