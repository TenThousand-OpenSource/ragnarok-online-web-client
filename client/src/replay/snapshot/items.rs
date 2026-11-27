//! Section 8 of a replay file: the lists of items (`CSession::BuildItemReplayRecords`, `Replay.cpp`, read by
//! `CSession::ReadReplayItems`, `Contents/ClientState.cpp`). Between 4500 and 4501: the zeny of an exchange
//! (4701, 4702), thirteen lists (4510 to 4522, `SerializeItems`, `Client/Session/Session.cpp`) and six rows of
//! exactly ten (4601 to 4606), each list its items one after another with no count. One item is 21 tagged
//! records between 281 and 282 (`ITEM_INFO::Serialize`, `Contents/Items/ItemNaming.cpp`, read by
//! `ITEM_INFO::Deserialize`, `Contents/Items/ItemInfo.cpp`).

use super::*;

/// The lists by their tags: the inventory (`m_inventoryItems`), the storage (`storageItems`) and the cart
/// (`cartItems`). The others are a shop's (4511 to 4513), a merchant's (4516 to 4521) and the inventory as
/// filtered (4522).
pub const INVENTORY: u16 = 4510;
pub const STORAGE: u16 = 4514;
pub const CART: u16 = 4515;
/// The rows of ten: what is worn (`equippedItems`); the others are its second set (4602), the costumes and
/// theirs (4603, 4604) and the two sides of an exchange (4605, 4606).
pub const EQUIPPED: u16 = 4601;
/// How many bytes an item takes beside its name: 21 records and the 95 bytes in them, by the sizes
/// `ITEM_INFO::Serialize` gives each.
pub const ITEM_BYTES: usize = 221;

/// An item as the client's lists keep it (`ITEM_INFO`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    /// The kind of item, as the packets tell it (`identity[0]`).
    pub kind: u32,
    /// Its place in its list (`identity[1]`).
    pub index: u32,
    /// Where it can be worn, the server's bits (`identity[2]`).
    pub location: u32,
    /// Where it is worn; 0 when it is not (`identity[3]`).
    pub worn: u32,
    pub quantity: i32,
    pub prices: [u32; 2],
    /// The cards in it.
    pub cards: [u32; 4],
    /// The item's number (the client keeps it as a text of digits and writes it as a number).
    pub id: i32,
    /// The name as shown (`displayName`).
    pub name: Vec<u8>,
    pub identified: bool,
    pub damaged: bool,
    /// How far it is refined.
    pub refine: i32,
    /// `bindOnEquip`.
    pub bind: u16,
    /// When a rented one runs out (`expireTime`).
    pub expires: u32,
    /// What it looks like when worn (`sprite`).
    pub sprite: i32,
    /// Its grade of enchanting.
    pub grade: i16,
    /// Its random options, five at most: each the option, its value and its parameter.
    pub options: Vec<(i16, i16, u8)>,
}

/// The lists of section 8.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Items {
    pub inventory: Vec<Item>,
    pub cart: Vec<Item>,
    pub storage: Vec<Item>,
    /// The other ten lists that hold anything, each by its tag (4511 to 4513, 4516 to 4522).
    pub other: Vec<(u16, Vec<Item>)>,
    /// What is worn, by the client's ten places (0 the lower headgear, 1 the right hand, 2 the garment, 3
    /// an accessory, 4 the armour, 5 the left hand, 6 the shoes, 7 the other accessory, 8 the upper
    /// headgear, 9 the middle one).
    pub equipped: [Item; 10],
    /// The other five rows that hold anything, each by its tag (4602 to 4606) and of ten items.
    pub rows: Vec<(u16, Vec<Item>)>,
}

/// One item: its 21 records in the client's order.
fn item(to: &mut Vec<u8>, item: &Item) {
    let options = &item.options[..item.options.len().min(5)];
    let mut option_bytes: Vec<u8> = options.iter().flat_map(|(id, value, with)| [&id.to_le_bytes()[..], &value.to_le_bytes(), &[*with]].concat()).collect();
    option_bytes.resize(25, 0);
    tagged(to, 281, &[]);
    tagged(to, 283, &item.kind.to_le_bytes());
    tagged(to, 285, &item.index.to_le_bytes());
    tagged(to, 284, &item.location.to_le_bytes());
    tagged(to, 286, &item.worn.to_le_bytes());
    tagged(to, 287, &item.quantity.to_le_bytes());
    tagged(to, 288, &item.prices[0].to_le_bytes());
    tagged(to, 289, &item.prices[1].to_le_bytes());
    tagged(to, 290, &item.cards.iter().flat_map(|card| card.to_le_bytes()).collect::<Vec<u8>>());
    tagged(to, 291, &item.id.to_le_bytes());
    tagged(to, 292, &item.name);
    tagged(to, 293, &[item.identified.into()]);
    tagged(to, 294, &[item.damaged.into()]);
    tagged(to, 295, &item.refine.to_le_bytes());
    tagged(to, 296, &item.bind.to_le_bytes());
    tagged(to, 297, &item.expires.to_le_bytes());
    tagged(to, 298, &item.sprite.to_le_bytes());
    tagged(to, 299, &item.grade.to_le_bytes());
    tagged(to, 300, &(options.len() as i32).to_le_bytes());
    tagged(to, 301, &option_bytes);
    tagged(to, 282, &[]);
}

