//! The sections of a replay file about other people. Section 7: the party and the friends
//! (`CSession::BuildMessengerReplayRecords`, `Replay.cpp`, read by `CSession::ReadReplayMessenger`,
//! `Contents/ClientState.cpp`; one of them is `MessengerMember::Serialize`, read by
//! `MessengerMember::Deserialize`, `Contents/Messenger/MessengerMember.cpp`). Section 22: the names over heads,
//! the guilds' emblems and the character's own guild (`CGameMode::SaveNameCardState`, read by
//! `CGameMode::ReadReplayGuildInfo`, `GameMode.cpp`; the names are `CActorNameCardHolder::Save` and `Load`,
//! `Contents/Actors/ActorNameCache.cpp`).

use super::*;

/// One of the party, or a friend (`MessengerMember`; its job, level and whether it is dead are not kept).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Member {
    /// Whether the entry holds someone (`valid`).
    pub valid: i32,
    pub account: u32,
    pub character: u32,
    /// Whether it leads the party.
    pub leader: i32,
    /// Whether it is away from the game.
    pub offline: i32,
    /// The colour its name is written in (`color`).
    pub colour: i32,
    pub name: Vec<u8>,
    /// The map it is on.
    pub map: Vec<u8>,
}

/// The party and the friends.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Messenger {
    /// The session's `partyLeader`, as it keeps it (4012).
    pub leader: i32,
    pub party_name: Vec<u8>,
    pub party: Vec<Member>,
    pub friends: Vec<Member>,
}

/// How many bytes one of them takes beside its name and its map: ten records, six of them four bytes.
pub const MEMBER_BYTES: usize = 84;

/// One of them: 253 to 260, the name (256) and the map (257) between 251 and 252, in the client's order.
fn member(to: &mut Vec<u8>, member: &Member) {
    tagged(to, 251, &[]);
    tagged(to, 253, &member.valid.to_le_bytes());
    tagged(to, 254, &member.account.to_le_bytes());
    tagged(to, 255, &member.character.to_le_bytes());
    tagged(to, 258, &member.leader.to_le_bytes());
    tagged(to, 259, &member.offline.to_le_bytes());
    tagged(to, 260, &member.colour.to_le_bytes());
    tagged(to, 256, &member.name);
    tagged(to, 257, &member.map);
    tagged(to, 252, &[]);
}

/// Those of one list, one after another with no count (`ReadMessengerMembers` reads until the list ends).
fn members(list: &[u8]) -> Vec<Member> {
    let (mut found, mut one) = (Vec::new(), Member::default());
    for (tag, bytes) in tags(list) {
        match tag {
            251 => one = Member::default(),
            253 => one.valid = i32_of(bytes),
            254 => one.account = u32_of(bytes),
            255 => one.character = u32_of(bytes),
            258 => one.leader = i32_of(bytes),
            259 => one.offline = i32_of(bytes),
            260 => one.colour = i32_of(bytes),
            256 => one.name = bytes.to_vec(),
            257 => one.map = bytes.to_vec(),
            252 => found.push(std::mem::take(&mut one)),
            _ => {}
        }
    }
    found
}

/// Section 7: between 4000 and 4001 the session's flag (4010, a byte), whether invitations are refused (4011)
/// and the name of who asked into a party (4013), none of which this client keeps; the leader (4012), the
/// party's name (4014), the party (4015) and the friends (4016).
pub fn messenger(messenger: &Messenger) -> Vec<u8> {
    let list = |members: &[Member]| {
        let mut bytes = Vec::new();
        members.iter().for_each(|one| member(&mut bytes, one));
        bytes
    };
    let mut section = Vec::new();
    tagged(&mut section, 4000, &[]);
    tagged(&mut section, 4010, &[0]);
    tagged(&mut section, 4011, &0i32.to_le_bytes());
    tagged(&mut section, 4012, &messenger.leader.to_le_bytes());
    tagged(&mut section, 4013, &[]);
    tagged(&mut section, 4014, &messenger.party_name);
    tagged(&mut section, 4015, &list(&messenger.party));
    tagged(&mut section, 4016, &list(&messenger.friends));
    tagged(&mut section, 4001, &[]);
    section
}

