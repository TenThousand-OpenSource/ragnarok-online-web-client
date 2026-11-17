//! The client's own tables (which sprite a job or a headgear is), read out of its Lua files.
//!
//! The files are Lua 5.1 bytecode compiled for the 32-bit client. `lunify` rewrites it for the Lua this program
//! embeds, which then simply runs it; the tables are what the files leave in the globals.

use std::collections::BTreeMap;

use mlua::{Lua, Table, Value};

use crate::grf::Grf;

/// What Lua itself puts in the globals.
const BUILT_IN: [&str; 30] = [
    "print", "pairs", "ipairs", "type", "next", "tostring", "tonumber", "assert", "collectgarbage", "dofile", "error", "gcinfo", "getfenv", "getmetatable", "load",
    "loadfile", "loadstring", "module", "newproxy", "pcall", "rawequal", "rawget", "rawset", "require", "select", "setfenv", "setmetatable", "unpack", "xpcall", "_VERSION",
];

const ROOT: &str = "data\\luafiles514\\lua files\\";

/// Runs the client's Lua files, in order, in one Lua state. A file that is missing or does not run is skipped
/// with a note: the tables it would have filled stay empty.
pub fn run(grf: &Grf, files: &[&str]) -> Lua {
    // The files only build tables: they get the string, table and math libraries, and neither files nor the system.
    let lua = Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state");
    run_in(&lua, grf, files);
    lua
}

fn run_in(lua: &Lua, grf: &Grf, files: &[&str]) {
    for file in files {
        // A name may carry its own folder under `lua files`; the others are in `datainfo`.
        // …and one that starts with a backslash is right under `lua files`.
        let path = match file.strip_prefix('\\') {
            Some(top) => format!("{ROOT}{top}.lub"),
            None if file.contains('\\') => format!("{ROOT}{file}.lub"),
            None => format!("{ROOT}datainfo\\{file}.lub"),
        };
        let outcome = grf.read(&path).map_err(|error| error.to_string()).and_then(|bytes| {
            // Most files are bytecode; the translated ones are Lua source.
            let chunk = if bytes.starts_with(b"\x1bLua") {
                let bytecode = lunify::unify(&bytes, &lunify::Format::default(), &lunify::Settings::default()).map_err(|error| format!("{error:?}"))?;
                lua.load(bytecode).set_mode(mlua::ChunkMode::Binary)
            } else {
                lua.load(bytes).set_mode(mlua::ChunkMode::Text)
            };
            chunk.set_name(*file).exec().map_err(|error| error.to_string())
        });
        if let Err(error) = outcome {
            eprintln!("tables: {file}.lub: {error}");
        }
    }
}

/// The names of the global tables a state holds and how many entries each has, for looking around.
pub fn describe(lua: &Lua) -> Vec<String> {
    let mut lines = Vec::new();
    for pair in lua.globals().pairs::<String, Value>() {
        let Ok((name, value)) = pair else { continue };
        match value {
            Value::Table(table) if !["_G", "package", "string", "table", "math", "coroutine"].contains(&name.as_str()) => {
                let entries: Vec<String> = table.pairs::<Value, Value>().flatten().take(4).map(|(key, value)| format!("{} = {}", show(&key), show(&value))).collect();
                lines.push(format!("{name}: {} entries; {}", table.pairs::<Value, Value>().count(), entries.join(", ")));
            }
            Value::Function(_) if !BUILT_IN.contains(&name.as_str()) => lines.push(format!("{name}: function")),
            _ => {}
        }
    }
    lines.sort();
    lines
}

fn show(value: &Value) -> String {
    match value {
        Value::String(text) => format!("{:?}", korean(&text.as_bytes())),
        Value::Integer(number) => number.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Table(_) => "{…}".to_owned(),
        other => other.type_name().to_owned(),
    }
}

