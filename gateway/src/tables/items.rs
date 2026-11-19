//! The items: names, pictures and descriptions, in the client's own words, in English and in Russian.

use super::*;

/// What an item is called and which pictures are its, by item ID: from the client's `iteminfo.lub`.
pub struct Items(std::collections::HashMap<u32, (String, String)>, std::collections::HashMap<u32, About>, Korean, Translated, Unidentified);

/// What an item is before it is identified (the table's `unidentifiedDisplayName`, `unidentifiedResourceName`
/// and `unidentifiedDescriptionName`): its name, the name of its pictures, its description.
pub type Unidentified = std::collections::HashMap<u32, (String, String, Vec<String>)>;

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
    /// How many slots for cards it has (the table's `slotCount`: what the client writes as `[n]` after a name).
    slots: u8,
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
                    if matches!(key, "Type" | "SubType" | "Buy" | "Sell" | "Weight" | "Attack" | "MagicAttack" | "Defense" | "Range" | "Slots" | "WeaponLevel" | "ArmorLevel" | "EquipLevelMin" | "View")
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
        let mut unidentified = Unidentified::new();
        if let Ok(table) = lua.globals().get::<Table>("tbl") {
            for (id, item) in table.pairs::<i64, Table>().flatten() {
                let field = |name: &str| match item.get::<Value>(name) {
                    Ok(Value::String(text)) => korean(&text.as_bytes()),
                    _ => String::new(),
                };
                if let Ok(id) = u32::try_from(id) {
                    items.insert(id, (field("identifiedDisplayName"), field("identifiedResourceName")));
                    let lines: Vec<String> = item.get::<Table>("unidentifiedDescriptionName").map(|lines| lines.sequence_values::<mlua::String>().flatten().map(|line| korean(&line.as_bytes())).collect()).unwrap_or_default();
                    unidentified.insert(id, (field("unidentifiedDisplayName"), field("unidentifiedResourceName"), lines));
                    if let Ok(lines) = item.get::<Table>("identifiedDescriptionName") {
                        let description: Vec<String> = lines.sequence_values::<mlua::String>().flatten().map(|line| korean(&line.as_bytes())).collect();
                        about.insert(id, About { description, facts: Vec::new(), slots: 0 });
                    }
                    if let Ok(slots @ 1..) = item.get::<u8>("slotCount") {
                        about.entry(id).or_default().slots = slots;
                    }
                }
            }
        }
        Self(items, about, Korean::new(), Translated::new(), unidentified)
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
                // An item the client's own table lacks takes its slots from this one too.
                if about.slots == 0 {
                    about.slots = item.get::<u8>("slotCount").unwrap_or(0);
                }
                // What the item is before it is identified, in English too.
                let unknown = self.4.entry(id).or_default();
                if let Ok(Value::String(text)) = item.get::<Value>("unidentifiedDisplayName") {
                    unknown.0 = english(&text.as_bytes());
                }
                if unknown.1.is_empty() {
                    if let Ok(Value::String(resource)) = item.get::<Value>("unidentifiedResourceName") {
                        unknown.1 = korean(&resource.as_bytes());
                    }
                }
                let lines: Vec<String> = item.get::<Table>("unidentifiedDescriptionName").map(|lines| lines.sequence_values::<mlua::String>().flatten().map(|line| english(&line.as_bytes())).collect()).unwrap_or_default();
                if !lines.is_empty() {
                    unknown.2 = lines;
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
                let (description, facts, slots) = self.1.get(&id).map_or((&[][..], &[][..], 0), |about| (&about.description[..], &about.facts[..], about.slots));
                let facts: Vec<[&str; 2]> = facts.iter().map(|(key, value)| [key.as_str(), value.as_str()]).collect();
                let (name, description) = match self.3.get(&id) {
                    Some((russian, lines)) => (if russian.is_empty() { name.clone() } else { russian.clone() }, if lines.is_empty() { description.iter().map(|line| russian_line(line)).collect() } else { lines.clone() }),
                    None => (name.clone(), description.iter().map(|line| russian_line(line)).collect()),
                };
                let unidentified = self.4.get(&id).map(|(name, resource, lines)| serde_json::json!({ "name": name, "resource": resource, "description": lines.iter().map(|line| russian_line(line)).collect::<Vec<String>>() }));
                Some((id.to_string(), serde_json::json!({ "name": name, "resource": resource, "description": description, "facts": facts, "unidentified": unidentified, "slots": slots })))
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
                let (description, facts, slots) = self.1.get(&id).map_or((&[][..], &[][..], 0), |about| (&about.description[..], &about.facts[..], about.slots));
                let facts: Vec<[&str; 2]> = facts.iter().map(|(key, value)| [key.as_str(), value.as_str()]).collect();
                let unidentified = self.4.get(&id).map(|(name, resource, lines)| serde_json::json!({ "name": name, "resource": resource, "description": lines }));
                Some((id.to_string(), serde_json::json!({ "name": name, "resource": resource, "description": description, "facts": facts, "unidentified": unidentified, "slots": slots })))
            })
            .collect();
        found.into()
    }
}
