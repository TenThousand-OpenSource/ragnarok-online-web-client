//! The instances (the client's memorial dungeons): the place in the queue for one, what the server tells of
//! the party's instance, that one ended, and the command that destroys it. The layouts are rAthena's
//! (`clif_instance_create`, `clif_instance_changewait`, `clif_instance_status`, `clif_instance_changestatus`
//! and `clif_parse_MemorialDungeonCommand` in `src/map/clif.cpp`) and the client's
//! (`PACKET_ZC_MEMORIALDUNGEON_*` in `Source/Client/Mode/GameModePackets.cpp`).

use super::bytes::{Writer, long, short, text_at};

/// ZC_MEMORIALDUNGEON_SUBSCRIPTION_INFO (0x02cb, 65 bytes): `char name[61]`, `i16 place in the queue`.
pub const ZC_MEMORIALDUNGEON_SUBSCRIPTION_INFO: u16 = 0x02cb;
/// ZC_MEMORIALDUNGEON_SUBSCRIPTION_NOTIFY (0x02cc, 4 bytes): `i16 place in the queue`; under zero, the place
/// was given up.
pub const ZC_MEMORIALDUNGEON_SUBSCRIPTION_NOTIFY: u16 = 0x02cc;
/// ZC_MEMORIALDUNGEON_INFO (0x02cd, 71 bytes): `char name[61]`, `u32` when it ends at the latest, `u32` when
/// it goes if nobody enters (nothing while someone is in it).
pub const ZC_MEMORIALDUNGEON_INFO: u16 = 0x02cd;
/// ZC_MEMORIALDUNGEON_NOTIFY (0x02ce, 10 bytes): `u32 kind`, `u32` the time to enter by. Kind 0 only changes
/// that time; any other ends the instance (`crate::instances::ended_line`).
pub const ZC_MEMORIALDUNGEON_NOTIFY: u16 = 0x02ce;

/// A ZC_MEMORIALDUNGEON_SUBSCRIPTION_INFO: the instance's name and the place in the queue for it.
pub fn subscribed(packet: &[u8]) -> Option<(String, i16)> {
    Some((text_at(packet, 2, 61)?, short(packet, 63)?))
}

/// A ZC_MEMORIALDUNGEON_SUBSCRIPTION_NOTIFY: the place in the queue now.
pub fn queued(packet: &[u8]) -> Option<i16> {
    short(packet, 2)
}

/// A ZC_MEMORIALDUNGEON_INFO: the name, when it ends at the latest, and when it goes if nobody enters.
pub fn told(packet: &[u8]) -> Option<(String, u32, u32)> {
    Some((text_at(packet, 2, 61)?, long(packet, 63)?, long(packet, 67)?))
}

/// A ZC_MEMORIALDUNGEON_NOTIFY: its kind and the time to enter by.
pub fn noticed(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

/// CZ_MEMORIALDUNGEON_COMMAND (0x02cf, 6 bytes): `u32 command` (`crate::instances::DESTROY` is the one the
/// client's window sends).
pub fn instance_command(command: u32) -> Vec<u8> {
    Writer::new(0x02cf).u32(command).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing;

    #[test]
    fn reads_the_place_in_the_queue() {
        let packet = testing::packet(ZC_MEMORIALDUNGEON_SUBSCRIPTION_INFO, &[&testing::field("Orc's Memory", 61), &3i16.to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_MEMORIALDUNGEON_SUBSCRIPTION_INFO), Some(packet.len() as i32));
        assert_eq!(subscribed(&packet), Some(("Orc's Memory".to_owned(), 3)));
        assert_eq!(subscribed(&packet[..64]), None);
        let packet = testing::packet(ZC_MEMORIALDUNGEON_SUBSCRIPTION_NOTIFY, &[&(-1i16).to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_MEMORIALDUNGEON_SUBSCRIPTION_NOTIFY), Some(packet.len() as i32));
        assert_eq!(queued(&packet), Some(-1));
    }

    #[test]
    fn reads_what_is_told_of_an_instance() {
        let times = testing::longs(&[1_795_810_000, 1_795_803_300]);
        let packet = testing::packet(ZC_MEMORIALDUNGEON_INFO, &[&testing::field("Endless Tower", 61), &times]);
        assert_eq!(crate::net::packet_length(ZC_MEMORIALDUNGEON_INFO), Some(packet.len() as i32));
        assert_eq!(told(&packet), Some(("Endless Tower".to_owned(), 1_795_810_000, 1_795_803_300)));
        assert_eq!(told(&packet[..70]), None);
    }

    #[test]
    fn reads_that_an_instance_changed_or_ended() {
        let packet = testing::packet(ZC_MEMORIALDUNGEON_NOTIFY, &[&testing::longs(&[3, 0])]);
        assert_eq!(crate::net::packet_length(ZC_MEMORIALDUNGEON_NOTIFY), Some(packet.len() as i32));
        assert_eq!(noticed(&packet), Some((3, 0)));
        assert_eq!(noticed(&packet[..9]), None);
    }

    #[test]
    fn the_command_has_the_server_s_length() {
        testing::assert_lengths(&[instance_command(crate::instances::DESTROY)]);
        assert_eq!(instance_command(3), [0xcf, 0x02, 3, 0, 0, 0]);
    }
}
