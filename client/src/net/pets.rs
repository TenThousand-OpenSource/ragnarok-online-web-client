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
}
