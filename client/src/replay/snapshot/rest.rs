//! The smaller sections of the session in a replay file: the memorial dungeon (10,
//! `CSession::BuildExtraReplayRecords`, read by `ReadReplayExtra`), the queue of the battlegrounds (12,
//! `BuildEntryQueueReplayRecords`, `ReadReplayEntryQueue`), the shortcut bar (13,
//! `CSession::BuildShortcutReplayRecords`, `ReadReplayShortcuts`) and the shops by their names (14,
//! `CSession::BuildShopReplayRecords`, `ReadReplayShops`). The writers are in `Client/Replay/Replay.cpp`, the
//! readers in `Contents/ClientState.cpp`.

use super::*;

/// The memorial dungeon as the session keeps it (`memorial…`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memorial {
    /// Whether its window is open (1 opens it when the file is read), and where the window stands.
    pub open: i32,
    pub x: i32,
    pub y: i32,
    /// `memorialDungeonState`.
    pub state: i32,
    /// `memorialReplayValue`, which the client does not name further.
    pub value: i32,
    pub name: Vec<u8>,
    /// The place in the queue for one.
    pub position: i32,
    /// When it was made and when it ends.
    pub created: u32,
    pub expires: u32,
}

/// Section 10: 5510 to 5518 between 5500 and 5501.
pub fn memorial(memorial: &Memorial) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 5500, &[]);
    for (tag, value) in (5510..).zip([memorial.open, memorial.x, memorial.y, memorial.state, memorial.value]) {
        tagged(&mut section, tag, &value.to_le_bytes());
    }
    tagged(&mut section, 5515, &memorial.name);
    tagged(&mut section, 5516, &memorial.position.to_le_bytes());
    tagged(&mut section, 5517, &memorial.created.to_le_bytes());
    tagged(&mut section, 5518, &memorial.expires.to_le_bytes());
    tagged(&mut section, 5501, &[]);
    section
}

/// Section 10 read back.
pub fn memorial_read(section: &[u8]) -> Option<Memorial> {
    let mut memorial = Memorial::default();
    for (tag, bytes) in opened(section, 5500)? {
        match tag {
            5510 => memorial.open = i32_of(bytes),
            5511 => memorial.x = i32_of(bytes),
            5512 => memorial.y = i32_of(bytes),
            5513 => memorial.state = i32_of(bytes),
            5514 => memorial.value = i32_of(bytes),
            5515 => memorial.name = bytes.to_vec(),
            5516 => memorial.position = i32_of(bytes),
            5517 => memorial.created = u32_of(bytes),
            5518 => memorial.expires = u32_of(bytes),
            _ => {}
        }
    }
    Some(memorial)
}

/// The window of one who waits in a queue (`UIEntryQueueStandByWnd`): how many wait, the place, the queue.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Standby {
    pub waiting: i32,
    pub ranking: i32,
    pub name: Vec<u8>,
}

/// The queue of the battlegrounds (`g_entryQueueMgr` and its two windows).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Queue {
    /// The number of the entry the character is in (6511), when it is in one.
    pub entry: Option<i32>,
    /// `ready` (6512).
    pub ready: u8,
    /// The queue's name and the character's (6513, 6514), each written only when it is not empty.
    pub queue_name: Vec<u8>,
    pub character_name: Vec<u8>,
    /// The line picked in the window of the battlegrounds (6515), when that window is open.
    pub selected: Option<i32>,
    /// The window of waiting (6517, 6518, then 6516), when it is open.
    pub standby: Option<Standby>,
}

/// Section 12: 6512 always, the others when there is what they tell; an empty queue is 6500, 6512, 6501.
pub fn queue(queue: &Queue) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 6500, &[]);
    if let Some(entry) = queue.entry {
        tagged(&mut section, 6511, &entry.to_le_bytes());
    }
    tagged(&mut section, 6512, &[queue.ready]);
    for (tag, text) in [(6513, &queue.queue_name), (6514, &queue.character_name)] {
        if !text.is_empty() {
            tagged(&mut section, tag, text);
        }
    }
    if let Some(selected) = queue.selected {
        tagged(&mut section, 6515, &selected.to_le_bytes());
    }
    if let Some(standby) = &queue.standby {
        tagged(&mut section, 6517, &standby.waiting.to_le_bytes());
        tagged(&mut section, 6518, &standby.ranking.to_le_bytes());
        tagged(&mut section, 6516, &standby.name);
    }
    tagged(&mut section, 6501, &[]);
    section
}

/// Section 12 read back.
pub fn queue_read(section: &[u8]) -> Option<Queue> {
    let mut queue = Queue::default();
    for (tag, bytes) in opened(section, 6500)? {
        match tag {
            6511 => queue.entry = Some(i32_of(bytes)),
            6512 => queue.ready = u8_of(bytes),
            6513 => queue.queue_name = bytes.to_vec(),
            6514 => queue.character_name = bytes.to_vec(),
            6515 => queue.selected = Some(i32_of(bytes)),
            6517 => queue.standby.get_or_insert_with(Standby::default).waiting = i32_of(bytes),
            6518 => queue.standby.get_or_insert_with(Standby::default).ranking = i32_of(bytes),
            6516 => queue.standby.get_or_insert_with(Standby::default).name = bytes.to_vec(),
            _ => {}
        }
    }
    Some(queue)
}