/// The client's strings are EUC-KR. Those of converted bytecode arrive with every byte widened to a character
/// of its own (as Latin-1 read into UTF-8), which is undone first.
pub fn korean(bytes: &[u8]) -> String {
    let narrowed: Option<Vec<u8>> = std::str::from_utf8(bytes).ok().and_then(|text| text.chars().map(|character| u8::try_from(u32::from(character)).ok()).collect());
    encoding_rs::EUC_KR.decode_without_bom_handling(narrowed.as_deref().unwrap_or(bytes)).0.into_owned()
}

/// A table of the globals as numbers to strings; entries of other types are left out.
pub fn names(lua: &Lua, table: &str) -> BTreeMap<i64, String> {
    let Ok(table) = lua.globals().get::<Table>(table) else { return BTreeMap::new() };
    table
        .pairs::<Value, Value>()
        .flatten()
        .filter_map(|(key, value)| match (key, value) {
            (Value::Integer(key), Value::String(text)) => Some((key, korean(&text.as_bytes()))),
            (Value::Number(key), Value::String(text)) => Some((key as i64, korean(&text.as_bytes()))),
            _ => None,
        })
        .collect()
}

/// What the browser client needs of the client's tables, as JSON:
/// `{"jobs": {"<id>": {"name": …}}, "headgear": {"<view id>": …}, "skills": {…}}`: what each job is called, the
/// name each headgear's sprite files end in, and the skills (see `skills`).
pub fn for_client(grf: &Grf) -> serde_json::Value {
    let lua = run(grf, &["jobidentity", "npcidentity", "jobname", "pcjobnamegender_f", "accessoryid", "accname"]);
    // The translated job names are keyed by `JOBID.<identifier>`, which the files above know under other tables.
    // An identifier none of them has gets a key of its own, so the file still runs to its end.
    let prepared = lua
        .load(
            r#"local unknown = 0
            JOBID = setmetatable({}, { __index = function(_, key)
                local id = (pcJobTbl2 and pcJobTbl2[key]) or (JTtbl and JTtbl[key]) or (jobtbl and jobtbl[key])
                if id == nil then unknown = unknown - 1; id = unknown end
                return id
            end })"#,
        )
        .exec();
    if let Err(error) = prepared {
        eprintln!("tables: {error}");
    }
    let named = run_more(&lua, grf, &["pcjobname"]);
    let job_names = names(named, "PCJobNameTable");

    let mut jobs = serde_json::Map::new();
    for (id, name) in job_names.into_iter().filter(|(id, _)| *id >= 0) {
        jobs.insert(id.to_string(), serde_json::json!({ "name": name }));
    }

    let headgear: serde_json::Map<String, serde_json::Value> = names(&lua, "AccNameTable").into_iter().map(|(id, name)| (id.to_string(), name.into())).collect();
    // The sprite files of NPCs and monsters, by the number the server shows them under.
    let sprites: serde_json::Map<String, serde_json::Value> =
        names(&lua, "JobNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // The ending of a weapon's sprite files by the kind of weapon (`_단검` for daggers), from `weapontable.lub`.
    let weapons: serde_json::Map<String, serde_json::Value> =
        names(&run(grf, &["weapontable"]), "WeaponNameTable").into_iter().filter(|(id, _)| *id > 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // The picture of each pet in its window (a file of `유저인터페이스\illust`), by the monster's number: from
    // `petinfo.lub`, whose function `GetPetIllustName_V3` the client asks.
    let pets: serde_json::Map<String, serde_json::Value> =
        names(run_more(&lua, grf, &["petinfo"]), "PetIllustNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // Its `PetAccActNameTable`: by the accessory a pet wears (an item), the actions it is then drawn with.
    let pet_acts: serde_json::Map<String, serde_json::Value> = names(&lua, "PetAccActNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    serde_json::json!({ "jobs": jobs, "headgear": headgear, "skills": skills(grf), "sprites": sprites, "weapons": weapons, "tree": skill_tree(grf), "pets": pets, "pet_acts": pet_acts, "emotions": emotions(grf), "messages": messages(grf), "hotkeys": hotkeys(grf) })
}

/// The client's message table (`msgstringtable.txt`), line by line: what the client says, by the numbers its code
/// knows the lines by.
pub fn messages(grf: &Grf) -> Vec<String> {
    grf.read("data\\msgstringtable.txt").map(|bytes| korean(&bytes).lines().map(|line| line.trim_end_matches('#').to_owned()).collect()).unwrap_or_default()
}

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
    let lua = Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state");
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
        let names: Vec<String> = named.get::<Table>(id).map(|names| names.sequence_values::<mlua::String>().flatten().map(|name| korean(&name.as_bytes())).collect()).unwrap_or_default();
        found.insert(id.to_string(), serde_json::json!({ "action": action, "sprite": sprite.trim_end_matches(".spr"), "names": names }));
    }
    found.into()
}

/// The client's keys (`Lua Files\HotKey`): `{"tables": [[{"name": …, "keys": [key, with]}, …] ×4], "names": {"<key>": …}}`.
/// The four tables are the hotkey bar, the interface, the macros and the second hotkey bar, each entry in its
/// place (the client's code knows them by table and place); the keys are Windows' key numbers, the second the
/// key held with the first (0 for none); `names` is how the client writes each key.
pub fn hotkeys(grf: &Grf) -> serde_json::Value {
    let lua = Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state");
    run_in(&lua, grf, &["\\common", "\\msgstring_kr", "\\hotkey"]);
    let number = |table: &Table, field: &str| table.get::<Option<i64>>(field).ok().flatten().unwrap_or(0);
    let tables: Vec<serde_json::Value> = (1..=4)
        .map(|which| {
            let Ok(table) = lua.globals().get::<Table>(format!("HOTKEY_{which}")) else { return serde_json::json!([]) };
            let last = table.pairs::<i64, Value>().flatten().map(|(place, _)| place).max().unwrap_or(-1);
            (0..=last)
                .map(|place| match table.get::<Option<Table>>(place).ok().flatten() {
                    Some(entry) => {
                        let name = entry.get::<Option<mlua::String>>("EXE").ok().flatten().map(|name| korean(&name.as_bytes())).unwrap_or_default();
                        serde_json::json!({ "name": name, "keys": [number(&entry, "KEY1"), number(&entry, "KEY2")] })
                    }
                    None => serde_json::json!({ "name": "", "keys": [0, 0] }),
                })
                .collect()
        })
        .collect();
    let names: serde_json::Map<String, serde_json::Value> = names(&lua, "KEYNAME").into_iter().map(|(key, name)| (key.to_string(), name.into())).collect();
    serde_json::json!({ "tables": tables, "names": names })
}

/// Runs more files in a state that already holds others' tables.
fn run_more<'lua>(lua: &'lua Lua, grf: &Grf, files: &[&str]) -> &'lua Lua {
    run_in(lua, grf, files);
    lua
}

