//! Tests of the packets of the lists a skill or a card opens.

use super::*;
use crate::choosing::{made_effect, set_card};
use crate::net::actors::ask_name;
use crate::net::bytes::testing::sized as listing;

#[test]
fn the_places_of_a_warp_are_read_and_one_is_answered() {
    let mut body = 27u16.to_le_bytes().to_vec();
    for place in ["prontera.gat", "", "Random"] {
        let mut field = [0u8; 16];
        field[..place.len()].copy_from_slice(place.as_bytes());
        body.extend_from_slice(&field);
    }
    assert_eq!(warps(&listing(ZC_WARPLIST, &body)), Some((27, vec!["prontera.gat".to_owned(), "Random".to_owned()])));
    let answer = warp(27, "prontera.gat");
    assert_eq!((answer.len(), &answer[..4], &answer[4..17]), (20, &[0x1b, 0x01, 27, 0][..], &b"prontera.gat\0"[..]));
    assert_eq!(&warp(26, CANCEL)[4..11], b"cancel\0");
    assert_eq!(remembered(&[0x1e, 0x01, 2]), Some(2));
}

#[test]
fn lists_of_items_and_spells_are_four_bytes_a_row() {
    let rows: Vec<u8> = [909u32, 0, 1010].iter().flat_map(|id| id.to_le_bytes()).collect();
    assert_eq!(arrow_sources(&listing(ZC_MAKINGARROW_LIST, &rows)), vec![909, 0, 1010]);
    assert_eq!(spells(&listing(ZC_AUTOSPELLLIST, &rows)), vec![909, 1010]);
    assert_eq!(cookable(&listing(ZC_MAKINGITEM_LIST, &[&1u16.to_le_bytes()[..], &rows].concat())), Some((1, vec![909, 0, 1010])));
    assert_eq!(make_arrows(909), [0xae, 0x01, 0x8d, 0x03, 0, 0]);
    assert_eq!(pick_spell(14), [0xce, 0x01, 14, 0, 0, 0]);
    assert_eq!(cook(1, 12041), [0x5b, 0x02, 1, 0, 0x09, 0x2f, 0, 0]);
}

#[test]
fn what_can_be_made_is_sixteen_bytes_a_row() {
    let mut rows = Vec::new();
    for id in [1101u32, 998] {
        rows.extend_from_slice(&id.to_le_bytes());
        rows.extend_from_slice(&[0; 12]);
    }
    assert_eq!(makable(&listing(ZC_MAKABLEITEMLIST, &rows)), vec![1101, 998]);
    let packet = make(1101, [994, 1000, 0]);
    assert_eq!((packet.len(), u32_at(&packet, 2), u32_at(&packet, 6), u32_at(&packet, 10), u32_at(&packet, 14)), (18, 1101, 994, 1000, 0));
    assert_eq!(made(&[0x8f, 0x01, 2, 0, 0xf5, 0x01, 0, 0]), Some((2, 501)));
}

#[test]
fn a_thing_to_repair_goes_back_as_it_was_listed() {
    let mut row = vec![7, 0];
    row.extend_from_slice(&1201u32.to_le_bytes());
    for card in [4001u32, 0, 0, 0] {
        row.extend_from_slice(&card.to_le_bytes());
    }
    row.extend_from_slice(&[5, 0]);
    let listed = repairable(&listing(ZC_REPAIRITEMLIST, &row));
    assert_eq!(listed, vec![Listed { index: 7, id: 1201, refine: 5, cards: [4001, 0, 0, 0] }]);
    let answer = repair(listed.first());
    assert_eq!((answer.len(), &answer[..2], &answer[2..26]), (26, &[0x66, 0x0b][..], &row[..]));
    assert_eq!(&repair(None)[2..4], &[0xff, 0xff]);
    assert_eq!((repaired(&[0xfe, 0x01, 7, 0, 0]), repaired(&[0xfe, 0x01, 7, 0, 1])), (Some(true), Some(false)));
}

#[test]
fn a_weapon_to_refine_is_named_by_its_place() {
    let mut row = vec![9, 0];
    row.extend_from_slice(&1101u32.to_le_bytes());
    row.push(3);
    row.extend_from_slice(&[0; 16]);
    assert_eq!(refinable(&listing(ZC_NOTIFY_WEAPONITEMLIST, &row)), vec![Listed { index: 9, id: 1101, refine: 3, cards: [0; 4] }]);
    assert_eq!(refine(Some(9)), [0x22, 0x02, 9, 0, 0, 0]);
    assert_eq!(refine(None), [0x22, 0x02, 0xff, 0xff, 0xff, 0xff]);
    assert_eq!(refined(&[0x23, 0x02, 1, 0, 0, 0, 0x4d, 0x04, 0, 0]), Some((1, 1101)));
    assert_eq!(item_refined(&[0x88, 0x01, 2, 0, 9, 0, 6, 0]), Some((2, 9, 6)));
    assert_eq!((made_effect(0), made_effect(3), made_effect(6)), (Some(154), Some(306), None));
}

#[test]
fn a_card_is_asked_about_set_and_answered() {
    assert_eq!(ask_compound(12), [0x7a, 0x01, 12, 0]);
    assert_eq!(compoundable(&listing(ZC_ITEMCOMPOSITION_LIST, &[3, 0, 8, 0])), vec![3, 8]);
    assert_eq!(compound(12, 8), [0x7c, 0x01, 12, 0, 8, 0]);
    assert_eq!(compounded(&[0x7d, 0x01, 8, 0, 12, 0, 0]), Some((8, 12, true)));
    assert_eq!(compounded(&[0x7d, 0x01, 8, 0, 12, 0, 1]), Some((8, 12, false)));
    let mut cards = [4001, 0, 0, 0];
    assert!(set_card(&mut cards, 4002));
    assert_eq!(cards, [4001, 4002, 0, 0]);
    assert!(!set_card(&mut [1, 2, 3, 4], 5));
}

#[test]
fn a_makers_name_is_asked_by_number() {
    assert_eq!(ask_name(150_001), [0x69, 0x03, 0xf1, 0x49, 0x02, 0]);
}
