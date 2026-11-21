//! What a skill needs by the rules the server plays: the SP of each level, and for each job the skills (and
//! the job level) asked for before it can be learned. Read from the server's database, not from the client's
//! `skillinfolist.lub`, whose `SpAmount` and `_NeedSkillList` are Renewal's (the owner's decision of
//! 2026-11-21, `docs/DEBTS.md`): `skill_db.yml` and `skill_tree.yml` of the folder the item database is in
//! (`db/pre-re` for the classic game, `db/re` for Renewal), and the jobs' numbers from the server's own
//! `e_job` (`src/common/mmo.hpp`), by which its `SkillTreeDatabase::parseBodyNode` knows a job's name.

use super::*;

use std::collections::HashMap;

/// A line of a database file as its depth (the blanks before it, a list's dash among them), its key and its
/// value; none for an empty line, a comment, or a line that is no `key: value`.
fn field(line: &str) -> Option<(usize, bool, &str, &str)> {
    let line = line.split(" #").next().unwrap_or(line);
    let inner = line.trim_start();
    if inner.is_empty() || inner.starts_with('#') {
        return None;
    }
    let depth = line.len() - inner.len();
    let (dashed, inner) = match inner.strip_prefix("- ") {
        Some(inner) => (true, inner),
        None => (false, inner),
    };
    let (key, value) = inner.split_once(':')?;
    Some((depth, dashed, key.trim(), value.trim()))
}

/// Out of `skill_db.yml`: each skill's number by its name, and the SP of each level of those that take any
/// (`Requires.SpCost`: one number for every level, written as a list of one, or a list of `Level`/`Amount`,
/// written level by level with 0 for a level not named).
pub(super) fn costs(text: &str) -> (HashMap<String, u32>, BTreeMap<u32, Vec<u32>>) {
    let (mut names, mut costs) = (HashMap::new(), BTreeMap::<u32, Vec<u32>>::new());
    let mut id = None;
    // Inside the skill's `SpCost` list, and the level its next `Amount` is of.
    let (mut listing, mut level) = (false, 0usize);
    for (depth, dashed, key, value) in text.lines().filter_map(field) {
        match (depth, dashed, key) {
            (2, true, "Id") => (id, listing) = (value.parse::<u32>().ok(), false),
            (4, false, "Name") => drop(id.map(|id| names.insert(value.to_owned(), id))),
            (6, false, "SpCost") => {
                listing = value.is_empty();
                if let (Some(id), Ok(amount)) = (id, value.parse::<u32>()) {
                    costs.insert(id, vec![amount]);
                }
            }
            (8, true, "Level") if listing => level = value.parse().unwrap_or(0),
            (10, false, "Amount") if listing => {
                if let (Some(id), Ok(amount), 1..=200) = (id, value.parse::<u32>(), level) {
                    let levels = costs.entry(id).or_default();
                    levels.resize(levels.len().max(level), 0);
                    levels[level - 1] = amount;
                }
            }
            (depth, ..) if depth <= 6 => listing = false,
            _ => {}
        }
    }
    costs.retain(|_, levels| levels.iter().any(|amount| *amount != 0));
    (names, costs)
}

/// The jobs' numbers out of the server's `enum e_job` (`src/common/mmo.hpp`), by the name `skill_tree.yml`
/// calls each by, in small letters: `JOB_KNIGHT2` is `knight2`. A name counts one on from the name before it
/// unless it is given a number. `Supernovice`, which the database still says, is the server's
/// `Job_SuperNovice` for `JOB_SUPER_NOVICE` (`src/map/script_constants.hpp`).
pub(super) fn job_numbers(header: &str) -> HashMap<String, u32> {
    let mut numbers = HashMap::new();
    let Some((_, list)) = header.split_once("enum e_job") else { return numbers };
    let mut next = 0;
    for line in list.split_once('}').map_or(list, |(list, _)| list).lines() {
        let line = line.split("//").next().unwrap_or_default().trim().trim_end_matches(',');
        let (name, given) = line.split_once('=').map_or((line, None), |(name, number)| (name.trim(), number.trim().parse::<u32>().ok()));
        let Some(name) = name.strip_prefix("JOB_") else { continue };
        next = given.unwrap_or(next);
        numbers.insert(name.to_lowercase(), next);
        next += 1;
    }
    if let Some(number) = numbers.get("super_novice").copied() {
        numbers.insert("supernovice".to_owned(), number);
    }
    numbers
}

/// One skill of a job's tree as the database writes it.
#[derive(Clone, Default)]
struct Entry {
    skill: String,
    /// `MaxLevel: 0` takes the skill out of the tree.
    removed: bool,
    /// `Exclude`: not handed to a job that inherits this one's tree.
    kept: bool,
    job_level: u32,
    needs: Vec<(String, u32)>,
}

