//! The family as rules: the lines the client says of a divorce and of an adoption that cannot be
//! (`CGameMode::ReceiveDivorceNotice`, `ReceiveBabyMessage`). The packets are read in `net/family.rs`.

/// The line of a divorce (the client's line 836, with the player's own name and the other's;
/// `ReceiveDivorceNotice`).
pub fn divorce_line(line: &str, own: &str, other: &str) -> String {
    line.replacen("%s", own, 1).replacen("%s", other, 1)
}

/// Why an adoption cannot be, by the number the server gives: the line of the client's message table for it,
/// and the words until the table is read (`ReceiveBabyMessage`: 0, 1 and 2).
pub fn adoption_refusal(why: u32) -> Option<(usize, &'static str)> {
    match why {
        0 => Some((899, "You cannot adopt more than 1 child.")),
        1 => Some((900, "You must be at least character level 70 in order to adopt someone.")),
        2 => Some((916, "You cannot adopt a married person.")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lines_are_filled_and_chosen() {
        assert_eq!(divorce_line("%s has divorced %s.", "Hero", "Partner"), "Hero has divorced Partner.");
        assert_eq!([0, 1, 2, 3].map(|why| adoption_refusal(why).map(|said| said.0)), [Some(899), Some(900), Some(916), None]);
    }
}
