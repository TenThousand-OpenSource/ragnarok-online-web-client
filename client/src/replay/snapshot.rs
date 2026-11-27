//! The sections of a replay file in which the Windows client keeps the rest of what its session and its map
//! held when the recording began (6 to 24 of the file's table; 3 and 4 are `super::sections`), each as plain
//! data with a writer that lays it out as the client does and a reader of what the client writes. Sections 6
//! to 14 are tagged records of the session (`CSession::Build…ReplayRecords` in `Client/Replay/Replay.cpp`,
//! read by `CSession::ReadReplay…` in `Contents/ClientState.cpp`); 15 to 18, 22 and 23 are the map's chunks
//! (`CGameMode::Save…` in `GameMode.cpp`), 19 to 21 the windows' (`UIWindowMgr::SaveReplay…`), 24 the
//! navigation's. A chunk and a record are the same six bytes before their own: `u16 id, u32 bytes`.
//!
//! Every text is kept as the bytes of the client's code page; who calls turns them into letters. A member of
//! the client's this project gives no field for is written as zeros of its size and passed by when read; a
//! tag a reader does not know is passed by its size, as the client's readers do.
//!
//! `sections` writes all of them for `File::client`; `of_file` reads them back. `session` turns what this
//! client keeps of a session into these and back, and `skills` is what stands at the head of the packets.

pub mod actors;
pub mod companions;
pub mod ground;
pub mod items;
pub mod people;
pub mod quests;
pub mod rest;
pub mod screen;
pub mod session;
pub mod skills;

use super::{tagged, tags};

/// All that sections 6 to 24 hold, as this client gives and takes it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    /// Section 6: the quests under way.
    pub quests: Vec<quests::Quest>,
    /// Section 7: the party and the friends.
    pub messenger: people::Messenger,
    /// Section 8: the lists of items.
    pub items: items::Items,
    /// Section 9: the homunculus, the mercenary, the pet and the elemental.
    pub companions: companions::Companions,
    /// Section 10: the memorial dungeon.
    pub memorial: rest::Memorial,
    /// Section 12: the queue of the battlegrounds.
    pub queue: rest::Queue,
    /// Section 13: the shortcut bar.
    pub shortcuts: rest::Shortcuts,
    /// Section 14: the shops by their names.
    pub shops: rest::Shops,
    /// Section 15: those in sight, each a row of chunks.
    pub actors: Vec<actors::ActorChunk>,
    /// Section 16: what lies on the ground.
    pub ground_items: Vec<ground::GroundItem>,
    /// Section 17: the skills' units on the ground.
    pub ground_skills: ground::GroundSkills,
    /// Section 18: the status icons.
    pub icons: Vec<screen::StatusIcon>,
    /// Section 19: the windows open, each the client's number of it and what it kept.
    pub windows: Vec<(u32, Vec<u8>)>,
    /// Section 22: the names over heads, the emblems and the character's guild.
    pub guild: people::Guild,
    /// Section 23: whether the character is on a quest (`isOnQuest`).
    pub on_quest: i32,
}

/// The numbers of the sections in the file's table.
pub mod number {
    pub const QUESTS: usize = 6;
    pub const MESSENGER: usize = 7;
    pub const ITEMS: usize = 8;
    pub const COMPANIONS: usize = 9;
    pub const MEMORIAL: usize = 10;
    pub const QUEUE: usize = 12;
    pub const SHORTCUTS: usize = 13;
    pub const SHOPS: usize = 14;
    pub const ACTORS: usize = 15;
    pub const GROUND_ITEMS: usize = 16;
    pub const GROUND_SKILLS: usize = 17;
    pub const ICONS: usize = 18;
    pub const WINDOWS: usize = 19;
    pub const SETTINGS: usize = 20;
    pub const RESERVED: usize = 21;
    pub const GUILD: usize = 22;
    pub const ON_QUEST: usize = 23;
    pub const NAVIGATION: usize = 24;
}

