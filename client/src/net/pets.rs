//! The player's pet: what the map server tells of it, and the commands given to it.

/// ZC_PROPERTY_PET: everything about the pet at once.
pub const ZC_PROPERTY_PET: u16 = 0x01a2;
/// ZC_FEED_PET: whether the pet was fed.
pub const ZC_FEED_PET: u16 = 0x01a3;
/// ZC_CHANGESTATE_PET: one thing about a pet changed.
pub const ZC_CHANGESTATE_PET: u16 = 0x01a4;

/// What ZC_CHANGESTATE_PET tells of (its `type`).
pub mod change {
    /// This unit is the player's pet.
    pub const OWN: u8 = 0;
    pub const INTIMACY: u8 = 1;
    pub const HUNGER: u8 = 2;
    pub const ACCESSORY: u8 = 3;
    /// The pet does a trick: the value is which (1 to 3, and 4 for its special one).
    pub const PERFORMANCE: u8 = 4;
    /// The pet went back into its egg when the value is 0.
    pub const EGG: u8 = 6;
}

/// The commands of CZ_COMMAND_PET, as the pet window's menu gives them.
pub mod order {
    pub const FEED: u8 = 1;
    pub const PERFORM: u8 = 2;
    pub const TO_EGG: u8 = 3;
    pub const UNEQUIP: u8 = 4;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Pet {
    pub name: String,
    /// It was given a name already; that can be done once.
    pub renamed: bool,
    pub level: u16,
    /// 0 to 100.
    pub hunger: i32,
    /// 0 to 1000.
    pub intimacy: i32,
    /// The item it wears, 0 for none.
    pub accessory: u16,
    /// The monster it is.
    pub job: u16,
}

/// ZC_PROPERTY_PET (0x01a2, 37 bytes): `char name[24]`, `u8 renamed`, `u16 level`, `u16 hunger`,
/// `u16 intimacy`, `u16 accessory`, `u16 job`.
pub fn property(packet: &[u8]) -> Option<Pet> {
    let packet = packet.get(..37)?;
    let word = |at: usize| u16::from_le_bytes([packet[at], packet[at + 1]]);
    let name = &packet[2..26];
    let end = name.iter().position(|byte| *byte == 0).unwrap_or(name.len());
    Some(Pet {
        name: String::from_utf8_lossy(&name[..end]).into_owned(),
        renamed: packet[26] != 0,
        level: word(27),
        hunger: i32::from(word(29)),
        intimacy: i32::from(word(31)),
        accessory: word(33),
        job: word(35),
    })
}

/// ZC_CHANGESTATE_PET (0x01a4, 11 bytes): `u8 type`, `u32 unit`, `i32 value`.
pub fn changed(packet: &[u8]) -> Option<(u8, u32, i32)> {
    let packet = packet.get(..11)?;
    Some((packet[2], u32::from_le_bytes(packet[3..7].try_into().ok()?), i32::from_le_bytes(packet[7..11].try_into().ok()?)))
}

/// ZC_FEED_PET (0x01a3, 7 bytes): `u8 fed`, `u32 item`: whether the pet ate, and the food it eats.
pub fn fed(packet: &[u8]) -> Option<(bool, u32)> {
    let packet = packet.get(..7)?;
    Some((packet[2] != 0, u32::from_le_bytes(packet[3..7].try_into().ok()?)))
}

/// CZ_COMMAND_PET (0x01a1): `u8 order`.
pub fn command(order: u8) -> Vec<u8> {
    vec![0xa1, 0x01, order]
}

/// CZ_RENAME_PET (0x01a5): `char name[24]`.
pub fn rename(name: &str) -> Vec<u8> {
    let mut packet = vec![0u8; 26];
    packet[..2].copy_from_slice(&0x01a5u16.to_le_bytes());
    let bytes = &name.as_bytes()[..name.len().min(23)];
    packet[2..2 + bytes.len()].copy_from_slice(bytes);
    packet
}

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

/// ZC_PET_ACT: a pet shows what it feels, or says something.
pub const ZC_PET_ACT: u16 = 0x01aa;

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

/// What a pet does, as ZC_PET_ACT tells it.
#[derive(Debug, PartialEq)]
pub enum Act {
    /// It shows that emotion.
    Emotion(u16),
    /// It says a line: for that monster, at that moment, in that state of hunger.
    Speech { job: u16, moment: i32, hunger: usize },
}

/// ZC_PET_ACT (0x01aa, 10 bytes): `u32 unit`, `i32 number`. A number ending in 2 is an emotion (the rest of
/// it); any other is something said (the client's `ReceivePetAct`).
pub fn acted(packet: &[u8]) -> Option<(u32, Act)> {
    let packet = packet.get(..10)?;
    let unit = u32::from_le_bytes(packet[2..6].try_into().ok()?);
    let number = i32::from_le_bytes(packet[6..10].try_into().ok()?);
    let rest = number / 10;
    let act = if number % 10 == 2 { Act::Emotion(u16::try_from(rest).ok()?) } else { Act::Speech { job: u16::try_from(rest / 100).ok()?, moment: rest / 10 % 10, hunger: usize::try_from(rest % 10).ok()? } };
    Some((unit, act))
}

/// CZ_PET_ACT (0x01a9): `i32 number`: the pet's owner tells what its pet shows or says.
pub fn act(number: i32) -> Vec<u8> {
    [&0x01a9u16.to_le_bytes()[..], &number.to_le_bytes()].concat()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_pet() {
        let mut packet = vec![0u8; 37];
        packet[..2].copy_from_slice(&ZC_PROPERTY_PET.to_le_bytes());
        packet[2..8].copy_from_slice(b"Poring");
        packet[26] = 1;
        packet[27..29].copy_from_slice(&3u16.to_le_bytes());
        packet[29..31].copy_from_slice(&70u16.to_le_bytes());
        packet[31..33].copy_from_slice(&1000u16.to_le_bytes());
        packet[33..35].copy_from_slice(&10013u16.to_le_bytes());
        packet[35..37].copy_from_slice(&1002u16.to_le_bytes());
        assert_eq!(property(&packet), Some(Pet { name: "Poring".into(), renamed: true, level: 3, hunger: 70, intimacy: 1000, accessory: 10013, job: 1002 }));

        let mut change = vec![0xa4, 0x01, change::HUNGER];
        change.extend(110_000u32.to_le_bytes());
        change.extend(55i32.to_le_bytes());
        assert_eq!(changed(&change), Some((change::HUNGER, 110_000, 55)));
        assert_eq!(fed(&[0xa3, 0x01, 1, 0x13, 0x02, 0, 0]), Some((true, 531)));
    }

    #[test]
    fn writes_the_commands() {
        assert_eq!(command(order::FEED), [0xa1, 0x01, 1]);
        let named = rename("Bob");
        assert_eq!((named.len(), &named[..6], named[25]), (26, &[0xa5, 0x01, b'B', b'o', b'b', 0][..], 0));
    }

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

        let packet = |number: i32| [&[0xaa, 0x01, 7, 0, 0, 0][..], &number.to_le_bytes()].concat();
        assert_eq!(acted(&packet(322)), Some((7, Act::Emotion(32))));
        assert_eq!(acted(&packet(1_002_031)), Some((7, Act::Speech { job: 1002, moment: 0, hunger: 3 })));
        assert_eq!(act(322), [0xa9, 0x01, 0x42, 0x01, 0, 0]);
        assert_eq!((hunger_tag(3), moment_tag(7)), ("full", "perfor_1"));
    }
}
