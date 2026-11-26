//! The achievements and the titles: the client's table of achievements, and what each title is called.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use super::*;

/// An achievement as the file's `main` hands it to the client (`CAchievementMgr`'s `AchievementInfo`).
/// Its texts are kept byte for byte, to be read in the file's own writing (`listed`).
#[derive(Debug, Default)]
struct Entry {
    title: Vec<u8>,
    summary: Vec<u8>,
    description: Vec<u8>,
    points: i64,
    item: i64,
    title_id: i64,
    buff: i64,
    /// Each goal's words, how many are to be reached, and the achievement its line leads to (-1 for none).
    goals: Vec<(Vec<u8>, i64, i64)>,
    counted: bool,
    icon: Vec<u8>,
    tab: Option<(i64, i64)>,
}

type Entries = Rc<RefCell<BTreeMap<i64, Entry>>>;

/// Gives a state one of the functions the client registers before it runs the file
/// (`CAchievementMgr::LoadAchievementInfo`): each answers `true, "good"` as the client's do, or `false` and
/// why for an achievement the file has not told of yet.
fn register<A: mlua::FromLuaMulti + 'static>(lua: &Lua, entries: &Entries, name: &str, set: fn(&mut BTreeMap<i64, Entry>, A) -> bool) {
    let entries = entries.clone();
    let function = lua.create_function(move |_, arguments: A| {
        let done = set(&mut entries.borrow_mut(), arguments);
        Ok((done, if done { "good" } else { "achievement info cannot be find" }))
    });
    if let Ok(function) = function {
        let _ = lua.globals().set(name, function);
    }
}

/// The seven functions of the client, over what they fill.
fn registered(lua: &Lua) -> Entries {
    let entries = Entries::default();
    register(lua, &entries, "InsertAchieveInfo", |entries, (id, title, summary, description, points): (i64, mlua::String, mlua::String, mlua::String, f64)| {
        let raw = |text: &mlua::String| text.as_bytes().to_vec();
        let entry = Entry { title: raw(&title), summary: raw(&summary), description: raw(&description), points: points as i64, ..Entry::default() };
        entries.insert(id, entry).is_none()
    });
    register(lua, &entries, "InsertAchieveResource", |entries, (id, text, count, link): (i64, mlua::String, f64, f64)| {
        let text = text.as_bytes().to_vec();
        let new = |entry: &&mut Entry| entry.goals.iter().all(|(known, ..)| *known != text);
        entries.get_mut(&id).filter(new).map(|entry| entry.goals.push((text, count as i64, link as i64))).is_some()
    });
    register(lua, &entries, "InsertAchieveRewardItem", |entries, (id, item): (i64, f64)| entries.get_mut(&id).map(|entry| entry.item = item as i64).is_some());
    register(lua, &entries, "InsertAchieveRewardTitle", |entries, (id, title): (i64, f64)| {
        entries.get_mut(&id).map(|entry| entry.title_id = title as i64).is_some()
    });
    register(lua, &entries, "InsertAchieveRewardBuff", |entries, (id, buff): (i64, f64)| entries.get_mut(&id).map(|entry| entry.buff = buff as i64).is_some());
    register(lua, &entries, "InsertAchieveUIType", |entries, (id, kind, icon): (i64, f64, mlua::String)| {
        let icon = icon.as_bytes().to_vec();
        entries.get_mut(&id).map(|entry| (entry.counted, entry.icon) = (kind as i64 == 1, icon)).is_some()
    });
    register(lua, &entries, "SetAchieveIDByTab", |entries, (id, group, sub): (i64, f64, f64)| {
        entries.get_mut(&id).map(|entry| entry.tab = Some((group as i64, sub as i64))).is_some()
    });
    entries
}

/// The achievements of a file of the client's (`System\achievement_list.lub`: the table `achievement_tbl` and
/// `main`, which hands every entry to the client's own functions), as the client's functions were told them.
fn told(source: &[u8]) -> Result<BTreeMap<i64, Entry>, String> {
    let lua = state();
    let entries = registered(&lua);
    exec(&lua, source, Some("achievement_list"))?;
    let main = lua.globals().get::<mlua::Function>("main").map_err(|error| error.to_string())?;
    let (good, why) = main.call::<(bool, Option<String>)>(()).map_err(|error| error.to_string())?;
    if !good {
        return Err(why.unwrap_or_default());
    }
    Ok(entries.take())
}

/// What each title is called (`Lua Files\DataInfo\TitleTable`: the client asks its `GetTitleString(title)`;
/// the table is `title_tbl`): `{"<title>": "Lucky", …}`.
fn titles(grf: &Grf) -> serde_json::Value {
    let lua = run(grf, &["titletable"]);
    let globals = lua.globals();
    let (Ok(table), Ok(called)) = (globals.get::<Table>("title_tbl"), globals.get::<mlua::Function>("GetTitleString")) else {
        return serde_json::json!({});
    };
    let mut found = serde_json::Map::new();
    for (title, _) in table.pairs::<Value, Value>().flatten() {
        let Some(title) = whole(&title) else { continue };
        if let Some(name) = called.call::<Value>(title).ok().as_ref().and_then(text_only).filter(|name| !name.is_empty()) {
            found.insert(title.to_string(), name.into());
        }
    }
    found.into()
}