/// What an item is called and which pictures are its, by item ID: from the client's `iteminfo.lub`.
pub struct Items(std::collections::HashMap<u32, (String, String)>, std::collections::HashMap<u32, About>, Korean, Translated);

/// A translation of names and descriptions (of items or of skills) kept in files of its own: by ID, the name and
/// the description's lines.
pub type Translated = std::collections::HashMap<u32, (String, Vec<String>)>;

/// Reads the translation files of a folder whose names start with `prefix` (`translations/ru/items-*.json`):
/// each `{"<id>": {"name": …, "description": […]}}`.
pub fn translated(folder: &std::path::Path, prefix: &str) -> Translated {
    let mut all = Translated::new();
    let Ok(files) = std::fs::read_dir(folder) else { return all };
    for file in files.flatten() {
        let name = file.file_name().to_string_lossy().into_owned();
        if !name.starts_with(prefix) || !name.ends_with(".json") {
            continue;
        }
        let parsed = std::fs::read_to_string(file.path()).map_err(|error| error.to_string()).and_then(|text| serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text).map_err(|error| error.to_string()));
        match parsed {
            Ok(entries) => {
                for (id, entry) in entries {
                    let Ok(id) = id.parse::<u32>() else { continue };
                    let name = entry.get("name").and_then(|name| name.as_str()).unwrap_or_default().to_owned();
                    let description = entry.get("description").and_then(|lines| lines.as_array()).map(|lines| lines.iter().filter_map(|line| line.as_str().map(str::to_owned)).collect()).unwrap_or_default();
                    all.insert(id, (name, description));
                }
            }
            Err(error) => eprintln!("tables: {}: {error}", file.path().display()),
        }
    }
    all
}