/// The items of a list one after another.
fn list(items: &[Item]) -> Vec<u8> {
    let mut bytes = Vec::new();
    items.iter().for_each(|one| item(&mut bytes, one));
    bytes
}

/// One record of an item taken into it.
fn told(item: &mut Item, counted: &mut usize, tag: u16, bytes: &[u8]) {
    match tag {
        283 => item.kind = u32_of(bytes),
        285 => item.index = u32_of(bytes),
        284 => item.location = u32_of(bytes),
        286 => item.worn = u32_of(bytes),
        287 => item.quantity = i32_of(bytes),
        288 => item.prices[0] = u32_of(bytes),
        289 => item.prices[1] = u32_of(bytes),
        290 => item.cards.iter_mut().zip(bytes.chunks(4)).for_each(|(card, bytes)| *card = u32_of(bytes)),
        291 => item.id = i32_of(bytes),
        292 => item.name = bytes.to_vec(),
        293 => item.identified = u8_of(bytes) != 0,
        294 => item.damaged = u8_of(bytes) != 0,
        295 => item.refine = i32_of(bytes),
        296 => item.bind = u16_of(bytes),
        297 => item.expires = u32_of(bytes),
        298 => item.sprite = i32_of(bytes),
        299 => item.grade = i16_of(bytes),
        300 => *counted = i32_of(bytes).clamp(0, 5) as usize,
        301 => item.options = bytes.chunks_exact(5).take(5).map(|one| (i16_of(one), i16_of(&one[2..]), one[4])).collect(),
        _ => {}
    }
}

/// The items of a list (`ReadItemList`): of the five options of each, as many as it says it has.
fn listed(list: &[u8]) -> Vec<Item> {
    let (mut found, mut one, mut counted) = (Vec::new(), Item::default(), 0);
    for (tag, bytes) in tags(list) {
        match tag {
            281 => (one, counted) = (Item::default(), 0),
            282 => {
                one.options.truncate(counted);
                found.push(std::mem::take(&mut one));
            }
            _ => told(&mut one, &mut counted, tag, bytes),
        }
    }
    found
}

/// A row of exactly ten: what is given of it, empty items after.
fn row(items: &[Item]) -> Vec<u8> {
    let mut ten = items[..items.len().min(10)].to_vec();
    ten.resize(10, Item::default());
    list(&ten)
}

/// Section 8: every tag the client writes, in its order; a list with nothing in it is a record of no bytes,
/// a row always its ten items. The zeny of an exchange (4701, 4702) is not kept by this client.
pub fn written(items: &Items) -> Vec<u8> {
    let by_tag = |lists: &[(u16, Vec<Item>)], tag: u16| lists.iter().find(|(of, _)| *of == tag).map_or(Vec::new(), |(_, list)| list.clone());
    let mut section = Vec::new();
    tagged(&mut section, 4500, &[]);
    tagged(&mut section, 4701, &0i32.to_le_bytes());
    tagged(&mut section, 4702, &0i32.to_le_bytes());
    for tag in 4510..=4522u16 {
        match tag {
            INVENTORY => tagged(&mut section, tag, &list(&items.inventory)),
            STORAGE => tagged(&mut section, tag, &list(&items.storage)),
            CART => tagged(&mut section, tag, &list(&items.cart)),
            _ => tagged(&mut section, tag, &list(&by_tag(&items.other, tag))),
        }
    }
    tagged(&mut section, EQUIPPED, &row(&items.equipped));
    for tag in 4602..=4606u16 {
        tagged(&mut section, tag, &row(&by_tag(&items.rows, tag)));
    }
    tagged(&mut section, 4501, &[]);
    section
}

/// Section 8 read back. A list of the other ten is kept when it holds an item, a row of the other five when
/// one of its ten is not empty. (The client ends at 4501 or 4703; nothing stands after either.)
pub fn read(section: &[u8]) -> Option<Items> {
    let mut items = Items::default();
    for (tag, bytes) in opened(section, 4500)? {
        let found = listed(bytes);
        match tag {
            4501 | 4703 => break,
            INVENTORY => items.inventory = found,
            STORAGE => items.storage = found,
            CART => items.cart = found,
            4511..=4522 if !found.is_empty() => items.other.push((tag, found)),
            EQUIPPED => items.equipped.iter_mut().zip(found).for_each(|(place, item)| *place = item),
            4602..=4606 if found.iter().any(|item| *item != Item::default()) => items.rows.push((tag, found)),
            _ => {}
        }
    }
    Some(items)
}
