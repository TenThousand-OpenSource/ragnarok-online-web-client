//! Tests of the accounts: what an ID, a password and an e-mail address may be, the letter, the wait.

use super::*;

#[test]
fn ids_are_what_the_login_server_takes() {
    for id in ["abcd", "Player_01", &"x".repeat(23)] {
        assert!(valid_id(id).is_ok(), "{id}");
    }
    for id in ["", "abc", &"x".repeat(24), "with space", "semi;colon", "quote'", "кириллица", "a\0bc"] {
        assert!(valid_id(id).is_err(), "{id:?}");
    }
}

#[test]
fn passwords_fit_the_login_packet() {
    for password in ["12345678", "p@ss-W0rd!", &"x".repeat(23)] {
        assert!(valid_password(password).is_ok(), "{password}");
    }
    for password in ["", "short7!", &"x".repeat(24), "has a space", "tab\there!", "пароль1234", "nul\0inside"] {
        assert!(valid_password(password).is_err(), "{password:?}");
    }
}

#[test]
fn emails_are_well_formed_and_fit_the_column() {
    for email in ["a@b.co", "first.last+tag@mail.example.org", "player_1@example-host.net"] {
        assert!(valid_email(email).is_ok(), "{email}");
    }
    let too_long = format!("{}@example.org", "x".repeat(30));
    for email in [
        "", "plain", "@example.org", "name@", "name@host", "two@@example.org", "a@b@c.org", "spa ce@example.org",
        "name@exa mple.org", "name@.org", "name@example.", "new\nline@example.org", &too_long
    ] {
        assert!(valid_email(email).is_err(), "{email:?}");
    }
}

#[test]
fn the_letter_is_in_the_players_language() {
    let (subject, text) = reset_letter("ru", "hero", "http://game.test/?reset=abc");
    assert_eq!(subject, "Установка нового пароля");
    assert!(text.contains("\"hero\"") && text.contains("http://game.test/?reset=abc") && text.contains("30 минут"), "{text}");
    // English for every other language, and for none.
    for lang in ["en", "", "de"] {
        let (subject, text) = reset_letter(lang, "hero", "http://game.test/?reset=abc");
        assert_eq!(subject, "Set a new password");
        assert!(text.contains("http://game.test/?reset=abc") && text.contains("30 minutes"));
    }
}

#[test]
fn says_how_long_to_wait() {
    let response = too_many(Duration::from_secs(61));
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response.headers()[header::RETRY_AFTER], "61");
}
