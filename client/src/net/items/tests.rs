//! Tests of the packets of the inventory, the skills known and the hotkey bar.

use super::*;
use crate::net::packet_length;

#[test]
fn requests_have_the_length_the_server_expects() {
    for request in [use_item(2, 2_000_000), wear(3, 2), take_off(3), raise_skill(142), use_skill(1, 142, 2_000_000), set_hotkey(0, Hotkey { skill: true, id: 142, count: 1 })] {
        let id = u16_at(&request, 0);
        assert_eq!(packet_length(id), Some(request.len() as i32), "{id:#06x}");
    }
    // And the fixed-length replies are as long as they are read here.
    for (id, length) in [(ZC_ITEM_PICKUP_ACK, 70), (ZC_USE_ITEM_ACK, 15), (ZC_DELETE_ITEM_FROM_BODY, 8), (ZC_ITEM_THROW_ACK, 6), (ZC_REQ_WEAR_EQUIP_ACK, 11), (ZC_REQ_TAKEOFF_EQUIP_ACK, 9), (ZC_SKILLINFO_UPDATE, 11), (ZC_ADD_SKILL, 39), (ZC_SHORTCUT_KEY_LIST, 5 + 7 * HOTKEYS as i32)] {
        assert_eq!(packet_length(id), Some(length), "{id:#06x}");
    }
}

#[test]
fn a_count_typed_is_a_number_of_what_there_is() {
    assert_eq!((count_typed("1", 30), count_typed(" 30 ", 30)), (Some(1), Some(30)));
    // Nothing, no number, none at all, more than there is, more than a count can be: nothing is done.
    assert_eq!([count_typed("", 30), count_typed("ten", 30), count_typed("0", 30), count_typed("31", 30), count_typed("70000", u16::MAX), count_typed("-1", 30)], [None; 6]);
}

fn stack(index: u16, id: u32, kind: u8, amount: u16) -> Vec<u8> {
    let mut item = vec![0u8; 34];
    item[..2].copy_from_slice(&index.to_le_bytes());
    item[2..6].copy_from_slice(&id.to_le_bytes());
    item[6] = kind;
    item[7..9].copy_from_slice(&amount.to_le_bytes());
    item[33] = 1;
    item
}

#[test]
fn reads_the_stackable_items() {
    let mut packet = vec![0x09, 0x0b, 0, 0, 0];
    packet.extend(stack(2, 501, 0, 30));
    packet.extend(stack(3, 909, 3, 7));
    let items = stackable(&packet);
    assert_eq!(items.len(), 2);
    assert_eq!((items[0].index, items[0].id, items[0].amount, items[0].identified), (2, 501, 30, true));
    assert_eq!((items[0].tab(), items[1].tab()), (Tab::Usable, Tab::Other));
    assert!(items[0].usable() && !items[1].usable());
    // The cart's and the storage's lists are not the character's.
    packet[4] = 1;
    assert!(stackable(&packet).is_empty());
}

#[test]
fn reads_equipment() {
    let mut item = vec![0u8; 68];
    item[..2].copy_from_slice(&4u16.to_le_bytes());
    item[2..6].copy_from_slice(&1201u32.to_le_bytes());
    item[6] = 5;
    item[7..11].copy_from_slice(&2u32.to_le_bytes());
    item[11..15].copy_from_slice(&2u32.to_le_bytes());
    item[15..19].copy_from_slice(&4001u32.to_le_bytes());
    item[65] = 7;
    item[67] = 1;
    let mut packet = vec![0x39, 0x0b, 0, 0, 0];
    packet.extend(&item);
    let items = equipment(&packet);
    assert_eq!(items, [Item { index: 4, id: 1201, kind: 5, amount: 1, wearable: 2, worn: 2, refine: 7, identified: true, cards: [4001, 0, 0, 0] }]);
    assert_eq!(items[0].tab(), Tab::Equipment);
}

#[test]
fn follows_items_coming_and_going() {
    let mut gain = vec![0u8; 70];
    gain[2..4].copy_from_slice(&5u16.to_le_bytes());
    gain[4..6].copy_from_slice(&3u16.to_le_bytes());
    gain[6..10].copy_from_slice(&501u32.to_le_bytes());
    gain[10] = 1;
    let item = gained(&gain).unwrap();
    assert_eq!((item.index, item.id, item.amount, item.identified), (5, 501, 3, true));
    // Result 2 is "too heavy": nothing came.
    gain[33] = 2;
    assert_eq!(gained(&gain), None);

    let mut use_ack = vec![0u8; 15];
    use_ack[2..4].copy_from_slice(&5u16.to_le_bytes());
    use_ack[4..8].copy_from_slice(&501u32.to_le_bytes());
    use_ack[8..12].copy_from_slice(&2000001u32.to_le_bytes());
    use_ack[12..14].copy_from_slice(&2u16.to_le_bytes());
    use_ack[14] = 1;
    assert_eq!(used(&use_ack), Some((5, 501, 2000001, 2)));
    use_ack[14] = 0;
    assert_eq!(used(&use_ack), None);

    assert_eq!(lost(ZC_DELETE_ITEM_FROM_BODY, &[0xfa, 0x07, 0, 0, 5, 0, 1, 0]), Some((5, 1)));
    assert_eq!(lost(ZC_ITEM_THROW_ACK, &[0xaf, 0x00, 5, 0, 2, 0]), Some((5, 2)));
    assert_eq!(put_on(&[0x99, 0x09, 4, 0, 2, 0, 0, 0, 0, 0, 0]), Some((4, 2)));
    assert_eq!(put_on(&[0x99, 0x09, 4, 0, 2, 0, 0, 0, 0, 0, 1]), None);
    assert_eq!(taken_off(&[0x9a, 0x09, 4, 0, 2, 0, 0, 0, 0]), Some(4));
}

