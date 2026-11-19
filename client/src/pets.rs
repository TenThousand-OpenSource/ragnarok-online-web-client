//! Pets as rules: the words of a pet's window for its hunger and its intimacy, what it feels and says at a
//! moment (the numbers its owner's client sends for them), and the lines it may say, out of the client's
//! `PetTalkTable.xml`. No packet is read here: those are in `net/pets.rs`.
//!
//! The table of what pets say is a tree three deep: the monster, how hungry it is, and the moment, each
//! moment's lines side by side: `<poring><hungry><feeding> Not bad, what else ya got? </feeding>…</hungry>…</poring>`.

/// How hungry a pet is, in the words of its window (the client's `UIPetInfoWnd::Paint`).
pub fn hunger_word(hunger: i32) -> &'static str {
    match hunger {
        0..=10 => "Very Hungry",
        11..=25 => "Hungry",
        26..=75 => "Neutral",
        76..=90 => "Satisfied",
        91..=100 => "Stuffed",
        _ => "Unknown",
    }
}

/// How close a pet is to its master, in the words of its window, and how far through that stage it is, in
/// hundredths, for the bar under the word (the client's `UIPetInfoWnd::UpdateFriendlyGauge`).
pub fn intimacy_word(intimacy: i32) -> (&'static str, i32) {
    let (word, low, high) = match intimacy {
        0..=100 => ("Awkward", 0, 100),
        101..=250 => ("Shy", 100, 250),
        251..=750 => ("Neutral", 250, 750),
        751..=900 => ("Cordial", 750, 900),
        901..=1000 => ("Loyal", 900, 1000),
        _ => return ("Unknown", 0),
    };
    (word, (intimacy - low) * 100 / (high - low))
}

/// What a pet's feeling or words are about (the client's `PM_*`): they pick the emotion, and the lines it may say.
pub mod moment {
    pub const FED: i32 = 0;
    /// Its master is badly hurt.
    pub const DANGER: i32 = 2;
    /// Standing about, a while after a trick.
    pub const IDLE: i32 = 4;
    /// Its special trick.
    pub const TRICK_SPECIAL: i32 = 5;
    pub const LEVEL_UP: i32 = 6;
    /// Its first trick; the second and third follow.
    pub const TRICK: i32 = 7;
}

/// How hungry a pet is, as one of five states (the client's `GetPetHungryState`): 0 the hungriest, 4 stuffed.
pub fn hunger_state(hunger: i32) -> usize {
    match hunger {
        91..=100 => 4,
        76..=90 => 3,
        26..=75 => 2,
        11..=25 => 1,
        _ => 0,
    }
}

/// How close a pet is, as one of five states (the client's `GetPetFriendlyState`): 0 awkward, 4 loyal.
pub fn intimacy_state(intimacy: i32) -> usize {
    match intimacy {
        ..=100 => 0,
        101..=250 => 1,
        251..=750 => 2,
        751..=900 => 3,
        _ => 4,
    }
}

/// The emotion a pet shows, by how hungry it is, how close, and the moment (0 to 6); -1 for none. The client's
/// table (`CSession::InitPetActionTable`), with its emotions by their numbers: 32 is the hungry one, 29 the
/// sneer, 20 the scratch, 33 the OK and so on. The entries the client leaves unset are none here.
const FEELINGS: [[[i16; 7]; 5]; 5] = [
    [[32, 32, 29, 29, 6, 23, -1], [32, 32, 29, 29, 6, 6, 4], [20, 32, -1, 32, 4, 20, 9], [33, -1, -1, 9, 20, 9, 5], [18, -1, -1, 20, 33, 2, 5]],
    [[20, 32, 29, 29, 32, 32, -1], [20, 32, -1, 32, 29, 32, 32], [33, -1, -1, 9, 20, 9, 5], [18, -1, -1, 20, 33, 2, 5], [15, -1, -1, 19, 3, 2, 21]],
    [[32, 32, -1, 32, 29, 4, -1], [20, -1, -1, 9, 9, 9, 20], [33, -1, -1, 20, 33, 2, 5], [18, -1, -1, 19, 18, 2, 21], [15, 21, -1, 23, 14, 33, 21]],
    [[32, -1, -1, 9, 9, 9, -1], [32, -1, -1, 20, 20, 20, 9], [20, -1, -1, 19, 21, 2, 5], [33, 21, 19, 23, 3, 33, 21], [18, 21, 26, 28, 30, 22, 18]],
    [[23, -1, -1, 20, 20, 20, -1], [32, -1, -1, 19, 20, 0, 0], [32, 21, 19, 23, 18, 33, 21], [20, 21, 26, 28, 3, 18, 18], [20, 21, 26, 28, 30, 18, 30]],
];

