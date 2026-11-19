//! Tests of the Adventure Guide's table: a line without its marks, the server's monsters, the pages.

use super::*;

#[test]
fn a_line_loses_its_marks() {
    assert_eq!(plain("Talk to <NAVI>Sprakki<INFO>prontera,150,180,0,100,0,0</INFO></NAVI> in ^ff0000Prontera^000000."), "Talk to Sprakki in Prontera.");
    assert_eq!(plain("<TIPBOX>Kafra<INFO>1042</INFO></TIPBOX> services"), "Kafra services");
    // The bullet of the client's font becomes a dot; a `^` that begins no colour stays.
    assert_eq!(plain("\u{25a0} 2^3 zeny"), "· 2^3 zeny");
    // A link whose target never closes takes the rest. A tag that never closes leaves the line as it is, after
    // what stood before the tag once more: so it is today, and this holds it (it looks like a slip).
    assert_eq!(plain("  before <broken"), "before   before <broken");
    assert_eq!(plain("a <INFO>hidden"), "a");
    assert_eq!(plain(""), "");
}

/// A folder with this `mob_db.yml` in it.
fn database(name: &str, monsters: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("roweb-guide-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("mob_db.yml"), monsters).unwrap();
    folder
}

const MONSTERS: &str = concat!(
    "Body:\n",
    "  - Id: 1002\n    AegisName: PORING\n    Name: Poring\n    Level: 1\n    Size: Medium\n    Race: Plant\n    Element: Water\n    ElementLevel: 1\n",
    "    Drops:\n      - Item: Jellopy\n        Name: Not the monster's\n",
    "  - Id: 1038\n    AegisName: OSIRIS\n    Name: Osiris\n    Level: 68\n    Class: Boss\n    Hp: 1175840\n",
);

#[test]
fn the_servers_monsters_are_known_by_their_names() {
    let folder = database("known", MONSTERS);
    let known = known_monsters(&folder);
    std::fs::remove_dir_all(&folder).unwrap();
    assert_eq!(known.len(), 2);
    assert_eq!(
        serde_json::Value::from(known["PORING"].clone()),
        serde_json::json!({ "id": 1002, "name": "Poring", "level": 1, "size": "Medium", "race": "Plant", "element": "Water", "element_level": 1 })
    );
    assert_eq!(serde_json::Value::from(known["OSIRIS"].clone()), serde_json::json!({ "id": 1038, "name": "Osiris", "level": 68, "class": "Boss" }));
    // No database, no monsters.
    assert!(known_monsters(&folder).is_empty());
}

#[test]
fn the_guides_monsters_come_in_the_order_of_their_levels() {
    let folder = database("guided", MONSTERS);
    let known = known_monsters(&folder);
    std::fs::remove_dir_all(&folder).unwrap();
    let guided = serde_json::json!({ "Contents": {
        "OSIRIS": { "RequireMinLevel": 60, "RequireMaxLevel": 99, "Reward": ["^i[616] Old Card Album", "^i[1285]^i[5053]"] },
        "PORING": { "RequireMinLevel": 1, "RequireMaxLevel": 10, "Reward": [] },
        "NOBODY": { "RequireMinLevel": 1, "RequireMaxLevel": 5 },
    } });
    let monsters = monsters(&guided, &known);
    assert_eq!(monsters.iter().map(|monster| monster["key"].as_str().unwrap()).collect::<Vec<_>>(), ["NOBODY", "PORING", "OSIRIS"]);
    assert_eq!(monsters[0], serde_json::json!({ "key": "NOBODY", "min": 1, "max": 5, "rewards": [] }));
    assert_eq!((&monsters[2]["id"], &monsters[2]["rewards"], &monsters[2]["max"]), (&serde_json::json!(1038), &serde_json::json!([616, 1285, 5053]), &serde_json::json!(99)));
    assert!(super::monsters(&serde_json::Value::Null, &known).is_empty());
}

#[test]
fn the_tabs_and_the_kinds_come_in_the_order_of_their_numbers() {
    let intro = serde_json::json!({ "Contents": { "10": { "Name": "Armor", "Desc": "What to wear" }, "2": { "Name": "Monster", "Desc": "What to hunt" }, "x": { "Name": "No number" } } });
    assert_eq!(tabs(&intro), [serde_json::json!(["Monster", "What to hunt"]), serde_json::json!(["Armor", "What to wear"])]);
    assert!(tabs(&serde_json::json!({})).is_empty());
    let page = serde_json::json!({ "SubMenu": {
        "3": { "SubMenuTitle": "Late", "SubMenuMinLevel": 50, "SubMenuMaxLevel": 99 },
        "1": { "SubMenuTitle": "Early", "SubMenuMinLevel": 1, "SubMenuMaxLevel": 49 },
    } });
    assert_eq!(kinds(&page), [serde_json::json!({ "at": 1, "title": "Early", "min": 1, "max": 49 }), serde_json::json!({ "at": 3, "title": "Late", "min": 50, "max": 99 })]);
}

#[test]
fn a_page_of_quests_shows_each_by_its_tip() {
    let table = serde_json::json!({ "SubMenu": {}, "Contents": {
        "20": { "SubMenu": 1, "RequireMinLevel": 30, "RequireMaxLevel": 60, "Prerequisite": "^0000ffNone^000000", "Reward": ["\u{25a0} Base EXP"] },
        "7": { "SubMenu": 2, "RequireMinLevel": 30, "RequireMaxLevel": 40 },
        "3": { "SubMenu": 1, "RequireMinLevel": 1, "RequireMaxLevel": 10 },
    } });
    let tips: std::collections::HashMap<u32, (String, String)> =
        [(20, ("Q Hello!(1)".to_owned(), "First ^ff0000line^000000\nSecond".to_owned())), (7, ("Seven#hidden".to_owned(), String::new()))].into_iter().collect();
    let page = quest_page(&table, &tips);
    assert_eq!(page["kinds"], serde_json::json!([]));
    assert_eq!(
        page["list"],
        serde_json::json!([
            { "id": 3, "kind": 1, "min": 1, "max": 10, "title": "3", "says": "", "needs": "", "rewards": [] },
            { "id": 7, "kind": 2, "min": 30, "max": 40, "title": "Seven", "says": "", "needs": "", "rewards": [] },
            { "id": 20, "kind": 1, "min": 30, "max": 60, "title": "Hello!", "says": "First line", "needs": "None", "rewards": ["· Base EXP"] },
        ])
    );
}

#[test]
fn a_page_of_items_says_what_each_can_and_who_wears_it() {
    let table = serde_json::json!({ "Contents": {
        "1201": { "SubMenu": 1, "RequireMinLevel": 1, "RequireMaxLevel": 20, "Refining": 1, "Enchant": 0, "ObtainGuide": ["<NAVI>Shop<INFO>prontera</INFO></NAVI>"] },
    } });
    let page = item_page(&table, None);
    assert_eq!(page["list"], serde_json::json!([{ "id": 1201, "kind": 1, "min": 1, "max": 20, "can": [true, false, false], "from": ["Shop"], "jobs": 0 }]));
}
