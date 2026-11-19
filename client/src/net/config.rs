//! The switches the server keeps for a character (ZC_CONFIG, CZ_CONFIG): which one, on or off. The layouts are
//! rAthena's for PACKETVER 20260219 (`clif_parse_configuration`, `clif_configuration` in `clif.cpp`). Which
//! number is which switch is told by the system that has it (`viewequip::OPEN_EQUIPMENT`, the homunculus fed
//! by itself in `net/homunculus.rs`).

use super::bytes::{u32_at, Writer};

/// ZC_CONFIG (0x02d9, 10 bytes): a setting of the player's was changed.
pub const ZC_CONFIG: u16 = 0x02d9;

/// ZC_CONFIG (0x02d9, 10 bytes): `u32 setting, u32 on`.
pub fn config(packet: &[u8]) -> Option<(u32, bool)> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), u32_at(packet, 6) != 0))
}

/// CZ_CONFIG (0x02d8, 10 bytes): `u32 setting, u32 on` (the client's `MM_OPEN_EQUIPEDITEM_CHANGE` sends the
/// setting 0).
pub fn set(setting: u32, on: bool) -> Vec<u8> {
    Writer::new(0x02d8).u32(setting).u32(u32::from(on)).done()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_switch_is_set_and_told() {
        crate::net::bytes::testing::assert_lengths(&[set(6, true)]);
        assert_eq!(set(6, true), [0xd8, 0x02, 6, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(crate::net::packet_length(ZC_CONFIG), Some(10));
        assert_eq!(config(&[0xd9, 0x02, 6, 0, 0, 0, 0, 1, 0, 0]), Some((6, true)));
        assert_eq!(config(&[0xd9, 0x02, 6, 0, 0, 0, 0, 0, 0]), None);
    }
}
