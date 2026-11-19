//! The quests: what the journal writes of each, what the client recommends, and how it sorts them.

use super::*;

/// What the client's quest tables tell (`CRenewQuestMgr::LoadQuestInfo`; the files lie beside the archives):
/// `{"ongoing": {"<quest>": {"title", "summary", "icon", "lines": […], "waits": true, "npc": [sprite, map, x, y],
/// "base", "job", "rewards": [[item, count], …]}}, "recommended": {"<number>": {"title", "summary", "icon",
/// "picture", "npc", "info": [[…], […], […]]}}, "tooltips": {"<icon>": …}, "filters": {"<number>": {"name",
/// "icons": […]}}}`. A field a quest has nothing for is left out.
///
/// - The quests of the server are `System\OngoingQuestInfoList_True.lub`, its English one
///   (`SystemEN\OngoingQuests.lub`, the ROenglishRE translation of the same table) where there is one: the
///   table `QuestInfoList`, whose fields the client asks through `GetOngoingQuestInfoByID`,
///   `GetOngoingDescription` and `GetOngoingRewardInfo` of its `QuestInfo_f.lub`. The table has every quest
///   the game ever had, some megabytes of it; only the quests the server's own database knows
///   (`quest_db.yml` beside the item database, and its `import` folder) are given, since the server can give
///   no other. That cut is this program's, not the client's; with no such database every quest is given.
/// - What the client recommends is `System\RecommendedQuestInfoList_True.lub` (`SystemEN\RecommendedQuests.lub`),
///   the table `RecommendedQuestInfoList`.
/// - The words for the icons and the filters of the journal's lower row are `System\QuestClassificationInfo.lub`
///   (its English one first): `QuestIconTooltipTable` and `QuestFilterTable`.
pub fn quests(client_dir: &std::path::Path, database: &std::path::Path) -> serde_json::Value {
    let known = known_quests(database);
    let list = |files: &[&str], name: &str| beside(client_dir, files).map(|lua| numbered(&lua, name)).unwrap_or_default();
    let ongoing: serde_json::Map<String, serde_json::Value> = list(&["SystemEN/OngoingQuests.lub", "System/OngoingQuestInfoList_True.lub", "System/OngoingQuestInfoList.lub"], "QuestInfoList")
        .iter()
        .filter(|(number, _)| known.is_empty() || known.contains(number))
        .map(|(number, entry)| (number.to_string(), ongoing(entry).into()))
        .collect();
    let recommended: serde_json::Map<String, serde_json::Value> =
        list(&["SystemEN/RecommendedQuests.lub", "System/RecommendedQuestInfoList_True.lub", "System/RecommendedQuestInfoList.lub"], "RecommendedQuestInfoList").iter().map(|(number, entry)| (number.to_string(), recommended(entry).into())).collect();
    let sorting = beside(client_dir, &["SystemEN/QuestClassificationInfo.lub", "System/QuestClassificationInfo_EN.lub", "System/QuestClassificationInfo.lub"]);
    let tooltips = sorting.as_ref().and_then(|lua| lua.globals().get::<Value>("QuestIconTooltipTable").ok()).map(|table| json_of(&table)).unwrap_or_default();
    let filters: serde_json::Map<String, serde_json::Value> = sorting.as_ref().map(|lua| numbered(lua, "QuestFilterTable")).unwrap_or_default().iter().map(|(number, entry)| (number.to_string(), filter(entry))).collect();
    serde_json::json!({ "ongoing": ongoing, "recommended": recommended, "tooltips": tooltips, "filters": filters })
}

/// The numbers of the quests the server's database has (`- Id: 1001` in `quest_db.yml` and in the `import`
/// folder's): empty when neither file is there.
fn known_quests(database: &std::path::Path) -> std::collections::HashSet<u64> {
    let files = [database.join("quest_db.yml"), database.join("../import/quest_db.yml")];
    files.iter().filter_map(|file| std::fs::read_to_string(file).ok()).flat_map(|text| quest_numbers(&text)).collect()
}