/// The words the English item descriptions are built of, in Russian: an item with no translation of its own is
/// shown with at least these translated.
const RUSSIAN_LABELS: &[(&str, &str)] = &[
    ("Weapon Level:", "Уровень оружия:"),
    ("Armor Level:", "Уровень брони:"),
    ("Base Level:", "Базовый уровень:"),
    ("Requirement:", "Требования:"),
    ("Refineable:", "Заточка:"),
    ("Refinable:", "Заточка:"),
    ("Position:", "Место:"),
    ("Location:", "Место:"),
    ("Property:", "Стихия:"),
    ("Element:", "Стихия:"),
    ("Defense:", "Защита:"),
    ("Attack:", "Атака:"),
    ("Weight:", "Вес:"),
    ("Type:", "Тип:"),
    ("Class:", "Класс:"),
    ("Heal:", "Лечение:"),
    ("Jobs:", "Профессии:"),
    ("Item ID:", "ID предмета:"),
    ("Restorative", "Восстанавливающий"),
    ("Collectible", "Трофей"),
    ("Headgear", "Головной убор"),
    ("Garment", "Накидка"),
    ("Accessory", "Аксессуар"),
    ("Footgear", "Обувь"),
    ("Shield", "Щит"),
    ("Armor", "Броня"),
    ("Dagger", "Кинжал"),
    ("One-Handed Sword", "Одноручный меч"),
    ("Two-Handed Sword", "Двуручный меч"),
    ("Bow", "Лук"),
    ("Staff", "Посох"),
    ("Mace", "Булава"),
    ("Spear", "Копьё"),
    ("Axe", "Топор"),
    ("Arrow", "Стрела"),
    ("Card", "Карта"),
    ("Upper", "Верх"),
    ("Middle", "Середина"),
    ("Lower", "Низ"),
    ("None", "Нет"),
    ("Yes", "Да"),
    ("No", "Нет"),
    ("All Jobs", "Все профессии"),
    ("All classes", "Все профессии"),
    ("Novice", "Новичок"),
    ("Swordsman", "Мечник"),
    ("Magician", "Маг"),
    ("Archer", "Лучник"),
    ("Merchant", "Торговец"),
    ("Thief", "Вор"),
    ("Acolyte", "Послушник"),
    ("Soul Linker", "Медиум"),
    ("Ninja", "Ниндзя"),
    ("Gunslinger", "Стрелок"),
    ("Taekwon", "Тхэквондист"),
    ("classes", "классы"),
    (" and ", " и "),
];

/// An English description line with its labels and common words put into Russian.
fn russian_line(line: &str) -> String {
    let mut line = line.to_owned();
    for (english, russian) in RUSSIAN_LABELS {
        if line.contains(english) {
            line = line.replace(english, russian);
        }
    }
    line
}

/// What the client's own (Korean) table calls each item and says of it, kept apart from what is shown: the name
/// and the description's lines, by item ID.
type Korean = std::collections::HashMap<u32, (String, Vec<String>)>;

/// What more is told of an item when it is looked at: the client's own description of it (line by line), and
/// what the server's database says of it (`Type`, `Weight` and the like, as it names them).
#[derive(Default)]
pub struct About {
    description: Vec<String>,
    facts: Vec<(String, String)>,
}

