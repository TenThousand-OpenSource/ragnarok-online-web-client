//! Tests of what the tables' files share (`tables.rs`): a Lua state as the client's files need it, and
//! walking what a file leaves in it.

use super::*;

/// A state that ran this Lua source.
fn ran(source: &str) -> Lua {
    let lua = state();
    exec(&lua, source.as_bytes(), Some("test")).expect("the source runs");
    lua
}

#[test]
fn a_state_has_tables_strings_and_numbers_and_nothing_of_the_system() {
    let lua = ran("T = { string.upper('a'), math.floor(2.5), table.concat({ 'x', 'y' }) }");
    assert_eq!(json_of(&lua.globals().get::<Value>("T").unwrap()), serde_json::json!(["A", 2, "xy"]));
    let lua = ran("HAS = { io = io ~= nil, os = os ~= nil }");
    assert_eq!(json_of(&lua.globals().get::<Value>("HAS").unwrap()), serde_json::json!({ "io": false, "os": false }));
}

#[test]
fn a_file_that_does_not_run_says_why_and_by_its_name() {
    let lua = state();
    let error = exec(&lua, b"T = {", Some("broken")).unwrap_err();
    assert!(error.contains("broken"), "{error}");
    assert!(exec(&lua, b"error('no')", None).is_err());
    // What is no bytecode after all is refused, not run.
    assert!(exec(&lua, b"\x1bLua nonsense", Some("bytecode")).is_err());
}

#[test]
fn unknown_names_stand_for_themselves_once_asked() {
    let lua = ran("BEFORE = TT_SNAP");
    names_stand_for_themselves(&lua).unwrap();
    exec(&lua, b"AFTER = { TT_SNAP, EFFECT }", None).unwrap();
    assert_eq!(json_of(&lua.globals().raw_get::<Value>("BEFORE").unwrap()), serde_json::Value::Null);
    assert_eq!(json_of(&lua.globals().raw_get::<Value>("AFTER").unwrap()), serde_json::json!(["TT_SNAP", "EFFECT"]));
}

#[test]
fn a_whole_number_is_read_however_it_is_kept() {
    assert_eq!((whole(&Value::Integer(7)), whole(&Value::Number(7.0)), whole(&Value::Number(-2.9))), (Some(7), Some(7), Some(-2)));
    assert_eq!((whole(&Value::Boolean(true)), whole(&Value::Nil)), (None, None));
}

#[test]
fn texts_and_lists_of_texts_are_read_as_korean() {
    // "사과" in EUC-KR, as the client's files have it.
    let lua = ran("T = { Title = '\\187\\231\\176\\250', Level = 3, Page = { 'one', '\\187\\231\\176\\250', 5, 'past a number' } }");
    let table = lua.globals().get::<Table>("T").unwrap();
    assert_eq!((text(&table, "Title").as_deref(), text(&table, "Missing"), text(&table, "Page")), (Some("사과"), None, None));
    let pages = table.get::<Table>("Page").unwrap();
    assert_eq!(lines(&pages), ["one", "사과", "5", "past a number"]);
    assert_eq!(korean(&[0xbb, 0xe7, 0xb0, 0xfa]), "사과");
}

#[test]
fn a_translated_text_is_utf8_or_else_latin() {
    assert_eq!(latin("caf\u{e9} \u{2013}".as_bytes()), "caf\u{e9} \u{2013}");
    // Windows-1252: 0xe9 is "é", 0x96 an en dash.
    assert_eq!(latin(&[b'c', b'a', b'f', 0xe9, b' ', 0x96]), "caf\u{e9} \u{2013}");
    assert_eq!(latin(b""), "");
}

#[test]
fn the_tables_of_a_table_are_listed_by_their_numbers() {
    let lua = ran("Q = { [30] = { Title = 'c' }, [2] = { Title = 'a' }, [10.0] = { Title = 'b' }, [-1] = { Title = \
        'under' }, [5] = 'no table', key = { Title = 'named' } }");
    let entries = numbered(&lua, "Q");
    assert_eq!(entries.iter().map(|(number, entry)| (*number, entry["Title"].as_str().unwrap())).collect::<Vec<_>>(), [(2, "a"), (10, "b"), (30, "c")]);
    assert!(numbered(&lua, "Missing").is_empty());
    assert_eq!(
        names(&ran("N = { [3] = 'c', [1.0] = 'a', [2] = 7, x = 'named' }"), "N").into_iter().collect::<Vec<_>>(),
        [(1, "a".to_owned()), (3, "c".to_owned())]
    );
}

