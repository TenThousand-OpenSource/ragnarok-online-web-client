//! The packets about everyone else on the map: other players, NPCs and monsters appearing, walking and leaving.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`packet_idle_unit`, `packet_spawn_unit` and `packet_unit_walking`
//! in `src/map/packets_struct.hpp`).

use super::bytes::{name_at, u16_at, u32_at};

/// A unit that stands (or sits, or lies dead) where it is.
pub const ZC_NOTIFY_STANDENTRY: u16 = 0x09ff;
/// A unit that has just come into being: spawned, logged in, teleported in.
pub const ZC_NOTIFY_NEWENTRY: u16 = 0x09fe;
/// A unit that is walking: one that walks into sight, or one in sight that starts to walk.
pub const ZC_NOTIFY_MOVEENTRY: u16 = 0x09fd;
pub const ZC_NOTIFY_VANISH: u16 = 0x0080;
/// ZC_STATE_CHANGE (0x0229): `u32 id, u16 body state, u16 health state, u32 effect state, u8 pk`. Someone's
/// state changed: the effect state says what they ride, among other things.
pub const ZC_STATE_CHANGE: u16 = 0x0229;

/// Whose state changed, and the bits of the state they are in now.
pub fn state_changed(packet: &[u8]) -> Option<(u32, u32)> {
    (packet.len() >= 14).then(|| (u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]), u32::from_le_bytes([packet[10], packet[11], packet[12], packet[13]])))
}

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
    let data: Vec<u16> = data[..5].iter().map(|&byte| u16::from(byte)).collect();
    Some(Walked {
        from: (data[0] << 2 | data[1] >> 6, (data[1] & 0x3f) << 4 | data[2] >> 4),
        to: ((data[2] & 0x0f) << 6 | data[3] >> 2, (data[3] & 0x03) << 8 | data[4]),
        within: (within >> 4, within & 0x0f),
        started,
    })
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
        let data: Vec<u16> = packet[position..position + 3].iter().map(|&byte| u16::from(byte)).collect();
        ((data[0] << 2 | data[1] >> 6, (data[1] & 0x3f) << 4 | data[2] >> 4), (data[2] & 0x0f) as u8, None)
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
        life: Some((u32_at(packet, after_position + 8) as i32, u32_at(packet, after_position + 4) as i32)).filter(|(left, most)| *left >= 0 && *most > 0),
        name: packet.get(after_position + 15..).map_or(String::new(), |_| name_at(packet, after_position + 15, 24)),
        cell,
        direction,
        walks,
        guild: u32_at(packet, after_headgear + 12),
        emblem: u16_at(packet, after_headgear + 16),
        third: u32_at(packet, after_headgear + 20),
    })
}

/// ZC_NPC_SHOWEFST_UPDATE (0x028a): `u32 id, u32 effect state, u32 level, u32 third option`: the states that
/// show on someone in sight changed (rAthena's `clif_changeoption2`, which it sends for a monster or an NPC).
/// Whose, the bits of the state it is in and those of its third option.
pub const ZC_NPC_SHOWEFST_UPDATE: u16 = 0x028a;

pub fn third_option(packet: &[u8]) -> Option<(u32, u32, u32)> {
    (packet.len() >= 18).then(|| (u32_at(packet, 2), u32_at(packet, 6), u32_at(packet, 14)))
}

/// ZC_CHANGE_DIRECTION (0x009c): `u32 id, u16 head, u8 direction`: someone turns where it stands, or turns
/// its head (0 ahead, 1 to the right, 2 to the left). Whose, the head's turn and the direction (0 north,
/// turning left, as the packets that show a unit count it).
pub const ZC_CHANGE_DIRECTION: u16 = 0x009c;

pub fn turned(packet: &[u8]) -> Option<(u32, u8, u8)> {
    (packet.len() >= 9).then(|| (u32_at(packet, 2), u16_at(packet, 6) as u8, packet[8] % 8))
}

/// ZC_FASTMOVE (0x08d2): `u32 id, u16 x, u16 y`: someone is on that cell at once (rAthena's
/// `clif_fast_movement`), laid out as ZC_HIGHJUMP and read by `slid`.
pub const ZC_FASTMOVE: u16 = 0x08d2;

/// ZC_SPIRITS (0x01d0) and ZC_SPIRITS2 (0x01e1): `u32 id, u16 count`: how many spirit spheres (or coins)
/// someone has about them.
pub const ZC_SPIRITS: u16 = 0x01d0;
pub const ZC_SPIRITS2: u16 = 0x01e1;

