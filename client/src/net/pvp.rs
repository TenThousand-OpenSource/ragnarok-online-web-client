//! Fighting other players: what kind of map the character is on, who on it may be fought, and the standing the
//! server keeps on a PVP map. The layouts are rAthena's for PACKETVER 20260219 (`clif_map_property`,
//! `clif_map_type`, `clif_pvpset` in `clif.cpp`); what each bit means to the client is the client's own
//! (`CGameMode::ReceiveMapProperty`, the `ID_ZC_NOTIFY_MAPPROPERTY` and `ID_ZC_NOTIFY_MAPPROPERTY2` arms of its
//! packet handler, `CGameMode::HandleActorPick`).

use super::bytes::{long, word};

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

/// What holds on the map the character is on. The flags are those of ZC_MAPPROPERTY_R2, each named for what the
/// client does with it (the numbers are the client's own for them, in `CMapPropertyMgr`).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Zone {
    /// Bit 0 (the client's 7): everyone who is not of the player's party is fought.
    pub party_foes: bool,
    /// Bit 1 (8): everyone who is not of the player's guild is fought.
    pub guild_foes: bool,
    /// Bit 2 (9): a siege: guilds fight, and a guild's emblem shows over its members.
    pub siege: bool,
    /// Bit 3 (15): effects are drawn the simple way, as with `/mineffect`.
    pub simple_effects: bool,
    /// Bit 4 (16): the client's `m_isSiegeZone`: a press on another player fights them.
    pub fights_on_press: bool,
    /// Bit 5 (17): the client's `m_isPvpZone`: the standing is counted and shown.
    pub counted: bool,
    /// Bit 6 (18): no party is made or changed here.
    pub no_party: bool,
    /// Bit 7 (19): a battleground: teams fight.
    pub battlefield: bool,
    /// Bit 8 (20): costumes are not shown.
    pub no_costumes: bool,
    /// The older packets' word for it (the client's `isPKZone`): a map, or a server, of free fighting.
    pub pk: bool,
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
    Some((long(packet, 2)?, long(packet, 6)? as i32, long(packet, 10)? as i32))
}

impl Zone {
    /// Whether another player is fought here as a monster is: the pointer over them is the sword, and a press
    /// attacks. `party`, `guild` and `ally` tell whether they are of the player's party, of the player's guild,
    /// of a guild allied to it. The client's rule for a player under the pointer with no skill aimed and Shift
    /// not held (`CGameMode::HandleActorPick`).
    ///
    /// On a battleground the client goes by the two teams (`playerTeam`, `battleTeam`), which this client does
    /// not know: nobody is fought there.
    pub fn fights(&self, party: bool, guild: bool, ally: bool) -> bool {
        if self.battlefield {
            return false;
        }
        if self.siege {
            return !(guild || ally);
        }
        let by_side = if self.party_foes { !party } else { self.guild_foes && !guild };
        by_side || ((self.pk || self.fights_on_press) && !party)
    }

    /// Whether the standing is shown (the client draws it when `isPKZone` or `m_isPvpZone` is set).
    pub fn ranked(&self) -> bool {
        self.pk || self.counted
    }
}

/// The action of `RankFont` that is the stroke between the rank and the total.
pub const STROKE: usize = 10;

/// The "rank / total" line the client draws in the corner of the screen on a PVP map
/// (`CGameMode::DrawMiniMapCoordinates`): each piece is an action of the sprite `이팩트\RankFont` (a digit, or
/// `STROKE`) and where its origin stands, from the right edge and from the bottom of the screen. Nothing while
/// either number is not above zero. The total is written up to 9999, as the client does; the rank too (the
/// client does not bound it, and writes nonsense past that).
pub fn rank_line(rank: i32, total: i32) -> Vec<(usize, i32, i32)> {
    if rank <= 0 || total <= 0 {
        return Vec::new();
    }
    // The digits of a number, the last first.
    let digits = |number: i32| {
        let mut left = number.min(9999);
        let mut digits = vec![(left % 10) as usize];
        while left >= 10 {
            left /= 10;
            digits.push((left % 10) as usize);
        }
        digits
    };
    let mut line = Vec::new();
    let total = digits(total);
    let last = -80 + total.len() as i32 * 25;
    line.extend(total.into_iter().enumerate().map(|(index, digit)| (digit, last - index as i32 * 25, -30)));
    line.push((STROKE, -80, -40));
    line.extend(digits(rank).into_iter().enumerate().map(|(index, digit)| (digit, -105 - index as i32 * 25, -50)));
    line
}

