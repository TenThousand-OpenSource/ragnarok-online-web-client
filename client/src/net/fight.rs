//! What the server says of a fight beside the blows themselves: the monk's combo window and blade stop, the
//! threads of Devotion, the MVP and its reward, what Sense tells of a monster, the Star Gladiator's places and
//! missions.

use super::bytes::{long, name_at, short, word, Writer};

// A skill changed or taken away is told with the skills known (`net/items/skills.rs`), a talkie box's words and
// what Graffiti writes with the units on the ground (`net/skill_units.rs`); they were here once, and are still
// named so.
pub use super::items::{skill_changed, skill_taken, ZC_SKILLINFO_DELETE, ZC_SKILLINFO_UPDATE2};
pub use super::skill_units::{talkie, written, Written, ZC_SKILL_ENTRY2, ZC_SKILL_UPDATE, ZC_TALKBOX_CHATCONTENTS};

/// ZC_COMBODELAY (0x01d2): `u32 who, u32 milliseconds`: how long the one who struck may follow with the next
/// blow of a combo.
pub const ZC_COMBODELAY: u16 = 0x01d2;
/// ZC_BLADESTOP (0x01d1): `u32 who, u32 whom, u32 held`: one catches another's blade, or lets it go.
pub const ZC_BLADESTOP: u16 = 0x01d1;
/// ZC_DEVOTIONLIST (0x01cf): `u32 who, u32 ward[5], u16 range`: whom a crusader's Devotion is on.
pub const ZC_DEVOTIONLIST: u16 = 0x01cf;
/// ZC_MVP (0x010c): `u32 who`: that one did the most to bring a boss down.
pub const ZC_MVP: u16 = 0x010c;
/// ZC_MVP_GETTING_ITEM (0x010a): `u32 item`: the MVP's reward.
pub const ZC_MVP_GETTING_ITEM: u16 = 0x010a;
/// ZC_MVP_GETTING_SPECIAL_EXP (0x010b): `u32 exp`: the MVP's reward in experience.
pub const ZC_MVP_GETTING_SPECIAL_EXP: u16 = 0x010b;
/// ZC_THROW_MVPITEM (0x010d): the reward could not be carried.
pub const ZC_THROW_MVPITEM: u16 = 0x010d;
/// ZC_MONSTER_INFO (0x018c): what Sense tells of a monster (`sensed`).
pub const ZC_MONSTER_INFO: u16 = 0x018c;
/// ZC_STARPLACE (0x0253): `u8 which`: the server asks whether this map is to be the place of the sun (0), the
/// moon (1) or the stars (2).
pub const ZC_STARPLACE: u16 = 0x0253;
/// ZC_STARSKILL (0x020e): what a Star Gladiator's or a Taekwon's skill tells (`star_told`).
pub const ZC_STARSKILL: u16 = 0x020e;
/// The window of a combo: `(who, milliseconds)`.
pub fn combo_delay(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

/// A blade caught or let go: `(who, whose, whether it is held)`.
pub fn blade_stop(packet: &[u8]) -> Option<(u32, u32, bool)> {
    Some((long(packet, 2)?, long(packet, 6)?, long(packet, 10)? != 0))
}

/// Devotion: who guards, and those guarded (the server names five places; an empty one is 0).
pub fn devotion(packet: &[u8]) -> Option<(u32, Vec<u32>)> {
    let wards = (0..5).map(|ward| long(packet, 6 + ward * 4)).collect::<Option<Vec<u32>>>()?;
    Some((long(packet, 2)?, wards.into_iter().filter(|ward| *ward != 0).collect()))
}

/// The one number ZC_MVP, ZC_MVP_GETTING_ITEM and ZC_MVP_GETTING_SPECIAL_EXP carry: who, which item, how much.
pub fn mvp(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// What Sense tells of a monster.
#[derive(Debug, Clone, PartialEq)]
pub struct Sensed {
    pub job: u16,
    pub level: u16,
    /// 0 small, 1 medium, 2 large.
    pub size: u16,
    pub life: u32,
    pub defence: i16,
    pub race: u16,
    pub magic_defence: i16,
    /// The element, with its level in the twenties above it.
    pub element: u16,
    /// How much of an attack of each element gets through, in hundredths: water, earth, fire, wind, poison,
    /// holy, shadow, ghost, undead.
    pub taken: [u8; 9],
}

/// ZC_MONSTER_INFO: `u16 job, u16 level, u16 size, u32 hp, i16 def, u16 race, i16 mdef, u16 element,
/// u8 taken[9]`.
pub fn sensed(packet: &[u8]) -> Option<Sensed> {
    Some(Sensed {
        job: word(packet, 2)?,
        level: word(packet, 4)?,
        size: word(packet, 6)?,
        life: long(packet, 8)?,
        defence: short(packet, 12)?,
        race: word(packet, 14)?,
        magic_defence: short(packet, 16)?,
        element: word(packet, 18)?,
        taken: packet.get(20..29)?.try_into().ok()?,
    })
}

/// The monsters Sense opens no window for (the client leaves the guardians out).
pub fn sense_shown(job: u16) -> bool {
    !matches!(job, 1285 | 1286 | 1287 | 1829 | 1830)
}

/// Which place the server asks about: 0 the sun's, 1 the moon's, 2 the stars'.
pub fn star_place(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// CZ_AGREE_STARPLACE (0x0254): `u8 which`: the map is to be that place.
pub fn agree_star_place(which: u8) -> Vec<u8> {
    Writer::new(0x0254).u8(which).done()
}

/// What ZC_STARSKILL tells: `char name[24], u32 value, u8 which, u8 kind`. The name is a map's or a monster's,
/// the value a monster's job, an item or an amount of life, by the kind.
#[derive(Debug, Clone, PartialEq)]
pub struct StarTold {
    pub name: String,
    pub value: u32,
    pub which: u8,
    pub kind: u8,
}

pub fn star_told(packet: &[u8]) -> Option<StarTold> {
    Some(StarTold { name: name_at(packet.get(..26)?, 2, 24), value: long(packet, 26)?, which: *packet.get(30)?, kind: *packet.get(31)? })
}

/// What a line of ZC_STARSKILL is made of (the client's `CGameMode::ReceiveSpecialBroadcast`): a line of the
/// message table and what goes into it.
#[derive(Debug, Clone, PartialEq)]
pub enum StarLine {
    /// A line with the player's own name and then the name sent (a map's).
    Place(usize),
    /// A line with the player's own name and then the name of the monster's job sent.
    Monster(usize),
    /// The Taekwon mission: a monster's name and how far it is, in hundredths.
    Mission,
    /// The taming mission: a monster's name.
    Taming,
    /// The collector's mission: an item's name.
    Collector,
    /// The places and monsters are forgotten.
    Reset,
    /// "Target HP : " and the value.
    Life,
}

impl StarTold {
    /// Which line it is, and whether the client shows its effect of hatred on the player's own character with
    /// it (`EF_HATED`; `Reset` shows `EF_ELECTRIC` instead). None for a kind the client does not know.
    pub fn line(&self) -> Option<(StarLine, bool)> {
        let which = usize::from(self.which);
        Some(match self.kind {
            0 => (StarLine::Place(837 + which), false),
            1 => (StarLine::Place(840 + which), true),
            10 => (StarLine::Monster(843 + which), false),
            11 => (StarLine::Monster(846 + which), true),
            20 => (StarLine::Mission, true),
            21 => (StarLine::Taming, true),
            22 => (StarLine::Collector, true),
            30 => (StarLine::Reset, false),
            40 => (StarLine::Life, false),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::packet;

    #[test]
    fn the_window_of_a_combo_and_a_blade_caught() {
        assert_eq!(combo_delay(&packet(ZC_COMBODELAY, &[&7u32.to_le_bytes(), &650u32.to_le_bytes()])), Some((7, 650)));
        assert_eq!(blade_stop(&packet(ZC_BLADESTOP, &[&7u32.to_le_bytes(), &9u32.to_le_bytes(), &1u32.to_le_bytes()])), Some((7, 9, true)));
        assert_eq!(blade_stop(&packet(ZC_BLADESTOP, &[&7u32.to_le_bytes(), &9u32.to_le_bytes(), &0u32.to_le_bytes()])), Some((7, 9, false)));
        assert_eq!(combo_delay(&[0xd2, 0x01, 7]), None);
    }

    #[test]
    fn devotion_names_those_guarded_and_leaves_the_empty_places_out() {
        let mut told = packet(ZC_DEVOTIONLIST, &[&5u32.to_le_bytes()]);
        for ward in [11u32, 0, 13, 0, 0] {
            told.extend(ward.to_le_bytes());
        }
        told.extend(9u16.to_le_bytes());
        assert_eq!(told.len(), 28);
        assert_eq!(devotion(&told), Some((5, vec![11, 13])));
    }

    #[test]
    fn what_sense_tells() {
        let mut told = packet(ZC_MONSTER_INFO, &[&1002u16.to_le_bytes(), &1u16.to_le_bytes(), &1u16.to_le_bytes(), &50u32.to_le_bytes()]);
        told.extend((-3i16).to_le_bytes());
        told.extend(3u16.to_le_bytes());
        told.extend(5i16.to_le_bytes());
        told.extend(21u16.to_le_bytes());
        told.extend([100, 100, 150, 100, 100, 100, 100, 100, 100]);
        assert_eq!(told.len(), 29);
        let sensed = sensed(&told).unwrap();
        assert_eq!((sensed.job, sensed.level, sensed.size, sensed.life, sensed.defence, sensed.race, sensed.magic_defence, sensed.element), (1002, 1, 1, 50, -3, 3, 5, 21));
        assert_eq!(sensed.taken[2], 150);
        assert!(sense_shown(1002) && !sense_shown(1285));
    }

    #[test]
    fn the_star_gladiators_places_and_missions() {
        let mut told = packet(ZC_STARSKILL, &[b"prontera"]);
        told.resize(26, 0);
        told.extend(1002u32.to_le_bytes());
        told.extend([2, 1]);
        let star = star_told(&told).unwrap();
        assert_eq!(star, StarTold { name: "prontera".to_owned(), value: 1002, which: 2, kind: 1 });
        assert_eq!(star.line(), Some((StarLine::Place(842), true)));
        assert_eq!(StarTold { kind: 10, which: 1, ..star.clone() }.line(), Some((StarLine::Monster(844), false)));
        assert_eq!(StarTold { kind: 30, ..star.clone() }.line(), Some((StarLine::Reset, false)));
        assert_eq!(StarTold { kind: 5, ..star }.line(), None);
        assert_eq!(star_place(&[0x53, 0x02, 1]), Some(1));
        assert_eq!(agree_star_place(2), [0x54, 0x02, 2]);
    }
}
