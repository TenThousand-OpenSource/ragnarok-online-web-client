//! The skills: what they are called, their pictures and descriptions, and where they stand in the skill window.

use super::*;

thread_local! {
    /// By job: the job it grew out of, and where its skills stand in the skill window (place to skill).
    static TREE: RefCell<HashMap<u16, (Option<u16>, Vec<(u16, u16)>)>> = RefCell::new(HashMap::new());
    /// What the client says of each skill, as far as it was asked for (`None` once the gateway said it has
    /// nothing).
    static SKILL_TEXTS: RefCell<HashMap<u16, Option<Vec<String>>>> = RefCell::new(HashMap::new());
    /// Skills by ID: the name of their pictures, what they are called, and their highest level.
    static SKILLS: RefCell<HashMap<u16, (String, String, u16)>> = RefCell::new(HashMap::new());
    /// What the translated skills are called in Russian, by ID.
    static SKILL_NAMES_RU: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
}

/// The pages of a job's skill window, as the client lays them out (`skilltreeview.lub`): one for each job it
/// grew through, the first with the Novice's skills among its own. Each page: the place of each skill (counting
/// the cells row by row, seven to a row) and the skill.
pub fn skill_pages(job: u16) -> Vec<Vec<(u16, u16)>> {
    TREE.with_borrow(|tree| {
        // From the job back to the Novice.
        let mut line = Vec::new();
        let mut at = Some(job);
        while let Some(job) = at.filter(|job| !line.contains(job) && line.len() < 8) {
            line.push(job);
            at = tree.get(&job).and_then(|(from, _)| *from).filter(|from| *from != job);
        }
        line.reverse();
        let mut pages: Vec<Vec<(u16, u16)>> = Vec::new();
        for (index, job) in line.iter().enumerate() {
            let slots = tree.get(job).map(|(_, slots)| slots.clone()).unwrap_or_default();
            // The Novice's page is the first job's too.
            if index == 1 && line[0] == 0 {
                pages[0].extend(slots);
            } else {
                pages.push(slots);
            }
        }
        pages
    })
}

/// The client's description of a skill, line by line, once the gateway has told it (see `want_skill`).
pub fn skill_text(id: u16) -> Option<Vec<String>> {
    SKILL_TEXTS.with_borrow(|texts| texts.get(&id).cloned().flatten())
}

/// Asks the gateway for a skill's description, unless it is known; `then` is called when it has come.
pub fn want_skill(id: u16, then: fn()) {
    if SKILL_TEXTS.with_borrow(|texts| texts.contains_key(&id)) {
        return;
    }
    SKILL_TEXTS.with_borrow_mut(|texts| texts.insert(id, None));
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(answer) = fetch_json(&format!("/tables/{}skills/{id}", if russian() { "ru/" } else { "" })).await {
            let lines = Some(field(&answer, &id.to_string())).filter(|lines| !lines.is_undefined()).map(|lines| texts(&lines));
            SKILL_TEXTS.with_borrow_mut(|texts| texts.insert(id, lines));
            then();
        }
    });
}

/// A skill's picture name, what it is called, and its highest level.
pub fn skill(id: u16) -> Option<(String, String, u16)> {
    let (resource, name, most) = SKILLS.with_borrow(|skills| skills.get(&id).cloned())?;
    // In Russian, a skill that has a translation goes by it.
    let name = if russian() { SKILL_NAMES_RU.with_borrow(|names| names.get(&id).cloned()).unwrap_or(name) } else { name };
    Some((resource, name, most))
}

/// Forgets the skills' descriptions: after the language changed they are asked for anew.
pub(super) fn forget() {
    SKILL_TEXTS.with_borrow_mut(HashMap::clear);
}

/// Takes its tables out of what the gateway gave: the skills, and where they stand in each job's skill window.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    SKILLS.set(
        entries::<u16>(&field(tables, "skills"))
            .into_iter()
            .filter_map(|(id, about)| {
                let max = number(&about, "max").unwrap_or(1.0) as u16;
                Some((id, (text_of(&about, "resource")?, text_of(&about, "name")?, max)))
            })
            .collect(),
    );
    TREE.set(
        entries::<u16>(&field(tables, "tree"))
            .into_iter()
            .map(|(job, about)| {
                let from = number(&about, "from").map(|from| from as u16);
                let slots = entries::<u16>(&field(&about, "slots"));
                (job, (from, slots.into_iter().filter_map(|(place, skill)| Some((place, skill.as_f64()? as u16))).collect()))
            })
            .collect(),
    );
}

/// Fetches what the translated skills are called in Russian.
pub(super) async fn read_russian() {
    if let Ok(names) = fetch_json("/tables/ru/skill-names.json").await {
        SKILL_NAMES_RU.set(super::names::<u16>(&names).into_iter().collect());
    }
}