/// What the chat says when the player's rank changed from `before` to `now`, by the client's rule
/// (`CGameMode::ReceiveEffectNumber`): the number of its message (582 for coming into the first ten, 583 for
/// falling out of them), or none.
pub fn rank_message(before: i32, now: i32) -> Option<usize> {
    match (before > 10, now > 10) {
        (true, false) => Some(582),
        (false, true) if before != now => Some(583),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pvp_map_as_rathena_tells_it() {
        // `clif_map_property` on a map with the `pvp` flag: FREEPVPZONE, bits 0 (PARTY), 4 (DISABLE_LOCKON),
        // 5 (COUNT_PK), 9 (USECART) and 10 (SUNMOONSTAR_MIRACLE).
        let packet = [0x9b, 0x09, 0x01, 0x00, 0x31, 0x06, 0x00, 0x00];
        let (kind, zone) = zone(&packet).unwrap();
        assert_eq!(kind, property::FREE_PVP);
        assert_eq!(zone, Zone { party_foes: true, fights_on_press: true, counted: true, ..Zone::default() });
        assert!(zone.ranked());
        // Everyone but the party is fought, a guildmate too.
        assert!(zone.fights(false, false, false));
        assert!(zone.fights(false, true, true));
        assert!(!zone.fights(true, false, false));
    }

    #[test]
    fn an_ordinary_map_has_nobody_to_fight() {
        let packet = [0x9b, 0x09, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00];
        let (kind, zone) = zone(&packet).unwrap();
        assert_eq!((kind, zone), (0, Zone::default()));
        assert!(!zone.ranked());
        assert!(!zone.fights(false, false, false));
    }

    #[test]
    fn a_castle_under_siege_spares_the_guild_and_its_allies() {
        // `mapdata_flag_gvg2`: AGITZONE, bits 1 (GUILD), 2 (SIEGE), 3 (USE_SIMPLE_EFFECT), 4, 9, 10.
        let packet = [0x9b, 0x09, 0x03, 0x00, 0x1e, 0x06, 0x00, 0x00];
        let (kind, zone) = zone(&packet).unwrap();
        assert_eq!(kind, property::AGIT);
        assert!(zone.guild_foes && zone.siege && zone.simple_effects && zone.fights_on_press && !zone.party_foes);
        assert!(zone.fights(false, false, false));
        // One of the party who is of another guild is fought; a guildmate and an ally are not.
        assert!(zone.fights(true, false, false));
        assert!(!zone.fights(false, true, false));
        assert!(!zone.fights(false, false, true));
    }

    #[test]
    fn a_battleground_is_left_alone() {
        let packet = [0x9b, 0x09, 0x00, 0x00, 0x86, 0x00, 0x00, 0x00];
        let (_, zone) = zone(&packet).unwrap();
        assert!(zone.battlefield);
        assert!(!zone.fights(false, false, false));
    }

    #[test]
    fn the_older_packets_make_the_same_zones() {
        assert!(old_zone(&[0x99, 0x01, 0x01, 0x00]).unwrap().1.pk);
        assert!(old_zone(&[0x99, 0x01, 0x03, 0x00]).unwrap().1.siege);
        assert_eq!(old_zone(&[0x99, 0x01, 0x04, 0x00]).unwrap(), (property::PK_SERVER, Zone::default()));
        assert!(old_zone(&[0x99, 0x01, 0x01, 0x00]).unwrap().1.fights(false, true, false));
        let known = Zone { counted: true, ..Zone::default() };
        assert_eq!(typed(&[0xd6, 0x01, 0x06, 0x00], known), Some(Zone { pk: true, counted: true, ..Zone::default() }));
        assert_eq!(typed(&[0xd6, 0x01, 0x13, 0x00], known), Some(Zone { pk: true, battlefield: true, counted: true, ..Zone::default() }));
        assert_eq!(typed(&[0xd6, 0x01, 0x00, 0x00], known), Some(known));
        assert_eq!(typed(&[0xd6, 0x01], known), None);
    }

    #[test]
    fn the_standing_is_read() {
        let mut packet = vec![0x9a, 0x01];
        packet.extend(2_000_001u32.to_le_bytes());
        packet.extend(3u32.to_le_bytes());
        packet.extend(12u32.to_le_bytes());
        assert_eq!(ranking(&packet), Some((2_000_001, 3, 12)));
        // A hidden player's rank comes as all ones.
        packet[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(ranking(&packet), Some((2_000_001, -1, 12)));
        assert_eq!(ranking(&packet[..13]), None);
    }

    #[test]
    fn the_line_is_laid_out_as_the_client_does() {
        // Rank 3 of 12: the total's digits end 30 from the right, the stroke stands at 80, the rank at 105.
        assert_eq!(rank_line(3, 12), vec![(2, -30, -30), (1, -55, -30), (STROKE, -80, -40), (3, -105, -50)]);
        assert_eq!(rank_line(10, 7), vec![(7, -55, -30), (STROKE, -80, -40), (0, -105, -50), (1, -130, -50)]);
        assert!(rank_line(0, 5).is_empty());
        assert!(rank_line(-1, 5).is_empty());
        assert!(rank_line(1, 0).is_empty());
        assert_eq!(rank_line(1, 123_456).len(), 6);
    }

    #[test]
    fn the_chat_tells_of_the_first_ten() {
        assert_eq!(rank_message(11, 10), Some(582));
        assert_eq!(rank_message(10, 11), Some(583));
        // The first standing told is counted from none, as the client counts it: one who starts within the
        // ten is told nothing, one who starts outside them is told of being out.
        assert_eq!(rank_message(0, 4), None);
        assert_eq!(rank_message(0, 14), Some(583));
        assert_eq!(rank_message(4, 4), None);
        assert_eq!(rank_message(12, 14), None);
    }
}
