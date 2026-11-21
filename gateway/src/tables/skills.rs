//! The skills: where they stand in the skill window, what they are called, what the client says of them.

use super::*;

/// Where each job's skills stand in the skill window (the client's `skilltreeview.lub`), and which job each job
/// grew out of (`jobinheritlist.lub`): `{"<job>": {"from": <job or null>, "slots": {"<place>": <skill>}}}`. A
/// place counts the window's cells row by row, seven to a row.
pub(super) fn skill_tree(read: &Sources) -> serde_json::Map<String, serde_json::Value> {
    let lua = read.run(&["skillinfoz\\jobinheritlist", "skillinfoz\\skillid", "skillinfoz\\skilltreeview"]);
    let mut tree = serde_json::Map::new();
    let Ok(views) = lua.globals().get::<Table>("SKILL_TREEVIEW_FOR_JOB") else { return tree };
    // Which job each grew out of, as the client reads it (`inherits_of`): the high form of a third job, which
    // has no page of its own, gives way to the plain one.
    let inherits = inherits_of(&lua);
    for (job, slots) in views.pairs::<i64, Table>().flatten() {
        let slots: serde_json::Map<String, serde_json::Value> =
            slots.pairs::<i64, i64>().flatten().map(|(place, skill)| (place.to_string(), skill.into())).collect();
        let from = inherits.get(&job.to_string()).and_then(serde_json::Value::as_i64);
        tree.insert(job.to_string(), serde_json::json!({ "from": from, "slots": slots }));
    }
    tree
}

/// Which job each job grew out of, for every job the client's `jobinheritlist.lub` names: `{"<job>": <job>}`.
/// As the client's `CSession::InheritedJob` reads it: the job of `JOB_INHERIT_LIST`, and in its place the one
/// `JOB_INHERIT_LIST2` gives for that, where it gives one.
pub(super) fn job_inherits(read: &Sources) -> serde_json::Map<String, serde_json::Value> {
    inherits_of(&read.run(&["skillinfoz\\jobinheritlist"]))
}

/// The same, of a state `jobinheritlist.lub` has run in.
fn inherits_of(lua: &Lua) -> serde_json::Map<String, serde_json::Value> {
    let mut inherits = serde_json::Map::new();
    let Ok(list) = lua.globals().get::<Table>("JOB_INHERIT_LIST") else { return inherits };
    let instead = lua.globals().get::<Table>("JOB_INHERIT_LIST2").ok();
    // The table's numbers are whole, but not all of them are kept as whole numbers (`whole`).
    let replaced: std::collections::HashMap<i64, i64> = instead
        .map(|instead| instead.pairs::<Value, Value>().flatten().filter_map(|(job, from)| Some((whole(&job)?, whole(&from)?))).collect())
        .unwrap_or_default();
    for (job, from) in list.pairs::<Value, Value>().flatten() {
        let (Some(job), Some(from)) = (whole(&job), whole(&from)) else { continue };
        inherits.insert(job.to_string(), (*replaced.get(&from).unwrap_or(&from)).into());
    }
    inherits
}

/// What the client says of each skill when it is looked at (its `skilldescript.lub`): the lines, by skill ID.
pub fn skill_texts(read: &Sources) -> std::collections::HashMap<u32, Vec<String>> {
    let lua = read.run(&["skillinfoz\\skillid", "skillinfoz\\skilldescript"]);
    let Ok(table) = lua.globals().get::<Table>("SKILL_DESCRIPT") else { return Default::default() };
    table
        .pairs::<i64, Table>()
        .flatten()
        .filter_map(|(id, described)| Some((u32::try_from(id).ok()?, lines(&described))))
        .collect()
}

/// The skills, by ID: `{"<id>": {"resource": "NV_BASIC", "name": "Basic Skill", "max": 9}}`. The resource names a
/// skill's pictures. One that warns of its area while it is cast has `"scale": [[11, 11], …]` too (`scale`).
pub(super) fn skills(read: &Sources) -> serde_json::Map<String, serde_json::Value> {
    // The skill table names the jobs it speaks of through `JOBID`: any number will do for those.
    let lua = read.run(&[]);
    if let Err(error) = lua.load("local next = 0; JOBID = setmetatable({}, { __index = function() next = next + 1; return next end })").exec() {
        eprintln!("tables: {error}");
    }
    read.run_in(&lua, &["skillinfoz\\skillid", "skillinfoz\\skillinfolist"]);
    let mut skills = serde_json::Map::new();
    let Ok(table) = lua.globals().get::<Table>("SKILL_INFO_LIST") else { return skills };
    for (id, skill) in table.pairs::<i64, Table>().flatten() {
        let text = |value: mlua::Result<Value>| value.ok().as_ref().and_then(text_only).unwrap_or_default();
        let resource = text(skill.get::<Value>(1));
        if resource.is_empty() {
            continue;
        }
        let max = skill.get::<i64>("MaxLv").unwrap_or(1);
        let mut about = serde_json::json!({ "resource": resource, "name": text(skill.get::<Value>("SkillName")), "max": max });
        let scale = scale(&skill);
        if !scale.is_empty() {
            about["scale"] = scale.into();
        }
        skills.insert(id.to_string(), about);
    }
    skills
}

/// The area a skill warns of while it is cast, level by level, in cells across and down: the skill's
/// `SkillScale` (`{ [1] = { x = 11, y = 11 }, … }`), which the client's `GetSkillScale` of `skillinfo_f.lub`
/// reads for ZC_SKILL_SCALE (that file is compiled; the function names `SKILL_INFO_LIST`, `SkillScale`, `x`
/// and `y` and nothing else). Empty for a skill that has none.
pub(super) fn scale(skill: &Table) -> Vec<[i64; 2]> {
    let Ok(levels) = skill.get::<Table>("SkillScale") else { return Vec::new() };
    levels.sequence_values::<Table>().flatten().map(|level| [level.get("x").unwrap_or(0), level.get("y").unwrap_or(0)]).collect()
}
