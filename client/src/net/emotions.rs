//! Emotions: the little pictures shown over someone's head.

/// ZC_EMOTION: someone shows an emotion.
pub const ZC_EMOTION: u16 = 0x00c0;

/// ZC_EMOTION (0x00c0, 7 bytes): `u32 unit`, `u8 emotion`.
pub fn shown(packet: &[u8]) -> Option<(u32, u16)> {
    let packet = packet.get(..7)?;
    Some((u32::from_le_bytes(packet[2..6].try_into().ok()?), u16::from(packet[6])))
}

/// CZ_REQ_EMOTION (0x00bf): `u8 emotion`.
///
/// The client of 2026 asks with CZ_REQ_EMOTION_EXPANSION (0x0be9: a pack and an emotion), which rAthena reads
/// and drops; this older request it answers, to everyone around, with ZC_EMOTION.
pub fn show(emotion: u8) -> Vec<u8> {
    vec![0xbf, 0x00, emotion]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_asks() {
        assert_eq!(shown(&[0xc0, 0x00, 0x40, 0x4b, 0x4c, 0x00, 18]), Some((5_000_000, 18)));
        assert_eq!(shown(&[0xc0, 0x00, 1]), None);
        assert_eq!(show(2), [0xbf, 0x00, 2]);
    }
}
