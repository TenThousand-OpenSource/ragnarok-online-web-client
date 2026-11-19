//! The quests: what the journal writes of each, what the client recommends, and how it sorts them.

use super::*;

/// What the client's table says of a quest of the server (`QuestInfoList`, as `CRenewQuestMgr::AddOngoingQuest`
/// asks it).
#[derive(Default, Clone)]
pub struct QuestText {
    pub title: String,
    pub summary: String,
    /// The picture of its kind, a file of the interface's `renew_questui`; empty for the plain one.
    pub icon: String,
    /// Its description, line by line.
    pub lines: Vec<String>,
    /// It only waits for a time to pass (`CoolTimeQuest`): it lies under the journal's last tab.
    pub waits: bool,
    /// What it gives: the experience, as the table writes it, and the items with how many of each.
    pub base: String,
    pub job: String,
    pub rewards: Vec<(u32, u32)>,
}

/// Something the client itself recommends (`RecommendedQuestInfoList`): not a quest of the server.
#[derive(Default, Clone)]
pub struct Recommended {
    pub title: String,
    pub summary: String,
    pub icon: String,
    /// The picture behind its texts, a file of the interface's `renew_questui`.
    pub picture: String,
    /// Its three texts, each line by line.
    pub info: [Vec<String>; 3],
}

thread_local! {
    /// The server's quests as the gateway gave them, by the quest's number: one is read out when it is asked
    /// for, since there are thousands and a character has a few.
    static ONGOING: RefCell<wasm_bindgen::JsValue> = const { RefCell::new(wasm_bindgen::JsValue::UNDEFINED) };
    /// What the client recommends, in the order of its numbers.
    static RECOMMENDED: RefCell<Vec<(u32, Recommended)>> = const { RefCell::new(Vec::new()) };
    /// What is said of each icon under the pointer (`QuestIconTooltipTable`).
    static TOOLTIPS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    /// The filters of the journal's lower row (`QuestFilterTable`): each its number, its words and the icons it takes.
    static FILTERS: RefCell<Vec<(u32, String, Vec<String>)>> = const { RefCell::new(Vec::new()) };
}

/// What the client's table says of a quest; none for a quest it does not have, and while the gateway is one
/// older than this table (the journal then shows the quest's number).
pub fn quest_text(quest: u32) -> Option<QuestText> {
    ONGOING.with_borrow(|quests| {
        let about = field(quests, &quest.to_string());
        if !about.is_object() {
            return None;
        }
        let text = |name: &str| text_of(&about, name).unwrap_or_default();
        let rewards = list(&field(&about, "rewards"));
        Some(QuestText {
            title: text("title"),
            summary: text("summary"),
            icon: text("icon"),
            lines: texts(&field(&about, "lines")),
            waits: field(&about, "waits").is_truthy(),
            base: text("base"),
            job: text("job"),
            rewards: rewards
                .into_iter()
                .filter_map(|reward| {
                    let reward: js_sys::Array = reward.dyn_into().ok()?;
                    Some((reward.get(0).as_f64()? as u32, reward.get(1).as_f64()? as u32))
                })
                .collect(),
        })
    })
}

/// What the client recommends, in the order of its numbers.
pub fn recommended_quests() -> Vec<(u32, Recommended)> {
    RECOMMENDED.with_borrow(Clone::clone)
}

/// What the client says of a quest's icon under the pointer (`CRenewQuestMgr::GetIconTooltip`): "QUEST" for
/// one its table has nothing for.
pub fn quest_tooltip(icon: &str) -> String {
    TOOLTIPS.with_borrow(|tooltips| tooltips.get(icon).cloned()).unwrap_or_else(|| "QUEST".to_owned())
}

/// The journal's filters: each its number and the icons it takes (for `crate::quests::in_filter`).
pub fn quest_filters() -> Vec<(u32, Vec<String>)> {
    FILTERS.with_borrow(|filters| filters.iter().map(|(number, _, icons)| (*number, icons.clone())).collect())
}

/// What a filter's button says under the pointer (`CRenewQuestMgr::GetFilterName`).
pub fn quest_filter_name(filter: u32) -> String {
    FILTERS.with_borrow(|filters| filters.iter().find(|(number, ..)| *number == filter).map(|(_, name, _)| name.clone())).unwrap_or_default()
}

/// Takes its tables out of what the gateway gave (`tables/quests.rs`). A gateway older than this table gives
/// none: then no quest has a text, nothing is recommended and the filters take nothing.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let quests = field(tables, "quests");
    ONGOING.set(field(&quests, "ongoing"));
    let mut recommended: Vec<(u32, Recommended)> = entries::<u32>(&field(&quests, "recommended"))
        .into_iter()
        .map(|(number, about)| {
            let text = |name: &str| text_of(&about, name).unwrap_or_default();
            let info = row(&field(&about, "info"));
            (number, Recommended {
                title: text("title"),
                summary: text("summary"),
                icon: text("icon"),
                picture: text("picture"),
                info: [0, 1, 2].map(|part| texts(&info.get(part)))
            })
        })
        .collect();
    recommended.sort_by_key(|(number, _)| *number);
    RECOMMENDED.set(recommended);
    TOOLTIPS.set(names::<String>(&field(&quests, "tooltips")).into_iter().collect());
    let mut filters: Vec<(u32, String, Vec<String>)> =
        entries::<u32>(&field(&quests, "filters"))
            .into_iter()
            .map(|(number, about)| (number, text_of(&about, "name").unwrap_or_default(), texts(&field(&about, "icons"))))
            .collect();
    filters.sort_by_key(|(number, ..)| *number);
    FILTERS.set(filters);
}
