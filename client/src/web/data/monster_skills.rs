//! The action a monster is shown in as it uses a skill (the gateway's `tables/monster_skills.rs`, the
//! client's `MonsterSkillInfo.xml`). The rule is `crate::monster_skills`.

use super::*;

thread_local! {
    /// By a monster's name in the table and a skill: the table's `action` and `loopType`.
    static ACTIONS: RefCell<HashMap<(String, u16), (i32, i32)>> = RefCell::new(HashMap::new());
}

/// What the table has for a job and a skill (`CSession::GetMonsterSkillInfo`): it is asked by the name of the
/// job's sprite (`GetJobName`) in small letters.
pub fn monster_skill(job: u16, skill: u16) -> Option<(i32, i32)> {
    let name = sprite(job)?.to_ascii_lowercase();
    ACTIONS.with_borrow(|actions| actions.get(&(name, skill)).copied())
}

/// Takes the table out of what the gateway gave. An older gateway gives none: then every monster swings.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let monsters = field(tables, "monster_skills");
    let Ok(monsters) = monsters.dyn_into::<js_sys::Object>() else { return };
    let mut actions = HashMap::new();
    for monster in js_sys::Object::entries(&monsters).iter() {
        let monster = row(&monster);
        let Some(name) = monster.get(0).as_string() else { continue };
        for (skill, figures) in entries::<u16>(&monster.get(1)) {
            let figures = row(&figures);
            if let (Some(action), Some(loops)) = (figures.get(0).as_f64(), figures.get(1).as_f64()) {
                actions.insert((name.clone(), skill), (action as i32, loops as i32));
            }
        }
    }
    ACTIONS.set(actions);
}