#[test]
fn a_json_object_keyed_by_numbers_is_gone_through_in_their_order() {
    let object = serde_json::json!({ "10": "ten", "9": "nine", "-1": "under", "name": "no number", "2": "two" });
    assert_eq!(
        by_number(&object).into_iter().map(|(at, entry)| (at, entry.as_str().unwrap())).collect::<Vec<_>>(),
        [(-1, "under"), (2, "two"), (9, "nine"), (10, "ten")]
    );
    assert!(by_number(&serde_json::json!(["a list"])).is_empty());
}

#[test]
fn a_hash_table_is_its_fields_line_by_line() {
    let table = "// cards\n4001#Lucky#\n\n  4002#of Vitality#1#  \n4003##\n#\n";
    assert_eq!(hash_fields(table), [vec!["4001", "Lucky"], vec!["4002", "of Vitality", "1"], vec!["4003"], vec![""]]);
}

#[test]
fn a_lua_value_becomes_json_a_list_or_an_object() {
    let lua = ran("T = { list = { 'a', 2, 2.5, true }, object = { [1] = 'one', [3] = 'three', name = \
        '\\187\\231\\176\\250', [2.5] = 'half' }, empty = {}, whole = 7.0, call = print }");
    let json = json_of(&lua.globals().get::<Value>("T").unwrap());
    assert_eq!(json["list"], serde_json::json!(["a", 2, 2.5, true]));
    // A table with a gap or a name among its keys is an object, its keys written as texts.
    assert_eq!(json["object"], serde_json::json!({ "1": "one", "3": "three", "name": "사과", "2.5": "half" }));
    // An empty table is an object too; a whole number kept with a fraction is written whole; a function is nothing.
    assert_eq!((&json["empty"], &json["whole"], &json["call"]), (&serde_json::json!({}), &serde_json::json!(7), &serde_json::Value::Null));
    assert_eq!(json_of(&Value::Nil), serde_json::Value::Null);
}

#[test]
fn korean_is_read_also_when_every_byte_was_widened() {
    // "사과" as converted bytecode leaves it: each byte of the EUC-KR a character of its own.
    let widened: String = [0xbbu8, 0xe7, 0xb0, 0xfa].iter().map(|byte| char::from(*byte)).collect();
    assert_eq!(korean(widened.as_bytes()), "사과");
    assert_eq!(korean(b"plain"), "plain");
    assert_eq!(korean(b""), "");
}

#[test]
fn the_global_tables_of_a_state_are_described() {
    let lua = ran("B = { 1, 2, 3 } A = { name = 'x' } function F() end N = 5");
    let lines = describe(&lua);
    let own: Vec<&String> = lines.iter().filter(|line| line.starts_with("A:") || line.starts_with("B:") || line.starts_with("F:")).collect();
    assert_eq!(own, ["A: 1 entries; \"name\" = \"x\"", "B: 3 entries; 1 = 1, 2 = 2, 3 = 3", "F: function"]);
    assert!(!lines.iter().any(|line| line.starts_with("N:") || line.starts_with("print:") || line.starts_with("string:")));
}

#[test]
fn the_service_is_koreas_unless_the_client_names_another() {
    assert!(jobs::service_korea("<clientinfo>\n\t<servicetype>korea</servicetype>\n</clientinfo>"));
    assert!(jobs::service_korea("<clientinfo><ServiceType> Korea </ServiceType></clientinfo>"));
    assert!(!jobs::service_korea("<clientinfo><servicetype>america</servicetype></clientinfo>"));
    // A file that names none: the client's `g_serviceType` starts as Korea's.
    assert!(jobs::service_korea("<clientinfo></clientinfo>"));
}

#[test]
fn a_skills_warned_area_is_read_level_by_level() {
    let lua = ran("QUAKE = { 'NPC_EARTHQUAKE', SkillScale = { [1] = { x = 11, y = 11 }, [2] = { x = 15, y = 13 } } } PLAIN = { 'NV_BASIC' }");
    assert_eq!(skills::scale(&lua.globals().get::<Table>("QUAKE").unwrap()), vec![[11, 11], [15, 13]]);
    assert!(skills::scale(&lua.globals().get::<Table>("PLAIN").unwrap()).is_empty());
}
