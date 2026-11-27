//! Section 9 of a replay file: the homunculus, the mercenary, the pet and the elemental
//! (`CSession::BuildCompanionReplayRecords`, `Replay.cpp`, read by `CSession::ReadReplayCompanions`,
//! `Contents/ClientState.cpp`). Between 5000 and 5001 one record a member of the session: 5101 to 5134 the
//! homunculus, 5201 to 5229 the mercenary, 5301 to 5310 the pet, 5401 to 5405 the elemental; each four bytes,
//! but the three names (32) and the homunculus's two experiences (8). The members about whom a homunculus or
//! a mercenary fights and what it is doing (5124 to 5129, 5131 to 5133, 5222 to 5227, 5229) have no field
//! here: they are written as zeros and passed by.

use super::*;

/// The homunculus (`m_homun…`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Homunculus {
    pub id: u32,
    /// Its name, 32 bytes at most.
    pub name: Vec<u8>,
    /// Its eight values of the window (`m_homunStats`), in the client's order.
    pub stats: [i32; 8],
    pub job: i32,
    pub level: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub sp: i32,
    pub max_sp: i32,
    /// How fond it is of its master (`homunFriendly`).
    pub friendly: i32,
    pub accessory: i32,
    pub exp: i64,
    pub max_exp: i64,
    pub hunger: i32,
    pub max_hunger: i32,
    /// Whether it was given a name (`homunNameModified`).
    pub renamed: i32,
    pub alive: i32,
    /// How far it reaches (`m_homunAttackRange`).
    pub range: i32,
}

/// The mercenary (`m_mercenary…`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mercenary {
    pub id: u32,
    pub name: Vec<u8>,
    pub stats: [i32; 8],
    pub hp: i32,
    pub max_hp: i32,
    /// `m_mercenaryClass`.
    pub job: i32,
    pub level: i32,
    pub sp: i32,
    pub max_sp: i32,
    pub loyalty: i32,
    /// How many times one was called (`m_mercenarySummons`).
    pub summons: i32,
    pub kills: i32,
    pub range: i32,
    /// When its contract ends.
    pub expires: u32,
    pub alive: i32,
}

/// The pet (`pet…`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pet {
    pub id: u32,
    /// Whether it was given a name (`m_petNameModified`).
    pub renamed: i32,
    pub name: Vec<u8>,
    pub accessory: i32,
    pub job: i32,
    pub level: i32,
    pub hunger: i32,
    pub friendly: i32,
    /// The place of its egg in the inventory (`petEggIndex`).
    pub egg: i32,
    /// `petPreviousHunger`.
    pub hunger_before: i32,
}

/// The elemental (`elemental…`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Elemental {
    pub id: i32,
    pub hp: i32,
    pub sp: i32,
    pub max_hp: i32,
    pub max_sp: i32,
}

/// The four.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Companions {
    pub homunculus: Homunculus,
    pub mercenary: Mercenary,
    pub pet: Pet,
    pub elemental: Elemental,
}

/// The tags in the order the client writes them.
fn all() -> impl Iterator<Item = u16> {
    (5101..=5134).chain(5201..=5229).chain(5301..=5310).chain(5401..=5405)
}

