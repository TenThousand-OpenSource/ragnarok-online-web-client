//! Tests of the packets that tell of fighting other players, each as rAthena writes it, and of who is fought
//! in the zone each makes.

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
