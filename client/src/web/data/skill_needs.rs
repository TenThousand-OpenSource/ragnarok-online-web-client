//! What a skill needs by the rules the server plays (the gateway's `tables/skill_needs.rs`, out of the
//! server's `skill_db.yml` and `skill_tree.yml`): the SP of each level, and for each job the skills and the
//! job level asked for before it is learned. It stands where the Windows client asks its `skillinfolist.lub`
//! (`SpAmount`, `_NeedSkillList`), whose figures are Renewal's: the owner's decision, `docs/DEBTS.md`.

use super::*;

thread_local! {
    /// By skill: the SP of its levels (one figure for a skill that takes the same at every level).
    static SP: RefCell<HashMap<u16, Vec<u16>>> = RefCell::new(HashMap::new());
    /// By job, by skill: the job level it asks for, and the skills with their levels.
    static ASKED: RefCell<HashMap<u16, HashMap<u16, (u16, Vec<(u16, u16)>)>>> = RefCell::new(HashMap::new());
}

/// The SP a skill takes at a level (`crate::skills::sp_at`); none for a skill the table does not have, as with
/// a gateway that found no database.
pub fn skill_sp(id: u16, level: u16) -> Option<u16> {
    SP.with_borrow(|sp| crate::skills::sp_at(sp.get(&id)?, level))
}

/// What a job's skill asks for before it is learned: the job level, and the skills with their levels. Nothing
/// for a skill that asks for nothing.
pub fn skill_asks(job: u16, id: u16) -> (u16, Vec<(u16, u16)>) {
    ASKED.with_borrow(|asked| asked.get(&job).and_then(|skills| skills.get(&id)).cloned()).unwrap_or_default()
}

/// Reads the table out of what the gateway gave: `{"sp": {"<skill>": [8, 8, 15]}, "jobs": {"<job>":
/// {"<skill>": {"job": 0, "needs": [[<skill>, <level>]]}}}}`.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let table = field(tables, "skill_needs");
    let whole = |value: wasm_bindgen::JsValue| value.as_f64().map(|number| number as u16);
    SP.set(
        entries::<u16>(&field(&table, "sp"))
            .into_iter()
            .map(|(skill, levels)| (skill, list(&levels).into_iter().filter_map(whole).collect()))
            .collect(),
    );
    ASKED.set(
        entries::<u16>(&field(&table, "jobs"))
            .into_iter()
            .map(|(job, skills)| {
                let skills = entries::<u16>(&skills)
                    .into_iter()
                    .map(|(skill, about)| {
                        let needs = list(&field(&about, "needs"))
                            .iter()
                            .filter_map(|need| Some((whole(row(need).get(0))?, whole(row(need).get(1))?)))
                            .collect();
                        (skill, (number(&about, "job").unwrap_or(0.0) as u16, needs))
                    })
                    .collect();
                (job, skills)
            })
            .collect(),
    );
}