impl Items {
    /// Takes the items' names from the server's item database instead (rAthena's `db/re` or `db/pre-re` folder):
    /// this client's own table is Korean, and the server's names are the ones its players know. Items the
    /// database lacks keep the client's name.
    pub fn with_server_names(mut self, database: &std::path::Path) -> Self {
        let mut named = 0;
        for file in ["item_db_usable.yml", "item_db_equip.yml", "item_db_etc.yml"] {
            let Ok(text) = std::fs::read_to_string(database.join(file)) else { continue };
            // Entries are `- Id: <id>` followed, a few lines on, by `Name: <name>`.
            let mut id = None;
            let mut last = None;
            for line in text.lines() {
                let line = line.trim_start();
                // What the database says of the item last named, as far as a player wants to know.
                if let (Some(item), Some((key, value))) = (last, line.split_once(':')) {
                    if matches!(key, "Type" | "SubType" | "Buy" | "Sell" | "Weight" | "Attack" | "MagicAttack" | "Defense" | "Range" | "Slots" | "WeaponLevel" | "ArmorLevel" | "EquipLevelMin")
                        && !value.trim().is_empty()
                    {
                        let value = value.trim();
                        // Weight is kept in tenths.
                        let value = if key == "Weight" { value.parse::<f64>().map_or(value.to_owned(), |weight| format!("{}", weight / 10.0)) } else { value.to_owned() };
                        self.1.entry(item).or_default().facts.push((key.to_owned(), value));
                    }
                }
                if let Some(value) = line.strip_prefix("- Id:") {
                    id = value.trim().parse::<u32>().ok();
                    last = id;
                } else if let Some(name) = line.strip_prefix("Name:") {
                    // The first name after an ID is the item's; later ones belong to other things in the entry.
                    let Some(item) = id.take() else { continue };
                    let name = name.trim().trim_matches(|quote| quote == '"' || quote == '\'').to_owned();
                    self.0.entry(item).or_default().0 = name;
                    named += 1;
                }
            }
        }
        eprintln!("tables: {named} item names from {}", database.display());
        self
    }

    /// Reads the item table. It is large (tens of thousands of items), so the browser asks for the items it
    /// meets instead of getting all of it.
    pub fn load(grf: &Grf) -> Self {
        let lua = run(grf, &["iteminfo"]);
        let mut items = std::collections::HashMap::new();
        let mut about = std::collections::HashMap::new();
        if let Ok(table) = lua.globals().get::<Table>("tbl") {
            for (id, item) in table.pairs::<i64, Table>().flatten() {
                let field = |name: &str| match item.get::<Value>(name) {
                    Ok(Value::String(text)) => korean(&text.as_bytes()),
                    _ => String::new(),
                };
                if let Ok(id) = u32::try_from(id) {
                    items.insert(id, (field("identifiedDisplayName"), field("identifiedResourceName")));
                    if let Ok(lines) = item.get::<Table>("identifiedDescriptionName") {
                        let description: Vec<String> = lines.sequence_values::<mlua::String>().flatten().map(|line| korean(&line.as_bytes())).collect();
                        about.insert(id, About { description, facts: Vec::new() });
                    }
                }
            }
        }
        Self(items, about, Korean::new(), Translated::new())
    }

