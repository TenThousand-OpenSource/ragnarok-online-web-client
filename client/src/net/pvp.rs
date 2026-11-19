//! Fighting other players: what kind of map the character is on, who on it may be fought, and the standing the
//! server keeps on a PVP map. The layouts are rAthena's for PACKETVER 20260219 (`clif_map_property`,
//! `clif_map_type`, `clif_pvpset` in `clif.cpp`); what each bit means to the client is the client's own
//! (`CGameMode::ReceiveMapProperty`, the `ID_ZC_NOTIFY_MAPPROPERTY` and `ID_ZC_NOTIFY_MAPPROPERTY2` arms of its
//! packet handler, `CGameMode::HandleActorPick`).

use super::bytes::{int, long, word};

/// ZC_NOTIFY_MAPPROPERTY (0x0199, 4 bytes): `u16 property`: the kind of the map, as servers older than
/// 2012-10-10 say it (`property`). rAthena does not send it to this client; it is read all the same.
pub const ZC_NOTIFY_MAPPROPERTY: u16 = 0x0199;
/// ZC_NOTIFY_MAPPROPERTY2 (0x01d6, 4 bytes): `u16 type`: the type of the map (`kind`). rAthena sends it on
/// entering a battleground only.
pub const ZC_NOTIFY_MAPPROPERTY2: u16 = 0x01d6;
/// ZC_MAPPROPERTY_R2 (0x099b, 8 bytes): `u16 property, u32 flags`: the kind of the map and what holds on it,
/// sent each time the character comes onto a map.
pub const ZC_MAPPROPERTY_R2: u16 = 0x099b;
/// ZC_NOTIFY_RANKING (0x019a, 14 bytes): `u32 account, u32 rank, u32 total`: someone's standing on a PVP map and
/// how many are on it.
pub const ZC_NOTIFY_RANKING: u16 = 0x019a;

/// The kinds ZC_NOTIFY_MAPPROPERTY and ZC_MAPPROPERTY_R2 name (the client's `MAPPROPERTY_*`).
pub mod property {
    pub const FREE_PVP: u16 = 1;
    pub const EVENT_PVP: u16 = 2;
    pub const AGIT: u16 = 3;
    /// A server where players fight everywhere: the client warns of it in the chat.
    pub const PK_SERVER: u16 = 4;
    pub const PVP_SERVER: u16 = 5;
}

/// The types ZC_NOTIFY_MAPPROPERTY2 names that the client acts on (its `MAPTYPE_*`).
pub mod kind {
    pub const PENALTY_FREE_PK: u16 = 5;
    pub const NO_PENALTY_FREE_PK: u16 = 6;
    pub const AGIT: u16 = 8;
    pub const PK_SERVER: u16 = 11;
    pub const PVP_SERVER: u16 = 12;
    pub const AGIT_SIEGE: u16 = 18;
    pub const BATTLEFIELD: u16 = 19;
}

/// ZC_MAPPROPERTY_R2: the kind of the map (`property`) and what holds on it.
pub fn zone(packet: &[u8]) -> Option<(u16, Zone)> {
    let (kind, flags) = (word(packet, 2)?, long(packet, 4)?);
    let bit = |bit: u32| flags >> bit & 1 != 0;
    let zone = Zone {
        party_foes: bit(0),
        guild_foes: bit(1),
        siege: bit(2),
        simple_effects: bit(3),
        fights_on_press: bit(4),
        counted: bit(5),
        no_party: bit(6),
        battlefield: bit(7),
        no_costumes: bit(8),
        pk: false,
    };
    Some((kind, zone))
}

/// ZC_NOTIFY_MAPPROPERTY: the kind of the map alone (`property`), and the zone the client makes of it: a map of
/// free fighting for the PVP kinds, a siege for a guild's castle.
pub fn old_zone(packet: &[u8]) -> Option<(u16, Zone)> {
    let kind = word(packet, 2)?;
    let zone = Zone { pk: matches!(kind, property::FREE_PVP | property::PVP_SERVER | property::EVENT_PVP), siege: kind == property::AGIT, ..Zone::default() };
    Some((kind, zone))
}

/// ZC_NOTIFY_MAPPROPERTY2: the type of the map (`kind`), laid over the zone known so far as the client does:
/// free fighting for types 5 and 6, a siege for 8 and 18, a battleground (which is free fighting too) for 19.
pub fn typed(packet: &[u8], known: Zone) -> Option<Zone> {
    Some(match word(packet, 2)? {
        kind::PENALTY_FREE_PK | kind::NO_PENALTY_FREE_PK => Zone { pk: true, ..known },
        kind::PK_SERVER | kind::PVP_SERVER => known,
        kind::AGIT | kind::AGIT_SIEGE => Zone { siege: true, ..known },
        kind::BATTLEFIELD => Zone { battlefield: true, pk: true, ..known },
        _ => known,
    })
}

/// ZC_NOTIFY_RANKING: whose standing, the rank, and how many are on the map. A rank of -1 is one the server
/// hides (someone hidden or cloaked).
pub fn ranking(packet: &[u8]) -> Option<(u32, i32, i32)> {
    Some((long(packet, 2)?, int(packet, 6)?, int(packet, 10)?))
}

/// The rules that are no packet, kept at the path they had: the zone and who is fought in it, the standing's
/// line and what the chat says of it.
pub use crate::pvp::{STROKE, Zone, rank_line, rank_message};

#[cfg(test)]
mod tests;
