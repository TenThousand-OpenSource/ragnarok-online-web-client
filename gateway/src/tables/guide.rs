//! The Adventure Guide: its pages, and the tips its quests are shown by.

use super::*;

/// The client's Adventure Guide (`Lua Files\\AdventureGuide`), as far as this client shows it: `{"intro": …,
/// "tabs": [[name, description], …], "monsters": [{…}, …]}`. The tabs are the guide's own, in its order. The
/// monsters are those of `Guide_Monster.lub`, each with the levels it is recommended for and the items shown as
/// its reward, in the order of those levels. The client's table knows a monster by its name in the server's
/// database only; what kind of monster it is the client takes from its navigation tables, and here it is taken
/// from the server's own database (`mob_db.yml`): its number, name, level, size, race, element and whether it is
/// a boss.
pub fn guide(grf: &Grf, database: &std::path::Path, client_dir: &std::path::Path) -> serde_json::Value {
    let lua = run(grf, &["adventureguide\\availablejoblist", "adventureguide\\availablejoblist_f", "adventureguide\\adventureguide_f", "adventureguide\\guide_intro", "adventureguide\\guide_monster", "adventureguide\\guide_episode", "adventureguide\\guide_quest", "adventureguide\\guide_dungeon", "adventureguide\\guide_weapon", "adventureguide\\guide_armor"]);
    let tips = tips(client_dir);
    let global = |name: &str| lua.globals().get::<Value>(name).map(|value| json_of(&value)).unwrap_or_default();
    let intro = global("GuideIntro");
    let mut tabs: Vec<(i64, serde_json::Value)> = intro["Contents"]
        .as_object()
        .map(|contents| contents.iter().filter_map(|(at, tab)| Some((at.parse().ok()?, serde_json::json!([tab["Name"], tab["Desc"]])))).collect())
        .unwrap_or_default();
    tabs.sort_by_key(|(at, _)| *at);
    // The server's monsters, by the name its database knows each by.
    let mut known: std::collections::HashMap<String, serde_json::Map<String, serde_json::Value>> = std::collections::HashMap::new();
    let mut entry: Option<(String, serde_json::Map<String, serde_json::Value>)> = None;
    let mut id = None;
    for line in std::fs::read_to_string(database.join("mob_db.yml")).unwrap_or_default().lines() {
        // Only the entry's own lines, not those of what it drops and the like, which lie deeper.
        let own = line.starts_with("    ") && !line.starts_with("     ");
        let line = line.trim_start();
        if let Some(value) = line.strip_prefix("- Id:") {
            if let Some((name, about)) = entry.take() {
                known.insert(name, about);
            }
            id = value.trim().parse::<u32>().ok();
        } else if let (Some(number), Some(name)) = (id, line.strip_prefix("AegisName:")) {
            let mut about = serde_json::Map::new();
            about.insert("id".to_owned(), number.into());
            entry = Some((name.trim().to_owned(), about));
            id = None;
        } else if let (true, Some((_, about)), Some((key, value))) = (own, &mut entry, line.split_once(':')) {
            let value = value.trim();
            match key {
                "Name" | "Size" | "Race" | "Element" | "Class" => drop(about.insert(key.to_lowercase(), value.into())),
                "Level" | "ElementLevel" => drop(about.insert(if key == "Level" { "level" } else { "element_level" }.to_owned(), value.parse::<u32>().unwrap_or(0).into())),
                _ => {}
            }
        }
    }
    if let Some((name, about)) = entry.take() {
        known.insert(name, about);
    }
    let guided = global("GuideMonster");
    let mut monsters: Vec<serde_json::Value> = guided["Contents"]
        .as_object()
        .map(|contents| {
            contents
                .iter()
                .map(|(name, about)| {
                    let mut monster = known.get(name).cloned().unwrap_or_default();
                    // The rewards are written as the client writes an item's picture into text: `^i[<id>]`.
                    let rewards: Vec<u32> = about["Reward"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|line| line.as_str())
                        .flat_map(|line| line.split("^i[").skip(1).filter_map(|item| item.split(']').next()?.parse().ok()).collect::<Vec<u32>>())
                        .collect();
                    monster.insert("key".to_owned(), name.as_str().into());
                    monster.insert("min".to_owned(), about["RequireMinLevel"].clone());
                    monster.insert("max".to_owned(), about["RequireMaxLevel"].clone());
                    monster.insert("rewards".to_owned(), rewards.into());
                    monster.into()
                })
                .collect()
        })
        .unwrap_or_default();
    monsters.sort_by_key(|monster| (monster["min"].as_i64().unwrap_or(0), monster["key"].as_str().unwrap_or_default().to_owned()));
    // The other pages: each a table with its kinds (`SubMenu`, by number) and its entries (`Contents`).
    let kinds = |table: &serde_json::Value| -> Vec<serde_json::Value> {
        let mut kinds: Vec<(i64, serde_json::Value)> = table["SubMenu"]
            .as_object()
            .map(|kinds| kinds.iter().filter_map(|(at, kind)| Some((at.parse().ok()?, serde_json::json!({ "at": at.parse::<i64>().ok()?, "title": kind["SubMenuTitle"], "min": kind["SubMenuMinLevel"], "max": kind["SubMenuMaxLevel"] })))).collect())
            .unwrap_or_default();
        kinds.sort_by_key(|(at, _)| *at);
        kinds.into_iter().map(|(_, kind)| kind).collect()
    };
    let lines = |lines: &serde_json::Value| -> Vec<String> { lines.as_array().into_iter().flatten().filter_map(|line| line.as_str()).map(plain).collect() };
    let entries = |table: &serde_json::Value, entry: &dyn Fn(u32, &serde_json::Value) -> serde_json::Value| -> Vec<serde_json::Value> {
        let mut entries: Vec<(i64, u32, serde_json::Value)> = table["Contents"]
            .as_object()
            .map(|contents| contents.iter().filter_map(|(id, about)| Some((about["RequireMinLevel"].as_i64().unwrap_or(0), id.parse().ok()?, entry(id.parse().ok()?, about)))).collect())
            .unwrap_or_default();
        entries.sort_by_key(|(least, id, _)| (*least, *id));
        entries.into_iter().map(|(_, _, entry)| entry).collect()
    };
    // Who can wear an item: a number the client's `GetEquipJob` gives for it (`AvailableJobList.lub`), which
    // stands for a set of jobs (see `job_codes.json`).
    let wearers = lua.globals().get::<mlua::Function>("GetEquipJob").ok();
    let worn_by = |id: u32| wearers.as_ref().and_then(|wearers| wearers.call::<i64>(id).ok()).unwrap_or(0);
    // An item: its number, its kind, the levels it is for, whether it can be refined, enchanted and enchanted
    // by grade, where it is had from, and the number that says who can wear it.
    let items = |name: &str| {
        let table = global(name);
        serde_json::json!({
            "kinds": kinds(&table),
            "list": entries(&table, &|id, about| serde_json::json!({
                "id": id, "kind": about["SubMenu"], "min": about["RequireMinLevel"], "max": about["RequireMaxLevel"],
                "can": [about["Refining"] == 1, about["Enchant"] == 1, about["GradeEnchant"] == 1],
                "from": lines(&about["ObtainGuide"]),
                "jobs": worn_by(id),
            })),
        })
    };
    // A quest, an episode's part or a dungeon: the entry is a tip of the client's tip box, shown by the tip's
    // title and the first line of what it says (`CUIQuestInfo::SetTitle`: of a title like `Q Hello!(1)` only
    // what is between the `Q` and the bracket; of any other, what stands before a `#`).
    let quests = |name: &str| {
        let table = global(name);
        serde_json::json!({
            "kinds": kinds(&table),
            "list": entries(&table, &|id, about| {
                let (title, says) = tips.get(&id).cloned().unwrap_or_else(|| (id.to_string(), String::new()));
                let title = match (title.strip_prefix('Q'), title.rfind('(')) {
                    (Some(_), Some(bracket)) if bracket > 1 => title[1..bracket].trim().to_owned(),
                    _ => title.split('#').next().unwrap_or_default().trim().to_owned(),
                };
                serde_json::json!({
                    "id": id, "kind": about["SubMenu"], "min": about["RequireMinLevel"], "max": about["RequireMaxLevel"],
                    "title": title, "says": says.lines().next().map(plain).unwrap_or_default(),
                    "needs": about["Prerequisite"].as_str().map(plain).unwrap_or_default(),
                    "rewards": lines(&about["Reward"]),
                })
            }),
        })
    };
    serde_json::json!({
        "intro": intro["TitleText"],
        "menu": intro["MainMenuTitle"],
        "tabs": tabs.into_iter().map(|(_, tab)| tab).collect::<Vec<_>>(),
        "monsters": monsters,
        "episode": quests("GuideEpisode"),
        "quest": quests("GuideQuest"),
        "dungeon": quests("GuideDungeon"),
        "weapon": items("GuideWeapon"),
        "armor": items("GuideArmor"),
        // Which jobs each of those numbers stands for. The client has this compiled into it
        // (`CJobCodeForItem::InitJobGroups` and `InitJobCodes`); the file is that table, written out from the
        // decompiled client: `{"novices": [jobs], "codes": {"<number>": [jobs]}}`. 0xFFFFF is everyone, 0xFFFFE
        // everyone but the novices.
        "job_codes": serde_json::from_str::<serde_json::Value>(include_str!("job_codes.json")).unwrap_or_default(),
    })
}