pub fn spirits(packet: &[u8]) -> Option<(u32, u16)> {
    (packet.len() >= 8).then(|| (u32_at(packet, 2), u16_at(packet, 6)))
}

/// ZC_RESURRECTION (0x0148): `u32 id, u16 kind`: someone who lay dead stands up again.
pub const ZC_RESURRECTION: u16 = 0x0148;

pub fn risen(packet: &[u8]) -> Option<u32> {
    (packet.len() >= 6).then(|| u32_at(packet, 2))
}

/// ZC_SPRITE_CHANGE (0x01d7): `u32 id, u8 kind, u32 value, u32 second`: what someone wears or holds
/// changes. With the kind 2 the value is the weapon and the second the shield.
pub const ZC_SPRITE_CHANGE: u16 = 0x01d7;

pub fn look(packet: &[u8]) -> Option<(u32, u8, u32, u32)> {
    (packet.len() >= 15).then(|| (u32_at(packet, 2), packet[6], u32_at(packet, 7), u32_at(packet, 11)))
}

/// ZC_NOTIFY_VANISH (0x0080): `u32 id, u8 how`: a unit is gone from sight (0), dead (1), logged out (2) or
/// teleported away (3). Returns the ID and whether it died.
pub fn vanished(packet: &[u8]) -> Option<(u32, bool)> {
    (packet.len() >= 7).then(|| (u32_at(packet, 2), packet[6] == 1))
}

/// ZC_NOTIFY_MOVE (0x0086): `u32 id, u8[6] move, u32 start_time`: someone already in sight walks from one cell to
/// another (packed as the player's own move is).
pub const ZC_NOTIFY_MOVE: u16 = 0x0086;

pub fn moved(packet: &[u8]) -> Option<(u32, Walked)> {
    (packet.len() >= 16).then(|| walked(&packet[6..12], u32_at(packet, 12)).map(|walk| (u32_at(packet, 2), walk)))?
}

/// ZC_HIGHJUMP (0x01ff): `u32 id, u16 x, u16 y`: someone is on that cell at once (a leap, a slide, being
/// thrown back: rAthena's `clif_slide`).
pub const ZC_HIGHJUMP: u16 = 0x01ff;

pub fn slid(packet: &[u8]) -> Option<(u32, (u16, u16))> {
    (packet.len() >= 10).then(|| (u32_at(packet, 2), (u16_at(packet, 6), u16_at(packet, 8))))
}

/// ZC_ACK_REQNAMEALL_NPC (0x0adf): `u32 who, u32 group, char name[24], char title[24]`: what a monster, an NPC
/// or a pet is called, with the title and the group a script gave it.
pub const ZC_ACK_REQNAMEALL_NPC: u16 = 0x0adf;

/// Who is named, in which group, by what name and title.
pub fn named(packet: &[u8]) -> Option<(u32, u32, String, String)> {
    let packet = packet.get(..58)?;
    Some((u32_at(packet, 2), u32_at(packet, 6), name_at(packet, 10, 24), name_at(packet, 34, 24)))
}

/// ZC_NOTIFY_ACT (0x08c8): `u32 source, u32 target, u32 tick, i32 source_speed, i32 damage_speed, i32 damage,
/// u8 sp, u16 hits, u8 kind, i32 damage2`: someone does something to someone.
pub const ZC_NOTIFY_ACT: u16 = 0x08c8;

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
    let number = |at: usize| Some(i32::from_le_bytes(packet.get(at..at + 4)?.try_into().ok()?));
    Some(Act { source: number(2)? as u32, target: number(6)? as u32, speed: number(14)?, delay: number(18)?, damage: number(22)?, kind: *packet.get(29)?, hits: super::bytes::word(packet, 27)?, extra: number(30)? })
}

#[cfg(test)]
mod act_tests {
    use super::*;

    #[test]
    fn a_monsters_name_with_its_title_and_group() {
        let mut packet = vec![0xdf, 0x0a];
        packet.extend(9u32.to_le_bytes());
        packet.extend(3u32.to_le_bytes());
        for words in ["Poring", "Angry"] {
            let mut field = words.as_bytes().to_vec();
            field.resize(24, 0);
            packet.extend(field);
        }
        assert_eq!(named(&packet), Some((9, 3, "Poring".to_owned(), "Angry".to_owned())));
        assert_eq!(named(&packet[..57]), None);
    }