    /// Takes the items' names and descriptions from the English table that comes with the client
    /// (`SystemEN/LuaFiles514/itemInfo.lua`, the ROenglishRE translation: the same table as the client's own,
    /// in English). What the client's own table said is kept aside as the Korean translation. Items the English
    /// table lacks keep what they had.
    pub fn with_english(mut self, client: &std::path::Path) -> Self {
        let path = client.join("SystemEN/LuaFiles514/itemInfo.lua");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("tables: no English item table at {}", path.display());
            return self;
        };
        let lua = Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state");
        if let Err(error) = lua.load(bytes).set_mode(mlua::ChunkMode::Text).set_name("itemInfo").exec() {
            eprintln!("tables: {}: {error}", path.display());
            return self;
        }
        // The translation is plain text; the few bytes beyond ASCII in it are Latin letters and signs.
        let english = |bytes: &[u8]| match std::str::from_utf8(bytes) {
            Ok(text) => text.to_owned(),
            Err(_) => encoding_rs::WINDOWS_1252.decode_without_bom_handling(bytes).0.into_owned(),
        };
        let mut translated = 0;
        if let Ok(table) = lua.globals().get::<Table>("tbl") {
            for (id, item) in table.pairs::<i64, Table>().flatten() {
                let Ok(id) = u32::try_from(id) else { continue };
                let name = match item.get::<Value>("identifiedDisplayName") {
                    Ok(Value::String(text)) => english(&text.as_bytes()),
                    _ => String::new(),
                };
                let description: Vec<String> = item.get::<Table>("identifiedDescriptionName").map(|lines| lines.sequence_values::<mlua::String>().flatten().map(|line| english(&line.as_bytes())).collect()).unwrap_or_default();
                if name.is_empty() && description.is_empty() {
                    continue;
                }
                let entry = self.0.entry(id).or_default();
                let about = self.1.entry(id).or_default();
                self.2.insert(id, (entry.0.clone(), about.description.clone()));
                if !name.is_empty() {
                    entry.0 = name;
                }
                // An item the client's own table lacks takes its pictures' name from this one.
                if entry.1.is_empty() {
                    if let Ok(Value::String(resource)) = item.get::<Value>("identifiedResourceName") {
                        entry.1 = korean(&resource.as_bytes());
                    }
                }
                if !description.is_empty() {
                    about.description = description;
                }
                translated += 1;
            }
        }
        eprintln!("tables: {translated} items in English from {}", path.display());
        self
    }

    /// Takes the Russian names and descriptions from the translation files (`items-*.json` of the folder).
    pub fn with_russian(mut self, folder: &std::path::Path) -> Self {
        self.3 = translated(folder, "items");
        eprintln!("tables: {} items in Russian from {}", self.3.len(), folder.display());
        self
    }

    /// The items with these IDs in Russian, as `json` gives them in English: an item with a translation of its
    /// own is shown by it; any other keeps its English name, with the labels of its description in Russian.
    pub fn russian_json(&self, ids: impl Iterator<Item = u32>) -> serde_json::Value {
        let found: serde_json::Map<String, serde_json::Value> = ids
            .filter_map(|id| {
                let (name, resource) = self.0.get(&id)?;
                let (description, facts) = self.1.get(&id).map_or((&[][..], &[][..]), |about| (&about.description[..], &about.facts[..]));
                let facts: Vec<[&str; 2]> = facts.iter().map(|(key, value)| [key.as_str(), value.as_str()]).collect();
                let (name, description) = match self.3.get(&id) {
                    Some((russian, lines)) => (if russian.is_empty() { name.clone() } else { russian.clone() }, if lines.is_empty() { description.iter().map(|line| russian_line(line)).collect() } else { lines.clone() }),
                    None => (name.clone(), description.iter().map(|line| russian_line(line)).collect()),
                };
                Some((id.to_string(), serde_json::json!({ "name": name, "resource": resource, "description": description, "facts": facts })))
            })
            .collect();
        found.into()
    }

    /// The Korean names and descriptions of the items with these IDs, as JSON: `{"<id>": {"name": …,
    /// "description": […]}}`.
    pub fn korean_json(&self, ids: impl Iterator<Item = u32>) -> serde_json::Value {
        let found: serde_json::Map<String, serde_json::Value> = ids.filter_map(|id| self.2.get(&id).map(|(name, description)| (id.to_string(), serde_json::json!({ "name": name, "description": description })))).collect();
        found.into()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The items with these IDs, as JSON: `{"<id>": {"name": …, "resource": …}}`. IDs the table lacks are left
    /// out.
    pub fn json(&self, ids: impl Iterator<Item = u32>) -> serde_json::Value {
        let found: serde_json::Map<String, serde_json::Value> =
            ids.filter_map(|id| {
                let (name, resource) = self.0.get(&id)?;
                let (description, facts) = self.1.get(&id).map_or((&[][..], &[][..]), |about| (&about.description[..], &about.facts[..]));
                let facts: Vec<[&str; 2]> = facts.iter().map(|(key, value)| [key.as_str(), value.as_str()]).collect();
                Some((id.to_string(), serde_json::json!({ "name": name, "resource": resource, "description": description, "facts": facts })))
            })
            .collect();
        found.into()
    }
}