/// A line of the guide or of a tip without its marks: a link to a tip or to a place (`<TIPBOX>name<INFO>…</INFO>
/// </TIPBOX>`, `<NAVI>…`) is left as its name, a colour (`^rrggbb`) is dropped, and the bullet the line starts
/// with, which is a sign of the client's own font, becomes a dot.
fn plain(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('>') else { break };
        let tag = &rest[start + 1..start + end];
        rest = &rest[start + end + 1..];
        // What a link points at is not shown.
        if tag == "INFO" {
            rest = rest.find("</INFO>").map_or("", |close| &rest[close + 7..]);
        }
    }
    out.push_str(rest);
    let mut cleaned = String::new();
    let mut letters = out.chars().peekable();
    while let Some(letter) = letters.next() {
        if letter == '^' {
            let colour: String = letters.clone().take(6).collect();
            if colour.len() == 6 && colour.chars().all(|digit| digit.is_ascii_hexdigit()) {
                letters.nth(5);
                continue;
            }
        }
        cleaned.push(if letter.is_ascii() || letter.is_alphanumeric() { letter } else { '·' });
    }
    cleaned.trim().to_owned()
}

/// The client's tip box (`System\\tipbox.lub`; the English one beside it where there is one), as far as the guide
/// needs it: each tip's title and its first page, by the tip's number.
fn tips(client_dir: &std::path::Path) -> std::collections::HashMap<u32, (String, String)> {
    let Some(lua) = beside(client_dir, &["SystemEN/tipbox.lub", "System/tipbox.lub"]) else { return Default::default() };
    let Ok(table) = lua.globals().get::<Table>("tbl") else { return Default::default() };
    // The English file is plain text with a few signs beyond ASCII; the client's own is Korean.
    let words = |text: &mlua::String| match std::str::from_utf8(&text.as_bytes()) {
        Ok(text) => text.to_owned(),
        Err(_) => encoding_rs::WINDOWS_1252.decode_without_bom_handling(&text.as_bytes()).0.into_owned(),
    };
    table
        .pairs::<i64, Table>()
        .flatten()
        .filter_map(|(id, tip)| {
            let title = tip.get::<Option<mlua::String>>("Title").ok().flatten().map(|title| words(&title)).unwrap_or_default();
            let page = tip.get::<Option<Table>>("Page").ok().flatten().and_then(|pages| pages.get::<Option<mlua::String>>(1).ok().flatten()).map(|page| words(&page)).unwrap_or_default();
            Some((u32::try_from(id).ok()?, (title, page)))
        })
        .collect()
}
