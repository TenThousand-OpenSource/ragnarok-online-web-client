//! What the client's item naming reads beside the item table (the gateway's `tables/cards.rs`): the cards
//! with the word each adds to a name and its illustration, what may not be dropped, traded or stored, what
//! each thing a smith makes is made of; the names of characters by their numbers, as the server told them
//! (the client's `CPlayerNameDictionary`: who made a thing); and `item_title`, the one way an item's name is
//! written (`crate::item_names`).

use super::*;
use crate::item_names::{Affix, Named, Tables};

thread_local! {
    /// By card: the word it adds to a name, whether the word goes after the name, the name of its illustration.
    static CARDS: RefCell<HashMap<u32, (String, bool, String)>> = RefCell::new(HashMap::new());
    /// By item the client's `itemmoveinfov5.txt` names: a bit for each thing it is kept from.
    static MOVES: RefCell<HashMap<u32, u8>> = RefCell::new(HashMap::new());
    /// By thing a smith makes: the lines that say what it is made of (`metalprocessitemlist.txt`).
    static MATERIALS: RefCell<HashMap<u32, Vec<String>>> = RefCell::new(HashMap::new());
    /// Characters' names by their numbers, as the server told them.
    static MAKERS: RefCell<HashMap<u32, String>> = RefCell::new(HashMap::new());
    /// The numbers a name was looked up for and not found, not asked of the server yet; and those asked.
    static WANTED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    static ASKED: RefCell<std::collections::HashSet<u32>> = RefCell::new(std::collections::HashSet::new());
}

/// What an item is kept from, a bit each (`crate::item_names::restrictions`), if the client's table names it
/// at all (`FindItemMoveInfo`).
pub fn item_moves(id: u32) -> Option<u8> {
    MOVES.with_borrow(|moves| moves.get(&id).copied())
}

/// The lines that say what a thing a smith makes is made of (`FindMetalProcessItems`).
pub fn materials(id: u32) -> Vec<String> {
    MATERIALS.with_borrow(|materials| materials.get(&id).cloned().unwrap_or_default())
}

/// The name of a card's illustration (a file of `cardbmp`), for what is a card with one
/// (`num2cardillustnametable.txt`).
pub fn card_picture(id: u32) -> Option<String> {
    CARDS.with_borrow(|cards| cards.get(&id).map(|(_, _, picture)| picture.clone()).filter(|picture| !picture.is_empty()))
}

/// The server told what a character is called.
pub fn maker_named(character: u32, name: String) {
    MAKERS.with_borrow_mut(|makers| makers.insert(character, name));
}

/// The characters whose names were looked up in vain since this was last asked: each is given once, to be
/// asked of the server (the client's dictionary queues its requests the same way).
pub fn makers_wanted() -> Vec<u32> {
    WANTED.with_borrow_mut(std::mem::take)
}

/// The tables as `crate::item_names` asks them.
struct Client;

impl Tables for Client {
    fn name(&self, id: u32) -> String {
        item(id).map_or_else(|| format!("{} {id}", crate::i18n::tr("Item")), |(name, _)| name)
    }

    fn unidentified(&self, id: u32) -> String {
        item_as(id, false).map_or_else(|| self.name(id), |(name, _)| name)
    }

    fn slots(&self, id: u32) -> u8 {
        item_slots(id)
    }

    fn card(&self, id: u32) -> Option<Affix> {
        CARDS.with_borrow(|cards| cards.get(&id).map(|(word, after, _)| if *after { Affix::After(word.clone()) } else { Affix::Before(word.clone()) }))
    }

    fn message(&self, line: usize) -> String {
        message(line).unwrap_or_default()
    }

    fn player(&self, character: u32) -> Option<String> {
        let known = MAKERS.with_borrow(|makers| makers.get(&character).cloned());
        if known.is_none() && ASKED.with_borrow_mut(|asked| asked.insert(character)) {
            WANTED.with_borrow_mut(|wanted| wanted.push(character));
        }
        known
    }
}

/// An item's name as the client writes it everywhere (`ITEM_INFO::DisplayName`): the one function every
/// window names an item with. `detailed` is the client's argument: true only where it tells of a thing
/// somebody obtained.
pub fn item_title(item: &Named, detailed: bool) -> String {
    crate::item_names::display_name(item, detailed, &Client)
}

/// Takes its tables out of what the gateway gave. A gateway older than these tables gives none: then no card
/// adds a word to a name.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let cards = field(tables, "cards");
    CARDS.set(
        entries::<u32>(&field(&cards, "names"))
            .into_iter()
            .map(|(id, card)| {
                let card = js_sys::Array::from(&card);
                (id, (card.get(0).as_string().unwrap_or_default(), card.get(1).as_f64() == Some(1.0), card.get(2).as_string().unwrap_or_default()))
            })
            .collect(),
    );
    MOVES.set(entries::<u32>(&field(&cards, "moves")).into_iter().filter_map(|(id, bits)| Some((id, bits.as_f64()? as u8))).collect());
    MATERIALS.set(entries::<u32>(&field(&cards, "materials")).into_iter().map(|(id, lines)| (id, texts(&lines))).collect());
}