/// Where each job's skills stand in the skill window (the client's `skilltreeview.lub`), and which job each job
/// grew out of (`jobinheritlist.lub`): `{"<job>": {"from": <job or null>, "slots": {"<place>": <skill>}}}`. A
/// place counts the window's cells row by row, seven to a row.
fn skill_tree(grf: &Grf) -> serde_json::Map<String, serde_json::Value> {
    let lua = run(grf, &["skillinfoz\\jobinheritlist", "skillinfoz\\skillid", "skillinfoz\\skilltreeview"]);
    let mut tree = serde_json::Map::new();
    let Ok(views) = lua.globals().get::<Table>("SKILL_TREEVIEW_FOR_JOB") else { return tree };
    let inherits = lua.globals().get::<Table>("JOB_INHERIT_LIST").ok();
    for (job, slots) in views.pairs::<i64, Table>().flatten() {
        let slots: serde_json::Map<String, serde_json::Value> = slots.pairs::<i64, i64>().flatten().map(|(place, skill)| (place.to_string(), skill.into())).collect();
        let from = inherits.as_ref().and_then(|inherits| inherits.get::<i64>(job).ok());
        tree.insert(job.to_string(), serde_json::json!({ "from": from, "slots": slots }));
    }
    tree
}

/// What the client says of each skill when it is looked at (its `skilldescript.lub`): the lines, by skill ID.
pub fn skill_texts(grf: &Grf) -> std::collections::HashMap<u32, Vec<String>> {
    let lua = run(grf, &["skillinfoz\\skillid", "skillinfoz\\skilldescript"]);
    let Ok(table) = lua.globals().get::<Table>("SKILL_DESCRIPT") else { return Default::default() };
    table
        .pairs::<i64, Table>()
        .flatten()
        .filter_map(|(id, lines)| Some((u32::try_from(id).ok()?, lines.sequence_values::<mlua::String>().flatten().map(|line| korean(&line.as_bytes())).collect())))
        .collect()
}

/// The skills, by ID: `{"<id>": {"resource": "NV_BASIC", "name": "Basic Skill", "max": 9}}`. The resource names a
/// skill's pictures.
fn skills(grf: &Grf) -> serde_json::Map<String, serde_json::Value> {
    // The skill table names the jobs it speaks of through `JOBID`: any number will do for those.
    let lua = run(grf, &[]);
    if let Err(error) = lua.load("local next = 0; JOBID = setmetatable({}, { __index = function() next = next + 1; return next end })").exec() {
        eprintln!("tables: {error}");
    }
    run_in(&lua, grf, &["skillinfoz\\skillid", "skillinfoz\\skillinfolist"]);
    let mut skills = serde_json::Map::new();
    let Ok(table) = lua.globals().get::<Table>("SKILL_INFO_LIST") else { return skills };
    for (id, skill) in table.pairs::<i64, Table>().flatten() {
        let text = |value: mlua::Result<Value>| match value {
            Ok(Value::String(text)) => korean(&text.as_bytes()),
            _ => String::new(),
        };
        let resource = text(skill.get::<Value>(1));
        if resource.is_empty() {
            continue;
        }
        let max = skill.get::<i64>("MaxLv").unwrap_or(1);
        skills.insert(id.to_string(), serde_json::json!({ "resource": resource, "name": text(skill.get::<Value>("SkillName")), "max": max }));
    }
    skills
}
