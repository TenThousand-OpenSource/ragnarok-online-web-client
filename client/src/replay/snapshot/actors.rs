//! Section 15 of a replay file: those in sight when the recording began (`CGameMode::SaveActorStates`, read by
//! `CGameMode::ReadReplayActors`, `GameMode.cpp`). Between 10000 and 10001 a row of chunks an actor: the packet
//! that shows it, whole with its number and its length (0xD6 one that walks, ZC_NOTIFY_MOVEENTRY 0x09FD; 0xD9
//! one that does not, ZC_NOTIFY_STANDENTRY 0x09FF: `BuildActorEntryPacket`, `Client/Mode/EntityInit.cpp`), or
//! 0xDA for one made by its looks alone; then what hangs over it: its chat room (0xC9), its stall's sign (0xCA
//! selling, 0xCC buying), a bar that fills (0xCB), the Madogear's status (0x191). Each chunk is handed to the
//! reader of the packet it holds.

use super::*;

/// The kinds of chunk the client writes, and how many bytes those of one size hold.
pub mod kind {
    /// One that walks: the packet 0x09FD, 0x5A bytes and the name.
    pub const WALKS: u16 = 0xd6;
    /// One that stands, sits or lies: the packet 0x09FF, 0x54 bytes and the name.
    pub const STANDS: u16 = 0xd9;
    /// One made by its looks (`GameModeStoredRecord00DA`, 0x3F bytes).
    pub const LOOKS: u16 = 0xda;
    /// Its chat room (`ChatRoomEntryPacket`, 0x11 bytes and the title).
    pub const ROOM: u16 = 0xc9;
    /// The sign of its stall (`ShopEntryPacket`, 0x56 bytes), selling and buying.
    pub const STALL: u16 = 0xca;
    pub const BUYING: u16 = 0xcc;
    /// A bar that fills over it (`ProgressPacket`, 0xA bytes).
    pub const BAR: u16 = 0xcb;
    /// The Madogear it sits in (`GameModeStoredRecord0191`, 0x1C bytes).
    pub const MADOGEAR: u16 = 0x191;
}

/// One chunk of the section: its kind and what it holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActorChunk {
    pub kind: u16,
    pub bytes: Vec<u8>,
}

/// Section 15: the chunks as they are given, between 10000 and 10001.
pub fn written(chunks: &[ActorChunk]) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 10000, &[]);
    chunks.iter().for_each(|chunk| tagged(&mut section, chunk.kind, &chunk.bytes));
    tagged(&mut section, 10001, &[]);
    section
}

/// Section 15 read back: every chunk between the two, also of kinds only older clients wrote (0xD0 to 0xD8).
pub fn read(section: &[u8]) -> Option<Vec<ActorChunk>> {
    let chunks = opened(section, 10000)?.into_iter().filter(|(kind, _)| !matches!(kind, 10000 | 10001));
    Some(chunks.map(|(kind, bytes)| ActorChunk { kind, bytes: bytes.to_vec() }).collect())
}

/// One who does not walk, as ZC_NOTIFY_STANDENTRY tells of it (`ActorStandEntryPacket`): the fields of
/// `crate::net::actors::Unit` and those of the packet it does not read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Standing {
    /// What it is (`objectType`): 0 a player, 5 a monster, 6 an NPC, 7 to 10 a pet, a homunculus, a
    /// mercenary, an elemental.
    pub kind: u8,
    pub id: u32,
    /// A player's character.
    pub character: u32,
    /// Milliseconds a cell walked.
    pub speed: u16,
    /// The three states: the body's, the health's, and the bits of the third (`effectState`).
    pub body_state: u16,
    pub health_state: u16,
    pub effect_state: u32,
    pub job: u16,
    /// The hair's style.
    pub hair: u16,
    pub weapon: u32,
    pub shield: u32,
    /// What is worn on the head as it looks: the lower, the upper, the middle (`accessory`, `accessory2`,
    /// `accessory3`).
    pub headgear: [u16; 3],
    pub hair_colour: u16,
    pub clothes_colour: u16,
    /// Which way the head is turned.
    pub head_direction: u16,
    pub robe: u16,
    pub guild: u32,
    /// The version of the guild's emblem.
    pub emblem: u16,
    pub honor: u16,
    /// The packet's `virtue`: the server's third option.
    pub virtue: u32,
    /// `isPkModeOn`.
    pub pk: u8,
    pub sex: u8,
    pub x: u16,
    pub y: u16,
    /// Which way it faces, as the packet counts it.
    pub direction: u8,
    /// `xSize` and `ySize`.
    pub size: [u8; 2],
    /// 0 it stands, 1 it lies dead, 2 it sits.
    pub state: u8,
    pub level: u16,
    pub font: u16,
    /// A monster's life, the most and what is left; -1 both when it is not told.
    pub max_hp: i32,
    pub hp: i32,
    /// 1 a boss, 2 an MVP.
    pub boss: u8,
    /// The body's style (`body`).
    pub body: u16,
    /// The name, as long as it is.
    pub name: Vec<u8>,
}

