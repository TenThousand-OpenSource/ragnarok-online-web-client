//! The navigation as the map server starts one (rAthena's `clif_navigateTo`, the script command `navigateto`):
//! the one packet of it. Everything else of the navigation is the client's own (`crate::navigation`).

use super::bytes::{name_at, short};

/// ZC_NAVIGATION_ACTIVE (0x08e2, 27 bytes): `u8 kind, u8 services, u8 hide, char[16] map, u16 x, u16 y, u16
/// monster`: the client is to find the way there (`CGameMode::ReceiveNavigation`).
pub const ZC_NAVIGATION_ACTIVE: u16 = 0x08e2;

/// What ZC_NAVIGATION_ACTIVE asks for.
#[derive(Debug, PartialEq)]
pub struct Asked {
    /// What is sought: 0 a cell of the map, 1 the map, 3 its monster of that number (the client takes 2 too,
    /// the NPC on the cell).
    pub kind: u8,
    /// Which ways may be taken, as the server writes it: 100 for those that cost zeny, 10 for scrolls, 1 for
    /// the airship, added (`crate::navigation::texts::mode_of` reads it).
    pub services: u8,
    /// The navigation's window is not to open.
    pub hide: bool,
    pub map: String,
    pub cell: (i16, i16),
    pub monster: i16,
}

/// ZC_NAVIGATION_ACTIVE, read.
pub fn asked(packet: &[u8]) -> Option<Asked> {
    let packet = packet.get(..27)?;
    Some(Asked {
        kind: packet[2],
        services: packet[3],
        hide: packet[4] != 0,
        map: name_at(packet, 5, 16),
        cell: (short(packet, 21)?, short(packet, 23)?),
        monster: short(packet, 25)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_server_asks_for_a_navigation() {
        let mut packet = vec![0xe2, 0x08, 0, 101, 1];
        let mut map = [0u8; 16];
        map[..8].copy_from_slice(b"prontera");
        packet.extend(map);
        packet.extend(150u16.to_le_bytes());
        packet.extend(180u16.to_le_bytes());
        packet.extend(1002u16.to_le_bytes());
        assert_eq!(packet.len(), 27);
        assert_eq!(crate::net::packet_length(ZC_NAVIGATION_ACTIVE), Some(27));
        assert_eq!(asked(&packet), Some(Asked { kind: 0, services: 101, hide: true, map: "prontera".to_owned(), cell: (150, 180), monster: 1002 }));
        assert_eq!(asked(&packet[..26]), None);
    }
}
