//! The packets about those in sight on the map, players, NPCs and monsters: one appearing, walking, turning,
//! leaping and leaving, what it is called and what it does to another (this file); its state, its look and
//! its life changing (`units/changes.rs`); the things that lie on the ground (`units/ground.rs`); and what the
//! client asks for its own character: to attack, to sit, to turn (`units/requests.rs`).
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`packet_idle_unit`, `packet_spawn_unit` and `packet_unit_walking`
//! in `src/map/packets_struct.hpp`).

use super::bytes::{i32_at, int, long, name_at, u16_at, u32_at, word, Writer};

mod changes;
mod ground;
mod requests;

pub use changes::*;
pub use ground::*;
pub use requests::*;
// The spirit spheres are told in packets of their own file; they were here once, and are still named so.
pub use super::spirits::{told as spirits, ZC_SPIRITS, ZC_SPIRITS2};

/// A unit that stands (or sits, or lies dead) where it is.
pub const ZC_NOTIFY_STANDENTRY: u16 = 0x09ff;
/// A unit that has just come into being: spawned, logged in, teleported in.
pub const ZC_NOTIFY_NEWENTRY: u16 = 0x09fe;
/// A unit that is walking: one that walks into sight, or one in sight that starts to walk.
pub const ZC_NOTIFY_MOVEENTRY: u16 = 0x09fd;
/// ZC_NOTIFY_VANISH (0x0080): `u32 id, u8 how`: a unit is gone from sight (0), dead (1), logged out (2) or
/// teleported away (3).
pub const ZC_NOTIFY_VANISH: u16 = 0x0080;
/// ZC_NOTIFY_MOVE (0x0086): `u32 id, u8[6] move, u32 start_time`: someone already in sight walks from one cell to
/// another (packed as the player's own move is).
pub const ZC_NOTIFY_MOVE: u16 = 0x0086;
/// ZC_HIGHJUMP (0x01ff): `u32 id, u16 x, u16 y`: someone is on that cell at once (a leap, a slide, being
/// thrown back: rAthena's `clif_slide`).
pub const ZC_HIGHJUMP: u16 = 0x01ff;
/// ZC_FASTMOVE (0x08d2): `u32 id, u16 x, u16 y`: someone is on that cell at once (rAthena's
/// `clif_fast_movement`), laid out as ZC_HIGHJUMP and read by `slid`.
pub const ZC_FASTMOVE: u16 = 0x08d2;
/// ZC_CHANGE_DIRECTION (0x009c): `u32 id, u16 head, u8 direction`: someone turns where it stands, or turns
/// its head (0 ahead, 1 to the right, 2 to the left). Whose, the head's turn and the direction (0 north,
/// turning left, as the packets that show a unit count it).
pub const ZC_CHANGE_DIRECTION: u16 = 0x009c;
/// ZC_ACK_REQNAMEALL_NPC (0x0adf): `u32 who, u32 group, char name[24], char title[24]`: what a monster, an NPC
/// or a pet is called, with the title and the group a script gave it.
pub const ZC_ACK_REQNAMEALL_NPC: u16 = 0x0adf;
/// ZC_ACK_REQNAME_BYGID (0x0af7, 32 bytes): `u16 flag, u32 character, char name[24]`: what a character is
/// called, by its own number.
pub const ZC_ACK_REQNAME_BYGID: u16 = 0x0af7;
/// ZC_NOTIFY_ACT (0x08c8): `u32 source, u32 target, u32 tick, i32 source_speed, i32 damage_speed, i32 damage,
/// u8 sp, u16 hits, u8 kind, i32 damage2`: someone does something to someone.
pub const ZC_NOTIFY_ACT: u16 = 0x08c8;