/// How many bytes ZC_NOTIFY_STANDENTRY has before the name.
pub const STAND_BYTES: usize = 0x54;

/// A cell and the way faced in the three bytes the packet has for them (`ActorStandEntryPacket::posDir`).
pub(super) fn place(x: u16, y: u16, direction: u8) -> [u8; 3] {
    [(x >> 2) as u8, ((x << 6) | ((y >> 4) & 0x3f)) as u8, ((y << 4) as u8) | (direction & 0x0f)]
}

/// The three runs of fields the packets that show one have alike (`ActorStandEntryPacket`,
/// `ActorMoveEntryPacket`): up to the lower headgear, from the upper headgear to the sex, and from the level on
/// with the name. Between the first two the one that walks has the time it set out at; between the last two
/// each has its place, and the one that stands how it stands.
fn runs(one: &Standing) -> [Vec<u8>; 3] {
    let short = |value: u16| value.to_le_bytes().to_vec();
    let long = |value: u32| value.to_le_bytes().to_vec();
    let [lower, upper, middle] = one.headgear;
    let begun = [vec![one.kind], long(one.id), long(one.character), short(one.speed), short(one.body_state), short(one.health_state)];
    let dressed = [long(one.effect_state), short(one.job), short(one.hair), long(one.weapon), long(one.shield), short(lower)];
    let looked = [short(upper), short(middle), short(one.hair_colour), short(one.clothes_colour), short(one.head_direction), short(one.robe)];
    let named = [long(one.guild), short(one.emblem), short(one.honor), long(one.virtue), vec![one.pk, one.sex]];
    let told = [short(one.level), short(one.font), long(one.max_hp as u32), long(one.hp as u32), vec![one.boss], short(one.body), one.name.clone()];
    [[begun.concat(), dressed.concat()].concat(), [looked.concat(), named.concat()].concat(), told.concat()]
}

/// A packet of that number out of its parts, its length after its number.
fn entry(packet: u16, parts: &[&[u8]]) -> Vec<u8> {
    let body = parts.concat();
    [&packet.to_le_bytes()[..], &((4 + body.len()) as u16).to_le_bytes(), &body].concat()
}

/// The whole ZC_NOTIFY_STANDENTRY (0x09FF) of one, as a chunk 0xD9 holds it: 0x54 bytes and the name, its
/// length after its number.
pub fn stand_entry(one: &Standing) -> Vec<u8> {
    let [first, second, third] = runs(one);
    let placed = [&place(one.x, one.y, one.direction)[..], &one.size, &[one.state]].concat();
    entry(crate::net::actors::ZC_NOTIFY_STANDENTRY, &[&first, &second, &placed, &third])
}

/// How many bytes ZC_NOTIFY_MOVEENTRY has before the name.
pub const MOVE_BYTES: usize = 0x5a;

/// The whole ZC_NOTIFY_MOVEENTRY (0x09FD) of one that walks, as a chunk 0xD6 holds it: 0x5A bytes and the
/// name. It walks from its cell to `to`, having set out at the server's time `started` from the middle of
/// its cell (`moveData`: the two cells in ten bits a number, then where within the first, in sixteenths).
pub fn move_entry(one: &Standing, to: (u16, u16), started: u32) -> Vec<u8> {
    let [first, second, third] = runs(one);
    let (x0, y0, (x1, y1)) = (one.x, one.y, to);
    let low = |value: u16| value as u8;
    let walk = [low(x0 >> 2), low(x0 << 6) | low(y0 >> 4) & 0x3f, low(y0 << 4) | low(x1 >> 6) & 0x0f, low(x1 << 2) | low(y1 >> 8) & 3, low(y1), 0x88];
    entry(crate::net::actors::ZC_NOTIFY_MOVEENTRY, &[&first, &started.to_le_bytes(), &second, &walk, &one.size, &third])
}

/// The chunk of one that walks.
pub fn walks(one: &Standing, to: (u16, u16), started: u32) -> ActorChunk {
    ActorChunk { kind: kind::WALKS, bytes: move_entry(one, to, started) }
}

