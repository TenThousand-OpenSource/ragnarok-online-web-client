//! What the client sends about a shop: opening one's own out of the cart and closing it, asking what a vendor
//! sells, and buying from a vendor's list.

use crate::net::bytes::Writer;

/// CZ_REQ_OPENSTORE2 (0x01b2): `u16 length`, `char title[80]`, `u8 open`, then for each thing `u16 index` (its
/// place in the cart), `u16 amount`, `u32 price`. Not opening after all is the same with `open` nought (the
/// client's `MM_REQ_OPENSTORE`, which also sends nought with nothing to sell).
pub fn open(title: &str, open: bool, things: &[(u16, u16, u32)]) -> Vec<u8> {
    // 79 bytes of the title at the most, so that it ends in a zero, and never half a character.
    let packet = Writer::sized(0x01b2).field_whole_chars::<80>(title).u8(u8::from(open && !things.is_empty()));
    things.iter().fold(packet, |packet, (index, amount, price)| packet.u16(*index).u16(*amount).u32(*price)).done()
}

/// CZ_REQ_CLOSESTORE (0x012e): one's own shop is closed.
pub fn close() -> Vec<u8> {
    Writer::new(0x012e).done()
}

/// CZ_REQ_BUY_FROMMC (0x0130, 6 bytes): `u32 account`: what does that vendor sell?
pub fn ask(vendor: u32) -> Vec<u8> {
    Writer::new(0x0130).u32(vendor).done()
}

/// CZ_PC_PURCHASE_ITEMLIST_FROMMC2 (0x0801): `u16 length`, `u32 account`, `u32 shop`, then for each thing
/// `u16 amount`, `u16 index`.
pub fn buy(vendor: u32, shop: u32, wanted: &[(u16, u16)]) -> Vec<u8> {
    let packet = Writer::sized(0x0801).u32(vendor).u32(shop);
    wanted.iter().fold(packet, |packet, (index, amount)| packet.u16(*amount).u16(*index)).done()
}
