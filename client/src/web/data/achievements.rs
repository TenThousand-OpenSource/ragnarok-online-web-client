//! The achievements and the titles: the client's table of achievements, and what each title is called (the
//! gateway's `tables/achievements.rs`).

use super::*;
use crate::achievements::{Goal, Info};

thread_local! {
    /// The achievements, in the order of their numbers (`System\achievement_list.lub`).
    static LIST: RefCell<std::rc::Rc<Vec<Info>>> = RefCell::new(std::rc::Rc::new(Vec::new()));
    /// What each title is called (`Lua Files\DataInfo\TitleTable`).
    static TITLES: RefCell<HashMap<u32, String>> = RefCell::new(HashMap::new());
}

/// The client's achievements.
pub fn achievements() -> std::rc::Rc<Vec<Info>> {
    LIST.with_borrow(Clone::clone)
}

/// What a title is called (the client's `GetJobTitleString`): nothing for no title, and for one the table
/// does not have.
pub fn title_name(title: u32) -> String {
    TITLES.with_borrow(|titles| titles.get(&title).cloned().unwrap_or_default())
}

/// Takes the tables out of what the gateway gave.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let told = field(tables, "achievements");
    let whole = |object: &wasm_bindgen::JsValue, name: &str| number(object, name).unwrap_or(0.0).max(0.0) as u32;
    let list = list(&field(&told, "list"))
        .into_iter()
        .map(|one| Info {
            id: whole(&one, "id"),
            title: text_of(&one, "title").unwrap_or_default(),
            summary: text_of(&one, "summary").unwrap_or_default(),
            description: text_of(&one, "description").unwrap_or_default(),
            points: whole(&one, "points"),
            item: whole(&one, "item"),
            title_id: whole(&one, "title_id"),
            buff: whole(&one, "buff"),
            goals: list(&field(&one, "goals"))
                .iter()
                .map(|goal| {
                    let goal = row(goal);
                    let whole = |place: u32| goal.get(place).as_f64().unwrap_or(0.0).max(0.0) as u32;
                    Goal { text: goal.get(0).as_string().unwrap_or_default(), count: whole(1), link: whole(2) }
                })
                .collect(),
            counted: flag(&one, "counted"),
            icon: text_of(&one, "icon").unwrap_or_default(),
            group: whole(&one, "group"),
            sub: whole(&one, "sub"),
        })
        .collect();
    LIST.set(std::rc::Rc::new(list));
    TITLES.set(names::<u32>(&field(&told, "titles")).into_iter().collect());
}
