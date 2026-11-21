//! The action a monster is shown in as it uses a skill (`data\MonsterSkillInfo.xml`, read by
//! `CSession::LoadXmlTables` and asked by `CSession::GetMonsterSkillInfo`, rodecomp
//! `Source/Contents/Monster/MonsterXmlActions.cpp`; the rule is the browser client's `crate::monster_skills`).

use super::*;

/// A name as a client that is not Korea's asks the file for it (`DecodeMessageRot`, rodecomp
/// `Source/Contents/Tables/MessageTable.cpp`): every Latin letter for the one as far from the other end of
/// the alphabet. Turning a name twice gives it back, so this also reads a name of such a client's file.
fn turned(name: &str) -> String {
    name.chars()
        .map(|letter| match letter {
            'a'..='z' => (b'a' + b'z' - letter as u8) as char,
            'A'..='Z' => (b'A' + b'Z' - letter as u8) as char,
            other => other,
        })
        .collect()
}

/// `{"<monster>": {"<skill>": [action, loop]}}` out of the file's text: under `Monster_Action_Table` an
/// element for each monster, named as its sprite is in small letters (the client asks for the job's name made
/// small, and `FindChild` tells capitals apart: a name with a capital is never found, and is given here as it
/// is), in it one for each skill (`_28` for skill 28) with its `action` and its `loopType`. A skill that lacks
/// either is left out (the client says nothing of it but to its own makers), and of two elements of one name
/// the first counts, as `FindChild` finds it. What the file says between `<!--` and `-->` or in `<?…?>` is
/// passed by.
///
/// A client that is not Korea's (`korea`, `jobs::korea`) asks for every name turned letter for letter
/// (`turned`): it finds its table only in a file written so, and nothing in Korea's.
pub fn actions(text: &str, korea: bool) -> serde_json::Value {
    let mut found: BTreeMap<String, BTreeMap<u32, [Option<i64>; 2]>> = BTreeMap::new();
    let mut seen: Vec<String> = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once('<') {
        if let Some(comment) = after.strip_prefix("!--") {
            rest = comment.split_once("-->").map_or("", |(_, after)| after);
            continue;
        }
        let Some((tag, after)) = after.split_once('>') else { break };
        rest = after;
        if tag.starts_with('?') || tag.starts_with('!') {
            continue;
        }
        if tag.starts_with('/') {
            // The text an element of the fourth depth closes over is its figure.
            if let [table, monster, skill, field] = &open[..] {
                let place = match field.as_str() {
                    "action" => Some(0),
                    "loopType" => Some(1),
                    _ => None,
                };
                let skill = skill.strip_prefix('_').and_then(|skill| skill.parse::<u32>().ok());
                let first = table == "Monster_Action_Table" && !seen.iter().any(|name| name == monster);
                if let (true, Some(place), Some(skill)) = (first, place, skill) {
                    let figures = found.entry(monster.clone()).or_default().entry(skill).or_default();
                    figures[place] = figures[place].or(Some(before.trim().parse().unwrap_or(0)));
                }
            }
            // A monster's element closed: one more of its name is not looked at.
            if let [_, monster] = &open[..] {
                seen.push(monster.clone());
            }
            open.pop();
        } else if !tag.ends_with('/') {
            let name = tag.split_whitespace().next().unwrap_or_default();
            open.push(if korea { name.to_owned() } else { turned(name) });
        }
    }
    found
        .into_iter()
        .map(|(monster, skills)| {
            let skills: serde_json::Map<String, serde_json::Value> = skills
                .into_iter()
                .filter_map(|(skill, figures)| Some((skill.to_string(), serde_json::json!([figures[0]?, figures[1]?]))))
                .collect();
            (monster, skills.into())
        })
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into()
}

/// The table out of the archives; an empty one, with a note, when the file is not there (the client then
/// shows every monster's skill as its attack).
pub fn monster_skills(grf: &Grf) -> serde_json::Value {
    match grf.read("data\\monsterskillinfo.xml") {
        Ok(bytes) => actions(&korean(&bytes), jobs::korea(grf)),
        Err(error) => {
            eprintln!("tables: monsterskillinfo.xml: {error}");
            serde_json::json!({})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_monster_has_an_action_and_a_loop_for_a_skill() {
        let table = actions(
            "<?xml version=\"1.0\" encoding=\"euc-kr\"?>\n<Monster_Action_Table>\n<!-- <poring><_1><action>9</action></_1></poring> -->\n\
             <ork_hero><_21><action>6</action><loopType>0</loopType></_21></ork_hero>\n\
             <choco>\n<_28>\n<action> 7 </action>\n<loopType>0</loopType>\n</_28>\n<_6><action>8</action><loopType>1</loopType></_6>\n\
             <_9><action>2</action></_9></choco>\n\
             <choco><_28><action>1</action><loopType>1</loopType></_28><_152><action>6</action><loopType>0</loopType></_152></choco>\n\
             </Monster_Action_Table>",
            true,
        );
        assert_eq!(table, serde_json::json!({ "ork_hero": { "21": [6, 0] }, "choco": { "28": [7, 0], "6": [8, 1] } }));
    }

    #[test]
    fn another_table_of_the_file_is_not_this_one() {
        assert_eq!(actions("<Other><poring><_1><action>2</action><loopType>0</loopType></_1></poring></Other>", true), serde_json::json!({}));
    }

    #[test]
    fn a_client_of_another_service_reads_only_a_file_turned_for_it() {
        let plain = "<Monster_Action_Table><poring><_1><action>2</action><loopType>0</loopType></_1></poring></Monster_Action_Table>";
        assert_eq!(actions(plain, false), serde_json::json!({}));
        let theirs = "<Nlmhgvi_Zxgrlm_Gzyov><klirmt><_1><zxgrlm>2</zxgrlm><ollkGbkv>0</ollkGbkv></_1></klirmt></Nlmhgvi_Zxgrlm_Gzyov>";
        assert_eq!(actions(theirs, false), serde_json::json!({ "poring": { "1": [2, 0] } }));
        assert_eq!(turned(&turned("Monster_Action_Table")), "Monster_Action_Table");
    }
}