/// The member behind a tag, by its kind.
enum Slot<'held> {
    Whole(&'held mut i32),
    Id(&'held mut u32),
    Long(&'held mut i64),
    /// A name in 32 bytes.
    Name(&'held mut Vec<u8>),
    /// A member of four bytes this client has no field for.
    Nothing,
}

fn homunculus(homunculus: &mut Homunculus, tag: u16) -> Slot<'_> {
    match tag {
        5101 => Slot::Id(&mut homunculus.id),
        5102 => Slot::Name(&mut homunculus.name),
        5103..=5110 => Slot::Whole(&mut homunculus.stats[usize::from(tag - 5103)]),
        5111 => Slot::Whole(&mut homunculus.job),
        5112 => Slot::Whole(&mut homunculus.level),
        5113 => Slot::Whole(&mut homunculus.hp),
        5114 => Slot::Whole(&mut homunculus.max_hp),
        5115 => Slot::Whole(&mut homunculus.sp),
        5116 => Slot::Whole(&mut homunculus.max_sp),
        5117 => Slot::Whole(&mut homunculus.friendly),
        5118 => Slot::Whole(&mut homunculus.accessory),
        5119 => Slot::Long(&mut homunculus.exp),
        5120 => Slot::Long(&mut homunculus.max_exp),
        5121 => Slot::Whole(&mut homunculus.hunger),
        5122 => Slot::Whole(&mut homunculus.max_hunger),
        5123 => Slot::Whole(&mut homunculus.renamed),
        5130 => Slot::Whole(&mut homunculus.alive),
        5134 => Slot::Whole(&mut homunculus.range),
        _ => Slot::Nothing,
    }
}

fn mercenary(mercenary: &mut Mercenary, tag: u16) -> Slot<'_> {
    match tag {
        5201 => Slot::Id(&mut mercenary.id),
        5202 => Slot::Name(&mut mercenary.name),
        5203..=5210 => Slot::Whole(&mut mercenary.stats[usize::from(tag - 5203)]),
        5211 => Slot::Whole(&mut mercenary.hp),
        5212 => Slot::Whole(&mut mercenary.max_hp),
        5213 => Slot::Whole(&mut mercenary.job),
        5214 => Slot::Whole(&mut mercenary.level),
        5215 => Slot::Whole(&mut mercenary.sp),
        5216 => Slot::Whole(&mut mercenary.max_sp),
        5217 => Slot::Whole(&mut mercenary.loyalty),
        5218 => Slot::Whole(&mut mercenary.summons),
        5219 => Slot::Whole(&mut mercenary.kills),
        5220 => Slot::Whole(&mut mercenary.range),
        5221 => Slot::Id(&mut mercenary.expires),
        5228 => Slot::Whole(&mut mercenary.alive),
        _ => Slot::Nothing,
    }
}

fn pet(pet: &mut Pet, tag: u16) -> Slot<'_> {
    match tag {
        5301 => Slot::Id(&mut pet.id),
        5302 => Slot::Whole(&mut pet.renamed),
        5303 => Slot::Name(&mut pet.name),
        5304 => Slot::Whole(&mut pet.accessory),
        5305 => Slot::Whole(&mut pet.job),
        5306 => Slot::Whole(&mut pet.level),
        5307 => Slot::Whole(&mut pet.hunger),
        5308 => Slot::Whole(&mut pet.friendly),
        5309 => Slot::Whole(&mut pet.egg),
        5310 => Slot::Whole(&mut pet.hunger_before),
        _ => Slot::Nothing,
    }
}

fn elemental(elemental: &mut Elemental, tag: u16) -> Slot<'_> {
    match tag {
        5401 => Slot::Whole(&mut elemental.id),
        5402 => Slot::Whole(&mut elemental.hp),
        5403 => Slot::Whole(&mut elemental.sp),
        5404 => Slot::Whole(&mut elemental.max_hp),
        5405 => Slot::Whole(&mut elemental.max_sp),
        _ => Slot::Nothing,
    }
}

/// The member behind a tag of the section; none for a tag that is not one of it.
fn slot(companions: &mut Companions, tag: u16) -> Option<Slot<'_>> {
    Some(match tag {
        5101..=5134 => homunculus(&mut companions.homunculus, tag),
        5201..=5229 => mercenary(&mut companions.mercenary, tag),
        5301..=5310 => pet(&mut companions.pet, tag),
        5401..=5405 => elemental(&mut companions.elemental, tag),
        _ => return None,
    })
}

/// Section 9: every member's record at the member's size, in the client's order.
pub fn written(companions: &Companions) -> Vec<u8> {
    let mut companions = companions.clone();
    let mut section = Vec::new();
    tagged(&mut section, 5000, &[]);
    for tag in all() {
        match slot(&mut companions, tag) {
            Some(Slot::Whole(value)) => tagged(&mut section, tag, &value.to_le_bytes()),
            Some(Slot::Id(value)) => tagged(&mut section, tag, &value.to_le_bytes()),
            Some(Slot::Long(value)) => tagged(&mut section, tag, &value.to_le_bytes()),
            Some(Slot::Name(name)) => tagged(&mut section, tag, &padded(name, 32)),
            Some(Slot::Nothing) | None => tagged(&mut section, tag, &[0; 4]),
        }
    }
    tagged(&mut section, 5001, &[]);
    section
}

/// Section 9 read back.
pub fn read(section: &[u8]) -> Option<Companions> {
    let mut companions = Companions::default();
    for (tag, bytes) in opened(section, 5000)? {
        match slot(&mut companions, tag) {
            Some(Slot::Whole(value)) => *value = i32_of(bytes),
            Some(Slot::Id(value)) => *value = u32_of(bytes),
            Some(Slot::Long(value)) => *value = i64::from_le_bytes(le(bytes)),
            Some(Slot::Name(name)) => *name = unpadded(&bytes[..bytes.len().min(32)]),
            Some(Slot::Nothing) | None => {}
        }
    }
    Some(companions)
}
