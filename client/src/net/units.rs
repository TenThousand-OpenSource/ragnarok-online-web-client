//! The packets about everyone else on the map: other players, NPCs and monsters appearing, walking and leaving.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`packet_idle_unit`, `packet_spawn_unit` and `packet_unit_walking`
//! in `src/map/packets_struct.hpp`).

/// A unit that stands (or sits, or lies dead) where it is.
pub const ZC_NOTIFY_STANDENTRY: u16 = 0x09ff;
/// A unit that has just come into being: spawned, logged in, teleported in.
pub const ZC_NOTIFY_NEWENTRY: u16 = 0x09fe;
/// A unit that is walking: one that walks into sight, or one in sight that starts to walk.
pub const ZC_NOTIFY_MOVEENTRY: u16 = 0x09fd;
pub const ZC_NOTIFY_VANISH: u16 = 0x0080;

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
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
    /// Milliseconds per cell walked.
    pub speed: u16,
    pub sex: u8,
    pub hair_style: u16,
    pub hair_colour: u16,
    pub body_palette: u16,
    /// Headgear as view IDs: bottom, top, middle.
    pub headgear: [u16; 3],
    pub level: u16,
    pub name: String,
    /// The cell the unit is on, or walks from.
    pub cell: (u16, u16),
    /// Which way it faces: 0 is south, then clockwise.
    pub direction: u8,
    /// The cell it walks to, when it is walking.
    pub walks_to: Option<(u16, u16)>,
}

fn kind(object: u8) -> Kind {
    match object {
        0 => Kind::Player,
        5 => Kind::Monster,
        7..=10 => Kind::Companion,
        _ => Kind::Npc,
    }
}

fn name_at(packet: &[u8], at: usize) -> String {
    let field = &packet[at..(at + 24).min(packet.len())];
    let end = field.iter().position(|&byte| byte == 0).unwrap_or(field.len());
    match std::str::from_utf8(&field[..end]) {
        Ok(name) => name.to_owned(),
        Err(_) => field[..end].iter().map(|&byte| char::from(byte)).collect(),
    }
}

/// A unit out of the packet that shows it: standing (0x09ff), new (0x09fe) or walking (0x09fd). The three share
/// their start; the walking one carries the time it set off and two cells where the others have one.
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
    let (cell, direction, walks_to) = if walking {
        let data: Vec<u16> = packet[position..position + 6].iter().map(|&byte| u16::from(byte)).collect();
        let from = (data[0] << 2 | data[1] >> 6, (data[1] & 0x3f) << 4 | data[2] >> 4);
        let to = ((data[2] & 0x0f) << 6 | data[3] >> 2, (data[3] & 0x03) << 8 | data[4]);
        (from, 0, Some(to))
    } else {
        let data: Vec<u16> = packet[position..position + 3].iter().map(|&byte| u16::from(byte)).collect();
        ((data[0] << 2 | data[1] >> 6, (data[1] & 0x3f) << 4 | data[2] >> 4), (data[2] & 0x0f) as u8, None)
    };
    Some(Unit {
        id: u32_at(packet, 5),
        kind: kind(packet[4]),
        speed: u16_at(packet, 13),
        job: u16_at(packet, 23),
        hair_style: u16_at(packet, 25),
        headgear: [u16_at(packet, 35), u16_at(packet, after_headgear), u16_at(packet, after_headgear + 2)],
        hair_colour: u16_at(packet, after_headgear + 4),
        body_palette: u16_at(packet, after_headgear + 6),
        sex: packet[position - 1],
        level: u16_at(packet, after_position),
        name: packet.get(after_position + 15..).map_or(String::new(), |_| name_at(packet, after_position + 15)),
        cell,
        direction,
        walks_to,
    })
}

/// ZC_NOTIFY_VANISH (0x0080): `u32 id, u8 how`: a unit is gone from sight (0), dead (1), logged out (2) or
/// teleported away (3). Returns the ID and whether it died.
pub fn vanished(packet: &[u8]) -> Option<(u32, bool)> {
    (packet.len() >= 7).then(|| (u32_at(packet, 2), packet[6] == 1))
}

/// ZC_NOTIFY_MOVE (0x0086): `u32 id, u8[6] move, u32 start_time`: someone already in sight walks from one cell to
/// another (packed as the player's own move is).
pub const ZC_NOTIFY_MOVE: u16 = 0x0086;

pub fn moved(packet: &[u8]) -> Option<(u32, (u16, u16), (u16, u16))> {
    let id = u32::from_le_bytes(packet.get(2..6)?.try_into().ok()?);
    let data: Vec<u16> = packet.get(6..12)?.iter().map(|&byte| u16::from(byte)).collect();
    let from = (data[0] << 2 | data[1] >> 6, (data[1] & 0x3f) << 4 | data[2] >> 4);
    let to = ((data[2] & 0x0f) << 6 | data[3] >> 2, (data[3] & 0x03) << 8 | data[4]);
    Some((id, from, to))
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
}

