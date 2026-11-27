//! Tests of the records of the windows: each press written as the client's record and read back as the press,
//! the layouts of the four kinds, and what is passed by.

use super::*;

fn open() -> Open {
    Open { quests_shown: true, ..Open::default() }
}

/// A press is written as one record of kind 900 with that window, command and values, and read back as `back`.
fn held(press: impl Into<Press>, open: &Open, window: u32, command: u32, values: [u32; 3], back: Vec<Press>) {
    let press = press.into();
    let records = records_of(&press, open);
    assert_eq!(records, vec![response(window, command, values)], "{press:?}");
    assert_eq!(deed(&records[0], open), Some(Deed::Presses(back)), "{press:?}");
}

#[test]
fn a_response_is_laid_out_as_the_clients() {
    let record = response(16, 218, [1, 2, 22]);
    assert_eq!(record.len(), 26);
    // The marker 0x038408AE and the length, then five numbers.
    assert_eq!(&record[..6], [0xae, 0x08, 0x84, 0x03, 26, 0]);
    assert_eq!(&record[6..14], [16, 0, 0, 0, 218, 0, 0, 0]);
    assert_eq!(&record[22..], [22, 0, 0, 0]);
}

#[test]
fn the_say_window_tells_next_from_close_by_its_third_value() {
    let says = Open { talk: Some(Talk::Says), ..open() };
    held(talk::Press::Next, &says, 16, 218, [0, 0, 22], vec![talk::Press::Next.into()]);
    held(talk::Press::Close, &says, 16, 218, [0, 0, 89], vec![talk::Press::Close.into()]);
    // Enter on "close" sends the window's own 201; the button after "next" (600) does nothing.
    assert_eq!(deed(&response(16, 201, [0, 0, 89]), &says), Some(Deed::Presses(vec![talk::Press::Close.into()])));
    assert_eq!(deed(&response(16, 600, [0, 0, 22]), &says), None);
}

#[test]
fn the_menu_is_answered_with_its_row() {
    let menu = Open { talk: Some(Talk::Menu(2)), ..open() };
    held(talk::Press::Ok, &menu, 17, 184, [2, 0, 0], vec![talk::Press::Pick(2).into(), talk::Press::Ok.into()]);
    held(talk::Press::Cancel, &menu, 17, 185, [0; 3], vec![talk::Press::Cancel.into()]);
    // A row only picked and a link to an item are not recorded at all.
    assert!(records_of(&talk::Press::Pick(1).into(), &menu).is_empty());
    assert!(records_of(&talk::Press::Item(501).into(), &menu).is_empty());
}

#[test]
fn a_field_of_the_talk_is_written_with_what_was_typed() {
    for (talk, window) in [(Talk::Number, 56), (Talk::Words, 100)] {
        let asked = Open { talk: Some(talk), typed: "250".to_owned(), ..open() };
        let records = records_of(&talk::Press::Ok.into(), &asked);
        assert_eq!(records, vec![texts(window, &["250"]), response(window, 184, [0; 3])]);
        assert_eq!(deed(&records[0], &asked), Some(Deed::Typed { window, texts: vec!["250".to_owned()] }));
        assert_eq!(deed(&records[1], &asked), Some(Deed::Presses(vec![talk::Press::Ok.into()])));
    }
    // The marker 0x038608AE, the whole length, the window, one field, its length and its text.
    assert_eq!(texts(56, &["250"]), [&[0xae, 0x08, 0x86, 0x03, 21, 0, 56, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0][..], b"250"].concat());
}

#[test]
fn a_shop_is_answered_window_by_window() {
    let choose = Open { shop: Some(Shop::Choose), ..open() };
    held(shops::Press::Buy, &choose, 25, 224, [0; 3], vec![shops::Press::Buy.into()]);
    held(shops::Press::Sell, &choose, 25, 223, [0; 3], vec![shops::Press::Sell.into()]);
    held(shops::Press::Cancel, &choose, 25, 185, [0; 3], vec![shops::Press::Cancel.into()]);
    let buy = Open { shop: Some(Shop::Buy), ..open() };
    held(shops::Press::Ok, &buy, 23, 184, [0; 3], vec![shops::Press::Ok.into()]);
    held(shops::Press::Cancel, &buy, 23, 185, [0; 3], vec![shops::Press::Cancel.into()]);
    let sell = Open { shop: Some(Shop::Sell), ..open() };
    held(shops::Press::Ok, &sell, 24, 184, [0; 3], vec![shops::Press::Ok.into()]);
    held(shops::Press::Cancel, &sell, 24, 185, [0; 3], vec![shops::Press::Cancel.into()]);
    // The box is recorded with what it becomes, and pressed again only if it stands otherwise.
    held(shops::Press::WholeStacks, &sell, 22, 213, [1, 0, 0], vec![shops::Press::WholeStacks.into()]);
    let ticked = Open { whole_stacks: true, ..sell.clone() };
    assert_eq!(deed(&response(22, 213, [1, 0, 0]), &ticked), Some(Deed::Presses(Vec::new())));
    // It cannot be ticked while buying; the count box's OK is a form's.
    assert!(records_of(&shops::Press::WholeStacks.into(), &buy).is_empty());
    assert!(records_of(&shops::Press::CountOk.into(), &buy).is_empty());
}

