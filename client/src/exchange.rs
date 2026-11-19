//! A deal between two players as rules: the most zeny a side puts into it, the zeny typed into it and the
//! colour it is written in (the client's `CUIExchangeUI`). The packets of a deal are in `net/exchange.rs`.

/// The most zeny one side puts into a deal (the client's `CUIExchangeUI::OnProcess`).
pub const MOST_ZENY: u32 = 1_000_000_000;

/// The zeny typed into the deal's window, as the client reads it (`CUIExchangeUI::SubmitZeny`): digits, with
/// commas between them or not; more than the character has is all it has; none for anything else.
pub fn zeny_typed(text: &str, has: u32) -> Option<u32> {
    crate::numbers::typed(text).map(|amount| if amount > u64::from(has) { has } else { amount as u32 }.min(MOST_ZENY))
}

/// The colour an amount of zeny is written in, by how large it is (the first colour of the client's
/// `PriceColor`, which it keeps blue first: written here as the page wants it).
pub fn zeny_colour(amount: u32) -> &'static str {
    crate::numbers::price_colour(amount).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeny_is_coloured_by_how_much_it_is() {
        assert_eq!(
            (zeny_colour(0), zeny_colour(99), zeny_colour(5_000), zeny_colour(10_000_000), zeny_colour(999_999)),
            ("#00ffff", "#c800c8", "#ffff00", "#ff0000", "#0000ff")
        );
    }

    #[test]
    fn zeny_is_digits_with_or_without_commas() {
        assert_eq!(zeny_typed("1,500", 2000), Some(1500));
        assert_eq!(zeny_typed(" 0 ", 2000), Some(0));
        assert_eq!(zeny_typed("5000", 2000), Some(2000));
        assert_eq!(zeny_typed("2,000,000,000", u32::MAX), Some(MOST_ZENY));
        assert_eq!(zeny_typed("12a", 2000), None);
        assert_eq!(zeny_typed("", 2000), None);
        assert_eq!(zeny_typed("-5", 2000), None);
    }
}
