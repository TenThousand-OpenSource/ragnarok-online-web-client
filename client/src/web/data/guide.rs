//! The Adventure Guide: its pages, and who can wear what it recommends.

use super::*;

thread_local! {
    /// The client's Adventure Guide.
    static GUIDE: std::rc::Rc<Guide> = std::rc::Rc::new(Guide::default());
    static GUIDE_READ: RefCell<Option<std::rc::Rc<Guide>>> = const { RefCell::new(None) };
}

/// The client's Adventure Guide (`Lua Files\AdventureGuide`): the title of its first page and of that page's
/// tab, its other tabs with what each is for, and the monsters it recommends.
#[derive(Default)]
pub struct Guide {
    pub intro: String,
    pub menu: String,
    pub tabs: Vec<(String, String)>,
    pub monsters: Vec<GuideMonster>,
    /// The pages of quests, in the guide's order of tabs: episodes, quests, dungeons.
    pub quests: [GuidePage<GuideQuest>; 3],
    /// The pages of items: weapons, armour.
    pub items: [GuidePage<GuideItem>; 2],
    /// The jobs each number of `GuideItem::jobs` stands for, and the novices' jobs.
    pub job_codes: HashMap<u32, Vec<u16>>,
    pub novices: Vec<u16>,
}

impl Guide {
    /// Whether a job can wear an item of the guide (the client's `CJobCodeForItem::IsJobAllowed`): 0xFFFFF is
    /// for everyone, 0xFFFFE for everyone but the novices, any other number for the jobs it stands for.
    pub fn wears(&self, job: u16, item: &GuideItem) -> bool {
        match item.jobs {
            0xfffff => true,
            0xffffe => !self.novices.contains(&job),
            code => self.job_codes.get(&code).is_some_and(|jobs| jobs.contains(&job)),
        }
    }
}

/// A page of the guide with kinds to pick from (the tab's own smaller tabs): each kind's number, name and the
/// levels it is shown to; and the page's entries, in the order of the levels they are for.
pub struct GuidePage<Entry> {
    pub kinds: Vec<(u32, String, u32, u32)>,
    pub list: Vec<Entry>,
}

impl<Entry> Default for GuidePage<Entry> {
    fn default() -> Self {
        Self { kinds: Vec::new(), list: Vec::new() }
    }
}

/// A quest, a part of an episode or a dungeon: a tip of the client's tip box (its number, its title, the first
/// line of what it says), the kind it is listed under, the levels it is for, what must be done before it, and
/// what the guide shows as its reward.
#[derive(Default)]
pub struct GuideQuest {
    pub id: u32,
    pub kind: u32,
    pub min: u32,
    pub max: u32,
    pub title: String,
    pub says: String,
    pub needs: String,
    pub rewards: Vec<String>,
}

/// An item the guide recommends: its number, its kind, the levels it is for, whether it can be refined,
/// enchanted and enchanted by grade, and where it is had from.
#[derive(Default)]
pub struct GuideItem {
    pub id: u32,
    pub kind: u32,
    pub min: u32,
    pub max: u32,
    pub can: [bool; 3],
    pub from: Vec<String>,
    /// Who can wear it: a number of the client's that stands for a set of jobs.
    pub jobs: u32,
}

/// A monster of the guide: the name the guide knows it by, the levels it is recommended for, the items shown as
/// its reward; and, from the server's database, its number (0 when the database lacks it), name, element with
/// the element's level, race, size and whether it is a boss.
#[derive(Default)]
pub struct GuideMonster {
    pub key: String,
    pub id: u32,
    pub name: String,
    pub min: u32,
    pub max: u32,
    pub rewards: Vec<u32>,
    pub element: String,
    pub element_level: u32,
    pub race: String,
    pub size: String,
    pub boss: bool,
}

pub fn guide() -> std::rc::Rc<Guide> {
    GUIDE_READ.with_borrow(|read| read.clone()).unwrap_or_else(|| GUIDE.with(std::rc::Rc::clone))
}

