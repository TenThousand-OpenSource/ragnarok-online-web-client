//! The reputation: the points the character has with each of the game's peoples, as the map server tells
//! them, and the window opened by a script. The layouts are rAthena's (`clif_reputation_list`,
//! `clif_reputation_type`, `clif_reputation_open` in `src/map/clif.cpp`); the client's readers are
//! `CGameMode::ReceiveReputeInfo` and `ReceiveReputeOpen`. The client sends nothing of it.

use super::bytes::{body, i64_at, u64_at};

/// ZC_REPUTE_INFO (0x0b8d, its length in it): `u8 success`, then `u64 kind, i64 points` for each: all of them
/// once on entering (none where the server's table of reputations is empty), and one as it changes.
pub const ZC_REPUTE_INFO: u16 = 0x0b8d;
/// ZC_REPUTE_OPEN (0x0b8e, 18 bytes): `u64 group, u32 kind, u32 mode`: a script opens the window on a group,
/// with one reputation marked (rAthena's `open_reputation_ui`).
pub const ZC_REPUTE_OPEN: u16 = 0x0b8e;

/// Reads ZC_REPUTE_INFO: whether the points are there, and each kind with its points.
pub fn info(packet: &[u8]) -> Option<(bool, Vec<(u64, i64)>)> {
    let success = *packet.get(4)? != 0;
    let list = body(packet, 5)?.chunks_exact(16).map(|entry| (u64_at(entry, 0), i64_at(entry, 8))).collect();
    Some((success, list))
}

/// Reads ZC_REPUTE_OPEN: the group (0 for all of them) and the reputation marked. The client reads the two
/// as `u64`s (`PACKET_ZC_UI_OPEN_REPUTE`: the second is rAthena's kind with its mode above it).
pub fn opened(packet: &[u8]) -> Option<(u64, u64)> {
    let packet = packet.get(..18)?;
    Some((u64_at(packet, 2), u64_at(packet, 10)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::{packet, sized};

    #[test]
    fn reads_the_points() {
        let list = sized(ZC_REPUTE_INFO, &[&[1u8][..], &3u64.to_le_bytes(), &(-2500i64).to_le_bytes(), &4u64.to_le_bytes(), &1200i64.to_le_bytes()].concat());
        assert_eq!(info(&list), Some((true, vec![(3, -2500), (4, 1200)])));
        // What the classic server sends: its table is empty.
        assert_eq!(info(&sized(ZC_REPUTE_INFO, &[1])), Some((true, Vec::new())));
        assert_eq!(info(&[0x8d, 0x0b, 4, 0]), None);
    }

    #[test]
    fn reads_the_window_opened() {
        let open = packet(ZC_REPUTE_OPEN, &[&2u64.to_le_bytes(), &3u32.to_le_bytes(), &0u32.to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_REPUTE_OPEN), Some(open.len() as i32));
        assert_eq!(opened(&open), Some((2, 3)));
    }
}