/// `{"list": [{"id": 127001, "title": …, "summary": …, "description": …, "points": 10, "item": 0, "title_id":
/// 0, "buff": 0, "goals": [[text, count, link], …], "counted": true, "icon": "CHATTING", "group": 4, "sub": 1}, …],
/// "titles": {"<title>": name}}`: the client's achievements in the order of their numbers
/// (`System\achievement_list.lub`, the English one beside it where there is one, read as
/// `CAchievementMgr::LoadAchievementInfo` reads it: by running its `main`), and what the titles are called.
/// A goal's `link` is the achievement its line leads to when pressed (the file's `shortcut`), 0 for none.
pub fn achievements(grf: &Grf, client_dir: &std::path::Path) -> serde_json::Value {
    let files: [(&str, fn(&[u8]) -> String); 2] = [("SystemEN/achievements.lub", latin), ("System/achievement_list.lub", korean)];
    let entries = files.iter().find_map(|(file, words)| Some((file, std::fs::read(client_dir.join(file)).ok()?, words))).map(|(file, bytes, words)| {
        let told = told(&bytes).unwrap_or_else(|error| {
            eprintln!("tables: {file}: {error}");
            BTreeMap::new()
        });
        listed(&told, *words)
    });
    serde_json::json!({ "list": entries.unwrap_or_else(|| serde_json::json!([])), "titles": titles(grf) })
}

/// The achievements as JSON, their texts read in the file's own writing (`words`). One the file gave no tab
/// is left out: the client lists none such anywhere.
fn listed(entries: &BTreeMap<i64, Entry>, words: fn(&[u8]) -> String) -> serde_json::Value {
    entries
        .iter()
        .filter_map(|(id, entry)| {
            let (group, sub) = entry.tab?;
            let goals: Vec<serde_json::Value> = entry.goals.iter().map(|(text, count, link)| serde_json::json!([words(text), count, link.max(&0)])).collect();
            Some(serde_json::json!({
                "id": id, "title": words(&entry.title), "summary": words(&entry.summary), "description": words(&entry.description),
                "points": entry.points, "item": entry.item.max(0), "title_id": entry.title_id.max(0), "buff": entry.buff.max(0), "goals": goals,
                "counted": entry.counted, "icon": words(&entry.icon), "group": group, "sub": sub,
            }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file laid out as the client's is, with its `main` cut to what these tests need of it.
    const FILE: &str = r#"
        achievement_tbl = {
            [127001] = { UI_Type = 1, group = "CHATTING", major = 4, minor = 1, title = "Prontera", content = { summary = "s", details = "d" },
                resource = { [1] = { text = "Stranger", count = 3000 }, [2] = { text = "Citizen", count = 1000, shortcut = 110000 } }, score = 10 },
            [110000] = { UI_Type = 0, group = "EAT", major = 4, minor = 0, title = "Snack", content = { summary = "1 a.m.", details = "Bring" },
                resource = { [1] = { text = "Bring the food" } }, reward = { item = 22876, title = 1013, buff = 7 }, score = 20 },
        }
        function main()
            for id, info in pairs(achievement_tbl) do
                result, msg = InsertAchieveInfo(id, info.title, info.content.summary, info.content.details, info.score)
                if not result then return false, msg end
                for index, resource in ipairs(info.resource) do
                    local shortcut = resource.shortcut or -1
                    if 0 == info.UI_Type then result, msg = InsertAchieveResource(id, resource.text, 0, shortcut)
                    else result, msg = InsertAchieveResource(id, resource.text, resource.count, shortcut) end
                    if not result then return false, msg end
                end
                if nil ~= info.reward then
                    if nil ~= info.reward.item then InsertAchieveRewardItem(id, info.reward.item) end
                    if nil ~= info.reward.title then InsertAchieveRewardTitle(id, info.reward.title) end
                    if nil ~= info.reward.buff then InsertAchieveRewardBuff(id, info.reward.buff) end
                end
                InsertAchieveUIType(id, info.UI_Type, info.group)
                SetAchieveIDByTab(id, info.major, info.minor)
            end
            return true, "good"
        end
    "#;

    #[test]
    fn the_file_is_read_by_running_its_main() {
        let entries = told(FILE.as_bytes()).unwrap();
        let list = listed(&entries, latin);
        assert_eq!(list[0]["id"], 110000);
        assert_eq!(list[0]["goals"], serde_json::json!([["Bring the food", 0, 0]]));
        assert_eq!((&list[0]["item"], &list[0]["title_id"], &list[0]["buff"]), (&22876.into(), &1013.into(), &7.into()));
        assert_eq!((&list[0]["counted"], &list[0]["icon"], &list[0]["group"], &list[0]["sub"]), (&false.into(), &"EAT".into(), &4.into(), &0.into()));
        assert_eq!(list[1]["title"], "Prontera");
        assert_eq!(list[1]["goals"], serde_json::json!([["Stranger", 3000, 0], ["Citizen", 1000, 110000]]));
        assert_eq!((&list[1]["counted"], &list[1]["points"], &list[1]["item"]), (&true.into(), &10.into(), &0.into()));
    }

    #[test]
    fn a_text_is_read_in_the_files_own_writing() {
        // "사과" in EUC-KR, as the client's own file has its texts.
        let file = FILE.replace("\"Snack\"", "\"\\187\\231\\176\\250\"");
        assert_eq!(listed(&told(file.as_bytes()).unwrap(), korean)[0]["title"], "사과");
        assert_eq!(listed(&told(FILE.replace("Snack", "Café").as_bytes()).unwrap(), latin)[0]["title"], "Café");
    }

    #[test]
    fn a_file_whose_main_fails_says_why() {
        assert!(told(b"function main() return false, 'duplicate' end").unwrap_err().contains("duplicate"));
        assert!(told(b"x = 1").is_err());
    }
}