#[test]
fn the_count_box_is_written_with_its_field_and_its_number() {
    let records = counted(asked::SOLD, "12");
    assert_eq!(records, vec![texts(15, &["12"]), response(15, 184, [100, 12, 2])]);
    assert_eq!(deed(&records[1], &open()), Some(Deed::Counted { asked: 2, amount: 12 }));
    assert_eq!([shop_asked(true, true), shop_asked(true, false), shop_asked(false, true), shop_asked(false, false)], [1, 2, 3, 4]);
    assert_eq!([shop_window(true, false), shop_window(true, true), shop_window(false, true)], [23, 24, 22]);
    // What is no number counts as nought, as `atoi` has it; another command of the box is nothing.
    assert_eq!(counted(asked::BOUGHT, "x")[1], response(15, 184, [100, 0, 1]));
    assert_eq!(deed(&response(15, 185, [0; 3]), &open()), None);
}

#[test]
fn a_drop_is_laid_out_as_the_clients_and_read_back() {
    let dropped = Dropped { onto: 24, from: from::SHOP_LIST, id: 7, item: 909, amount: 30, kind: 3 };
    let record = drop_record(&dropped);
    assert_eq!(record.len(), 0x191);
    assert_eq!(&record[..6], [0xae, 0x08, 0x85, 0x03, 0x91, 0x01]);
    assert_eq!(&record[0x12..0x1a], [3, 0, 0, 0, 7, 0, 0, 0]);
    assert_eq!(&record[0x97..0x9b], b"909\0");
    assert_eq!(deed(&record, &open()), Some(Deed::Dropped(dropped)));
    // One without the item's number says nothing.
    let mut nameless = record.clone();
    nameless[0x97] = 0;
    assert_eq!(deed(&nameless, &open()), None);
}

#[test]
fn each_list_keeps_its_row_where_its_window_does() {
    let rows = [
        (List::Warp, 39, [3, 0, 0]),
        (List::Spells, 99, [3, 0, 0]),
        (List::Compound, 74, [0, 3, 0]),
        (List::Arrows, 94, [0, 0, 3]),
        (List::Make, 79, [0, 0, 3]),
        (List::Repair, 107, [0, 0, 3]),
        (List::Refine, 111, [0, 0, 3]),
    ];
    for (list, window, values) in rows {
        let listed = Open { list: Some((list, Some(3))), ..open() };
        held(choosing::Press::Ok, &listed, window, 184, values, vec![choosing::Press::Pick(3).into(), choosing::Press::Ok.into()]);
        held(choosing::Press::Cancel, &listed, window, 185, values, vec![choosing::Press::Cancel.into()]);
        // The record of a list that is not the one open is passed by.
        assert_eq!(deed(&response(window, 184, values), &open()), None);
    }
    // A forging has no rows; a slot emptied has no record of the client's and is kept as this project's own.
    let forge = Open { list: Some((List::Forge, None)), ..open() };
    held(choosing::Press::Ok, &forge, 80, 184, [0; 3], vec![choosing::Press::Ok.into()]);
    assert_eq!(records_of(&choosing::Press::Material(1).into(), &forge), vec![press_record("choice-material", "1")]);
    assert!(records_of(&choosing::Press::Pick(1).into(), &forge).is_empty());
    // No row picked: -1 where the window keeps its own, and no row is picked again.
    let none = Open { list: Some((List::Refine, None)), ..open() };
    held(choosing::Press::Ok, &none, 111, 184, [0, 0, u32::MAX], vec![choosing::Press::Ok.into()]);
}

