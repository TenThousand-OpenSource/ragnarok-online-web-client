//! The jobs: what each is called, the sprite files of NPCs and monsters, the name a headgear's sprite files end
//! in, and the weapons' sprite endings and sounds.

use super::*;

type Names = serde_json::Map<String, serde_json::Value>;

/// The Lua state the jobs' files were run in (the pets' file is run in it after them: `pets::pets`), then:
/// what each job is called (`{"<id>": {"name": …}}`), the name each headgear's sprite files end in by its view
/// number, the sprite files of NPCs and monsters, and by the kind of weapon the ending of its sprite files
/// and the sound of its blow.
pub(super) fn jobs(read: &Sources) -> (Lua, Names, Names, Names, Names, Names) {
    let lua = read.run(&["jobidentity", "npcidentity", "jobname", "pcjobnamegender_f", "accessoryid", "accname"]);
    // The translated job names are keyed by `JOBID.<identifier>`, which the files above know under other tables.
    // An identifier none of them has gets a key of its own, so the file still runs to its end.
    let prepared = lua
        .load(
            r#"local unknown = 0
            JOBID = setmetatable({}, { __index = function(_, key)
                local id = (pcJobTbl2 and pcJobTbl2[key]) or (JTtbl and JTtbl[key]) or (jobtbl and jobtbl[key])
                if id == nil then unknown = unknown - 1; id = unknown end
                return id
            end })"#,
        )
        .exec();
    if let Err(error) = prepared {
        eprintln!("tables: {error}");
    }
    read.run_in(&lua, &["pcjobname"]);
    let job_names = names(&lua, "PCJobNameTable");

    let mut jobs = serde_json::Map::new();
    for (id, name) in job_names.into_iter().filter(|(id, _)| *id >= 0) {
        jobs.insert(id.to_string(), serde_json::json!({ "name": name }));
    }

    let headgear: serde_json::Map<String, serde_json::Value> =
        names(&lua, "AccNameTable").into_iter().map(|(id, name)| (id.to_string(), name.into())).collect();
    // The sprite files of NPCs and monsters, by the number the server shows them under.
    let sprites: serde_json::Map<String, serde_json::Value> =
        names(&lua, "JobNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // The ending of a weapon's sprite files by the kind of weapon (`_단검` for daggers), from `weapontable.lub`.
    let weapon_table = read.run(&["weapontable"]);
    let weapons: serde_json::Map<String, serde_json::Value> = names(&weapon_table, "WeaponNameTable")
        .into_iter()
        .filter(|(id, _)| *id > 0)
        .map(|(id, name)| (id.to_string(), name.into()))
        .collect();
    // The sound of a blow by the kind of weapon, from the same file (`WeaponHitWaveNameTable`).
    let weapon_hits: serde_json::Map<String, serde_json::Value> = names(&weapon_table, "WeaponHitWaveNameTable")
        .into_iter()
        .filter(|(id, _)| *id >= 0)
        .map(|(id, name)| (id.to_string(), name.into()))
        .collect();
    (lua, jobs, headgear, sprites, weapons, weapon_hits)
}