/// What kind of thing a unit is, as the server tells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Player,
    Monster,
    Npc,
    /// Pets, homunculi, mercenaries and elementals.
    Companion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub id: u32,
    pub kind: Kind,
    /// The job of a player, or the number of a monster's or an NPC's sprite.
    pub job: u16,
    /// The bits of the state it is in (the server's `option`): mounted, hidden, and so on.
    pub state: u32,
    /// Its body state and its health state (the server's `opt1` and `opt2`): stone, frozen, poisoned, cursed
    /// and the like (`crate::ailments`).
    pub body: u16,
    pub health: u16,
    /// Milliseconds per cell walked.
    pub speed: u16,
    pub sex: u8,
    pub hair_style: u16,
    pub hair_colour: u16,
    pub body_palette: u16,
    /// Headgear as view IDs: bottom, top, middle.
    pub headgear: [u16; 3],
    pub level: u16,
    /// Whether it is a boss (1) or an MVP (2), as the server marks a monster.
    pub boss: u8,
    /// A monster's life, left and most, when the server tells it as the monster comes into sight (it does
    /// for one already wounded; the packet has -1 for both otherwise).
    pub life: Option<(i32, i32)>,
    pub name: String,
    /// The cell the unit is on, or walks from.
    pub cell: (u16, u16),
    /// Which way it faces: 0 is south, then clockwise.
    pub direction: u8,
    /// The walk it is on, when it is walking.
    pub walks: Option<Walked>,
    /// A player's guild (0 for none) and the version of its emblem (`GUID`, `GEmblemVer`).
    pub guild: u32,
    pub emblem: u16,
    /// What a player holds, as item IDs (0 for nothing): the weapon and the shield (`weapon`, `shield`).
    pub weapon: u32,
    pub shield: u32,
    /// Which way a player's head is turned (`headDir`): 0 ahead, 1 to the right, 2 to the left.
    pub head: u8,
    /// How one that stands where it is, is there (`state` of the standing packet alone: rAthena's
    /// `dead_sit`).
    pub posture: Posture,
    /// The bits of its third option (the packet's `virtue`, the server's `opt3`): states that show on the one
    /// in them, as Fury or Steel Body (`crate::state_effects::seen`).
    pub third: u32,
}

/// How one who does not walk is there, as the packet that shows it standing says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Posture {
    #[default]
    Stands,
    Lies,
    Sits,
}

/// A walk as the server tells it: the cell walked from and the cell walked to, where within the first cell the
/// walker sets out (sixteenths from the cell's corner, 8 its middle), and the server's time at which it sets out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Walked {
    pub from: (u16, u16),
    pub to: (u16, u16),
    pub within: (u8, u8),
    pub started: u32,
}

/// A walk out of the six bytes the move packets pack it into (the client's `DecodeSrcDst`): two cells of 10
/// bits a coordinate, then the place within the first cell, 4 bits each way.
pub fn walked(data: &[u8], started: u32) -> Option<Walked> {
    let within = *data.get(5)?;
    let from = cell(data);
    let data: Vec<u16> = data[..5].iter().map(|&byte| u16::from(byte)).collect();
    Some(Walked { from, to: ((data[2] & 0x0f) << 6 | data[3] >> 2, (data[3] & 0x03) << 8 | data[4]), within: (within >> 4, within & 0x0f), started })
}

/// A cell out of the three bytes a packet packs it into, 10 bits a coordinate; the four bits left over are the
/// way the one on it faces, or the start of a second cell. The bytes must be there.
fn cell(data: &[u8]) -> (u16, u16) {
    super::packets::cell_unpacked(data).0
}

fn kind(object: u8) -> Kind {
    match object {
        0 => Kind::Player,
        5 => Kind::Monster,
        7..=10 => Kind::Companion,
        _ => Kind::Npc,
    }
}

/// A unit out of the packet that shows it: standing (0x09ff), new (0x09fe) or walking (0x09fd). The three share
/// their start; the walking one carries the time it set off (after the bottom headgear) and two cells where the
/// others have one.
pub fn unit(id: u16, packet: &[u8]) -> Option<Unit> {
    // Where the fields after the bottom headgear start, and where those after the position do.
    let (after_headgear, after_position, walking) = match id {
        ZC_NOTIFY_STANDENTRY => (37, 69, false),
        ZC_NOTIFY_NEWENTRY => (37, 68, false),
        ZC_NOTIFY_MOVEENTRY => (41, 75, true),
        _ => return None,
    };
    if packet.len() < after_position + 15 {
        return None;
    }
    let position = after_headgear + 26;
    let (cell, direction, walks) = if walking {
        let walk = walked(&packet[position..position + 6], u32_at(packet, 37))?;
        (walk.from, 0, Some(walk))
    } else {
        (cell(&packet[position..position + 3]), packet[position + 2] & 0x0f, None)
    };
    Some(Unit {
        id: u32_at(packet, 5),
        kind: kind(packet[4]),
        speed: u16_at(packet, 13),
        body: u16_at(packet, 15),
        health: u16_at(packet, 17),
        job: u16_at(packet, 23),
        state: u32_at(packet, 19),
        hair_style: u16_at(packet, 25),
        weapon: u32_at(packet, 27),
        shield: u32_at(packet, 31),
        head: u16_at(packet, after_headgear + 8) as u8,
        // The byte before the level, which only the standing packet has.
        posture: match (id, packet[after_position - 1]) {
            (ZC_NOTIFY_STANDENTRY, 1) => Posture::Lies,
            (ZC_NOTIFY_STANDENTRY, 2) => Posture::Sits,
            _ => Posture::Stands,
        },
        headgear: [u16_at(packet, 35), u16_at(packet, after_headgear), u16_at(packet, after_headgear + 2)],
        hair_colour: u16_at(packet, after_headgear + 4),
        body_palette: u16_at(packet, after_headgear + 6),
        sex: packet[position - 1],
        level: u16_at(packet, after_position),
        boss: packet[after_position + 12],
        life: Some((i32_at(packet, after_position + 8), i32_at(packet, after_position + 4))).filter(|(left, most)| *left >= 0 && *most > 0),
        name: packet.get(after_position + 15..).map_or(String::new(), |_| name_at(packet, after_position + 15, 24)),
        cell,
        direction,
        walks,
        guild: u32_at(packet, after_headgear + 12),
        emblem: u16_at(packet, after_headgear + 16),
        third: u32_at(packet, after_headgear + 20),
    })
}