/// Takes the guide out of what the gateway gave.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let guide = field(tables, "guide");
    let number = |object: &wasm_bindgen::JsValue, name: &str| field(object, name).as_f64().map_or(0, |number| number as u32);
    let words = |object: &wasm_bindgen::JsValue, name: &str| field(object, name).as_string().unwrap_or_default();
    let list = |value: wasm_bindgen::JsValue| js_sys::Array::from(&value).iter().collect::<Vec<_>>();
    let texts = |value: wasm_bindgen::JsValue| js_sys::Array::from(&value).iter().filter_map(|line| line.as_string()).collect::<Vec<String>>();
    let kinds = |page: &wasm_bindgen::JsValue| list(field(page, "kinds")).iter().map(|kind| (number(kind, "at"), words(kind, "title"), number(kind, "min"), number(kind, "max"))).collect::<Vec<_>>();
    let quests = |name: &str| {
        let page = field(&guide, name);
        GuidePage {
            kinds: kinds(&page),
            list: list(field(&page, "list"))
                .iter()
                .map(|quest| GuideQuest {
                    id: number(quest, "id"),
                    kind: number(quest, "kind"),
                    min: number(quest, "min"),
                    max: number(quest, "max"),
                    title: words(quest, "title"),
                    says: words(quest, "says"),
                    needs: words(quest, "needs"),
                    rewards: texts(field(quest, "rewards")),
                })
                .collect(),
        }
    };
    let items = |name: &str| {
        let page = field(&guide, name);
        GuidePage {
            kinds: kinds(&page),
            list: list(field(&page, "list"))
                .iter()
                .map(|item| {
                    let can = js_sys::Array::from(&field(item, "can"));
                    GuideItem {
                        id: number(item, "id"),
                        kind: number(item, "kind"),
                        min: number(item, "min"),
                        max: number(item, "max"),
                        can: [0, 1, 2].map(|at| can.get(at).as_bool().unwrap_or(false)),
                        from: texts(field(item, "from")),
                        jobs: number(item, "jobs"),
                    }
                })
                .collect(),
        }
    };
    let job_codes = field(&guide, "job_codes");
    let jobs_of = |value: wasm_bindgen::JsValue| js_sys::Array::from(&value).iter().filter_map(|job| job.as_f64()).map(|job| job as u16).collect::<Vec<u16>>();
    let read = Guide {
        job_codes: entries::<u32>(&field(&job_codes, "codes")).into_iter().map(|(code, jobs)| (code, jobs_of(jobs))).collect(),
        novices: jobs_of(field(&job_codes, "novices")),
        quests: [quests("episode"), quests("quest"), quests("dungeon")],
        items: [items("weapon"), items("armor")],
        intro: words(&guide, "intro"),
        menu: words(&guide, "menu"),
        tabs: js_sys::Array::from(&field(&guide, "tabs")).iter().map(|tab| js_sys::Array::from(&tab)).map(|tab| (tab.get(0).as_string().unwrap_or_default(), tab.get(1).as_string().unwrap_or_default())).collect(),
        monsters: js_sys::Array::from(&field(&guide, "monsters"))
            .iter()
            .map(|monster| GuideMonster {
                key: words(&monster, "key"),
                id: number(&monster, "id"),
                name: words(&monster, "name"),
                min: number(&monster, "min"),
                max: number(&monster, "max"),
                rewards: js_sys::Array::from(&field(&monster, "rewards")).iter().filter_map(|item| item.as_f64()).map(|item| item as u32).collect(),
                element: words(&monster, "element"),
                element_level: number(&monster, "element_level"),
                race: words(&monster, "race"),
                size: words(&monster, "size"),
                boss: words(&monster, "class") == "Boss",
            })
            .collect(),
    };
    GUIDE_READ.set(Some(std::rc::Rc::new(read)));
}