    #[test]
    fn a_blow_of_several_hits_with_the_other_hands_damage() {
        let mut packet = vec![0xc8, 0x08];
        for number in [7u32, 9, 1000, 432, 288, 300] {
            packet.extend(number.to_le_bytes());
        }
        packet.push(0);
        packet.extend(3u16.to_le_bytes());
        packet.push(8);
        packet.extend(40i32.to_le_bytes());
        assert_eq!(act(&packet), Some(Act { source: 7, target: 9, speed: 432, delay: 288, damage: 300, kind: 8, hits: 3, extra: 40 }));
        assert_eq!(act(&packet[..33]), None);
    }
}

/// ZC_HP_INFO (0x0977): `u32 id, i32 hp, i32 most`: how much life a monster has left.
pub const ZC_HP_INFO: u16 = 0x0977;

pub fn life(packet: &[u8]) -> Option<(u32, i32, i32)> {
    let number = |at: usize| Some(i32::from_le_bytes(packet.get(at..at + 4)?.try_into().ok()?));
    Some((number(2)? as u32, number(6)?, number(10)?))
}

/// ZC_ITEM_FALL_ENTRY (0x0add): `u32 id, u32 item, u16 type, u8 identified, i16 x, i16 y, u8 sub_x, u8 sub_y,
/// i16 count, …`: a thing falls to the ground. ZC_ITEM_ENTRY (0x009d): `u32 id, u32 item, u8 identified, u16 x,
/// u16 y, u16 count, …`: one that lay there comes into sight. ZC_ITEM_DISAPPEAR (0x00a1): `u32 id`.
pub const ZC_ITEM_FALL_ENTRY: u16 = 0x0add;
pub const ZC_ITEM_ENTRY: u16 = 0x009d;
pub const ZC_ITEM_DISAPPEAR: u16 = 0x00a1;

/// `(id, item, cell, where within the cell in sixteenths, identified, how many)` of a thing on the ground, from
/// either packet (the count stands after the place within the cell in the one, before it in the other).
pub fn dropped(id: u16, packet: &[u8]) -> Option<(u32, u32, (u16, u16), (u8, u8), bool, u16)> {
    let (at, within, identified, count) = if id == ZC_ITEM_FALL_ENTRY { (13, 17, 12, 19) } else { (11, 17, 10, 15) };
    let (short, long) = (|at: usize| super::bytes::word(packet, at), |at: usize| super::bytes::long(packet, at));
    Some((long(2)?, long(6)?, (short(at)?, short(at + 2)?), (*packet.get(within)?, *packet.get(within + 1)?), *packet.get(identified)? != 0, short(count)?))
}

/// CZ_CANCEL_LOCKON (0x0118): the server stops the blows it was dealing on its own.
pub fn stop_attack() -> Vec<u8> {
    0x0118u16.to_le_bytes().to_vec()
}

/// CZ_ITEM_PICKUP (0x0362): `u32 id`.
pub fn pick_up(id: u32) -> Vec<u8> {
    let mut packet = 0x0362u16.to_le_bytes().to_vec();
    packet.extend(id.to_le_bytes());
    packet
}

/// CZ_REQUEST_ACT2 (0x0437): `u32 target, u8 action`: 7 attacks until told otherwise, 0 once.
pub fn attack(target: u32) -> Vec<u8> {
    request(target, 7)
}

/// CZ_CHANGE_DIRECTION (0x0361 for this client): `u16 head` (how the head is turned: 0 ahead, 1 to the
/// right, 2 to the left), `u8 direction` (0 north, turning left).
pub fn face(direction: u8) -> Vec<u8> {
    turn(0, direction)
}

/// The same request with the head turned.
pub fn turn(head: u8, direction: u8) -> Vec<u8> {
    vec![0x61, 0x03, head.min(2), 0, direction % 8]
}

/// The same request for one blow only.
pub fn attack_once(target: u32) -> Vec<u8> {
    request(target, 0)
}

/// The same request to sit down (2) or to stand up (3): no target.
pub fn sit(down: bool) -> Vec<u8> {
    request(0, if down { 2 } else { 3 })
}

fn request(target: u32, action: u8) -> Vec<u8> {
    let mut packet = 0x0437u16.to_le_bytes().to_vec();
    packet.extend(target.to_le_bytes());
    packet.push(action);
    packet
}

#[cfg(test)]
mod tests;
