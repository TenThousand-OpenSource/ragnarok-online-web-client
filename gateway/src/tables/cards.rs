//! What the client's item naming reads beside the item table (`LoadItemNameTables`, rodecomp
//! `Source/Contents/Items/ItemNaming.cpp`): the cards with the word each adds to a name and its illustration,
//! what may not be dropped, traded or stored, what each thing a smith makes is made of, and where a thing is
//! worn with the two figures the panel beside an NPC's shop compares.

use super::*;

/// `{"<id>": [word, after, picture]}` for every card: the word it adds to the name of the thing it is set into
/// (`data\cardprefixnametable.txt`), 1 when the word goes after the name (the cards `cardpostfixnametable.txt`
/// lists) and 0 when before, and the name of its illustration (`num2cardillustnametable.txt`: a file of
/// `texture\유저인터페이스\cardbmp`). A card is what any of the four tables names, as for the client.
pub fn card_names(prefixes: &str, postfixes: &str, listed: &str, pictures: &str) -> serde_json::Value {
    let mut cards: BTreeMap<u32, (String, u8, String)> = BTreeMap::new();
    for line in hash_fields(listed) {
        if let Some(id) = line.first().and_then(|id| id.parse().ok()) {
            cards.entry(id).or_default();
        }
    }
    for line in hash_fields(pictures) {
        if let (Some(id), Some(picture)) = (line.first().and_then(|id| id.parse().ok()), line.get(1)) {
            cards.entry(id).or_default().2 = (*picture).to_owned();
        }
    }
    for line in hash_fields(prefixes) {
        if let (Some(id), Some(word)) = (line.first().and_then(|id| id.parse().ok()), line.get(1)) {
            cards.entry(id).or_default().0 = (*word).to_owned();
        }
    }
    for line in hash_fields(postfixes) {
        if let Some(id) = line.first().and_then(|id| id.parse().ok()) {
            cards.entry(id).or_default().1 = 1;
        }
    }
    cards
        .into_iter()
        .map(|(id, (word, after, picture))| (id.to_string(), serde_json::json!([word, after, picture])))
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into()
}

/// `{"<id>": bits}` out of `data\itemmoveinfov5.txt`, whose lines are an item and eight numbers: whether it
/// is kept from being dropped (bit 1), traded (2), stored (4), put into the cart (8), sold to an NPC (16),
/// mailed (32), auctioned (64), put into the guild's storage (128). The client keeps every item the file names
/// (`g_itemMoveInfo`), so one with no bit set is given too.
pub fn moves(text: &str) -> serde_json::Value {
    let mut all = serde_json::Map::new();
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or_default();
        let numbers: Vec<u32> = line.split_whitespace().map_while(|number| number.parse().ok()).collect();
        if numbers.len() >= 9 {
            let bits = (1..9).fold(0u32, |bits, at| bits | (u32::from(numbers[at] != 0) << (at - 1)));
            all.insert(numbers[0].to_string(), bits.into());
        }
    }
    all.into()
}

/// `{"<id>": [line, …]}` out of `data\metalprocessitemlist.txt`: under the number of a thing, the lines that
/// say what it is made of ("1 Iron Ore"), up to the next number (`g_metalProcessItems`).
pub fn materials(text: &str) -> serde_json::Value {
    let mut all: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut item = 0;
    for line in text.lines().map(|line| line.trim().trim_end_matches('#').trim()) {
        if line.is_empty() {
            continue;
        }
        match line.parse::<u32>() {
            Ok(id) => item = id,
            Err(_) => all.entry(item).or_default().push(line.to_owned()),
        }
    }
    all.into_iter().map(|(id, lines)| (id.to_string(), serde_json::json!(lines))).collect::<serde_json::Map<String, serde_json::Value>>().into()
}

