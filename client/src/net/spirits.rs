//! The spirit spheres of a monk and the coins of a gunslinger, as the server tells how many someone has.

use super::bytes::{u16_at, u32_at};

/// ZC_SPIRITS (0x01d0) and ZC_SPIRITS2 (0x01e1): `u32 id, u16 count`: how many spirit spheres (or coins)
/// someone has about them.
pub const ZC_SPIRITS: u16 = 0x01d0;
pub const ZC_SPIRITS2: u16 = 0x01e1;

/// Reads either: who, and how many they have.
pub fn told(packet: &[u8]) -> Option<(u32, u16)> {
    (packet.len() >= 8).then(|| (u32_at(packet, 2), u16_at(packet, 6)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_how_many_spheres_someone_has() {
        assert_eq!(told(&[0xd0, 0x01, 7, 0, 0, 0, 5, 0]), Some((7, 5)));
        assert_eq!(told(&[0xe1, 0x01, 7, 0, 0, 0, 5]), None);
    }
}
