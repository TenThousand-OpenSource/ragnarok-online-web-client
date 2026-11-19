//! Tests of the item table: the server's names and facts, the translations, what the browser is given.

use super::*;

/// A folder of this test's own, emptied.
fn folder(name: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("roweb-items-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

fn empty() -> Items {
    Items(Default::default(), Default::default(), Korean::new(), Translated::new(), Unidentified::new())
}

const USABLE: &str = concat!(
    "Body:\n",
    "  - Id: 501\n    AegisName: Red_Potion\n    Name: Red Potion\n    Type: Healing\n    Buy: 50\n    Weight: 70\n    Script: |\n      itemheal rand(45,65),0;\n",
    "  - Id: 502\n    AegisName: Orange_Potion\n    Name: \"Orange Potion\"\n    Type: Healing\n    Weight: abc\n    Slots:\n",
);
const EQUIPMENT: &str = "Body:\n  - Id: 1101\n    AegisName: Sword\n    Name: 'Sword'\n    Type: Weapon\n    SubType: 1hSword\n    Attack: 25\n    Slots: 3\n    Jobs:\n      Swordman: true\n";

#[test]
fn the_server_names_the_items_and_says_what_a_player_wants_to_know() {
    let database = folder("names");
    std::fs::write(database.join("item_db_usable.yml"), USABLE).unwrap();
    std::fs::write(database.join("item_db_equip.yml"), EQUIPMENT).unwrap();
    let items = empty().with_server_names(&database);
    std::fs::remove_dir_all(&database).unwrap();
    assert_eq!(items.len(), 3);
    assert!(!items.is_empty());
    let json = items.json([501, 502, 1101, 9999].into_iter());
    assert_eq!(json.as_object().unwrap().len(), 3);
    // Weight is written in whole units; what the entry says of other things (its script) is passed by.
    assert_eq!(
        json["501"],
        serde_json::json!({ "name": "Red Potion", "resource": "", "description": [], "facts": [["Type", "Healing"], ["Buy", "50"], ["Weight", "7"]], "unidentified": null, "slots": 0 })
    );
    // A name loses its quotes; a weight that is no number stays as written; a fact with nothing after it is none.
    assert_eq!(json["502"]["name"], "Orange Potion");
    assert_eq!(json["502"]["facts"], serde_json::json!([["Type", "Healing"], ["Weight", "abc"]]));
    assert_eq!(json["1101"]["facts"], serde_json::json!([["Type", "Weapon"], ["SubType", "1hSword"], ["Attack", "25"], ["Slots", "3"]]));
    // No database: nothing is named.
    assert!(empty().with_server_names(&database).is_empty());
}

#[test]
fn a_line_of_a_description_gets_its_labels_in_russian() {
    assert_eq!(russian_line("Weight: 7"), "Вес: 7");
    assert_eq!(russian_line("Type: Restorative"), "Тип: Восстанавливающий");
    assert_eq!(russian_line("Jobs: Swordsman and Thief classes"), "Профессии: Мечник и Вор классы");
    // The longer label is taken before the shorter one that is a part of it.
    assert_eq!(russian_line("Weapon Level: 3"), "Уровень оружия: 3");
    assert_eq!(russian_line("A potion made from grinded Red Herbs."), "A potion made from grinded Red Herbs.");
}

#[test]
fn the_translation_files_of_a_folder_are_read_by_their_prefix() {
    let translations = folder("translated");
    let russian = r#"{ "501": { "name": "Красное зелье", "description": ["Лечит.", 5, "Вес: 7"] }, "x": { "name": "no number" }, "502": {} }"#;
    std::fs::write(translations.join("items-1.json"), russian).unwrap();
    std::fs::write(translations.join("items-broken.json"), "{").unwrap();
    std::fs::write(translations.join("skills-1.json"), r#"{ "1": { "name": "Базовые навыки" } }"#).unwrap();
    std::fs::write(translations.join("items-notes.txt"), "not a table").unwrap();
    let items = translated(&translations, "items");
    let skills = translated(&translations, "skills");
    assert_eq!(items.len(), 2);
    assert_eq!(items[&501], ("Красное зелье".to_owned(), vec!["Лечит.".to_owned(), "Вес: 7".to_owned()]));
    assert_eq!(items[&502], (String::new(), Vec::new()));
    assert_eq!(skills[&1].0, "Базовые навыки");

    // In Russian an item with a translation is shown by it; another keeps its name, its labels translated.
    let mut all = empty();
    all.0.insert(501, ("Red Potion".to_owned(), "빨간포션".to_owned()));
    all.0.insert(502, ("Orange Potion".to_owned(), "주황포션".to_owned()));
    all.0.insert(503, ("Yellow Potion".to_owned(), "노란포션".to_owned()));
    all.1.insert(502, About { description: vec!["Weight: 10".to_owned()], facts: vec![("Buy".to_owned(), "200".to_owned())], slots: 0 });
    all.1.insert(503, About { description: vec!["Type: Restorative".to_owned()], facts: Vec::new(), slots: 2 });
    all.2.insert(501, ("빨간 포션".to_owned(), vec!["설명".to_owned()]));
    all.4.insert(503, ("Unknown Potion".to_owned(), "포션".to_owned(), vec!["Weight: 1".to_owned()]));
    let all = all.with_russian(&translations);
    std::fs::remove_dir_all(&translations).unwrap();
    let russian = all.russian_json([501, 502, 503, 9999].into_iter());
    assert_eq!(russian.as_object().unwrap().len(), 3);
    assert_eq!(
        russian["501"],
        serde_json::json!({ "name": "Красное зелье", "resource": "빨간포션", "description": ["Лечит.", "Вес: 7"], "facts": [], "unidentified": null, "slots": 0 })
    );
    assert_eq!(
        russian["502"],
        serde_json::json!({ "name": "Orange Potion", "resource": "주황포션", "description": ["Вес: 10"], "facts": [["Buy", "200"]], "unidentified": null, "slots": 0 })
    );
    let unknown = serde_json::json!({ "name": "Unknown Potion", "resource": "포션", "description": ["Вес: 1"] });
    assert_eq!(
        russian["503"],
        serde_json::json!({ "name": "Yellow Potion", "resource": "노란포션", "description": ["Тип: Восстанавливающий"], "facts": [], "unidentified": unknown, "slots": 2 })
    );
    // The same in English is as the table has it.
    let unknown = serde_json::json!({ "name": "Unknown Potion", "resource": "포션", "description": ["Weight: 1"] });
    assert_eq!(
        all.json([503].into_iter())["503"],
        serde_json::json!({ "name": "Yellow Potion", "resource": "노란포션", "description": ["Type: Restorative"], "facts": [], "unidentified": unknown, "slots": 2 })
    );
    assert_eq!(all.korean_json([501, 502].into_iter()), serde_json::json!({ "501": { "name": "빨간 포션", "description": ["설명"] } }));
}

#[test]
fn the_english_table_beside_the_client_names_and_describes() {
    let client = folder("english");
    std::fs::create_dir_all(client.join("SystemEN/LuaFiles514")).unwrap();
    // 0xe9 is "é" in Windows-1252: the translation is no UTF-8 there.
    let mut table = b"tbl = { [501] = { unidentifiedDisplayName = 'Red Potion', unidentifiedResourceName = 'potion', unidentifiedDescriptionName = { 'Unknown.' }, ".to_vec();
    table.extend_from_slice(b"identifiedDisplayName = 'Red Potion', identifiedResourceName = 'red', identifiedDescriptionName = { 'A caf");
    table.push(0xe9);
    table.extend_from_slice(b" potion.', 'Weight: 7' }, slotCount = 2 }, [502] = { identifiedDisplayName = '', identifiedDescriptionName = {} }, [-1] = { identifiedDisplayName = 'Under' } }");
    std::fs::write(client.join("SystemEN/LuaFiles514/itemInfo.lua"), table).unwrap();
    let mut items = empty();
    items.0.insert(501, ("빨간 포션".to_owned(), "빨간포션".to_owned()));
    items.1.insert(501, About { description: vec!["설명".to_owned()], facts: Vec::new(), slots: 0 });
    let items = items.with_english(&client);
    std::fs::remove_dir_all(&client).unwrap();
    // What the client's own table had is kept as its pictures' name and, aside, as the Korean.
    let unknown = serde_json::json!({ "name": "Red Potion", "resource": "potion", "description": ["Unknown."] });
    assert_eq!(
        items.json([501, 502].into_iter()),
        serde_json::json!({ "501": { "name": "Red Potion", "resource": "빨간포션", "description": ["A caf\u{e9} potion.", "Weight: 7"], "facts": [], "unidentified": unknown, "slots": 2 } })
    );
    assert_eq!(items.korean_json([501].into_iter()), serde_json::json!({ "501": { "name": "빨간 포션", "description": ["설명"] } }));
    // No English table: the items stay as they were.
    assert_eq!(empty().with_english(&client).len(), 0);
}