/// The chunk of one who does not walk.
pub fn stands(one: &Standing) -> ActorChunk {
    ActorChunk { kind: kind::STANDS, bytes: stand_entry(one) }
}

/// The chunk of a chat room over its owner (`ChatRoomEntryPacket`): two bytes nobody reads, the length (17
/// and the title's), the owner, the room, the most it takes and how many are in it, its kind, the title.
pub fn room(owner: u32, room: u32, limit: u16, inside: u16, kind: u8, title: &[u8]) -> ActorChunk {
    let length = (17 + title.len()) as u16;
    let numbers = [&[0, 0][..], &length.to_le_bytes(), &owner.to_le_bytes(), &room.to_le_bytes(), &limit.to_le_bytes(), &inside.to_le_bytes()];
    let bytes = [&numbers.concat()[..], &[kind], title].concat();
    ActorChunk { kind: kind::ROOM, bytes }
}

/// The chunk of a stall's sign over its owner (`ShopEntryPacket`, 0x56 bytes): the number of the packet
/// that tells of one (ZC_STORE_ENTRY 0x0131 selling, ZC_BUYING_STORE_ENTRY 0x0814 buying), the owner, the
/// title in 80 bytes.
pub fn sign(buying: bool, owner: u32, title: &[u8]) -> ActorChunk {
    let (kind, packet) = if buying { (kind::BUYING, 0x0814u16) } else { (kind::STALL, 0x0131) };
    ActorChunk { kind, bytes: [&packet.to_le_bytes()[..], &owner.to_le_bytes(), &padded(title, 80)].concat() }
}

/// One made by its looks alone (`GameModeStoredRecord00DA`, 0x3F bytes: two bytes nobody reads, the account,
/// the job, the cell, the sex, the head, the weapon and the shield, the three headgears, the two colours, the
/// robe, the name in 24 bytes, the body's costume) as one who stands: a player facing as the client makes it
/// (`SpawnStandingPc2`: the direction 4). None for a record too short.
pub fn looks(record: &[u8]) -> Option<Standing> {
    let record = record.get(..0x3f)?;
    let short = |at: usize| u16_of(&record[at..]);
    Some(Standing {
        id: u32_of(&record[2..]),
        job: u32_of(&record[6..]) as u16,
        x: short(0x0a),
        y: short(0x0c),
        sex: record[0x0e],
        hair: short(0x0f),
        weapon: u32_of(&record[0x11..]),
        shield: u32_of(&record[0x15..]),
        headgear: [short(0x19), short(0x1b), short(0x1d)],
        hair_colour: short(0x1f),
        clothes_colour: short(0x21),
        robe: short(0x23),
        name: unpadded(&record[0x25..0x3d]),
        body: short(0x3d),
        direction: 4,
        max_hp: -1,
        hp: -1,
        ..Standing::default()
    })
}

/// The packet a chunk is handed to this client's readers as; none for a kind this client shows nothing of
/// (a bar that fills, a Madogear, a stall that buys, the shapes only older clients wrote) or a chunk too
/// short for what it should hold. The client's readers take the chunk as it is and do not look at the
/// packet's number in it: here the number is written anew, and the length a packet carries is set to the
/// bytes there are, whatever the file says.
pub fn packet(chunk: &ActorChunk) -> Option<Vec<u8>> {
    use crate::net::{actors, chat_rooms, vending};
    let sized = |id: u16, least: usize| {
        let mut bytes = chunk.bytes.get(..chunk.bytes.len().min(usize::from(u16::MAX))).filter(|bytes| bytes.len() >= least)?.to_vec();
        bytes[..2].copy_from_slice(&id.to_le_bytes());
        let length = (bytes.len() as u16).to_le_bytes();
        bytes[2..4].copy_from_slice(&length);
        Some(bytes)
    };
    match chunk.kind {
        kind::WALKS => sized(actors::ZC_NOTIFY_MOVEENTRY, MOVE_BYTES),
        kind::STANDS => sized(actors::ZC_NOTIFY_STANDENTRY, STAND_BYTES),
        kind::LOOKS => looks(&chunk.bytes).map(|one| stand_entry(&one)),
        kind::ROOM => sized(chat_rooms::ZC_ROOM_NEWENTRY, 17),
        kind::STALL => {
            let mut bytes = chunk.bytes.get(..0x56)?.to_vec();
            bytes[..2].copy_from_slice(&vending::ZC_STORE_ENTRY.to_le_bytes());
            Some(bytes)
        }
        _ => None,
    }
}