/// The numbers after `- Id:` in a database file; a line commented out is not one.
fn quest_numbers(text: &str) -> Vec<u64> {
    text.lines().filter_map(|line| line.trim_start().strip_prefix("- Id:")?.trim().parse().ok()).collect()
}

/// A field of an entry when it has something in it: a text that is not empty, a number that is not 0, a list
/// with something in it.
fn filled<'entry>(entry: &'entry serde_json::Value, field: &str) -> Option<&'entry serde_json::Value> {
    entry.get(field).filter(|value| match value {
        serde_json::Value::String(text) => !text.is_empty(),
        serde_json::Value::Number(number) => number.as_f64() != Some(0.0),
        serde_json::Value::Array(list) => !list.is_empty(),
        serde_json::Value::Object(map) => !map.is_empty(),
        _ => false,
    })
}

/// Copies the fields of a table's entry that have something in them, each under the name the browser knows it by.
fn copied(entry: &serde_json::Value, fields: &[(&str, &str)]) -> serde_json::Map<String, serde_json::Value> {
    fields.iter().filter_map(|(theirs, ours)| Some(((*ours).to_owned(), filled(entry, theirs)?.clone()))).collect()
}

/// Whom a quest sends the character to: `[sprite, map, x, y]`, when the table names a sprite.
fn npc(entry: &serde_json::Value) -> Option<serde_json::Value> {
    let sprite = filled(entry, "NpcSpr")?;
    Some(serde_json::json!([sprite, entry.get("NpcNavi").and_then(|map| map.as_str()).unwrap_or(""), entry.get("NpcPosX").and_then(|x| x.as_i64()).unwrap_or(0), entry.get("NpcPosY").and_then(|y| y.as_i64()).unwrap_or(0)]))
}

/// What the journal writes of one of the server's quests, out of its entry of `QuestInfoList`.
fn ongoing(entry: &serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    let mut about = copied(entry, &[("Title", "title"), ("Summary", "summary"), ("IconName", "icon"), ("Description", "lines"), ("RewardEXP", "base"), ("RewardJEXP", "job")]);
    if filled(entry, "CoolTimeQuest").is_some() {
        about.insert("waits".to_owned(), true.into());
    }
    if let Some(npc) = npc(entry) {
        about.insert("npc".to_owned(), npc);
    }
    let rewards: Vec<serde_json::Value> = entry
        .get("RewardItemList")
        .and_then(|list| list.as_array())
        .map(|list| list.iter().filter_map(|reward| Some(serde_json::json!([reward.get("ItemID")?.as_i64()?, reward.get("ItemNum").and_then(|count| count.as_i64()).unwrap_or(1)]))).collect())
        .unwrap_or_default();
    if !rewards.is_empty() {
        about.insert("rewards".to_owned(), rewards.into());
    }
    about
}

/// One of the things the client recommends, out of its entry of `RecommendedQuestInfoList`.
fn recommended(entry: &serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    let mut about = copied(entry, &[("Title", "title"), ("Summary", "summary"), ("IconName", "icon"), ("BgName", "picture")]);
    if let Some(npc) = npc(entry) {
        about.insert("npc".to_owned(), npc);
    }
    let info: Vec<serde_json::Value> = ["QuestInfo1", "QuestInfo2", "QuestInfo3"].iter().map(|part| filled(entry, part).cloned().unwrap_or_else(|| serde_json::json!([]))).collect();
    about.insert("info".to_owned(), info.into());
    about
}

