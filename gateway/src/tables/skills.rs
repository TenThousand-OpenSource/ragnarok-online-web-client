//! The skills: where they stand in the skill window, what they are called, what the client says of them.

use super::*;

/// Where each job's skills stand in the skill window (the client's `skilltreeview.lub`), and which job each job
/// grew out of (`jobinheritlist.lub`): `{"<job>": {"from": <job or null>, "slots": {"<place>": <skill>}}}`. A
/// place counts the window's cells row by row, seven to a row.
pub(super) fn skill_tree(grf: &Grf) -> serde_json::Map<String, serde_json::Value> {
    let lua = run(grf, &["skillinfoz\\jobinheritlist", "skillinfoz\\skillid", "skillinfoz\\skilltreeview"]);
    let mut tree = serde_json::Map::new();
    let Ok(views) = lua.globals().get::<Table>("SKILL_TREEVIEW_FOR_JOB") else { return tree };
    // Which job each grew out of, as the client reads it (`job_inherits`): the high form of a third job, which
    // has no page of its own, gives way to the plain one.
    let inherits = job_inherits(grf);
    for (job, slots) in views.pairs::<i64, Table>().flatten() {
        let slots: serde_json::Map<String, serde_json::Value> = slots.pairs::<i64, i64>().flatten().map(|(place, skill)| (place.to_string(), skill.into())).collect();
        let from = inherits.get(&job.to_string()).and_then(serde_json::Value::as_i64);
        tree.insert(job.to_string(), serde_json::json!({ "from": from, "slots": slots }));
    }
    tree
}

/// Which job each job grew out of, for every job the client's `jobinheritlist.lub` names: `{"<job>": <job>}`.
/// As the client's `CSession::InheritedJob` reads it: the job of `JOB_INHERIT_LIST`, and in its place the one
/// `JOB_INHERIT_LIST2` gives for that, where it gives one.
pub(super) fn job_inherits(grf: &Grf) -> serde_json::Map<String, serde_json::Value> {
    let lua = run(grf, &["skillinfoz\\jobinheritlist"]);
    let mut inherits = serde_json::Map::new();
    let Ok(list) = lua.globals().get::<Table>("JOB_INHERIT_LIST") else { return inherits };
    let instead = lua.globals().get::<Table>("JOB_INHERIT_LIST2").ok();
    // The table's numbers are whole, but not all of them are kept as whole numbers.
    let whole = |value: &Value| match value {
        Value::Integer(number) => Some(*number),
        Value::Number(number) => Some(*number as i64),
        _ => None,
    };
    let replaced: std::collections::HashMap<i64, i64> = instead.map(|instead| instead.pairs::<Value, Value>().flatten().filter_map(|(job, from)| Some((whole(&job)?, whole(&from)?))).collect()).unwrap_or_default();
    for (job, from) in list.pairs::<Value, Value>().flatten() {
        let (Some(job), Some(from)) = (whole(&job), whole(&from)) else { continue };
        inherits.insert(job.to_string(), (*replaced.get(&from).unwrap_or(&from)).into());
    }
    inherits
}

/// What the client says of each skill when it is looked at (its `skilldescript.lub`): the lines, by skill ID.
pub fn skill_texts(grf: &Grf) -> std::collections::HashMap<u32, Vec<String>> {
    let lua = run(grf, &["skillinfoz\\skillid", "skillinfoz\\skilldescript"]);
    let Ok(table) = lua.globals().get::<Table>("SKILL_DESCRIPT") else { return Default::default() };
    table
        .pairs::<i64, Table>()
        .flatten()
        .filter_map(|(id, lines)| Some((u32::try_from(id).ok()?, lines.sequence_values::<mlua::String>().flatten().map(|line| korean(&line.as_bytes())).collect())))
        .collect()
}

/// The skills, by ID: `{"<id>": {"resource": "NV_BASIC", "name": "Basic Skill", "max": 9}}`. The resource names a
/// skill's pictures.
pub(super) fn skills(grf: &Grf) -> serde_json::Map<String, serde_json::Value> {
    // The skill table names the jobs it speaks of through `JOBID`: any number will do for those.
    let lua = run(grf, &[]);
    if let Err(error) = lua.load("local next = 0; JOBID = setmetatable({}, { __index = function() next = next + 1; return next end })").exec() {
        eprintln!("tables: {error}");
    }
    run_in(&lua, grf, &["skillinfoz\\skillid", "skillinfoz\\skillinfolist"]);
    let mut skills = serde_json::Map::new();
    let Ok(table) = lua.globals().get::<Table>("SKILL_INFO_LIST") else { return skills };
    for (id, skill) in table.pairs::<i64, Table>().flatten() {
        let text = |value: mlua::Result<Value>| match value {
            Ok(Value::String(text)) => korean(&text.as_bytes()),
            _ => String::new(),
        };
        let resource = text(skill.get::<Value>(1));
        if resource.is_empty() {
            continue;
        }
        let max = skill.get::<i64>("MaxLv").unwrap_or(1);
        skills.insert(id.to_string(), serde_json::json!({ "resource": resource, "name": text(skill.get::<Value>("SkillName")), "max": max }));
    }
    skills
}