#[test]
fn reads_skills() {
    let mut skill = vec![0u8; 37];
    skill[..2].copy_from_slice(&142u16.to_le_bytes());
    skill[2..6].copy_from_slice(&4u32.to_le_bytes());
    skill[6..8].copy_from_slice(&1u16.to_le_bytes());
    skill[8..10].copy_from_slice(&3u16.to_le_bytes());
    skill[10..12].copy_from_slice(&1u16.to_le_bytes());
    skill[36] = 0;
    let mut basic = vec![0u8; 37];
    basic[0] = 1;
    basic[36] = 1;
    let mut packet = vec![0x0f, 0x01, 0, 0];
    packet.extend(&skill);
    packet.extend(&basic);
    let list = skills(&packet);
    assert_eq!(list[0], Skill { id: 142, target: 4, level: 1, sp: 3, range: 1, raisable: false });
    assert!(list[0].on_self() && !list[0].passive());
    assert!(list[1].passive() && list[1].raisable);

    assert_eq!(skill_raised(&[0x0e, 0x01, 1, 0, 2, 0, 0, 0, 1, 0, 1]), Some((1, 2, 0, 1, true)));
    let mut added = vec![0x11, 0x01];
    added.extend(&skill);
    assert_eq!(skill_added(&added).unwrap().id, 142);
}

#[test]
fn reads_and_sets_the_hotkey_bar() {
    let mut packet = vec![0x20, 0x0b, 0, 0, 0];
    packet.extend([1, 142, 0, 0, 0, 1, 0]);
    packet.extend([0, 0xf5, 0x01, 0, 0, 0, 0]);
    packet.resize(5 + 7 * HOTKEYS, 0);
    let bar = hotkeys(&packet);
    assert_eq!(bar.len(), HOTKEYS);
    assert_eq!(bar[0], Hotkey { skill: true, id: 142, count: 1 });
    assert_eq!(bar[1], Hotkey { skill: false, id: 501, count: 0 });
    assert_eq!(bar[2], Hotkey::default());
    assert_eq!(set_hotkey(9, bar[0]), [0x21, 0x0b, 0, 0, 9, 0, 1, 142, 0, 0, 0, 1, 0]);
    // The list says which of the two bars it is of, and a slot of the second is set by naming it.
    assert_eq!(hotkeys_bar(&packet), Some(0));
    packet[3] = 1;
    assert_eq!(hotkeys_bar(&packet), Some(1));
    assert_eq!(set_hotkey_of(1, 9, bar[0]), [0x21, 0x0b, 1, 0, 9, 0, 1, 142, 0, 0, 0, 1, 0]);
}

#[test]
fn only_what_is_used_shot_or_worn_goes_on_the_hotkey_bar() {
    let item = |kind: u8, wearable: u32, identified: bool| Item { index: 2, id: 501, kind, amount: 1, wearable, worn: 0, refine: 0, identified, cards: [0; 4] };
    // A potion and an arrow do; loot does not.
    assert!(item(0, 0, true).fits_hotkeys());
    assert!(item(10, 0, true).fits_hotkeys());
    assert!(!item(3, 0, true).fits_hotkeys());
    // A weapon does if it has a place to be worn, and nothing does until it is identified.
    assert!(item(5, 2, true).fits_hotkeys());
    assert!(!item(5, 0, true).fits_hotkeys());
    assert!(!item(5, 2, false).fits_hotkeys());
}

#[test]
fn reads_what_is_to_identify_and_the_ammunition() {
    assert_eq!(unidentified(&[0x77, 0x01, 8, 0, 4, 0, 11, 0]), vec![4, 11]);
    assert_eq!((identified(&[0x79, 0x01, 4, 0, 0]), identified(&[0x79, 0x01, 4, 0, 1]), identified(&[0x79, 0x01])), ((4, Some(0)), (4, Some(1)), (0, None)));
    assert_eq!((ammunition(&[0x3c, 0x01, 7, 0]), ammunition(&[0x3c, 0x01, 7])), (Some(7), None));
}
