//! Putting things on and taking them off, as the server answers and as the Equipment window asks: why a thing
//! could not be worn or taken off, and what the window's "off" button sends (rAthena, `clif_equipitemack`,
//! `clif_unequipitemack`, `clif_parse_RemoveOption`, at PACKETVER 20260219). What the server says of an attack
//! that cannot be made (ZC_ACTION_FAILURE) is read in `net::notices`.

/// Why the server would not have a thing put on, from ZC_REQ_WEAR_EQUIP_ACK (0x0999: its last byte): `None`
/// when it was put on. The client's `CGameMode::ReceiveWearEquipAck` says its line 1953 for 1
/// (`ITEM_EQUIP_ACK_FAILLEVEL`) and 372 for 2 (`ITEM_EQUIP_ACK_FAIL`), and nothing for any other.
pub fn wear_refused(packet: &[u8]) -> Option<u8> {
    packet.get(10).copied().filter(|result| *result != 0)
}

/// Why a thing was not taken off, from ZC_REQ_TAKEOFF_EQUIP_ACK (0x099a: its last byte); `None` when it was.
/// The client (`ReceiveTakeOffEquipResult`) says its line 1953 for 1.
pub fn take_off_refused(packet: &[u8]) -> Option<u8> {
    packet.get(8).copied().filter(|result| *result != 0)
}

/// The line of the client's message table for a refusal to put a thing on, or to take one off.
pub fn refusal_line(result: u8, putting_on: bool) -> Option<usize> {
    match (result, putting_on) {
        (1, _) => Some(1953),
        (2, true) => Some(372),
        _ => None,
    }
}

/// CZ_UNINSTALLATION (0x0b35): `u8 what`: the Equipment window's "off" button (`MM_REQ_CARTOFF`). What it
/// names is `crate::mounts::off`'s number, or `CART_OFF` for the cart.
pub fn uninstall(what: u8) -> Vec<u8> {
    vec![0x35, 0x0b, what]
}

/// What the "off" button names when it lets go of the cart (`UIEquipWnd::UpdateCharacterView`).
pub const CART_OFF: u8 = 6;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::packet_length;

    #[test]
    fn a_refusal_is_told_by_the_servers_code() {
        let mut worn = vec![0x99, 0x09, 3, 0, 2, 0, 0, 0, 0, 0, 0];
        assert_eq!(wear_refused(&worn), None);
        worn[10] = 2;
        assert_eq!(wear_refused(&worn).and_then(|result| refusal_line(result, true)), Some(372));
        worn[10] = 1;
        assert_eq!(wear_refused(&worn).and_then(|result| refusal_line(result, true)), Some(1953));
        assert_eq!(take_off_refused(&[0x9a, 0x09, 3, 0, 2, 0, 0, 0, 1]).and_then(|result| refusal_line(result, false)), Some(1953));
        assert_eq!(take_off_refused(&[0x9a, 0x09, 3, 0, 2, 0, 0, 0, 0]), None);
        assert_eq!(refusal_line(2, false), None);
    }

    #[test]
    fn the_off_button_sends_what_the_server_expects() {
        assert_eq!(uninstall(CART_OFF), [0x35, 0x0b, 6]);
        assert_eq!(packet_length(0x0b35), Some(3));
    }
}
