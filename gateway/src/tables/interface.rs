//! The tables of the interface: the emotions, the keys, the options of the Game Settings window.

use super::*;

/// Where the client's message table (`msgstringtable.txt`, counted from 0) has the name of each emotion: the
/// numbers of the client's own `MSGSTRINGID` names, which its emotion list uses and which are compiled into it.
const EMOTION_MESSAGES: &[(&str, usize)] = &[
    ("MSI_EMOTION_ABS", 1512), ("MSI_EMOTION_AHA", 550), ("MSI_EMOTION_ANGER", 552), ("MSI_EMOTION_ANTENNA0", 1507),
    ("MSI_EMOTION_ANTENNA1", 1508), ("MSI_EMOTION_ANTENNA2", 1509), ("MSI_EMOTION_ANTENNA3", 1510),
    ("MSI_EMOTION_BEST", 561), ("MSI_EMOTION_BIGTHROB", 548), ("MSI_EMOTION_BLABLA", 949), ("MSI_EMOTION_CHUP", 570),
    ("MSI_EMOTION_CHUPCHUP", 571), ("MSI_EMOTION_COMEON", 780), ("MSI_EMOTION_CONFUSE", 946),
    ("MSI_EMOTION_CONGRATULATION", 782), ("MSI_EMOTION_COOL", 774), ("MSI_EMOTION_CRY", 568),
    ("MSI_EMOTION_DELIGHT", 546), ("MSI_EMOTION_DICE", 951), ("MSI_EMOTION_ENE", 1515), ("MSI_EMOTION_FRET", 551),
    ("MSI_EMOTION_GO", 567), ("MSI_EMOTION_GOODBOY", 777), ("MSI_EMOTION_HELP", 566), ("MSI_EMOTION_HNG", 572),
    ("MSI_EMOTION_HPTIME", 783), ("MSI_EMOTION_HUK", 563), ("MSI_EMOTION_HUM", 948), ("MSI_EMOTION_HUM2", 1511),
    ("MSI_EMOTION_HUNGRY", 773), ("MSI_EMOTION_KEK", 556), ("MSI_EMOTION_KIK", 569), ("MSI_EMOTION_LUV", 1504),
    ("MSI_EMOTION_MAIL", 1506), ("MSI_EMOTION_MERONG", 775), ("MSI_EMOTION_MOBILE", 1505), ("MSI_EMOTION_MONEY", 553),
    ("MSI_EMOTION_O", 564), ("MSI_EMOTION_OHNO", 947), ("MSI_EMOTION_OK", 573), ("MSI_EMOTION_OOPS", 1513),
    ("MSI_EMOTION_OTL", 950), ("MSI_EMOTION_PANIC", 1516), ("MSI_EMOTION_PROFUSELY_SWEAT", 559),
    ("MSI_EMOTION_QUESTION", 545), ("MSI_EMOTION_ROCK", 1501), ("MSI_EMOTION_SCISSOR", 1502),
    ("MSI_EMOTION_SCRATCH", 560), ("MSI_EMOTION_SEXY", 779), ("MSI_EMOTION_SHY", 776), ("MSI_EMOTION_SLEEPY", 781),
    ("MSI_EMOTION_SMILE", 558), ("MSI_EMOTION_SORRY", 557), ("MSI_EMOTION_SPARK", 945), ("MSI_EMOTION_SPIT", 1514),
    ("MSI_EMOTION_SPTIME", 778), ("MSI_EMOTION_STARE", 772), ("MSI_EMOTION_STARE_ABOUT", 562),
    ("MSI_EMOTION_SURPRISE", 544), ("MSI_EMOTION_SWEAT", 549), ("MSI_EMOTION_THANKS", 555),
    ("MSI_EMOTION_THINK", 554), ("MSI_EMOTION_THROB", 547), ("MSI_EMOTION_WHISP", 1517), ("MSI_EMOTION_WRAP", 1503),
    ("MSI_EMOTION_X", 565),
];