/// The number the pet's owner sends for the emotion its pet shows at that moment (the client's
/// `GetPetActionNumber`): ten times the emotion, and 2.
pub fn feeling(hunger: i32, intimacy: i32, moment: i32) -> Option<i32> {
    let emotion = *FEELINGS[hunger_state(hunger)][intimacy_state(intimacy)].get(usize::try_from(moment).ok()?)?;
    (emotion >= 0).then(|| i32::from(emotion) * 10 + 2)
}

/// The number the pet's owner sends for something its pet says at that moment (the client's
/// `GetPetEmotionNumber`): the monster, the moment, the hunger state and the owner's sex, digit after digit.
pub fn speech(job: u16, moment: i32, hunger: i32, sex: u8) -> i32 {
    ((i32::from(job) * 10 + moment) * 10 + hunger_state(hunger) as i32) * 10 + i32::from(sex & 1)
}

/// The client sends a pet's words only for a pet that is loyal (its `MM_PET_ACT`): intimacy from 900 on.
pub fn speaks(intimacy: i32) -> bool {
    intimacy / 10 >= 90
}

/// The words of a pet's hunger in the client's table of what pets say (`PetTalkTable.xml`), by its state.
pub fn hunger_tag(state: usize) -> &'static str {
    match state {
        1 => "bit_hungry",
        2 => "noting",
        3 => "full",
        4 => "so_full",
        _ => "hungry",
    }
}

/// The words of a moment in that table (the client's `GetPetTalkActionName`).
pub fn moment_tag(moment: i32) -> &'static str {
    match moment {
        0 => "feeding",
        1 => "hunting",
        2 => "danger",
        3 => "dead",
        4 => "stand",
        5 => "perfor_s",
        6 => "levelup",
        7 => "perfor_1",
        8 => "perfor_2",
        9 => "perfor_3",
        10 => "connect",
        _ => "normal",
    }
}

/// What lies between a tag's opening and its closing, the first time it comes in `text`.
fn inside<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let from = text.find(&open)? + open.len();
    let to = text[from..].find(&format!("</{tag}>"))?;
    Some(&text[from..from + to])
}

/// The lines a monster may say at a moment, in a state of hunger (`hunger_tag`, `moment_tag`); none
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
    fn words_of_the_window() {
        assert_eq!((hunger_word(10), hunger_word(11), hunger_word(76), hunger_word(100)), ("Very Hungry", "Hungry", "Satisfied", "Stuffed"));
        assert_eq!(intimacy_word(250), ("Shy", 100));
        assert_eq!(intimacy_word(500), ("Neutral", 50));
        assert_eq!(intimacy_word(1000), ("Loyal", 100));
    }

    #[test]
    fn feelings_and_words() {
        // Starving and awkward, fed: the hungry emotion (32). Stuffed and loyal, its master hurt: none.
        assert_eq!(feeling(5, 50, moment::FED), Some(322));
        assert_eq!(feeling(100, 1000, moment::DANGER), Some(262));
        assert_eq!(feeling(50, 500, moment::DANGER), None);
        assert_eq!(feeling(5, 50, moment::LEVEL_UP), None);
        assert_eq!(speech(1002, moment::FED, 80, 1), 1_002_031);
        assert!(speaks(900) && !speaks(899));
        assert_eq!((hunger_tag(3), moment_tag(7)), ("full", "perfor_1"));
    }

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
