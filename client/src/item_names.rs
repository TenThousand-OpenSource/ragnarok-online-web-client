//! An item's name as the client composes it wherever it writes one (`ITEM_INFO::DisplayName`,
//! rodecomp `Source/Contents/Items/ItemNaming.cpp`): the refinement before it, the prefixes and suffixes of
//! the cards set into it, its slots after it, the forger's name and the element of a forged weapon with the
//! strength its star crumbs gave it, the word before the egg of a pet that was given a name.
//!
//! The one function every window names an item with is `display_name`; what it needs of the client's tables
//! it asks through `Tables` (in the browser: `web/data/cards.rs` and `web/data/items.rs`).

/// The first card of a thing somebody made: 254 for what was brewed or cooked, 255 for what was forged
/// (rAthena's `CARD0_CREATE` and `CARD0_FORGE`; the client compares with the same two numbers).
pub const MADE: u32 = 254;
pub const FORGED: u32 = 255;

/// The lines of the client's message table the names are made of (`MSGSTRINGID.h`).
pub mod line {
    /// "Double", "Triple", "Quadruple": the same card two, three, four times (`MSI_DOUBLE`…).
    pub const DOUBLE: usize = 446;
    pub const TRIPLE: usize = 447;
    pub const QUADRUPLE: usize = 448;
    /// "'s ", "'s Fire ", "'s Water ", "'s Wind ", "'s Earth " (`MSI_MANUFACTURED_NORMAL_ITEM`…).
    pub const MADE_PLAIN: usize = 450;
    pub const MADE_FIRE: usize = 451;
    pub const MADE_WATER: usize = 452;
    pub const MADE_WIND: usize = 453;
    pub const MADE_EARTH: usize = 454;
    /// "Very Strong ", "Very Very Strong ", "Very Very Very Strong " (`MSI_ONE_STARPIECE`…).
    pub const ONE_STAR: usize = 459;
    pub const TWO_STARS: usize = 460;
    pub const THREE_STARS: usize = 461;
    /// "Nameless": a maker whose name the server has not told (`MSI_NAMELESS_USER`).
    pub const NAMELESS: usize = 581;
    /// "Beloved ": before the egg of a pet that was named (`MSI_NAMED_PET`).
    pub const NAMED_PET: usize = 756;
    /// What stands between the name of the spouse and a wedding ring (the client asks line 2976 by number).
    pub const WEDDING_RING: usize = 2976;
}

/// The lines of the client's message table that say what an item is kept from, for the bits of its entry in
/// `itemmoveinfov5.txt` (a bit for each column, in the file's order: dropping 1, trading 2, storing 4, the
/// cart 8, selling to an NPC 16, mailing 32, the auction 64, the guild's storage 128), in the order the
/// client's tooltip lists them (`UIRoundBoxTooltipWnd`, `UM_SET_ITEM_MOVE_TOOLTIP`: "Can't be dropped",
/// "Can't be put in Storage", "…in Cart", "…attached in Mail", "…traded with player", "…put in Guild
/// Storage", "…sold to NPC"; it does not list the auction).
pub fn restrictions(bits: u8) -> Vec<usize> {
    [(1, 2788), (4, 2789), (8, 2790), (32, 2791), (2, 2792), (128, 2794), (16, 2795)]
        .into_iter()
        .filter(|(bit, _)| bits & bit != 0)
        .map(|(_, line)| line)
        .collect()
}

/// What of an item its name is made of.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Named {
    pub id: u32,
    /// The item's type as the server tells it (the client's `TYPE_…` of `ItemType.h`).
    pub kind: u8,
    pub identified: bool,
    pub refine: u8,
    pub cards: [u32; 4],
}

impl From<&crate::net::inventory::Item> for Named {
    fn from(item: &crate::net::inventory::Item) -> Self {
        Self { id: item.id, kind: item.kind, identified: item.identified, refine: item.refine, cards: item.cards }
    }
}

/// What a card adds to the name of the thing it is set into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Affix {
    /// A word before the name (`cardprefixnametable.txt`).
    Before(String),
    /// Words after the name: a card that `cardpostfixnametable.txt` lists.
    After(String),
}

/// The client's tables a name is made from.
pub trait Tables {
    /// What the item is called once identified, and before.
    fn name(&self, id: u32) -> String;
    fn unidentified(&self, id: u32) -> String;
    /// How many slots the item has (`itemslotcounttable.txt`, the item table's `slotCount`).
    fn slots(&self, id: u32) -> u8;
    /// What a card adds to a name; nothing for what is no card (`g_itemCardNames`).
    fn card(&self, id: u32) -> Option<Affix>;
    /// A line of the client's message table.
    fn message(&self, line: usize) -> String;
    /// The name of a character by its number, once the server has told it (`LookupPlayerName`).
    fn player(&self, character: u32) -> Option<String>;
}

impl Named {
    /// `ITEM_INFO::IsEquipment`: armour, weapons of every hold, guns (and the type 21).
    pub fn is_equipment(&self) -> bool {
        matches!(self.kind, 4 | 5 | 8 | 9 | 11..=15 | 21)
    }

    /// Whether the item is a pet's egg (the client's two ranges of item numbers: 9001 to 9499, 10000 to 10999).
    fn is_egg(&self) -> bool {
        (9001..=9499).contains(&self.id) || (10000..=10999).contains(&self.id)
    }

    /// The character who made the thing: the low words of the third and fourth card (`ITEM_INFO::Creator`).
    /// Nobody for a thing that was not made.
    pub fn maker(&self) -> Option<u32> {
        (self.identified && matches!(self.cards[0], MADE | FORGED)).then(|| self.cards[2].wrapping_add(self.cards[3] << 16))
    }

