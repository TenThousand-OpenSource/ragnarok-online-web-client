//! How the client writes a number in its windows (zeny, a price, a sum) and reads one typed into them, and
//! how it writes what a line of its message table asks for into the line.

/// A number with commas between its thousands.
pub fn grouped(number: i64) -> String {
    let digits = number.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if number < 0 { format!("-{grouped}") } else { grouped }
}

/// The colour the client writes a price or an amount of zeny in, and the colour of the rim it gives its
/// figures where it gives one (`PriceColor`, `UIWindow::DrawAmount`): by how many figures the number has. The
/// client keeps its colours blue first; they are written here as the page wants them.
pub fn price_colour(price: u32) -> (&'static str, Option<&'static str>) {
    match price {
        1_000_000_000.. => ("#ff007f", Some("#ff0000")),
        100_000_000.. => ("#c8c864", Some("#000000")),
        10_000_000.. => ("#ff0000", None),
        1_000_000.. => ("#00ff00", Some("#000000")),
        100_000.. => ("#0000ff", None),
        10_000.. => ("#ff1eff", None),
        1_000.. => ("#ffff00", Some("#ff0000")),
        100.. => ("#00ffff", Some("#0000ff")),
        10.. => ("#c800c8", Some("#0000ff")),
        _ => ("#00ffff", Some("#000000")),
    }
}

/// A number as it was typed into a window that asks for a price or for zeny: its figures, with commas between
/// them or not, and blanks around; none for anything else, nothing typed too. One too large to be kept is the
/// largest there is: the window that asked cuts it to its own most.
pub fn typed(text: &str) -> Option<u64> {
    let figures: String = text.trim().chars().filter(|figure| *figure != ',').collect();
    if figures.is_empty() || !figures.bytes().all(|figure| figure.is_ascii_digit()) {
        return None;
    }
    Some(figures.parse().unwrap_or(u64::MAX))
}

/// A line of the message table with what it asks for written into it, as the client's `sprintf` does: each
/// `%s`, `%d`, `%u` and `%lld` in turn takes the next of `with`, set to the width the line names between
/// (`%02d`: with noughts before it; `%3d`: with blanks); `%%` is a percent sign.
pub fn filled(line: &str, with: &[&str]) -> String {
    let mut out = String::with_capacity(line.len());
    let mut with = with.iter();
    let mut rest = line;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        rest = &rest[at + 1..];
        let width = rest.len() - rest.trim_start_matches(|digit: char| digit.is_ascii_digit()).len();
        let asked = &rest[width..];
        match ["lld", "I64d", "s", "d", "u"].into_iter().find(|kind| asked.starts_with(kind)) {
            Some(kind) => {
                let value = with.next().copied().unwrap_or_default();
                let (noughts, wide) = (rest.starts_with('0'), rest[..width].parse().unwrap_or(0));
                out.push_str(&if noughts { format!("{value:0>wide$}") } else { format!("{value:>wide$}") });
                rest = &asked[kind.len()..];
            }
            None if rest.starts_with('%') => {
                out.push('%');
                rest = &rest[1..];
            }
            None => out.push('%'),
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_price_is_coloured_by_how_many_figures_it_has() {
        assert_eq!(price_colour(0), ("#00ffff", Some("#000000")));
        assert_eq!((price_colour(9).0, price_colour(10).0, price_colour(99).0, price_colour(100).0), ("#00ffff", "#c800c8", "#c800c8", "#00ffff"));
        assert_eq!((price_colour(1_000), price_colour(99_999)), (("#ffff00", Some("#ff0000")), ("#ff1eff", None)));
        assert_eq!((price_colour(100_000), price_colour(1_000_000)), (("#0000ff", None), ("#00ff00", Some("#000000"))));
        assert_eq!((price_colour(10_000_000), price_colour(100_000_000)), (("#ff0000", None), ("#c8c864", Some("#000000"))));
        assert_eq!(price_colour(u32::MAX), ("#ff007f", Some("#ff0000")));
    }

    #[test]
    fn a_number_is_typed_with_commas_or_without() {
        assert_eq!((typed("1,234,567"), typed(" 500 "), typed("0")), (Some(1_234_567), Some(500), Some(0)));
        assert_eq!((typed(""), typed("  "), typed(","), typed("12a"), typed("-5"), typed("1.5")), (None, None, None, None, None, None));
        assert_eq!(typed("99999999999999999999999"), Some(u64::MAX));
    }

    #[test]
    fn a_line_takes_what_it_asks_for() {
        assert_eq!(filled("The [%s] item will disappear in %d minutes.", &["Apple", "5"]), "The [Apple] item will disappear in 5 minutes.");
        assert_eq!(filled("'%lld' Base Experience", &["-12"]), "'-12' Base Experience");
        assert_eq!(filled("100%% of %s", &["it"]), "100% of it");
        assert_eq!(filled("%s and %s", &["one"]), "one and ");
        assert_eq!(filled("50% off", &[]), "50% off");
        assert_eq!(filled("in %02d hour(s) and %02d minute(s)", &["3", "15"]), "in 03 hour(s) and 15 minute(s)");
        assert_eq!(filled("[%3d]", &["7"]), "[  7]");
    }

    #[test]
    fn groups_thousands() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        assert_eq!(grouped(1234567), "1,234,567");
        assert_eq!(grouped(-45000), "-45,000");
    }
}