/// Every section from 6 to 24 the client writes, each its number and its bytes before the cipher, for
/// `File::client`. Not 11, which the client leaves empty (`BuildReservedReplayRecords`). The one setting of
/// section 20 (`dragAllItems`) is written as off, and the navigation as one that shows no route.
pub fn sections(snapshot: &Snapshot) -> Vec<(usize, Vec<u8>)> {
    vec![
        (number::QUESTS, quests::written(&snapshot.quests)),
        (number::MESSENGER, people::messenger(&snapshot.messenger)),
        (number::ITEMS, items::written(&snapshot.items)),
        (number::COMPANIONS, companions::written(&snapshot.companions)),
        (number::MEMORIAL, rest::memorial(&snapshot.memorial)),
        (number::QUEUE, rest::queue(&snapshot.queue)),
        (number::SHORTCUTS, rest::shortcuts(&snapshot.shortcuts)),
        (number::SHOPS, rest::shops(&snapshot.shops)),
        (number::ACTORS, actors::written(&snapshot.actors)),
        (number::GROUND_ITEMS, ground::items(&snapshot.ground_items)),
        (number::GROUND_SKILLS, ground::skills(&snapshot.ground_skills)),
        (number::ICONS, screen::icons(&snapshot.icons)),
        (number::WINDOWS, screen::windows(&snapshot.windows)),
        (number::SETTINGS, screen::settings(0)),
        (number::RESERVED, screen::reserved()),
        (number::GUILD, people::guild(&snapshot.guild)),
        (number::ON_QUEST, screen::on_quest(snapshot.on_quest)),
        (number::NAVIGATION, screen::navigation()),
    ]
}

/// The sections of a file read back; one the file has not, or that does not open as the client's does, is
/// as empty as a new one.
pub fn of_file(file: &super::File) -> Snapshot {
    Snapshot {
        quests: quests::read(file.section(number::QUESTS)).unwrap_or_default(),
        messenger: people::messenger_read(file.section(number::MESSENGER)).unwrap_or_default(),
        items: items::read(file.section(number::ITEMS)).unwrap_or_default(),
        companions: companions::read(file.section(number::COMPANIONS)).unwrap_or_default(),
        memorial: rest::memorial_read(file.section(number::MEMORIAL)).unwrap_or_default(),
        queue: rest::queue_read(file.section(number::QUEUE)).unwrap_or_default(),
        shortcuts: rest::shortcuts_read(file.section(number::SHORTCUTS)).unwrap_or_default(),
        shops: rest::shops_read(file.section(number::SHOPS)).unwrap_or_default(),
        actors: actors::read(file.section(number::ACTORS)).unwrap_or_default(),
        ground_items: ground::items_read(file.section(number::GROUND_ITEMS)).unwrap_or_default(),
        ground_skills: ground::skills_read(file.section(number::GROUND_SKILLS)).unwrap_or_default(),
        icons: screen::icons_read(file.section(number::ICONS)).unwrap_or_default(),
        windows: screen::windows_read(file.section(number::WINDOWS)).unwrap_or_default(),
        guild: people::guild_read(file.section(number::GUILD)).unwrap_or_default(),
        on_quest: screen::on_quest_read(file.section(number::ON_QUEST)).unwrap_or_default(),
    }
}

/// The records of a section that opens with that record, as the client's readers ask of one; none for any
/// other.
fn opened(section: &[u8], begin: u16) -> Option<Vec<(u16, &[u8])>> {
    let records = tags(section);
    (records.first()?.0 == begin).then_some(records)
}

/// The first bytes of a record as a number's own, zeros for those it has not: the client's readers copy as
/// many bytes as the record has into the member.
fn le<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut whole = [0; N];
    let size = bytes.len().min(N);
    whole[..size].copy_from_slice(&bytes[..size]);
    whole
}

fn i32_of(bytes: &[u8]) -> i32 {
    i32::from_le_bytes(le(bytes))
}

fn u32_of(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(le(bytes))
}

fn i16_of(bytes: &[u8]) -> i16 {
    i16::from_le_bytes(le(bytes))
}

fn u16_of(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(le(bytes))
}

fn u8_of(bytes: &[u8]) -> u8 {
    bytes.first().copied().unwrap_or(0)
}

/// A text in a field of so many bytes, cut where the field ends, zeros after it.
fn padded(text: &[u8], size: usize) -> Vec<u8> {
    let mut field = text[..text.len().min(size)].to_vec();
    field.resize(size, 0);
    field
}

/// The text of such a field: up to its first zero.
fn unpadded(field: &[u8]) -> Vec<u8> {
    field[..field.iter().position(|byte| *byte == 0).unwrap_or(field.len())].to_vec()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_map;
