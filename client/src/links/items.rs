//! The links to items in a line of chat (`<ITEML>…</ITEML>`): how one is read, how one is written, and a line
//! cut into its words and its links. The client shows such a link as the item's name between `<` and `>`
//! (`UIChatHisBox::RenderItemLinks`).

/// A piece of a line of chat: plain words, or a link to an item (`<ITEML>…</ITEML>`), which the client shows as
/// the item's name between `<` and `>` (`UIChatHisBox::RenderItemLinks`).
#[derive(Debug, PartialEq)]
pub enum Piece<'line> {
    Words(&'line str),
    Item(Link),
}

/// What a link to an item says of it (`UIChatHisBox::DecodeItemLinkFields`): the item, and how far it is refined.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Link {
    pub item: u32,
    pub refine: u32,
}

/// The signs that part the fields of a link, as the client's table has them.
const PARTING: &str = "!#$%&'()*+,-/";

/// A number written with the digits, the small and the large letters, sixty-two in all.
fn base62(digits: &str) -> Option<u32> {
    digits.chars().try_fold(0u32, |number, digit| {
        let value = match digit {
            '0'..='9' => digit as u32 - '0' as u32,
            'a'..='z' => digit as u32 - 'a' as u32 + 10,
            'A'..='Z' => digit as u32 - 'A' as u32 + 36,
            _ => return None,
        };
        number.checked_mul(62)?.checked_add(value)
    })
}

/// The same number written so.
fn to_base62(mut number: u32) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut written = Vec::new();
    loop {
        written.push(DIGITS[(number % 62) as usize]);
        number /= 62;
        if number == 0 {
            break;
        }
    }
    written.reverse();
    String::from_utf8(written).unwrap_or_default()
}

/// Reads what is between `<ITEML>` and `</ITEML>`: five signs for where the thing is worn, one for whether it
/// is worn at all, the item's number up to the first parting sign, and after a `%` how far it is refined.
pub fn link(data: &str) -> Option<Link> {
    let rest = data.get(6..)?;
    let end = rest.find(|sign| PARTING.contains(sign)).unwrap_or(rest.len());
    let item = base62(&rest[..end]).filter(|item| *item != 0)?;
    let refine = rest[end..].strip_prefix('%').map(|after| &after[..after.find(|sign| PARTING.contains(sign)).unwrap_or(after.len())]).and_then(base62).unwrap_or(0);
    Some(Link { item, refine })
}

/// A link to an item as the client writes it into what is said (`UIItemTagCtrl::AddItemTag`): where it is worn,
/// whether it is equipment, the item, how far it is refined when it is, and what is set into its four slots.
pub fn link_to(item: u32, wearable: u32, equipment: bool, refine: u8, cards: [u32; 4]) -> String {
    let mut data = format!("{:0>5}{}{}", to_base62(wearable), u8::from(equipment), to_base62(item));
    if refine > 0 {
        data.push_str(&format!("%{:0>2}", to_base62(u32::from(refine))));
    }
    for card in cards {
        data.push_str(&format!("){:0>2}", to_base62(card)));
    }
    format!("<ITEML>{data}</ITEML>")
}

/// A line of chat cut into its words and its links. A link that cannot be read stays as the words it is.
pub fn pieces(line: &str) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("<ITEML>") {
        let Some(length) = rest[start..].find("</ITEML>") else { break };
        let Some(found) = link(&rest[start + 7..start + length]) else { break };
        if start > 0 {
            pieces.push(Piece::Words(&rest[..start]));
        }
        pieces.push(Piece::Item(found));
        rest = &rest[start + length + 8..];
    }
    if !rest.is_empty() {
        pieces.push(Piece::Words(rest));
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_is_read_as_the_server_writes_it() {
        // rAthena's `create_item_link` for a Knife [3] (1202) refined to 7 with a card 4001 in its first slot.
        let knife = "<ITEML>000021jo%07&01'00)12x)00)00)00</ITEML>";
        assert_eq!(pieces(&format!("Look: {knife} here")), vec![Piece::Words("Look: "), Piece::Item(Link { item: 1202, refine: 7 }), Piece::Words(" here")]);
        assert_eq!(link("000000b9"), Some(Link { item: 691, refine: 0 }));
        assert_eq!(link("0000"), None);
        assert_eq!(pieces("<ITEML>broken"), vec![Piece::Words("<ITEML>broken")]);
        assert_eq!(pieces("plain"), vec![Piece::Words("plain")]);
    }

    #[test]
    fn a_link_is_written_as_it_is_read() {
        let written = link_to(1202, 2, true, 7, [4001, 0, 0, 0]);
        assert_eq!(written, "<ITEML>000021jo%07)12x)00)00)00</ITEML>");
        assert_eq!(pieces(&written), vec![Piece::Item(Link { item: 1202, refine: 7 })]);
        assert_eq!(to_base62(0), "0");
        assert_eq!(base62("jo"), Some(1202));
    }
}