pub fn act(packet: &[u8]) -> Option<Act> {
    let number = |at: usize| Some(i32::from_le_bytes(packet.get(at..at + 4)?.try_into().ok()?));
    Some(Act { source: number(2)? as u32, target: number(6)? as u32, speed: number(14)?, delay: number(18)?, damage: number(22)?, kind: *packet.get(29)? })
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

/// `(id, item, cell, where within the cell in sixteenths)` of a thing on the ground, from either packet (the
/// count stands before the place within the cell in the one, after it in the other).
pub fn dropped(id: u16, packet: &[u8]) -> Option<(u32, u32, (u16, u16), (u8, u8))> {
    let (at, within) = if id == ZC_ITEM_FALL_ENTRY { (13, 17) } else { (11, 17) };
    let short = |at: usize| Some(u16::from_le_bytes(packet.get(at..at + 2)?.try_into().ok()?));
    let long = |at: usize| Some(u32::from_le_bytes(packet.get(at..at + 4)?.try_into().ok()?));
    Some((long(2)?, long(6)?, (short(at)?, short(at + 2)?), (*packet.get(within)?, *packet.get(within + 1)?)))
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
mod tests {
    use super::*;

    /// A packet of `length` bytes with the fields the three kinds share at their fixed places.
    fn base(id: u16, length: usize, object: u8) -> Vec<u8> {
        let mut packet = vec![0u8; length];
        packet[..2].copy_from_slice(&id.to_le_bytes());
        packet[2..4].copy_from_slice(&(length as u16).to_le_bytes());
        packet[4] = object;
        packet[5..9].copy_from_slice(&110_000_001u32.to_le_bytes());
        packet[13..15].copy_from_slice(&200u16.to_le_bytes());
        packet[23..25].copy_from_slice(&1002u16.to_le_bytes());
        packet[25..27].copy_from_slice(&7u16.to_le_bytes());
        packet[35..37].copy_from_slice(&5u16.to_le_bytes());
        packet
    }

    fn pack(x: u16, y: u16, direction: u8) -> [u8; 3] {
        [(x >> 2) as u8, ((x << 6) | ((y >> 4) & 0x3f)) as u8, ((y << 4) as u8) | direction]
    }

    #[test]
    fn reads_a_standing_unit() {
        let mut packet = base(ZC_NOTIFY_STANDENTRY, 108, 5);
        packet[37..39].copy_from_slice(&224u16.to_le_bytes());
        packet[39..41].copy_from_slice(&12u16.to_le_bytes());
        packet[41..43].copy_from_slice(&3u16.to_le_bytes());
        packet[43..45].copy_from_slice(&2u16.to_le_bytes());
        packet[62] = 1;
        packet[63..66].copy_from_slice(&pack(150, 180, 4));
        packet[69..71].copy_from_slice(&99u16.to_le_bytes());
        packet[84..90].copy_from_slice(b"Poring");

        let unit = unit(ZC_NOTIFY_STANDENTRY, &packet).unwrap();
        assert_eq!((unit.id, unit.kind, unit.job, unit.speed), (110_000_001, Kind::Monster, 1002, 200));
        assert_eq!((unit.hair_style, unit.hair_colour, unit.body_palette, unit.headgear, unit.sex), (7, 3, 2, [5, 224, 12], 1));
        assert_eq!((unit.cell, unit.direction, unit.walks_to), ((150, 180), 4, None));
        assert_eq!((unit.level, unit.name.as_str()), (99, "Poring"));
    }

    /// A new unit is laid out as a standing one, less the byte that says whether it sits or lies.
    #[test]
    fn reads_a_new_unit() {
        let mut packet = base(ZC_NOTIFY_NEWENTRY, 107, 6);
        packet[63..66].copy_from_slice(&pack(10, 20, 0));
        packet[68..70].copy_from_slice(&1u16.to_le_bytes());
        packet[83..89].copy_from_slice(b"Kafra\0");
        let unit = unit(ZC_NOTIFY_NEWENTRY, &packet).unwrap();
        assert_eq!((unit.kind, unit.cell, unit.level, unit.name.as_str()), (Kind::Npc, (10, 20), 1, "Kafra"));
    }

    #[test]
    fn reads_a_walking_unit() {
        let mut packet = base(ZC_NOTIFY_MOVEENTRY, 114, 0);
        packet[41..43].copy_from_slice(&224u16.to_le_bytes());
        packet[66] = 0;
        // From (150, 180) to (155, 1000), as the server packs two cells into six bytes.
        let ((x0, y0), (x1, y1)) = ((150u16, 180u16), (155u16, 1000u16));
        packet[67] = (x0 >> 2) as u8;
        packet[68] = ((x0 << 6) | ((y0 >> 4) & 0x3f)) as u8;
        packet[69] = ((y0 << 4) | ((x1 >> 6) & 0x0f)) as u8;
        packet[70] = ((x1 << 2) | ((y1 >> 8) & 0x03)) as u8;
        packet[71] = y1 as u8;
        packet[72] = 0x88;
        packet[90..96].copy_from_slice(b"Walker");
        let unit = unit(ZC_NOTIFY_MOVEENTRY, &packet).unwrap();
        assert_eq!((unit.kind, unit.cell, unit.walks_to), (Kind::Player, (150, 180), Some((155, 1000))));
        assert_eq!((unit.headgear, unit.name.as_str()), ([5, 224, 0], "Walker"));
    }

    #[test]
    fn refuses_what_is_not_a_unit() {
        assert_eq!(unit(0x1234, &[0u8; 120]), None);
        assert_eq!(unit(ZC_NOTIFY_STANDENTRY, &[0u8; 40]), None);
        assert_eq!(vanished(&[0x80, 0, 1, 0, 0, 0, 1]), Some((1, true)));
        assert_eq!(vanished(&[0x80, 0, 1, 0, 0, 0, 0]), Some((1, false)));
    }
}