/// A filter of the journal's lower row, out of its entry of `QuestFilterTable`: its words and the icons it takes.
fn filter(entry: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "name": entry.get("Tooltip").and_then(|name| name.as_str()).unwrap_or(""), "icons": entry.get("QuestIcons").cloned().unwrap_or_else(|| serde_json::json!([])) })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A table of the client's, as its Lua text leaves it in the globals: its entries by number.
    fn table(source: &str, name: &str) -> std::collections::BTreeMap<u64, serde_json::Value> {
        let lua = Lua::new();
        lua.load(source).exec().expect("the table runs");
        numbered(&lua, name).into_iter().collect()
    }

    const QUESTS: &str = r#"QuestInfoList = {
        [1001] = { Title = "Job Change to Acolyte", Description = { "Visit ^0000FFFather Rubalkabara^000000." }, Summary = "Talk to Father Rubalkabara",
                   NpcSpr = "4_M_ORIENT02", NpcNavi = "prt_fild03", NpcPosX = 365, NpcPosY = 255 },
        [5849] = { Title = "[Standby] Megaliths", IconName = "ico_dq.bmp", Description = { "Come back tomorrow." }, Summary = "Reset at 04:00", CoolTimeQuest = 1,
                   RewardEXP = "1120000", RewardJEXP = "650000", RewardItemList = { { ItemID = 25271, ItemNum = 3 } } },
        [9999] = { Title = "Nothing more", Description = {}, Summary = "" },
    }"#;

    #[test]
    fn a_quest_is_given_with_what_it_has() {
        let quests = table(QUESTS, "QuestInfoList");
        assert_eq!(quests.keys().copied().collect::<Vec<_>>(), [1001, 5849, 9999]);
        assert_eq!(
            serde_json::Value::from(ongoing(&quests[&1001])),
            serde_json::json!({ "title": "Job Change to Acolyte", "summary": "Talk to Father Rubalkabara", "lines": ["Visit ^0000FFFather Rubalkabara^000000."], "npc": ["4_M_ORIENT02", "prt_fild03", 365, 255] })
        );
        let waiting = ongoing(&quests[&5849]);
        assert_eq!(waiting["waits"], true);
        assert_eq!(waiting["icon"], "ico_dq.bmp");
        assert_eq!((&waiting["base"], &waiting["job"]), (&serde_json::json!("1120000"), &serde_json::json!("650000")));
        assert_eq!(waiting["rewards"], serde_json::json!([[25271, 3]]));
        assert!(waiting.get("npc").is_none());
        assert_eq!(serde_json::Value::from(ongoing(&quests[&9999])), serde_json::json!({ "title": "Nothing more" }));
    }

    #[test]
    fn the_database_names_its_quests() {
        assert_eq!(quest_numbers("Body:\n  - Id: 1001\n    Title: A\n#  - Id: 7\n  - Id: 5849\n    Targets:\n      - Mob: PORING\n"), [1001, 5849]);
    }

    #[test]
    fn what_is_recommended_has_its_three_texts() {
        let source = r#"RecommendedQuestInfoList = { [3] = { Title = "Festival", IconName = "ico_ev.bmp", Summary = "See you", BgName = "reco_03.bmp",
            QuestInfo1 = { "We invite you." }, QuestInfo3 = { "Not a quest." } } }"#;
        assert_eq!(
            serde_json::Value::from(recommended(&table(source, "RecommendedQuestInfoList")[&3])),
            serde_json::json!({ "title": "Festival", "summary": "See you", "icon": "ico_ev.bmp", "picture": "reco_03.bmp", "info": [["We invite you."], [], ["Not a quest."]] })
        );
    }

    #[test]
    fn the_filters_have_their_words_and_icons() {
        let source = r#"QuestFilterTable = { [0] = { Tooltip = "All Quests" }, [4] = { Tooltip = "Daily Quests", QuestIcons = { "ico_dq.bmp" } }, [1] = { Tooltip = "Other Quests" } }"#;
        let filters = table(source, "QuestFilterTable");
        assert_eq!(filter(&filters[&0]), serde_json::json!({ "name": "All Quests", "icons": [] }));
        assert_eq!(filter(&filters[&4]), serde_json::json!({ "name": "Daily Quests", "icons": ["ico_dq.bmp"] }));
        assert_eq!(filters.len(), 3);
    }
}
