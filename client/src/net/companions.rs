//! What the packets of a pet and of a homunculus share: one fed, a number of it that changed, a name given to
//! it. `net/pets.rs` and `net/homunculus.rs` name each by their own packet; the layout is written here once.

use super::bytes::{i32_at, u32_at, Writer};

/// What ZC_CHANGESTATE_PET (its `type`) and ZC_CHANGESTATE_MER (its `state`; the client's `HO_*`) both tell of.
/// The pet's packet tells of more (`net::pets::change`).
pub mod change {
    /// This unit is the player's pet, or the player's homunculus.
    pub const OWN: u8 = 0;
    pub const INTIMACY: u8 = 1;
    pub const HUNGER: u8 = 2;
}

/// ZC_FEED_PET (0x01a3) and ZC_FEED_MER (0x022f), 7 bytes both: `u8 fed`, `u32 item`: whether it ate, and the
/// food it eats.
pub fn fed(packet: &[u8]) -> Option<(bool, u32)> {
    let packet = packet.get(..7)?;
    Some((packet[2] != 0, u32_at(packet, 3)))
}

/// What ZC_CHANGESTATE_PET and ZC_CHANGESTATE_MER end with, from `at`: `u8 what`, `u32 unit`, `i32 value`. (The
/// homunculus's packet has one unused byte before it, so its `at` is one further.)
pub(super) fn changed(packet: &[u8], at: usize) -> Option<(u8, u32, i32)> {
    let packet = packet.get(..at + 9)?;
    Some((packet[at], u32_at(packet, at + 1), i32_at(packet, at + 5)))
}

/// CZ_RENAME_PET (0x01a5) and CZ_RENAME_MER (0x0231), 26 bytes both: `char name[24]`.
pub(super) fn rename(id: u16, name: &str) -> Vec<u8> {
    Writer::new(id).name(name).done()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_both_are_told_alike() {
        assert_eq!(fed(&[0xa3, 0x01, 1, 0x13, 0x02, 0, 0]), Some((true, 531)));
        assert_eq!(fed(&[0x2f, 0x02, 0, 0x19, 0x02, 0, 0]), Some((false, 537)));
        assert_eq!(fed(&[0x2f, 0x02, 0, 0x19, 0x02, 0]), None);
        // The pet's from the third byte, the homunculus's from the fourth; a value under zero stays so.
        assert_eq!(changed(&[0xa4, 0x01, 2, 7, 0, 0, 0, 0xff, 0xff, 0xff, 0xff], 2), Some((change::HUNGER, 7, -1)));
        assert_eq!(changed(&[0x30, 0x02, 0, 1, 0x10, 0x27, 0, 0, 0xb0, 0x01, 0, 0], 3), Some((change::INTIMACY, 10000, 432)));
        assert_eq!(changed(&[0x30, 0x02, 0, 1, 0x10, 0x27, 0, 0, 0xb0, 0x01, 0], 3), None);
    }

    #[test]
    fn names_either_in_24_bytes() {
        let named = rename(0x01a5, "Bob");
        assert_eq!((named.len(), &named[..6], named[25]), (26, &[0xa5, 0x01, b'B', b'o', b'b', 0][..], 0));
        assert_eq!(&rename(0x0231, "Bob")[..2], [0x31, 0x02]);
    }
}