/// `skill_tree.yml` as it is written: each job's name, the jobs whose trees it inherits, and its own entries.
fn written(text: &str) -> Vec<(String, Vec<String>, Vec<Entry>)> {
    let mut jobs: Vec<(String, Vec<String>, Vec<Entry>)> = Vec::new();
    let mut inheriting = false;
    for (depth, dashed, key, value) in text.lines().filter_map(field) {
        if (depth, dashed, key) == (2, true, "Job") {
            jobs.push((value.to_owned(), Vec::new(), Vec::new()));
            continue;
        }
        let Some((_, inherits, entries)) = jobs.last_mut() else { continue };
        match (depth, dashed, key) {
            (4, false, _) => inheriting = key == "Inherit",
            (6, false, job) if inheriting && value == "true" => inherits.push(job.to_owned()),
            (6, true, "Name") => entries.push(Entry { skill: value.to_owned(), ..Entry::default() }),
            _ => {}
        }
        let Some(entry) = entries.last_mut() else { continue };
        match (depth, dashed, key) {
            (8, false, "MaxLevel") => entry.removed = value == "0",
            (8, false, "Exclude") => entry.kept = value == "true",
            (8, false, "JobLevel") => entry.job_level = value.parse().unwrap_or(0),
            (10, true, "Name") => entry.needs.push((value.to_owned(), 0)),
            (12, false, "Level") => drop(entry.needs.last_mut().map(|need| need.1 = value.parse().unwrap_or(0))),
            _ => {}
        }
    }
    jobs
}

/// By job, the skills of its tree that ask for something: `{"<job>": {"<skill>": {"job": <job level>,
/// "needs": [[<skill>, <level>], …]}}}`. A job's tree is its own entries over those of the jobs it inherits
/// (theirs alone, not what they inherit in turn, and none marked `Exclude`), as the server's
/// `SkillTreeDatabase::parseBodyNode` puts it together; a need of level 0 is none. A job or a skill whose
/// name is not known is left out.
pub(super) fn tree(text: &str, skills: &HashMap<String, u32>, jobs: &HashMap<String, u32>) -> serde_json::Map<String, serde_json::Value> {
    let written = written(text);
    let mut trees = BTreeMap::new();
    for (name, inherits, own) in &written {
        let Some(job) = jobs.get(&name.to_lowercase()) else { continue };
        let inherited = inherits
            .iter()
            .filter_map(|from| written.iter().find(|(name, ..)| name == from))
            .flat_map(|(_, _, entries)| entries.iter().filter(|entry| !entry.kept));
        let mut asked: BTreeMap<u32, serde_json::Value> = BTreeMap::new();
        for entry in inherited.chain(own) {
            let Some(skill) = skills.get(&entry.skill) else { continue };
            let needs: Vec<[u32; 2]> =
                entry.needs.iter().filter(|(_, level)| *level > 0).filter_map(|(need, level)| Some([*skills.get(need)?, *level])).collect();
            if entry.removed || (needs.is_empty() && entry.job_level == 0) {
                asked.remove(skill);
            } else {
                asked.insert(*skill, serde_json::json!({ "job": entry.job_level, "needs": needs }));
            }
        }
        trees.insert(*job, serde_json::Value::Object(asked.into_iter().map(|(skill, about)| (skill.to_string(), about)).collect()));
    }
    trees.into_iter().map(|(job, tree)| (job.to_string(), tree)).collect()
}

