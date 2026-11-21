//! A deal between two players as rules: the most zeny a side puts into it and the zeny typed into it (the
//! client's `CUIExchangeUI`). The packets of a deal are in `net/exchange.rs`. (The colour the client writes
//! an amount in by its size, `PriceColor`, is `crate::numbers::price_colour`; the deal's window does not use
//! it: the owner's decision, `web/hud/exchange.rs exchange_zeny`.)

/// The most zeny one side puts into a deal (the client's `CUIExchangeUI::OnProcess`).
pub const MOST_ZENY: u32 = 1_000_000_000;

/// The zeny typed into the deal's window, as the client reads it (`CUIExchangeUI::SubmitZeny`): digits, with
/// commas between them or not; more than the character has is all it has; none for anything else.
pub fn zeny_typed(text: &str, has: u32) -> Option<u32> {
    crate::numbers::typed(text).map(|amount| if amount > u64::from(has) { has } else { amount as u32 }.min(MOST_ZENY))
}

#[cfg(test)]
mod tests {
    use super::*;

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