/// Reads ZC_NOTIFY_VANISH: the ID and whether it died.
pub fn vanished(packet: &[u8]) -> Option<(u32, bool)> {
    (packet.len() >= 7).then(|| (u32_at(packet, 2), packet[6] == 1))
}

/// How ZC_NOTIFY_VANISH says its unit is gone: out of sight (0), dead (1), logged out (2), teleported away (3).
pub fn vanished_how(packet: &[u8]) -> Option<u8> {
    packet.get(6).copied()
}

/// Reads ZC_NOTIFY_MOVE: who walks, and the walk.
pub fn moved(packet: &[u8]) -> Option<(u32, Walked)> {
    (packet.len() >= 16).then(|| walked(&packet[6..12], u32_at(packet, 12)).map(|walk| (u32_at(packet, 2), walk)))?
}

/// Reads ZC_HIGHJUMP and ZC_FASTMOVE: who, and the cell it is on.
pub fn slid(packet: &[u8]) -> Option<(u32, (u16, u16))> {
    (packet.len() >= 10).then(|| (u32_at(packet, 2), (u16_at(packet, 6), u16_at(packet, 8))))
}

/// Reads ZC_CHANGE_DIRECTION: whose, the head's turn and the direction.
pub fn turned(packet: &[u8]) -> Option<(u32, u8, u8)> {
    (packet.len() >= 9).then(|| (u32_at(packet, 2), u16_at(packet, 6) as u8, packet[8] % 8))
}

/// Reads ZC_ACK_REQNAMEALL_NPC: who is named, in which group, by what name and title.
pub fn named(packet: &[u8]) -> Option<(u32, u32, String, String)> {
    let packet = packet.get(..58)?;
    Some((u32_at(packet, 2), u32_at(packet, 6), name_at(packet, 10, 24), name_at(packet, 34, 24)))
}

/// What ZC_NOTIFY_ACT tells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Act {
    pub source: u32,
    pub target: u32,
    /// How long the source's motion takes, in milliseconds.
    pub speed: i32,
    /// How long after the motion starts the blow lands, in milliseconds.
    pub delay: i32,
    pub damage: i32,
    /// 0 a hit, 1 picking up, 2 sitting down, 3 standing up, 8 several hits, 10 a critical hit, 11 a lucky miss.
    pub kind: u8,
    /// How many hits the blow is of (believed for the kinds of several hits), and what the other hand adds.
    pub hits: u16,
    pub extra: i32,
}

pub fn act(packet: &[u8]) -> Option<Act> {
    Some(Act {
        source: long(packet, 2)?,
        target: long(packet, 6)?,
        speed: int(packet, 14)?,
        delay: int(packet, 18)?,
        damage: int(packet, 22)?,
        kind: *packet.get(29)?,
        hits: word(packet, 27)?,
        extra: int(packet, 30)?,
    })
}

/// CZ_REQNAME_BYGID (0x0369, 6 bytes): `u32 character`: asks what the character of that number is called
/// (`CPlayerNameDictionary`: who made a thing). The answer is ZC_ACK_REQNAME_BYGID, read by `character_name`.
pub fn ask_name(character: u32) -> Vec<u8> {
    Writer::new(0x0369).u32(character).done()
}

/// ZC_ACK_REQNAME_BYGID: the character's number and its name, as `CGameMode::ReceivePlayerNameUpdate` keeps
/// it: for flag 3 the name told, for flag 4 nothing (the client then writes line 581 of its message table,
/// `MSI_NAMELESS_USER`); any other flag is not read.
pub fn character_name(packet: &[u8]) -> Option<(u32, Option<String>)> {
    let packet = packet.get(..32)?;
    match word(packet, 2)? {
        3 => Some((long(packet, 4)?, Some(name_at(packet, 8, 24)))),
        4 => Some((long(packet, 4)?, None)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