#[test]
fn the_magnifier_and_the_eggs_keep_their_row_in_the_second_value() {
    let picked = Open { identify: Some(1), egg: 2, ..open() };
    held(inventory::Press::IdentifyOk, &picked, 73, 184, [0, 1, 0], vec![
        inventory::Press::IdentifyPick(1).into(),
        inventory::Press::IdentifyOk.into(),
    ]);
    held(inventory::Press::IdentifyCancel, &picked, 73, 185, [0, 1, 0], vec![inventory::Press::IdentifyCancel.into()]);
    held(pets::Press::EggOk, &picked, 90, 184, [0, 2, 0], vec![pets::Press::EggPick(2).into(), pets::Press::EggOk.into()]);
    held(pets::Press::EggCancel, &picked, 90, 185, [0, 2, 0], vec![pets::Press::EggCancel.into()]);
}

#[test]
fn the_boxes_that_ask_are_answered() {
    let asked = Open { party_invited: true, guild_invited: true, ..open() };
    held(exchange::Press::Accept, &asked, 32, 184, [0; 3], vec![exchange::Press::Accept.into()]);
    held(exchange::Press::Refuse, &asked, 32, 185, [0; 3], vec![exchange::Press::Refuse.into()]);
    held(party::Press::Yes, &asked, 35, 184, [0; 3], vec![party::Press::Yes.into()]);
    held(party::Press::No, &asked, 35, 185, [0; 3], vec![party::Press::No.into()]);
    held(guild::Press::Yes, &asked, 70, 184, [0; 3], vec![guild::Press::Yes.into()]);
    held(friends::Press::No, &asked, 109, 185, [0; 3], vec![friends::Press::No.into()]);
    // A box of the party that asks something else is no window the client records, and not recorded here.
    assert!(records_of(&party::Press::Yes.into(), &open()).is_empty());
    assert_eq!(deed(&response(35, 184, [0; 3]), &open()), None);
}

#[test]
fn the_deals_window_keeps_its_field_and_its_buttons_as_this_projects_own() {
    let typed = Open { typed: "500".to_owned(), ..open() };
    assert_eq!(records_of(&exchange::Press::Lock.into(), &typed), vec![texts(10011, &["500"]), press_record("exchange-lock", "")]);
    assert_eq!(records_of(&exchange::Press::Trade.into(), &typed), vec![press_record("exchange-trade", "")]);
    // A press of a window nothing records is not written.
    assert!(records_of(&crate::press::general::Press::Menu.into(), &typed).is_empty());
}

#[test]
fn the_quest_windows_tabs_and_box_are_values() {
    let record = values(10008, 22, [4, 0, 0, 0]);
    assert_eq!(record.len(), 30);
    assert_eq!(&record[..6], [0xae, 0x08, 0x91, 0x03, 30, 0]);
    assert_eq!(records_of(&quests::Press::Tab(2).into(), &open()), vec![record.clone()]);
    assert_eq!(deed(&record, &open()), Some(Deed::Presses(vec![quests::Press::Tab(2).into()])));
    // The box as it stands after the press; set again only where it stands otherwise.
    let hidden = values(10008, 6, [213, 0, 0, 0]);
    assert_eq!(records_of(&quests::Press::Display.into(), &open()), vec![hidden.clone()]);
    assert_eq!(deed(&hidden, &open()), Some(Deed::Presses(vec![quests::Press::Display.into()])));
    assert_eq!(deed(&hidden, &Open::default()), Some(Deed::Presses(Vec::new())));
}

#[test]
fn what_is_cut_short_or_unknown_is_passed_by() {
    let whole = response(17, 184, [1, 0, 0]);
    for length in 0..whole.len() {
        assert_eq!(deed(&whole[..length], &open()), None);
    }
    // A text said to be longer than the record, and more fields than there are.
    let mut lying = texts(56, &["12"]);
    lying[14] = 200;
    assert_eq!(deed(&lying, &open()), None);
    lying = texts(56, &["12"]);
    lying[10] = 9;
    assert_eq!(deed(&lying, &open()), None);
    lying[14..18].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(deed(&lying, &open()), None);
    let cut = drop_record(&Dropped { onto: 23, from: 3, id: 1, item: 501, amount: 1, kind: 0 });
    assert_eq!(deed(&cut[..200], &open()), None);
    // A window, a command and a kind nothing here answers to.
    assert_eq!(deed(&response(203, 184, [0; 3]), &open()), None);
    assert_eq!(deed(&response(25, 999, [0; 3]), &open()), None);
    assert_eq!(deed(&values(321, 22, [1, 0, 0, 0]), &open()), None);
    assert_eq!(deed(&ui(908, &[0; 40]), &open()), None);
    assert_eq!(deed(&[0x8d, 0, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22], &open()), None);
}