/// The client's emotions (`Lua Files\CashEmotion\*`, as its `CCashEmotionMgr::LoadScripts` runs them), those of
/// its basic pack: `{"<id>": {"action": <which action of the sprite>, "sprite": "emotion/basic", "names": [...]}}`.
/// The names are what is typed after a slash to show one: first the name from the message table (`/!`), then the
/// others the list gives (`/delight`).
pub fn emotions(grf: &Grf) -> serde_json::Value {
    let lua = state();
    let messages = messages(grf);
    // What the scripts call in the client, written to gather what they tell; the list's names come back from
    // the function that takes them, as in the client.
    let prepared = lua.load(
        r#"EMOTIONS, NAMED, MESSAGES = {}, {}, {}
        local function nothing() return true end
        c_InsertPackInfo, c_InsertEmotionEnum, c_InsertEmotionMsgListTableForCustom, c_InsertEmotionVarientListTable, c_InsertEmotionGroup, c_InsertOrderList = nothing, nothing, nothing, nothing, nothing, nothing
        MsgStrID = setmetatable({}, { __index = function(_, key) return key end })
        local function named(pack, id, name)
            if pack == 0 and type(name) == "string" and name ~= "" then
                NAMED[id] = NAMED[id] or {}
                table.insert(NAMED[id], name)
            end
        end
        function c_InsertEmotionListTable(pack, id, key, action, frame, act, sprite)
            if pack == 0 then EMOTIONS[id] = { action = action, sprite = sprite } end
            return MESSAGES[key]
        end
        function c_InsertEmotionMsgListTable(pack, id, raw, count, ...)
            for index = 1, count do named(pack, id, (select(index, ...))) end
            return true
        end"#,
    );
    if let Err(error) = prepared.exec() {
        eprintln!("tables: emotions: {error}");
    }
    if let Ok(table) = lua.globals().get::<Table>("MESSAGES") {
        for (key, line) in EMOTION_MESSAGES {
            if let Some(name) = messages.get(*line) {
                let _ = table.set(*key, name.as_str());
            }
        }
    }
    run_in(&lua, grf, &["\\common", "cashemotion\\cashemotionclientsetting", "cashemotion\\cashemotionlistdefine_f", "cashemotion\\cashemotionlistdefine", "cashemotion\\cashemotionlist_f", "cashemotion\\cashemotionlist"]);
    if let Err(error) = lua.load("main_emotion()").exec() {
        eprintln!("tables: emotions: {error}");
    }
    let mut found = serde_json::Map::new();
    let (Ok(emotions), Ok(named)) = (lua.globals().get::<Table>("EMOTIONS"), lua.globals().get::<Table>("NAMED")) else { return found.into() };
    for (id, emotion) in emotions.pairs::<i64, Table>().flatten() {
        let (Ok(action), Ok(sprite)) = (emotion.get::<i64>("action"), emotion.get::<mlua::String>("sprite")) else { continue };
        let sprite = korean(&sprite.as_bytes()).replace('\\', "/");
        let names: Vec<String> = named.get::<Table>(id).map(|names| lines(&names)).unwrap_or_default();
        found.insert(id.to_string(), serde_json::json!({ "action": action, "sprite": sprite.trim_end_matches(".spr"), "names": names }));
    }
    found.into()
}

/// The client's keys (`Lua Files\HotKey`): `{"tables": [[{"name": …, "keys": [key, with]}, …] ×4], "names": {"<key>": …}}`.
/// The four tables are the hotkey bar, the interface, the macros and the second hotkey bar, each entry in its
/// place (the client's code knows them by table and place); the keys are Windows' key numbers, the second the
/// key held with the first (0 for none); `names` is how the client writes each key.
pub fn hotkeys(grf: &Grf) -> serde_json::Value {
    let lua = state();
    run_in(&lua, grf, &["\\common", "\\msgstring_kr", "\\hotkey"]);
    let number = |table: &Table, field: &str| table.get::<Option<i64>>(field).ok().flatten().unwrap_or(0);
    let tables: Vec<serde_json::Value> = (1..=4)
        .map(|which| {
            let Ok(table) = lua.globals().get::<Table>(format!("HOTKEY_{which}")) else { return serde_json::json!([]) };
            let last = table.pairs::<i64, Value>().flatten().map(|(place, _)| place).max().unwrap_or(-1);
            (0..=last)
                .map(|place| match table.get::<Option<Table>>(place).ok().flatten() {
                    Some(entry) => {
                        serde_json::json!({ "name": text(&entry, "EXE").unwrap_or_default(), "keys": [number(&entry, "KEY1"), number(&entry, "KEY2")] })
                    }
                    None => serde_json::json!({ "name": "", "keys": [0, 0] }),
                })
                .collect()
        })
        .collect();
    let names: serde_json::Map<String, serde_json::Value> = names(&lua, "KEYNAME").into_iter().map(|(key, name)| (key.to_string(), name.into())).collect();
    serde_json::json!({ "tables": tables, "names": names })
}

