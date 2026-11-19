//! What the game keeps in the browser between visits, as text: the two shapes its lists are written in, each
//! parsed and written here once. A list of pairs is `name=value;name=value` (the options set, with `=`; the
//! levels skills are used at, with `:`); a list of numbers is `1,2,3` (the sound's levels, the quests not
//! listed, the camera), and the player's own keys are numbers with `:` between, an entry to a key
//! (`entries`). Nothing kept is written as nothing at all: the key is taken away (`page::store(key, None)`).
//!
//! What cannot be read back (a browser's storage changed by hand, or by an older version) is passed by.

use std::fmt::Display;
use std::str::FromStr;

/// The entries of what is kept as a list with `;` between; none of an empty text.
pub fn entries(kept: &str) -> impl Iterator<Item = &str> {
    kept.split(';').filter(|entry| !entry.is_empty())
}

/// The pairs of a list `name=value;name=value`, `between` being what stands between a name and its value;
/// an entry without it is passed by.
pub fn pairs(kept: &str, between: char) -> impl Iterator<Item = (&str, &str)> {
    kept.split(';').filter_map(move |entry| entry.split_once(between))
}

/// Pairs as they are kept, or nothing when there are none.
pub fn paired<Name: Display, Value: Display>(pairs: impl IntoIterator<Item = (Name, Value)>, between: char) -> Option<String> {
    let kept: Vec<String> = pairs.into_iter().map(|(name, value)| format!("{name}{between}{value}")).collect();
    (!kept.is_empty()).then(|| kept.join(";"))
}

/// The numbers of a list with `between` between them; what is no number of that kind is passed by.
pub fn numbers<Number: FromStr>(kept: &str, between: char) -> Vec<Number> {
    kept.split(between).filter_map(|number| number.parse().ok()).collect()
}

/// Numbers as they are kept, or nothing when there are none.
pub fn numbered<Number: Display>(numbers: impl IntoIterator<Item = Number>, between: char) -> Option<String> {
    let kept: Vec<String> = numbers.into_iter().map(|number| number.to_string()).collect();
    (!kept.is_empty()).then(|| kept.join(&between.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_are_read_back_as_they_were_kept() {
        let kept = paired([("TT_SNAP", 1), ("TT_EFFECT", 0)], '=').unwrap();
        assert_eq!(kept, "TT_SNAP=1;TT_EFFECT=0");
        assert_eq!(pairs(&kept, '=').collect::<Vec<_>>(), [("TT_SNAP", "1"), ("TT_EFFECT", "0")]);
        assert_eq!(pairs("28:10;no pair;;5:1:2", ':').collect::<Vec<_>>(), [("28", "10"), ("5", "1:2")]);
        assert_eq!((pairs("", '=').count(), paired(Vec::<(u8, u8)>::new(), '=')), (0, None));
    }

    #[test]
    fn numbers_are_read_back_as_they_were_kept() {
        assert_eq!(numbered([100u8, 40, 0, 1], ',').as_deref(), Some("100,40,0,1"));
        assert_eq!(numbers::<u8>("100,40,0,1", ','), [100, 40, 0, 1]);
        // What is no number of that kind is passed by; nothing kept is no numbers.
        assert_eq!(numbers::<u8>("7,x,300,,9", ','), [7, 9]);
        assert_eq!((numbers::<u32>("", ','), numbered(Vec::<u32>::new(), ',')), (vec![], None));
        assert_eq!(numbers::<f32>("12.5,-3,45", ','), [12.5, -3.0, 45.0]);
    }

    #[test]
    fn the_keys_are_entries_of_numbers() {
        let kept = "0:3:112:0;1:0:65:2;";
        assert_eq!(entries(kept).map(|entry| numbers::<u16>(entry, ':')).collect::<Vec<_>>(), [[0, 3, 112, 0], [1, 0, 65, 2]]);
        assert_eq!(entries("").count(), 0);
    }
}
