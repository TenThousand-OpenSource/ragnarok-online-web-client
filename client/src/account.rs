//! The account as rules: what an ID, a password and an e-mail may be before a form is sent (the gateway's
//! account service decides: `gateway/src/accounts/rules.rs`), and how a wait is told. These are this
//! project's own: the Windows client has no registration.

/// An ID: 4 to 23 letters of English, digits and `_`.
pub fn valid_id(id: &str) -> bool {
    (4..=23).contains(&id.len()) && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// A password: 8 to 23 characters that show, none of them a space.
pub fn valid_password(password: &str) -> bool {
    (8..=23).contains(&password.len()) && password.bytes().all(|byte| byte.is_ascii_graphic())
}

/// A rough check, to catch slips before the form is sent; the account service decides.
pub fn valid_email(email: &str) -> bool {
    email.len() <= 39
        && !email.contains(' ')
        && email.split_once('@').is_some_and(|(name, domain)| !name.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.'))
}

/// A wait of so many seconds in whole minutes, a begun one counted, and never less than one.
pub fn minutes(seconds: u64) -> String {
    seconds.div_ceil(60).max(1).to_string()
}

/// When a character waiting to be deleted can be, as the list of characters writes it in two lines: the month
/// and the day, and under them the hour and the minute. (Where the Windows client writes this was not found
/// in its code.)
pub fn deletion_due((month, day): (u32, u32), (hours, minutes): (u32, u32)) -> [String; 2] {
    [format!("{month}/{day}"), format!("{hours:02}:{minutes:02}")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deletion_is_due_on_a_day_at_a_time() {
        assert_eq!(deletion_due((3, 9), (7, 5)), ["3/9", "07:05"]);
        assert_eq!(deletion_due((12, 31), (23, 59)), ["12/31", "23:59"]);
        assert_eq!(deletion_due((0, 0), (0, 0)), ["0/0", "00:00"]);
    }

    #[test]
    fn an_id_and_a_password_are_held_to_their_lengths_and_letters() {
        assert!(valid_id("abcd") && valid_id("player_01") && valid_id(&"a".repeat(23)));
        assert!(!valid_id("abc") && !valid_id(&"a".repeat(24)) && !valid_id("with space") && !valid_id("имя_героя"));
        assert!(valid_password("12345678") && valid_password("p@ss-w0rd!"));
        assert!(!valid_password("1234567") && !valid_password("with space") && !valid_password(&"a".repeat(24)));
    }

    #[test]
    fn an_email_is_looked_at_roughly() {
        assert!(valid_email("a@b.c") && valid_email("name.surname@mail.example.org"));
        for wrong in ["", "a", "a@b", "@b.c", "a@.c", "a@b.", "a b@c.d", &format!("{}@b.c", "a".repeat(36))] {
            assert!(!valid_email(wrong), "{wrong}");
        }
    }

    #[test]
    fn a_wait_is_told_in_minutes_begun() {
        assert_eq!(
            (minutes(0), minutes(1), minutes(60), minutes(61), minutes(3600)),
            ("1".to_owned(), "1".to_owned(), "1".to_owned(), "2".to_owned(), "60".to_owned())
        );
    }
}