/// Section 7 read back. (The client's reader ends at 4501, which its writer never writes, and so reads to
/// the section's end; so does this.)
pub fn messenger_read(section: &[u8]) -> Option<Messenger> {
    let mut messenger = Messenger::default();
    for (tag, bytes) in opened(section, 4000)? {
        match tag {
            4012 => messenger.leader = i32_of(bytes),
            4014 => messenger.party_name = bytes.to_vec(),
            4015 => messenger.party.extend(members(bytes)),
            4016 => messenger.friends.extend(members(bytes)),
            _ => {}
        }
    }
    Some(messenger)
}

/// What is written over and under someone (`CActorNameCardHolder`'s card), by the account's number.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NameCard {
    pub id: u32,
    pub name: Vec<u8>,
    pub party: Vec<u8>,
    pub guild: Vec<u8>,
    pub clan: Vec<u8>,
}

/// The names, the emblems and the character's guild.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Guild {
    pub cards: Vec<NameCard>,
    /// The emblems as the client keeps each (13002, `AppendGuildEmblem`): `u32` the length of the file's
    /// name, the name, then the file's bytes. See `emblem`.
    pub emblems: Vec<Vec<u8>>,
    /// The character's guild (`GetGdid`), and the version of its emblem.
    pub guild: i32,
    pub emblem_version: i32,
}

/// An emblem as 13002 holds it: the name of its file (in the client's `_tmpEmblem` folder) and the file.
pub fn emblem(file_name: &[u8], file: &[u8]) -> Vec<u8> {
    [&(file_name.len() as u32).to_le_bytes()[..], file_name, file].concat()
}

/// The names in one piece (13003): how many, then of each the number and the four texts, each after its length.
fn cards(cards: &[NameCard]) -> Vec<u8> {
    let mut sorted: Vec<&NameCard> = cards.iter().collect();
    sorted.sort_by_key(|card| card.id);
    let mut blob = (sorted.len() as u32).to_le_bytes().to_vec();
    for card in sorted {
        blob.extend(card.id.to_le_bytes());
        for text in [&card.name, &card.party, &card.guild, &card.clan] {
            blob.extend((text.len() as i32).to_le_bytes());
            blob.extend(text);
        }
    }
    blob
}

fn cards_read(blob: &[u8]) -> Vec<NameCard> {
    let mut found = Vec::new();
    let mut at = 4;
    for _ in 0..u32_of(blob) {
        let Some(id) = blob.get(at..at + 4) else { break };
        at += 4;
        let mut texts: [Vec<u8>; 4] = Default::default();
        for text in &mut texts {
            let Some(size) = blob.get(at..at + 4).map(|size| i32_of(size).max(0) as usize) else { return found };
            let Some(bytes) = blob.get(at + 4..at + 4 + size) else { return found };
            *text = bytes.to_vec();
            at += 4 + size;
        }
        let [name, party, guild, clan] = texts;
        found.push(NameCard { id: u32_of(id), name, party, guild, clan });
    }
    found
}

/// Section 22: between 13000 and 13001 the names (13003, only when there is one, by their numbers upwards),
/// the guild (13004) and then the version of its emblem (13005, which the reader takes the guild for), and
/// the emblems (13002). The client writes the emblem of the character's own guild before 13004 and those of
/// the others in sight after 13005: here the first is taken for the character's when it has a guild.
pub fn guild(guild: &Guild) -> Vec<u8> {
    let own = usize::from(guild.guild > 0 && !guild.emblems.is_empty());
    let mut section = Vec::new();
    tagged(&mut section, 13000, &[]);
    if !guild.cards.is_empty() {
        tagged(&mut section, 13003, &cards(&guild.cards));
    }
    guild.emblems[..own].iter().for_each(|emblem| tagged(&mut section, 13002, emblem));
    tagged(&mut section, 13004, &guild.guild.to_le_bytes());
    tagged(&mut section, 13005, &guild.emblem_version.to_le_bytes());
    guild.emblems[own..].iter().for_each(|emblem| tagged(&mut section, 13002, emblem));
    tagged(&mut section, 13001, &[]);
    section
}

/// Section 22 read back.
pub fn guild_read(section: &[u8]) -> Option<Guild> {
    let mut guild = Guild::default();
    for (tag, bytes) in opened(section, 13000)? {
        match tag {
            13003 => guild.cards = cards_read(bytes),
            13002 => guild.emblems.push(bytes.to_vec()),
            13004 => guild.guild = i32_of(bytes),
            13005 => guild.emblem_version = i32_of(bytes),
            _ => {}
        }
    }
    Some(guild)
}
