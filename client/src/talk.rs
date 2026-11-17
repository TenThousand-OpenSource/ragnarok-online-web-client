//! What pets say: the client's `PetTalkTable.xml`.
//!
//! The file is a tree three deep: the monster, how hungry it is, and the moment, each moment's lines side by
//! side: `<poring><hungry><feeding> Not bad, what else ya got? </feeding>…</hungry>…</poring>`.

/// What lies between a tag's opening and its closing, the first time it comes in `text`.
fn inside<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let from = text.find(&open)? + open.len();
    let to = text[from..].find(&format!("</{tag}>"))?;
    Some(&text[from..from + to])
}

/// The lines a monster may say at a moment, in a state of hunger (`net::pets::hunger_tag`, `moment_tag`); none
/// when the table has nothing for it.
pub fn lines<'a>(table: &'a str, monster: &str, hunger: &str, moment: &str) -> Vec<&'a str> {
    let Some(mut rest) = inside(table, monster).and_then(|monster| inside(monster, hunger)) else { return Vec::new() };
    let (open, close) = (format!("<{moment}>"), format!("</{moment}>"));
    let mut found = Vec::new();
    while let Some(from) = rest.find(&open) {
        let after = &rest[from + open.len()..];
        let Some(to) = after.find(&close) else { break };
        let line = after[..to].trim();
        if !line.is_empty() {
            found.push(line);
        }
        rest = &after[to + close.len()..];
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_lines() {
        let table = "<monster_talk_table>\n<poring>\n<hungry>\n<feeding>\tNot bad.\t</feeding>\n<feeding>\tMore?\t</feeding>\n<stand>\tHm.\t</stand>\n</hungry>\n<full>\n<feeding>\tEnough.\t</feeding>\n</full>\n</poring>\n<drops>\n<hungry>\n<feeding>\tYum.\t</feeding>\n</hungry>\n</drops>\n</monster_talk_table>";
        assert_eq!(lines(table, "poring", "hungry", "feeding"), ["Not bad.", "More?"]);
        assert_eq!(lines(table, "poring", "full", "feeding"), ["Enough."]);
        assert_eq!(lines(table, "drops", "hungry", "feeding"), ["Yum."]);
        assert!(lines(table, "poring", "full", "stand").is_empty());
        assert!(lines(table, "lunatic", "hungry", "feeding").is_empty());
    }
}
