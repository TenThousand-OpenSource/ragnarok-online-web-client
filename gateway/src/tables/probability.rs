//! The probabilities: the two lines of the client's message table that the menu's icon of them asks with and
//! opens.

use super::*;

/// The fields of a line of `msgstringtable.csv`: the name of a line of the message table and its text, each
/// written in base 64.
fn fields(line: &str) -> Option<(String, String)> {
    let (name, words) = line.trim_end_matches('\r').split_once(',')?;
    Some((String::from_utf8(base64(name)?).ok()?, String::from_utf8_lossy(&base64(words)?).into_owned()))
}

/// What a text of base 64 stands for (the usual letters, `=` at its end).
fn base64(text: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(text.len() / 4 * 3);
    let (mut bits, mut held) = (0u32, 0);
    for letter in text.trim_end_matches('=').bytes() {
        let six = match letter {
            b'A'..=b'Z' => letter - b'A',
            b'a'..=b'z' => letter - b'a' + 26,
            b'0'..=b'9' => letter - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        (bits, held) = (bits << 6 | u32::from(six), held + 6);
        if held >= 8 {
            held -= 8;
            bytes.push((bits >> held) as u8);
        }
    }
    Some(bytes)
}

/// The lines of the file that have those names.
fn named(csv: &str, names: [&str; 3]) -> [String; 3] {
    let mut found = [const { String::new() }; 3];
    for (name, words) in csv.trim_start_matches('\u{feff}').lines().filter_map(fields) {
        if let Some(place) = names.iter().position(|wanted| *wanted == name) {
            found[place] = words;
        }
    }
    found
}

/// `{"title": "Probablity", "asks": "Visit the probability list webpage?", "address": "https://…"}`: what the
/// menu's icon of the probabilities says when pointed at, asks when pressed, and opens on a yes
/// (`UIMenuIconWnd::OnMouseMove` and `SendMsg`, `MENUICON_PROBABILITY`: the lines 4119, 4129 and 4128). These
/// lines lie past the end of `msgstringtable.txt`: the client of this version fills its table from
/// `msgstringtable.csv` by the lines' names, which are `MSI_PROBABILITY_TITLE`,
/// `MSI_ETC_PROBABILITY_WEB_CONNECT` and `MSI_ETC_PROBABILITY_URL` in the list compiled into it
/// (`messageNames` of rodecomp's `MessageTable.cpp`, the places 4119, 4129 and 4128).
pub fn probability(grf: &Grf) -> serde_json::Value {
    let csv = grf.read("data\\msgstringtable.csv").map(|bytes| String::from_utf8_lossy(&bytes).into_owned()).unwrap_or_default();
    let [title, asks, address] = named(&csv, ["MSI_PROBABILITY_TITLE", "MSI_ETC_PROBABILITY_WEB_CONNECT", "MSI_ETC_PROBABILITY_URL"]);
    serde_json::json!({ "title": title, "asks": asks, "address": address })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_64_is_read() {
        assert_eq!(base64("TVNJX0RPX1lPVV9BR1JFRQ=="), Some(b"MSI_DO_YOU_AGREE".to_vec()));
        assert_eq!(base64("RG8geW91IGFncmVlPw=="), Some(b"Do you agree?".to_vec()));
        assert_eq!(base64(""), Some(Vec::new()));
        assert_eq!(base64("a b"), None);
    }

    #[test]
    fn the_lines_are_found_by_their_names() {
        let csv = "\u{feff}TVNJX0RPX1lPVV9BR1JFRQ==,RG8geW91IGFncmVlPw==\r\nTVNJX0E=,QQ==\nnot a line\n";
        assert_eq!(named(csv, ["MSI_A", "MSI_DO_YOU_AGREE", "MSI_NONE"]), ["A".to_owned(), "Do you agree?".to_owned(), String::new()]);
    }
}