/// `{"<id>": [places, first, second]}` out of `data\itemslottable.txt` and `data\itemparamtable.txt`, as the
/// client keeps them with an item (`descriptionFlags`, `descriptionValues`): where the thing is worn, in the
/// server's bits, and the two figures the client's panel beside an NPC's shop sums and compares (it writes
/// them as ATK and DEF; `crate::shops::compared` in the browser client). The files are not read by lines but
/// by their `#`: an item and its places take two fields, an item and its figures three. An item the second
/// file names more than once has four figures or more with the client, which then takes none of them: so here
/// (`null` for the two). An item of only one file has nought, or `null`, for what the other would say.
pub fn worn(places: &str, figures: &str) -> serde_json::Value {
    let fields = |text: &str| -> Vec<i64> { hash_fields(text).into_iter().flatten().map(|field| field.trim().parse().unwrap_or(0)).collect() };
    let mut all: BTreeMap<i64, (i64, Vec<i64>)> = BTreeMap::new();
    for pair in fields(places).chunks_exact(2) {
        all.entry(pair[0]).or_default().0 = pair[1];
    }
    for triple in fields(figures).chunks_exact(3) {
        all.entry(triple[0]).or_default().1.extend(&triple[1..]);
    }
    all.into_iter()
        .map(|(id, (places, figures))| {
            let figures = <[i64; 2]>::try_from(figures).ok();
            (id.to_string(), serde_json::json!([places, figures.map(|figures| figures[0]), figures.map(|figures| figures[1])]))
        })
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into()
}

/// `{"names": …, "moves": …, "materials": …, "worn": …}`: the tables above, read out of the client's archives.
pub fn cards(grf: &Grf) -> serde_json::Value {
    let read = |file: &str| grf.read(&format!("data\\{file}.txt")).map(|bytes| korean(&bytes)).unwrap_or_else(|error| {
        eprintln!("tables: {file}.txt: {error}");
        String::new()
    });
    serde_json::json!({
        "names": card_names(&read("cardprefixnametable"), &read("cardpostfixnametable"), &read("carditemnametable"), &read("num2cardillustnametable")),
        "moves": moves(&read("itemmoveinfov5")),
        "materials": materials(&read("metalprocessitemlist")),
        "worn": worn(&read("itemslottable"), &read("itemparamtable")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_card_has_its_word_its_side_and_its_picture() {
        let cards = card_names("4001#Lucky#\n4014#of Ghost#\n", "4014#\n", "4001#\n4014#\n4999#\n", "4001#poring#\n");
        assert_eq!(cards["4001"], serde_json::json!(["Lucky", 0, "poring"]));
        assert_eq!(cards["4014"], serde_json::json!(["of Ghost", 1, ""]));
        assert_eq!(cards["4999"], serde_json::json!(["", 0, ""]));
    }

    #[test]
    fn what_may_not_be_moved_is_a_bit_each() {
        let moves = moves("// id,\tdrop\n2634\t1\t1\t1\t1\t1\t1\t1\t1\t// ring\n501\t0\t1\t0\t0\t0\t1\t0\t0\n");
        assert_eq!(moves["2634"], 255);
        assert_eq!(moves["501"], 2 | 32);
    }

    #[test]
    fn where_a_thing_is_worn_and_its_two_figures_are_read_by_their_fields() {
        let worn = worn("2101#\r\n32#\r\n1101#\r\n2#\r\n5001#\r\n256#\r\n", "2101#\r\n0#2#\r\n1101#\r\n25#0#\r\n1201#\r\n17#0#\r\n1201#\r\n1#1#\r\n");
        assert_eq!(worn["2101"], serde_json::json!([32, 0, 2]));
        assert_eq!(worn["1101"], serde_json::json!([2, 25, 0]));
        assert_eq!(worn["5001"], serde_json::json!([256, null, null]));
        // Named twice: four figures, of which the client takes none.
        assert_eq!(worn["1201"], serde_json::json!([0, null, null]));
    }

    #[test]
    fn what_a_thing_is_made_of_follows_its_number() {
        let made = materials("998#\n1 Iron Ore\n#\n999#\n5 Iron\n1 Coal\n#\n");
        assert_eq!(made["998"], serde_json::json!(["1 Iron Ore"]));
        assert_eq!(made["999"], serde_json::json!(["5 Iron", "1 Coal"]));
    }
}
