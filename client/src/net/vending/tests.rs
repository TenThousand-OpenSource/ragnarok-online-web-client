use super::*;
use crate::net::bytes::testing::assert_lengths;
use crate::net::packet_length;

/// A thing of a list from its eighth byte on: Jellopy (909), identified, with a card in its first slot.
fn record(head: &[u8], length: usize) -> Vec<u8> {
    let mut record = head.to_vec();
    record.push(3);
    record.extend(909u32.to_le_bytes());
    record.extend([1, 0]);
    record.extend(4001u32.to_le_bytes());
    record.resize(length, 0);
    record
}

#[test]
fn reads_ones_own_shop() {
    let mut packet = vec![0x40, 0x0b, 66, 0, 1, 0, 0, 0];
    let mut thing = record(&[0x10, 0x27, 0, 0, 5, 0, 20, 0], 58);
    thing[56] = 7;
    packet.extend(thing);
    let sold = mine(&packet).unwrap();
    assert_eq!((sold[0].price, sold[0].item.index, sold[0].item.amount, sold[0].item.id, sold[0].item.refine), (10_000, 5, 20, 909, 7));
    assert_eq!((sold[0].item.identified, sold[0].item.cards[0]), (true, 4001));
}

#[test]
fn reads_a_vendors_list() {
    let mut packet = vec![0x3d, 0x0b, 76, 0, 9, 0, 0, 0, 0x2a, 0, 0, 0];
    let mut thing = record(&[0xe8, 3, 0, 0, 12, 0, 6, 0], 64);
    thing[56] = 2;
    thing[62] = 4;
    packet.extend(thing);
    let list = listed(&packet).unwrap();
    assert_eq!((list.vendor, list.shop, list.sold.len()), (9, 42, 1));
    let sold = &list.sold[0];
    assert_eq!((sold.price, sold.item.amount, sold.item.index, sold.item.id, sold.item.wearable, sold.item.refine), (1000, 12, 6, 909, 2, 4));
}

#[test]
fn reads_a_sign_a_sale_and_the_answers() {
    let mut packet = vec![0x31, 0x01, 7, 0, 0, 0];
    packet.extend(b"Cheap potions");
    packet.resize(86, 0);
    assert_eq!(sign(&packet), Some((7, "Cheap potions".to_owned())));
    assert_eq!(sign_gone(&[0x32, 0x01, 7, 0, 0, 0]), Some(7));
    let mut sold = vec![0xe5, 0x09, 5, 0, 3, 0];
    sold.resize(18, 0);
    assert_eq!(sale(&sold), Some((5, 3)));
    assert_eq!(
        [opened(&[0x28, 0x0a, 0]), opened(&[0x28, 0x0a, 1]), opened(&[0x28, 0x0a, 3]), opened(&[0x28, 0x0a, 2])],
        [Some(Opened::Open), Some(Opened::Failed), Some(Opened::NotHere), None]
    );
    assert_eq!(slots(&[0x2d, 0x01, 12, 0]), Some(12));
    assert_eq!(slots(&[0x2d, 0x01, 1, 0]), Some(3));
    assert_eq!(purchase_result(&[0x35, 0x01, 5, 0, 3, 0, 4]), Some((5, 3, 4)));
}

#[test]
fn the_requests_are_laid_out_as_the_server_reads_them() {
    let opening = open("Cheap", true, &[(5, 20, 10_000)]);
    assert_eq!(opening.len(), 93);
    assert_eq!((&opening[..4], &opening[4..10], opening[84]), (&[0xb2, 0x01, 93, 0][..], &b"Cheap\0"[..], 1));
    assert_eq!(opening[85..], [5, 0, 20, 0, 0x10, 0x27, 0, 0]);
    // Not opening, and nothing to sell, say nought.
    assert_eq!((open("", false, &[(5, 20, 1)])[84], open("x", true, &[])[84]), (0, 0));
    assert_eq!(open(&"я".repeat(60), true, &[(1, 1, 1)])[80..84], [0xd1, 0x8f, 0, 0]);
    assert_eq!(buy(9, 42, &[(6, 2)]), [0x01, 0x08, 16, 0, 9, 0, 0, 0, 42, 0, 0, 0, 2, 0, 6, 0]);
    assert_lengths(&[close(), ask(1)]);
    for (id, length) in [
        (0x01b2, -1), (0x0801, -1), (ZC_OPENSTORE, 4), (ZC_STORE_ENTRY, 86), (ZC_DISAPPEAR_ENTRY, 6),
        (ZC_PC_PURCHASE_RESULT_FROMMC, 7), (ZC_DELETEITEM_FROM_MCSTORE, 18), (ZC_ACK_OPENSTORE2, 3),
        (ZC_PC_PURCHASE_ITEMLIST_FROMMC, -1), (ZC_PC_PURCHASE_MYITEMLIST, -1)
    ] {
        assert_eq!(packet_length(id), Some(length), "{id:#06x}");
    }
}
