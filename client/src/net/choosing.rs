//! The lists a skill or a card opens to pick one thing of: the places of Teleport and Warp Portal, the
//! arrows to craft, the spells of Auto Spell, what can be forged, brewed and cooked, what can be repaired,
//! the weapons a smith can refine, the equipment a card goes into; the answers to each, and the name of a
//! character asked by its number (who made a thing).
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets.hpp`, `packets_struct.hpp`).

use super::bytes::{long, text, u16_at, u32_at, word, words_from};

/// ZC_WARPLIST (0x0abe; the older 0x011c of four fixed names is not sent at this version).
pub const ZC_WARPLIST: u16 = 0x0abe;
pub const ZC_ACK_REMEMBER_WARPPOINT: u16 = 0x011e;
pub const ZC_MAKINGARROW_LIST: u16 = 0x01ad;
/// ZC_AUTOSPELLLIST (0x0afb; the older 0x01cd of seven fixed skills is not sent at this version).
pub const ZC_AUTOSPELLLIST: u16 = 0x0afb;
pub const ZC_MAKABLEITEMLIST: u16 = 0x018d;
pub const ZC_ACK_REQMAKINGITEM: u16 = 0x018f;
pub const ZC_MAKINGITEM_LIST: u16 = 0x025a;
/// ZC_REPAIRITEMLIST (0x0b65; the older 0x01fc is not sent at this version).
pub const ZC_REPAIRITEMLIST: u16 = 0x0b65;
pub const ZC_ACK_ITEMREPAIR: u16 = 0x01fe;
pub const ZC_NOTIFY_WEAPONITEMLIST: u16 = 0x0221;
pub const ZC_ACK_WEAPONREFINE: u16 = 0x0223;
pub const ZC_ACK_ITEMREFINING: u16 = 0x0188;
pub const ZC_ITEMCOMPOSITION_LIST: u16 = 0x017b;
pub const ZC_ACK_ITEMCOMPOSITION: u16 = 0x017d;

/// What the client answers when a list of places is closed without a choice (`UIChooseWarpWnd`: the word
/// goes to the server as a place would).
pub const CANCEL: &str = "cancel";
/// The place the server lists for Teleport that is any cell of the map.
pub const RANDOM: &str = "Random";

/// ZC_WARPLIST (0x0abe): `u16 length, u16 skill`, then the places, 16 bytes each (a map's file name, or
/// "Random"). The skill, and the places that are not empty.
pub fn warps(packet: &[u8]) -> Option<(u16, Vec<String>)> {
    let skill = word(packet, 4)?;
    Some((skill, packet[6..].chunks_exact(16).map(text).filter(|place| !place.is_empty()).collect()))
}

/// CZ_SELECT_WARPPOINT (0x011b, 20 bytes): `u16 skill, char place[16]`: the place picked, as the server
/// named it, or `CANCEL`.
pub fn warp(skill: u16, place: &str) -> Vec<u8> {
    let mut packet = vec![0u8; 20];
    packet[..2].copy_from_slice(&0x011bu16.to_le_bytes());
    packet[2..4].copy_from_slice(&skill.to_le_bytes());
    let length = place.len().min(15);
    packet[4..4 + length].copy_from_slice(&place.as_bytes()[..length]);
    packet
}

/// ZC_ACK_REMEMBER_WARPPOINT (0x011e, 3 bytes): `u8 result`: 0 the place is kept, 1 the skill's level is too
/// low, 2 the skill is not known (the client knows a 3 too).
pub fn remembered(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// The numbers of items a packet lists from that place to its end, four bytes each.
fn ids_from(packet: &[u8], at: usize) -> Vec<u32> {
    packet.get(at..).unwrap_or_default().chunks_exact(4).map(|id| u32_at(id, 0)).collect()
}

/// ZC_MAKINGARROW_LIST (0x01ad): `u16 length`, then `u32 item` each: what arrows can be made of.
pub fn arrow_sources(packet: &[u8]) -> Vec<u32> {
    ids_from(packet, 4)
}

/// CZ_REQ_MAKINGARROW (0x01ae, 6 bytes): `u32 item`: the thing to make arrows of.
pub fn make_arrows(item: u32) -> Vec<u8> {
    [&0x01aeu16.to_le_bytes()[..], &item.to_le_bytes()].concat()
}

/// ZC_AUTOSPELLLIST (0x0afb): `u16 length`, then `u32 skill` each: the spells Auto Spell may cast (noughts
/// are left out, as the client leaves them).
pub fn spells(packet: &[u8]) -> Vec<u32> {
    ids_from(packet, 4).into_iter().filter(|skill| *skill != 0).collect()
}

/// CZ_SELECTAUTOSPELL (0x01ce, 6 bytes): `u32 skill`.
pub fn pick_spell(skill: u32) -> Vec<u8> {
    [&0x01ceu16.to_le_bytes()[..], &skill.to_le_bytes()].concat()
}

/// ZC_MAKABLEITEMLIST (0x018d): `u16 length`, then 16 bytes a thing: `u32 item, u32 material[3]`: what can
/// be forged or brewed now. The materials are noughts.
pub fn makable(packet: &[u8]) -> Vec<u32> {
    packet.get(4..).unwrap_or_default().chunks_exact(16).map(|thing| u32_at(thing, 0)).collect()
}

/// CZ_REQMAKINGITEM (0x018e, 18 bytes): `u32 item, u32 material[3]`: the thing to make, and what was put in
/// beside what it needs (the stone of an element, star crumbs). Item 0 calls the making off
/// (`UIMakeTargetListWnd`, Cancel).
pub fn make(item: u32, materials: [u32; 3]) -> Vec<u8> {
    let mut packet = 0x018eu16.to_le_bytes().to_vec();
    packet.extend_from_slice(&item.to_le_bytes());
    materials.iter().for_each(|material| packet.extend_from_slice(&material.to_le_bytes()));
    packet
}

/// The stones of the four elements (Flame Heart, Mystic Frozen, Rough Wind, Great Nature) and the Star Crumb:
/// what a smith may put into a weapon being forged beside what it needs (`ITEM_INFO::IsElementalStone`,
/// `IsForgeMaterial`).
pub const STAR_CRUMB: u32 = 1000;

pub fn elemental_stone(item: u32) -> bool {
    matches!(item, 994..=997)
}

/// Whether anything more may be put into the making of that thing: not into the stones, the iron, the steel
/// and the star crumb a smith makes (`UIMakeTargetProcessWnd::Paint` draws no slots for those).
pub fn takes_more(make: u32) -> bool {
    !matches!(make, 994..=1000)
}

/// Whether a thing dragged onto the window of a forging goes into one of its three slots
/// (`UIMakeTargetProcessWnd::DragAndDrop`): a star crumb or a stone of an element, while a slot is free, and
/// no second stone of an element.
pub fn goes_in(make: u32, put: &[u32], item: u32) -> bool {
    takes_more(make) && put.len() < 3 && (item == STAR_CRUMB || elemental_stone(item)) && !(elemental_stone(item) && put.iter().copied().any(elemental_stone))
}

/// What goes to the server of what was put in: three numbers, noughts for the empty slots.
pub fn put_in(put: &[u32]) -> [u32; 3] {
    [0, 1, 2].map(|slot| put.get(slot).copied().unwrap_or(0))
}

/// ZC_ACK_REQMAKINGITEM (0x018f, 8 bytes): `u16 result, u32 item`: how a making ended. The results the
/// client knows (`CGameMode::ReceiveItemEffectResult`): 0 forged, 1 failed, 2 brewed, 3 failed; 4 to 7 are
/// of later content.
pub fn made(packet: &[u8]) -> Option<(u16, u32)> {
    Some((word(packet, 2)?, long(packet, 4)?))
}

/// ZC_MAKINGITEM_LIST (0x025a): `u16 length, u16 kind`, then `u32 item` each: what can be cooked (kind 1)
/// or mixed by a later skill; the kind goes back with the choice.
pub fn cookable(packet: &[u8]) -> Option<(u16, Vec<u32>)> {
    Some((word(packet, 4)?, ids_from(packet, 6)))
}

/// CZ_REQ_MAKINGITEM (0x025b, 8 bytes): `u16 kind, u32 item`.
pub fn cook(kind: u16, item: u32) -> Vec<u8> {
    [&0x025bu16.to_le_bytes()[..], &kind.to_le_bytes(), &item.to_le_bytes()].concat()
}

/// A thing in a list of what can be repaired or refined: where its owner carries it, and what it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub index: u16,
    pub id: u32,
    pub refine: u8,
    pub cards: [u32; 4],
}

fn cards(bytes: &[u8], at: usize) -> [u32; 4] {
    [0, 1, 2, 3].map(|slot| u32_at(bytes, at + slot * 4))
}

/// ZC_REPAIRITEMLIST (0x0b65): `u16 length`, then 24 bytes a thing: `i16 index, u32 item, u32 card[4], u8
/// refine, u8 grade`: the broken things of the one the skill was used on.
pub fn repairable(packet: &[u8]) -> Vec<Listed> {
    packet.get(4..).unwrap_or_default().chunks_exact(24).map(|thing| Listed { index: u16_at(thing, 0), id: u32_at(thing, 2), cards: cards(thing, 6), refine: thing[22] }).collect()
}

