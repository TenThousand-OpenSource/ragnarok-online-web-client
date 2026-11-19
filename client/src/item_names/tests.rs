//! The tests of how an item is named (`item_names.rs`), with a few lines of the client's tables to name by.

use super::*;

/// A few lines of the client's tables as the client's files have them.
struct Client;

impl Tables for Client {
    fn name(&self, id: u32) -> String {
        match id {
            1201 => "Knife",
            1101 => "Sword",
            2301 => "Cotton Shirt",
            501 => "Red Potion",
            9001 => "Poring Egg",
            2634 => "Wedding Ring",
            _ => "Item",
        }
        .to_owned()
    }
    fn unidentified(&self, _: u32) -> String {
        "Dagger".to_owned()
    }
    fn slots(&self, id: u32) -> u8 {
        match id {
            1201 => 3,
            1101 => 3,
            _ => 0,
        }
    }
    fn card(&self, id: u32) -> Option<Affix> {
        match id {
            4001 => Some(Affix::Before("Lucky".to_owned())),
            4002 => Some(Affix::Before("Vital".to_owned())),
            4014 => Some(Affix::After("of Ghost".to_owned())),
            _ => None,
        }
    }
    fn message(&self, line: usize) -> String {
        match line {
            line::DOUBLE => "Double",
            line::TRIPLE => "Triple",
            line::QUADRUPLE => "Quadruple",
            line::MADE_PLAIN => "'s ",
            line::MADE_FIRE => "'s Fire ",
            line::THREE_STARS => "Very Very Very Strong ",
            line::NAMELESS => "Nameless",
            line::NAMED_PET => "Beloved ",
            line::WEDDING_RING => "With Mr/Miss",
            _ => "",
        }
        .to_owned()
    }
    fn player(&self, character: u32) -> Option<String> {
        (character == 150_001).then(|| "Smith".to_owned())
    }
}

fn item(id: u32, kind: u8, refine: u8, cards: [u32; 4]) -> Named {
    Named { id, kind, identified: true, refine, cards }
}

#[test]
fn a_plain_weapon_has_its_refinement_and_its_slots() {
    assert_eq!(display_name(&item(1201, 5, 0, [0; 4]), false, &Client), "Knife [3]");
    assert_eq!(display_name(&item(1201, 5, 7, [0; 4]), false, &Client), "+7 Knife [3]");
    assert_eq!(display_name(&item(2301, 4, 4, [0; 4]), false, &Client), "+4 Cotton Shirt");
}

#[test]
fn cards_stand_before_and_after_the_name() {
    assert_eq!(display_name(&item(1201, 5, 0, [4001, 0, 0, 0]), false, &Client), "Lucky Knife [3]");
    assert_eq!(display_name(&item(1201, 5, 5, [4001, 4002, 4014, 0]), false, &Client), "+5 Lucky Vital Knife of Ghost [3]");
    assert_eq!(display_name(&item(1201, 5, 0, [4001, 4001, 4001, 0]), false, &Client), "Triple Lucky Knife [3]");
    assert_eq!(display_name(&item(1201, 5, 0, [4014, 4001, 4014, 0]), false, &Client), "Lucky Knife of Ghost Double [3]");
    assert_eq!(display_name(&item(1201, 5, 0, [4001; 4]), false, &Client), "Quadruple Lucky Knife [3]");
}

#[test]
fn a_card_in_a_later_slot_alone_still_names_equipment() {
    assert_eq!(display_name(&item(1201, 5, 0, [0, 4002, 0, 0]), false, &Client), "Vital Knife [3]");
}

#[test]
fn what_is_no_equipment_has_its_bare_name_unless_told_in_full() {
    assert_eq!(display_name(&item(501, 0, 0, [0; 4]), false, &Client), "Red Potion");
    assert_eq!(display_name(&item(501, 0, 0, [0; 4]), true, &Client), "Red Potion");
}

#[test]
fn a_forged_weapon_names_its_strength_its_smith_and_its_element() {
    // The second card: the element in its low byte, five times the star crumbs in the next.
    let forged = item(1101, 5, 0, [FORGED, (15 << 8) | 3, 150_001 & 0xffff, 150_001 >> 16]);
    assert_eq!(display_name(&forged, false, &Client), "Very Very Very Strong Smith's Fire Sword");
    let unknown = item(1101, 5, 2, [FORGED, 0, 7, 0]);
    assert_eq!(display_name(&unknown, false, &Client), "+2 Nameless's Sword");
    assert_eq!(forged.maker(), Some(150_001));
}

#[test]
fn a_brewed_potion_names_who_brewed_it() {
    let brewed = item(501, 0, 0, [MADE, 0, 150_001 & 0xffff, 150_001 >> 16]);
    assert_eq!(display_name(&brewed, false, &Client), "Smith's Red Potion");
    assert_eq!(display_name(&item(2634, 4, 0, [MADE, 0, 150_001 & 0xffff, 150_001 >> 16]), false, &Client), "SmithWith Mr/MissWedding Ring");
}

#[test]
fn a_named_pets_egg_is_beloved() {
    assert_eq!(display_name(&item(9001, 7, 0, [0x100, 5, 0, 1]), false, &Client), "Beloved Poring Egg");
    assert_eq!(display_name(&item(9001, 7, 0, [0x100, 5, 0, 0]), false, &Client), "Poring Egg");
}

#[test]
fn what_is_not_identified_has_the_other_name_and_nothing_more() {
    let unknown = Named { id: 1201, kind: 5, identified: false, refine: 3, cards: [4001, 0, 0, 0] };
    assert_eq!(display_name(&unknown, false, &Client), "Dagger");
    assert_eq!(unknown.maker(), None);
}

#[test]
fn what_an_item_is_kept_from_is_listed_in_the_tooltips_order() {
    assert_eq!(restrictions(0), Vec::<usize>::new());
    assert_eq!(restrictions(1 | 2), vec![2788, 2792]);
    assert_eq!(restrictions(255), vec![2788, 2789, 2790, 2791, 2792, 2794, 2795]);
    assert_eq!(restrictions(64), Vec::<usize>::new());
}

#[test]
fn the_slots_row_shows_only_real_cards() {
    assert_eq!(item(1201, 5, 0, [4001, 0, 4002, 0]).cards_set(), vec![4001, 4002]);
    assert!(item(1101, 5, 0, [FORGED, 3, 1, 2]).cards_set().is_empty());
    assert!(item(9001, 7, 0, [0x100, 5, 0, 1]).cards_set().is_empty());
}