    /// `ITEM_INFO::HasCards`: the first slot holds a card, or, for equipment that nobody made and that is no
    /// egg, any other slot does.
    fn has_cards(&self, tables: &impl Tables) -> bool {
        let first = tables.card(self.cards[0]).is_some();
        if self.is_egg() || matches!(self.cards[0], MADE | FORGED) || !self.is_equipment() {
            return first;
        }
        first || self.cards[1..].iter().any(|card| tables.card(*card).is_some())
    }

    /// Whether the card fields mean something other than cards: a thing somebody made, or a pet's egg (for
    /// such the item's window draws no row of slots).
    pub fn made_or_egg(&self) -> bool {
        self.is_egg() || matches!(self.cards[0], MADE | FORGED)
    }

    /// The cards set into the item's slots, in their order (what the item's window draws in its row of slots):
    /// none for a thing somebody made or an egg, whose card fields mean other things.
    pub fn cards_set(&self) -> Vec<u32> {
        if self.made_or_egg() {
            return Vec::new();
        }
        self.cards.iter().copied().filter(|card| *card != 0).collect()
    }
}

/// `ITEM_INFO::Prefixes` and `Suffixes`: each different card's word once, with "Double", "Triple" or
/// "Quadruple" before a prefix that several of the same card give, after a suffix.
fn affixes(cards: [u32; 4], tables: &impl Tables) -> (String, String) {
    let (mut before, mut after) = (String::new(), String::new());
    let mut cards = cards;
    for index in 0..4 {
        let card = cards[index];
        if card == 0 {
            continue;
        }
        let Some(affix) = tables.card(card) else { continue };
        let mut count = 1;
        for next in index + 1..4 {
            if cards[next] == card {
                cards[next] = 0;
                count += 1;
            }
        }
        let times = match count {
            2 => Some(line::DOUBLE),
            3 => Some(line::TRIPLE),
            4 => Some(line::QUADRUPLE),
            _ => None,
        }
        .map(|line| tables.message(line));
        match affix {
            Affix::Before(word) if !word.is_empty() => {
                if let Some(times) = times {
                    before.push_str(&format!("{times} "));
                }
                before.push_str(&format!("{word} "));
            }
            Affix::After(word) if !word.is_empty() => {
                after.push_str(&format!(" {word}"));
                if let Some(times) = times {
                    after.push_str(&format!(" {times}"));
                }
            }
            _ => {}
        }
    }
    (before, after)
}

/// `ITEM_INFO::Element`: the words after the maker's name, by the element in the low byte of the second card.
fn element(item: &Named, tables: &impl Tables) -> String {
    match item.cards[1] & 0xff {
        // The wedding rings (2634, 2635) have a line of their own.
        0 if matches!(item.id, 2634 | 2635) => tables.message(line::WEDDING_RING),
        0 => tables.message(line::MADE_PLAIN),
        1 => tables.message(line::MADE_WATER),
        2 => tables.message(line::MADE_EARTH),
        3 => tables.message(line::MADE_FIRE),
        4 => tables.message(line::MADE_WIND),
        _ => String::new(),
    }
}

/// `ITEM_INFO::Star`: the strength the star crumbs gave, kept in the second byte of the second card.
fn star(item: &Named, tables: &impl Tables) -> String {
    match (item.cards[1] >> 8) & 0xff {
        5 => tables.message(line::ONE_STAR),
        10 => tables.message(line::TWO_STARS),
        15 => tables.message(line::THREE_STARS),
        _ => String::new(),
    }
}

/// `ITEM_INFO::Creator`: the maker's name, or "Nameless" until the server has told it.
fn creator(item: &Named, tables: &impl Tables) -> String {
    item.maker().and_then(|maker| tables.player(maker)).filter(|name| !name.is_empty()).unwrap_or_else(|| tables.message(line::NAMELESS))
}

/// ` [n]` after the name of an item with slots (`ITEM_INFO::Slots`).
fn slots(item: &Named, tables: &impl Tables) -> String {
    match tables.slots(item.id) {
        0 => String::new(),
        slots => format!(" [{slots}]"),
    }
}

/// An item's name as the client writes it (`ITEM_INFO::DisplayName`). `detailed` is the argument the client
/// passes as 1 only where it tells of a thing somebody obtained: there a thing that is no equipment is named
/// with its cards and slots too. The grade the client writes after the refinement (`ITEM_INFO::Grade`) belongs
/// to later content and is not written.
pub fn display_name(item: &Named, detailed: bool, tables: &impl Tables) -> String {
    if !item.identified {
        return tables.unidentified(item.id);
    }
    let name = tables.name(item.id);
    let refine = if item.refine > 0 { format!("+{} ", item.refine) } else { String::new() };
    if item.is_egg() && item.cards[3] == 1 {
        return format!("{}{name}", tables.message(line::NAMED_PET));
    }
    if item.cards[0] == MADE && item.cards[1] == 0 {
        return format!("{refine}{}{}{name}", creator(item, tables), element(item, tables));
    }
    if !item.is_equipment() && !detailed {
        return name;
    }
    if item.has_cards(tables) {
        let (before, after) = affixes(item.cards, tables);
        return format!("{refine}{before}{name}{after}{}", slots(item, tables));
    }
    if !matches!(item.cards[0], MADE | FORGED) {
        return format!("{refine}{name}{}", slots(item, tables));
    }
    format!("{refine}{}{}{}{name}", star(item, tables), creator(item, tables), element(item, tables))
}

#[cfg(test)]
mod tests;