/// CZ_REQ_ITEMREPAIR (0x0b66, 26 bytes): the thing picked, as the list had it (`i16 index, u32 item, u32
/// card[4], u8 refine, u8 grade`). The client answers a list closed without a choice with the index -1
/// (`MM_REPAIRITEM_SELECT_FROM_LIST`).
pub fn repair(thing: Option<&Listed>) -> Vec<u8> {
    let mut packet = vec![0u8; 26];
    packet[..2].copy_from_slice(&0x0b66u16.to_le_bytes());
    match thing {
        Some(thing) => {
            packet[2..4].copy_from_slice(&thing.index.to_le_bytes());
            packet[4..8].copy_from_slice(&thing.id.to_le_bytes());
            for (slot, card) in thing.cards.iter().enumerate() {
                packet[8 + slot * 4..12 + slot * 4].copy_from_slice(&card.to_le_bytes());
            }
            packet[24] = thing.refine;
        }
        None => packet[2..4].copy_from_slice(&0xffffu16.to_le_bytes()),
    }
    packet
}

/// ZC_ACK_ITEMREPAIR (0x01fe, 5 bytes): `u16 index, u8 result`: whether the repair failed.
pub fn repaired(packet: &[u8]) -> Option<bool> {
    packet.get(4).map(|result| *result == 0)
}