/// The table for the browser: `{"sp": {"<skill>": [<SP at level 1>, …]}, "jobs": {…}}` (`costs`, `tree`). A
/// file that is not there leaves its part empty, with a note: the browser then says the server's own figure
/// for the SP and shows no requirements.
pub fn skill_needs(database: &std::path::Path) -> serde_json::Value {
    let read = |file: std::path::PathBuf| {
        std::fs::read_to_string(&file).unwrap_or_else(|error| {
            eprintln!("tables: {}: {error}", file.display());
            String::new()
        })
    };
    let (names, costs) = costs(&read(database.join("skill_db.yml")));
    let jobs = job_numbers(&read(database.join("../../src/common/mmo.hpp")));
    let sp: serde_json::Map<String, serde_json::Value> = costs.into_iter().map(|(skill, levels)| (skill.to_string(), levels.into())).collect();
    serde_json::json!({ "sp": sp, "jobs": tree(&read(database.join("skill_tree.yml")), &names, &jobs) })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKILLS: &str = "\
Body:
  - Id: 5
    Name: SM_BASH
    MaxLevel: 10
    Requires:
      SpCost:
        - Level: 1
          Amount: 8
        - Level: 2
          Amount: 8
        - Level: 6
          Amount: 15
  - Id: 57
    Name: KN_BRANDISHSPEAR   # a note
    CastTime:
      - Level: 1
        Time: 700
    Requires:
      SpCost: 12
      Weapon:
        1hSpear: true
  - Id: 55
    Name: KN_SPEARMASTERY
    Unit:
      Id: Dummyskill
  - Id: 2
    Name: SM_SWORD
#  - Id: 9999
#    Name: OFF
";

    #[test]
    fn a_skill_costs_one_figure_or_one_for_each_level() {
        let (names, costs) = costs(SKILLS);
        assert_eq!(costs.get(&5), Some(&vec![8, 8, 0, 0, 0, 15]));
        assert_eq!(costs.get(&57), Some(&vec![12]));
        // A passive skill costs nothing and is not in the table; a line commented out names no skill.
        assert_eq!((costs.get(&55), costs.len()), (None, 2));
        assert_eq!((names.get("KN_BRANDISHSPEAR"), names.get("SM_SWORD"), names.get("OFF")), (Some(&57), Some(&2), None));
    }

    #[test]
    fn a_job_is_numbered_as_the_server_counts_it() {
        let header = "enum e_other { A };\nenum e_job {\n\tJOB_NOVICE,\n\tJOB_SWORDMAN, // the first\n\tJOB_SUPER_NOVICE = 23,\n\tJOB_GUNSLINGER,\n\
                      \tJOB_MAX_BASIC,\n\n\tJOB_NOVICE_HIGH = 4001,\n\tJOB_SWORDMAN_HIGH,\n};\nenum e_more { JOB_NOT = 7, };";
        let jobs = job_numbers(header);
        assert_eq!((jobs.get("novice"), jobs.get("swordman"), jobs.get("gunslinger")), (Some(&0), Some(&1), Some(&24)));
        assert_eq!((jobs.get("supernovice"), jobs.get("super_novice"), jobs.get("swordman_high")), (Some(&23), Some(&23), Some(&4002)));
        assert_eq!(jobs.get("not"), None);
        assert!(job_numbers("no enum here").is_empty());
    }

    #[test]
    fn a_tree_is_the_jobs_own_over_what_it_inherits() {
        let text = "\
Body:
  - Job: Novice
    Tree:
      - Name: NV_BASIC
        MaxLevel: 9
      - Name: NV_TRICKDEAD
        MaxLevel: 1
        Exclude: true
        JobLevel: 7
  - Job: Swordman
    Inherit:
      Novice: true
    Tree:
      - Name: SM_SWORD
        MaxLevel: 10
      - Name: SM_TWOHAND
        MaxLevel: 10
        Requires:
          - Name: SM_SWORD
            Level: 1
  - Job: Knight
    Inherit:
      Novice: true
      Swordman: true
    Tree:
      - Name: KN_BRANDISHSPEAR
        MaxLevel: 10
        JobLevel: 30
        Requires:
          - Name: KN_RIDING
            Level: 1
          - Name: SM_SWORD
            Level: 3
          - Name: NO_SUCH_SKILL
            Level: 1
      - Name: SM_TWOHAND
        MaxLevel: 0
  - Job: Unknown_Job
    Tree:
      - Name: SM_SWORD
        MaxLevel: 1
        JobLevel: 2
";
        let skills: HashMap<String, u32> =
            [("NV_BASIC", 1), ("NV_TRICKDEAD", 143), ("SM_SWORD", 2), ("SM_TWOHAND", 3), ("KN_BRANDISHSPEAR", 57), ("KN_RIDING", 63)]
                .into_iter()
                .map(|(name, id)| (name.to_owned(), id))
                .collect();
        let jobs: HashMap<String, u32> = [("novice", 0), ("swordman", 1), ("knight", 7)].into_iter().map(|(name, id)| (name.to_owned(), id)).collect();
        assert_eq!(
            serde_json::Value::Object(tree(text, &skills, &jobs)),
            serde_json::json!({
                "0": { "143": { "job": 7, "needs": [] } },
                // What the Novice keeps to itself is not the Swordman's.
                "1": { "3": { "job": 0, "needs": [[2, 1]] } },
                // The Knight took Two-Handed Sword Mastery out of its tree; a need nobody knows is left out.
                "7": { "57": { "job": 30, "needs": [[63, 1], [2, 3]] } },
            })
        );
    }

    #[test]
    fn no_database_leaves_the_table_empty() {
        let folder = std::env::temp_dir().join("roweb-no-skill-database");
        assert_eq!(skill_needs(&folder), serde_json::json!({ "sp": {}, "jobs": {} }));
    }
}