/// The options of the client's Game Settings window (`Lua Files\OptionInfo\GameSettings`, read as its
/// `CGameSettingsMgr::LoadSettings` does): `[{"id": "TT_…", "tab": 1 | 2 | 4, "run": false, "title": …,
/// "command": …, "description": …, "on": true}, …]`, in the file's order. The tab is the page it stands on
/// (1 effects, 2 controls, 4 the rest); `run` is a thing done at a press instead of switched on and off; `on`
/// is how it starts; the ID is the client's name for the option.
pub fn options(grf: &Grf) -> serde_json::Value {
    let lua = game_settings(grf);
    let words = |entry: &Table, field: &str| text(entry, field).unwrap_or_default();
    option_entries(&lua)
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": words(entry, "ID"),
                "tab": entry.raw_get::<Option<i64>>("Tab").ok().flatten().unwrap_or(0),
                "run": entry.raw_get::<Option<i64>>("Type").ok().flatten().unwrap_or(0) == 1,
                "title": words(entry, "Title"),
                "command": words(entry, "Tooltip"),
                "description": words(entry, "Description"),
                "on": entry.raw_get::<Option<bool>>("Default").ok().flatten().unwrap_or(false),
            })
        })
        .collect()
}

/// The client's `Lua Files\OptionInfo\GameSettings` run in a state of its own: the one reading of the file,
/// for the options (`options`) and for what they say when a command switches one (`commands::commands`). The
/// pages and kinds are numbered as the client numbers them; every other name the file uses (the options' own,
/// the messages') stands for itself.
pub(super) fn game_settings(grf: &Grf) -> Lua {
    let lua = state();
    let prepared = exec(&lua, b"EFFECT, CONTROL, ETC, ONOFF, EXE = 1, 2, 4, 0, 1", None).and_then(|()| names_stand_for_themselves(&lua));
    if let Err(error) = prepared {
        eprintln!("tables: options: {error}");
    }
    run_in(&lua, grf, &["optioninfo\\gamesettings"]);
    lua
}

/// The options that file leaves (`OptionTbl`), in its order.
pub(super) fn option_entries(settings: &Lua) -> Vec<Table> {
    settings.globals().raw_get::<Table>("OptionTbl").map(|table| table.sequence_values::<Table>().flatten().collect()).unwrap_or_default()
}

/// The chat window's tabs as the client starts with them (`Lua Files\\ChatWndInfo\\ChatWndInfo.lub`): `[{"name":
/// "MSI_ST_CHAT", "shows": [true, …]}, …]`, in the file's order (`ChatSubWnd_1`, `ChatSubWnd_2`…), those the file
/// has in the window ("On"). `shows` says, by the kind of a line (the number the client's code gives it: 0
/// anything else, 1 what is said, 2 whispers, 3 party, 4 guild, 6 things obtained, 7 equipment, 8 status, 11
/// skills that failed, 16 the fight, 18 experience…), whether the tab lists it: the file's `option<kind + 1>`.
/// The file names the messages it uses by the client's own names for them (`MsgStrID.…`, a table the client makes
/// for it); here each such name stands for itself.
pub fn chat_tabs(grf: &Grf) -> serde_json::Value {
    let lua = state();
    let _ = lua.load("MsgStrID = setmetatable({}, { __index = function(_, name) return name end })").exec();
    run_in(&lua, grf, &["chatwndinfo\\chatwndinfo"]);
    (1..=10)
        .filter_map(|tab| lua.globals().get::<Table>(format!("ChatSubWnd_{tab}")).ok())
        .filter(|tab| tab.get::<String>("TabState").is_ok_and(|state| state == "On"))
        .map(|tab| {
            let shows: Vec<bool> = (1..=30).map_while(|option| tab.get::<Option<i64>>(format!("option{option}")).ok().flatten()).map(|shown| shown != 0).collect();
            serde_json::json!({ "name": tab.get::<String>("TabName").unwrap_or_default(), "shows": shows })
        })
        .collect()
}