/// A place of the shortcut bar (`NormalItemSlot`, seven bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Slot {
    /// 0 an item, anything else a skill.
    pub kind: u8,
    /// The item or the skill.
    pub id: u32,
    /// A skill's level; of an item, how many.
    pub value: i16,
}

/// The shortcut bar: its 36 places (`m_shortcutSlots`) and the two quick ones (`quickSlot` and
/// `alternateQuickSlot`).
#[derive(Clone, Debug, PartialEq)]
pub struct Shortcuts {
    pub slots: [Slot; 36],
    pub quick: [Slot; 2],
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self { slots: [Slot::default(); 36], quick: [Slot::default(); 2] }
    }
}

fn slots(slots: &[Slot]) -> Vec<u8> {
    slots.iter().flat_map(|slot| [&[slot.kind][..], &slot.id.to_le_bytes(), &slot.value.to_le_bytes()].concat()).collect()
}

fn slots_read(to: &mut [Slot], bytes: &[u8]) {
    for (slot, bytes) in to.iter_mut().zip(bytes.chunks_exact(7)) {
        *slot = Slot { kind: bytes[0], id: u32_of(&bytes[1..]), value: i16_of(&bytes[5..]) };
    }
}

/// Section 13: 7100 (252 bytes) and 7101 (14) between 7000 and 7001.
pub fn shortcuts(shortcuts: &Shortcuts) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 7000, &[]);
    tagged(&mut section, 7100, &slots(&shortcuts.slots));
    tagged(&mut section, 7101, &slots(&shortcuts.quick));
    tagged(&mut section, 7001, &[]);
    section
}

/// Section 13 read back (each record clears its places first, as the client does).
pub fn shortcuts_read(section: &[u8]) -> Option<Shortcuts> {
    let mut shortcuts = Shortcuts::default();
    for (tag, bytes) in opened(section, 7000)? {
        match tag {
            7100 => {
                shortcuts.slots = [Slot::default(); 36];
                slots_read(&mut shortcuts.slots, bytes);
            }
            7101 => {
                shortcuts.quick = Default::default();
                slots_read(&mut shortcuts.quick, bytes);
            }
            _ => {}
        }
    }
    Some(shortcuts)
}

/// An item of a shop (`ReplayShopItemRecord` without its shop): the row it stands in, the item, and the two
/// numbers the session keeps for it (`first`, `second`, which the client does not name further).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShopItem {
    pub index: u32,
    pub item: i32,
    pub first: i32,
    pub second: i32,
}

/// A shop (an entry of `m_shops`): its number, its items and its flag.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shop {
    pub id: i32,
    pub items: Vec<ShopItem>,
    pub flag: u8,
}

/// The shops: their numbers by their names (`m_nameToId`, a name of 16 bytes at most), and each shop.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shops {
    pub names: Vec<(Vec<u8>, i32)>,
    pub shops: Vec<Shop>,
}

/// Section 14: between 7500 and 7501 a record of 22 bytes a name (7502), then of each shop its items (7504,
/// 20 bytes each) and its flag (7505, 8 bytes: the last three the client leaves as they were, here zeros).
pub fn shops(shops: &Shops) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 7500, &[]);
    for (name, id) in &shops.names {
        tagged(&mut section, 7502, &[&[0, 0][..], &padded(name, 16), &id.to_le_bytes()].concat());
    }
    for shop in &shops.shops {
        for item in &shop.items {
            let numbers = [shop.id, item.index as i32, item.item, item.first, item.second];
            tagged(&mut section, 7504, &numbers.iter().flat_map(|number| number.to_le_bytes()).collect::<Vec<u8>>());
        }
        tagged(&mut section, 7505, &[&shop.id.to_le_bytes()[..], &[shop.flag, 0, 0, 0]].concat());
    }
    tagged(&mut section, 7501, &[]);
    section
}

/// Section 14 read back.
pub fn shops_read(section: &[u8]) -> Option<Shops> {
    fn shop(shops: &mut Vec<Shop>, id: i32) -> &mut Shop {
        let place = shops.iter().position(|shop| shop.id == id).unwrap_or_else(|| {
            shops.push(Shop { id, ..Shop::default() });
            shops.len() - 1
        });
        &mut shops[place]
    }
    let mut shops = Shops::default();
    for (tag, bytes) in opened(section, 7500)? {
        match tag {
            7502 if bytes.len() >= 22 => shops.names.push((unpadded(&bytes[2..18]), i32_of(&bytes[18..]))),
            7504 if bytes.len() >= 20 => {
                let item = ShopItem { index: u32_of(&bytes[4..]), item: i32_of(&bytes[8..]), first: i32_of(&bytes[12..]), second: i32_of(&bytes[16..]) };
                shop(&mut shops.shops, i32_of(bytes)).items.push(item);
            }
            7505 if bytes.len() >= 5 => shop(&mut shops.shops, i32_of(bytes)).flag = bytes[4],
            _ => {}
        }
    }
    Some(shops)
}
