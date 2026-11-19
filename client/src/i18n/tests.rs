//! The tests of the texts of the pages before the game and of how values are put into them (`i18n.rs`).

use super::*;

#[test]
fn every_text_is_there_in_every_language() {
    for &key in Key::ALL {
        let places = key.text(Lang::En).matches("{}").count();
        for lang in Lang::ALL {
            let text = key.text(lang);
            assert!(!text.trim().is_empty(), "{key:?} in {lang:?}");
            assert_eq!(text.matches("{}").count(), places, "{key:?} in {lang:?} takes other values than in English");
        }
    }
}

#[test]
fn russian_is_russian() {
    let cyrillic = Key::ALL.iter().filter(|key| key.text(Lang::Ru).chars().any(|character| ('а'..='я').contains(&character))).count();
    // All but the few words that stay as they are (names, "ID", "OK", "E-mail").
    assert!(cyrillic >= Key::ALL.len() - 6, "{cyrillic} of {}", Key::ALL.len());
}

#[test]
fn languages_are_found_by_code() {
    assert_eq!(Lang::from_code("ru"), Some(Lang::Ru));
    assert_eq!(Lang::from_code("EN"), Some(Lang::En));
    assert_eq!(Lang::from_code("de"), None);
    assert_eq!(Lang::default(), Lang::En);
}

#[test]
fn fills_values_in_order() {
    assert_eq!(fill("from {} to {}.", &["4", "23"]), "from 4 to 23.");
    assert_eq!(fill("no places", &["x"]), "no places");
    assert_eq!(fill("{} and {}", &["one"]), "one and ");
}