/// ZC_NOTIFY_WEAPONITEMLIST (0x0221): `u16 length`, then 23 bytes a weapon: `i16 index, u32 item, u8 refine,
/// u32 card[4]`: the weapons carried that the smith's skill can refine.
pub fn refinable(packet: &[u8]) -> Vec<Listed> {
    packet.get(4..).unwrap_or_default().chunks_exact(23).map(|thing| Listed { index: u16_at(thing, 0), id: u32_at(thing, 2), refine: thing[6], cards: cards(thing, 7) }).collect()
}

/// CZ_REQ_WEAPONREFINE (0x0222, 6 bytes): `i32 index`: the weapon picked, or -1 for none
/// (`MM_REFINEITEM_SELECT_FROM_LIST`).
pub fn refine(index: Option<u16>) -> Vec<u8> {
    [&0x0222u16.to_le_bytes()[..], &index.map_or(-1, i32::from).to_le_bytes()].concat()
}

/// ZC_ACK_WEAPONREFINE (0x0223, 10 bytes): `i32 result, u32 item`: 0 refined, 1 failed, 2 the skill's level
/// is too low for it, 3 the stone for it is lacking.
pub fn refined(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

/// ZC_ACK_ITEMREFINING (0x0188, 8 bytes): `u16 result, u16 index, u16 refine`: a thing carried was refined
/// at an NPC: 0 it went well, 1 it failed, 2 it lost a level (3 is another failure the client says nothing
/// of); and how far it is refined now.
pub fn item_refined(packet: &[u8]) -> Option<(u16, u16, u8)> {
    Some((word(packet, 2)?, word(packet, 4)?, word(packet, 6)?.min(255) as u8))
}

/// The client's effects of a refining or a making that went well or did not, on the one who did it
/// (`EffectTypes.h`: `EF_REFINEOK`, `EF_REFINEFAIL`, `EF_PHARMACY_OK`, `EF_PHARMACY_FAIL`).
pub mod effect {
    pub const REFINE_OK: u32 = 154;
    pub const REFINE_FAIL: u32 = 155;
    pub const PHARMACY_OK: u32 = 305;
    pub const PHARMACY_FAIL: u32 = 306;
}

/// The effect the client shows for the result of a making (`CGameMode::ReceiveItemEffectResult`); the results
/// 4 to 7 belong to later content.
pub fn made_effect(result: u16) -> Option<u32> {
    match result {
        0 => Some(effect::REFINE_OK),
        1 => Some(effect::REFINE_FAIL),
        2 => Some(effect::PHARMACY_OK),
        3 => Some(effect::PHARMACY_FAIL),
        _ => None,
    }
}

/// CZ_REQ_ITEMCOMPOSITION_LIST (0x017a, 4 bytes): `u16 index`: asks what a card carried there can be set
/// into.
pub fn ask_compound(card: u16) -> Vec<u8> {
    [&0x017au16.to_le_bytes()[..], &card.to_le_bytes()].concat()
}

/// ZC_ITEMCOMPOSITION_LIST (0x017b): `u16 length`, then where each thing is carried that the card can be
/// set into.
pub fn compoundable(packet: &[u8]) -> Vec<u16> {
    words_from(packet, 4)
}

/// CZ_REQ_ITEMCOMPOSITION (0x017c, 6 bytes): `u16 card, u16 equipment`: where each is carried.
pub fn compound(card: u16, equipment: u16) -> Vec<u8> {
    [&0x017cu16.to_le_bytes()[..], &card.to_le_bytes(), &equipment.to_le_bytes()].concat()
}

/// ZC_ACK_ITEMCOMPOSITION (0x017d, 7 bytes): `u16 equipment, u16 card, u8 result` (0 the card is in).
pub fn compounded(packet: &[u8]) -> Option<(u16, u16, bool)> {
    Some((word(packet, 2)?, word(packet, 4)?, *packet.get(6)? == 0))
}

/// The card set into the first empty slot of a thing, as the client does it itself when the server says the
/// compounding went well (`CGameMode::ReceiveCardCompositionResult`). False when no slot is empty.
pub fn set_card(cards: &mut [u32; 4], card: u32) -> bool {
    match cards.iter_mut().find(|slot| **slot == 0) {
        Some(slot) => {
            *slot = card;
            true
        }
        None => false,
    }
}

/// CZ_REQNAME_BYGID (0x0369, 6 bytes): `u32 character`: asks what the character of that number is called
/// (`CPlayerNameDictionary`: who made a thing). The answer is ZC_ACK_REQNAME_BYGID, read by
/// `net::notices::character_name`.
pub fn ask_name(character: u32) -> Vec<u8> {
    [&0x0369u16.to_le_bytes()[..], &character.to_le_bytes()].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(id: u16, body: &[u8]) -> Vec<u8> {
        [&id.to_le_bytes()[..], &(body.len() as u16 + 4).to_le_bytes(), body].concat()
    }

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
    fn a_forging_takes_star_crumbs_and_one_stone() {
        assert!(goes_in(1101, &[], 994));
        assert!(goes_in(1101, &[994], STAR_CRUMB));
        assert!(!goes_in(1101, &[994], 995), "a second stone of an element");
        assert!(!goes_in(1101, &[1000, 1000, 1000], 994), "no slot is free");
        assert!(!goes_in(1101, &[], 998), "iron is no such material");
        assert!(!goes_in(999, &[], 1000), "nothing more goes into steel");
        assert_eq!(put_in(&[994, 1000]), [994, 1000, 0]);
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
}
