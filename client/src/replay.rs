//! Replays: the file a session is recorded into and played from, what of a session is recorded, the options
//! of recording, and the player's clock and speeds. The client's `Replay` (`Source/Client/Replay/Replay.cpp`,
//! `ReplayPacketRecorder.cpp`, `Replay.h`) and what its windows of `Source/UI/Dialogs/UIMessageBox.cpp`
//! (`UIReplayRECControlWnd`, `UISelectReplayDataWnd`, `UIReplayControlWnd`) decide without a page.
//!
//! The file is laid out as the client's `.rrf` is: a text of 100 bytes, the signature and the two keys (the
//! date and the time of the recording), the table of 26 sections at 112, then the sections, each ciphered by
//! the keys: section 2 the header as tagged records (`Replay::BuildMetadata`), section 1 the packets as they
//! came, each after `u32 frame, u32 tick, u16 bytes` (`Replay::Record`). Where it is this project's own: the
//! Windows client keeps in sections 3 to 25 what its session held when the recording began, as copies of its
//! own memory, which this client neither has nor can read. In their place this file has one section the
//! Windows client does not know (`STATE`): who the character is, and every packet of the map server from
//! entering the game to the beginning of the recording, from which this client is what it was then. So a
//! file of one client does not play in the other.

/// The file's signature (`ReplayHeader::Reset`), and how its text begins (`g_replayHeaderText`, version 5).
pub const SIGNATURE: u32 = 0x5241_4705;
const TEXT: &[u8] = b"<< Ragnarok Replay File Version 0.05 >>\r\n \xA1\xD8 Do not modify file! ";
/// Where the signature, the table of sections and the first section stand.
const KEYS_AT: usize = 100;
const TABLE_AT: usize = 112;
const DATA_AT: usize = TABLE_AT + 26 * 10;
/// The sections: the packets, the header, and this project's own with the state the recording began in (in
/// the 24th place of the table, under a number the Windows client has no section of).
const PACKETS: usize = 1;
const HEADER: usize = 2;
const STATE: usize = 24;
const STATE_ID: i16 = 124;

/// The options of recording, by their places among the thirty the header keeps (`ReplayRecordOption`): the
/// tenth is the box "ALL".
pub mod option {
    pub const BASIC: usize = 0;
    pub const EQUIP: usize = 1;
    pub const ITEM: usize = 2;
    pub const SKILL: usize = 3;
    pub const GUILD: usize = 4;
    pub const PARTY: usize = 5;
    pub const CHATTING: usize = 6;
    pub const SHORTCUT: usize = 7;
    pub const STATUS: usize = 8;
    pub const ALL: usize = 9;
}

/// What the header keeps of the options: thirty bytes, all set in a client that has recorded nothing yet.
pub type Options = [u8; 30];

/// A box of the options is ticked or cleared (`UIReplayRECControlWnd::SendMsg`, `ID_CHECK`): "ALL" ticked
/// ticks the nine; the first cleared clears all; any other ticked ticks the first, and cleared clears "ALL".
pub fn ticked(mut options: Options, place: usize, on: bool) -> Options {
    match place {
        option::ALL => {
            if on {
                options[..option::ALL].fill(1);
            }
        }
        option::BASIC => {
            if !on {
                options[1..=option::ALL].fill(0);
            }
        }
        1..=8 => {
            if on {
                options[option::BASIC] = 1;
            } else {
                options[option::ALL] = 0;
            }
        }
        _ => return options,
    }
    options[place] = on.into();
    options
}

/// The packets that are somebody's words: left out of a recording made without the chat (`Replay::Record`).
/// (The last is the whisper as rAthena sends it to this client, ZC_WHISPER02, which the Windows client's list
/// does not name.)
const CHAT: [u16; 8] = [0x008d, 0x008e, 0x0097, 0x0109, 0x017f, 0x02dc, 0x0bde, 0x09de];

/// Whether a packet of the map server goes into a recording made with those options.
pub fn recorded(id: u16, options: &Options) -> bool {
    options[option::CHATTING] != 0 || !CHAT.contains(&id)
}

/// The header of a file (`ReplayHeader`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Header {
    /// When it was recorded: the year, the month, the day, the hour, the minute, the second (the two keys).
    pub when: [u16; 6],
    pub service: i32,
    /// The kind of the service: 0 the main one ("Primary"), which this client writes.
    pub service_type: i32,
    pub name: String,
    pub map: String,
    pub account: u32,
    pub x: i32,
    pub y: i32,
    pub direction: i32,
    /// How long it is, in milliseconds.
    pub duration: u32,
    pub options: Vec<u8>,
}

impl Header {
    /// The two keys the file is ciphered by: the date and the time (`UISelectReplayDataWnd::UpdateInfo` reads
    /// them back so).
    fn keys(&self) -> (u32, u32) {
        let [year, month, day, hour, minute, second] = self.when.map(u32::from);
        (year | (month & 0xff) << 16 | (day & 0xff) << 24, (hour & 0xff) << 8 | (minute & 0xff) << 16 | (second & 0xff) << 24)
    }

    fn of_keys(key0: u32, key1: u32) -> [u16; 6] {
        [key0 as u16, (key0 >> 16) as u8 as u16, (key0 >> 24) as u16, (key1 >> 8) as u8 as u16, (key1 >> 16) as u8 as u16, (key1 >> 24) as u16]
    }
}

/// Who the character of a recording is, as this client needs it to stand in the game again.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Who {
    pub character: u32,
    pub job: u16,
    pub level: u16,
    /// What the list of characters told of it and the map server does not tell anew on entering: the job
    /// level, the zeny, the look (hair, its colour, the clothes' colour, the headgear's three views, the
    /// sex) and the six stats.
    pub job_level: u32,
    pub zeny: u32,
    pub look: [u16; 7],
    pub stats: [u8; 6],
}

impl Who {
    /// As the bytes of its record in the file, and back.
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.job_level.to_le_bytes());
        bytes.extend(self.zeny.to_le_bytes());
        bytes.extend(self.look.iter().flat_map(|part| part.to_le_bytes()));
        bytes.extend(self.stats);
        bytes
    }

    fn told(&mut self, bytes: &[u8]) {
        if bytes.len() < 28 {
            return;
        }
        self.job_level = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        self.zeny = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        for (place, part) in self.look.iter_mut().enumerate() {
            *part = u16::from_le_bytes([bytes[8 + place * 2], bytes[9 + place * 2]]);
        }
        self.stats.copy_from_slice(&bytes[22..28]);
    }
}

/// A file read: its header, who is in it, the packets that came before the recording began, and the
/// recorded ones, each with the milliseconds from the beginning at which it came.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct File {
    pub header: Header,
    pub who: Who,
    pub before: Vec<Vec<u8>>,
    pub packets: Vec<(u32, Vec<u8>)>,
}

/// Ciphers bytes, or brings them back: each whole four of them with the two keys and its place
/// (`Replay::Cipher`); what is left over of the last four stays as it is.
pub fn cipher(key0: u32, key1: u32, bytes: &mut [u8]) {
    let (base, key) = (key0 >> 5, key1 >> 3);
    for (place, word) in bytes.chunks_exact_mut(4).enumerate() {
        let mask = base.wrapping_add(place as u32 + 1).wrapping_mul(key);
        let value = u32::from_le_bytes([word[0], word[1], word[2], word[3]]) ^ mask;
        word.copy_from_slice(&value.to_le_bytes());
    }
}

/// A tagged record: `u16 id, u32 bytes`, then the bytes (`BuildReplayTaggedRecord`).
fn tagged(to: &mut Vec<u8>, id: u16, bytes: &[u8]) {
    to.extend(id.to_le_bytes());
    to.extend((bytes.len() as u32).to_le_bytes());
    to.extend(bytes);
}

/// The tagged records of a section, each its number and its bytes.
fn tags(mut section: &[u8]) -> Vec<(u16, &[u8])> {
    let mut found = Vec::new();
    while section.len() >= 6 {
        let (id, size) = (u16::from_le_bytes([section[0], section[1]]), u32::from_le_bytes([section[2], section[3], section[4], section[5]]) as usize);
        let Some(bytes) = section.get(6..6 + size) else { break };
        found.push((id, bytes));
        section = &section[6 + size..];
    }
    found
}

/// A text in a field of so many bytes, zeros after it.
fn in_field(text: &str, size: usize) -> Vec<u8> {
    let mut field = text.as_bytes()[..text.len().min(size - 1)].to_vec();
    field.resize(size, 0);
    field
}

fn of_field(field: &[u8]) -> String {
    let end = field.iter().position(|byte| *byte == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// Section 2: the header as the client's tagged records, 950 to 973 between 950 and 951
/// (`Replay::BuildMetadata`).
fn metadata(header: &Header, records: u32, end: u32) -> Vec<u8> {
    let mut options = header.options.clone();
    options.resize(30, 0);
    let mut section = Vec::new();
    tagged(&mut section, 950, &[]);
    tagged(&mut section, 961, &header.service.to_le_bytes());
    tagged(&mut section, 962, &header.service_type.to_le_bytes());
    tagged(&mut section, 963, &0i32.to_le_bytes());
    tagged(&mut section, 964, &in_field(&header.name, 64));
    tagged(&mut section, 965, &in_field(&header.map, 20));
    tagged(&mut section, 966, &header.account.to_le_bytes());
    tagged(&mut section, 967, &header.x.to_le_bytes());
    tagged(&mut section, 968, &header.y.to_le_bytes());
    tagged(&mut section, 969, &header.direction.to_le_bytes());
    tagged(&mut section, 970, &header.duration.to_le_bytes());
    tagged(&mut section, 971, &records.to_le_bytes());
    tagged(&mut section, 972, &end.to_le_bytes());
    tagged(&mut section, 973, &options);
    tagged(&mut section, 960, &SIGNATURE.to_le_bytes());
    tagged(&mut section, 951, &[]);
    section
}

/// Writes a file.
pub fn written(file: &File) -> Vec<u8> {
    let (key0, key1) = file.header.keys();
    let mut state = Vec::new();
    tagged(&mut state, 9000, &[]);
    tagged(&mut state, 9010, &file.who.character.to_le_bytes());
    tagged(&mut state, 9011, &file.who.job.to_le_bytes());
    tagged(&mut state, 9012, &file.who.level.to_le_bytes());
    tagged(&mut state, 9013, &file.who.bytes());
    for packet in &file.before {
        tagged(&mut state, 9020, packet);
    }
    tagged(&mut state, 9001, &[]);
    cipher(key0, key1, &mut state);
    let mut packets = Vec::new();
    for (tick, packet) in &file.packets {
        packets.extend(0u32.to_le_bytes());
        packets.extend(tick.to_le_bytes());
        packets.extend((packet.len() as u16).to_le_bytes());
        let mut ciphered = packet.clone();
        cipher(key0, key1, &mut ciphered);
        packets.extend(ciphered);
    }
    let state_at = DATA_AT;
    let packets_at = state_at + state.len();
    let header_at = packets_at + packets.len();
    let mut header = metadata(&file.header, file.packets.len() as u32, header_at as u32);
    cipher(key0, key1, &mut header);
    let mut bytes = vec![0u8; DATA_AT];
    bytes[..TEXT.len()].copy_from_slice(TEXT);
    bytes[95..97].copy_from_slice(b"\r\n");
    bytes[KEYS_AT..KEYS_AT + 4].copy_from_slice(&SIGNATURE.to_le_bytes());
    bytes[KEYS_AT + 4..KEYS_AT + 8].copy_from_slice(&key0.to_le_bytes());
    bytes[KEYS_AT + 8..KEYS_AT + 12].copy_from_slice(&key1.to_le_bytes());
    for (place, id, size, at) in [
        (PACKETS, PACKETS as i16, packets.len(), packets_at),
        (HEADER, HEADER as i16, header.len(), header_at),
        (STATE, STATE_ID, state.len(), state_at),
    ] {
        let entry = &mut bytes[TABLE_AT + place * 10..TABLE_AT + place * 10 + 10];
        entry[..2].copy_from_slice(&id.to_le_bytes());
        entry[2..6].copy_from_slice(&(size as u32).to_le_bytes());
        entry[6..10].copy_from_slice(&(at as u32).to_le_bytes());
    }
    // (The last place of the table is marked as the client marks it: `sections[25].id = 111`.)
    bytes[TABLE_AT + 250..TABLE_AT + 252].copy_from_slice(&111i16.to_le_bytes());
    bytes.extend(state);
    bytes.extend(packets);
    bytes.extend(header);
    bytes
}

/// A section of a file by its place in the table, its cipher taken off; none when the table has none there.
fn section(bytes: &[u8], place: usize, keys: (u32, u32), whole: bool) -> Option<Vec<u8>> {
    let entry = bytes.get(TABLE_AT + place * 10..TABLE_AT + place * 10 + 10)?;
    let size = u32::from_le_bytes([entry[2], entry[3], entry[4], entry[5]]) as usize;
    let at = u32::from_le_bytes([entry[6], entry[7], entry[8], entry[9]]) as usize;
    let mut section = bytes.get(at..at.checked_add(size)?)?.to_vec();
    if whole {
        cipher(keys.0, keys.1, &mut section);
    }
    Some(section)
}

/// The header of a file alone, for the list of files (`ReplayFileList::Open`, `Replay::ParseMetadata`): none
/// for what is no replay.
pub fn header(bytes: &[u8]) -> Option<Header> {
    let word = |at: usize| bytes.get(at..at + 4).map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]));
    if word(KEYS_AT)? != SIGNATURE {
        return None;
    }
    let (key0, key1) = (word(KEYS_AT + 4)?, word(KEYS_AT + 8)?);
    let section = section(bytes, HEADER, (key0, key1), true)?;
    let mut header = Header { when: Header::of_keys(key0, key1), ..Header::default() };
    let whole = |bytes: &[u8]| bytes.get(..4).map_or(0, |word| i32::from_le_bytes([word[0], word[1], word[2], word[3]]));
    for (id, bytes) in tags(&section) {
        match id {
            961 => header.service = whole(bytes),
            962 => header.service_type = whole(bytes),
            964 => header.name = of_field(bytes),
            965 => header.map = of_field(bytes),
            966 => header.account = whole(bytes) as u32,
            967 => header.x = whole(bytes),
            968 => header.y = whole(bytes),
            969 => header.direction = whole(bytes),
            970 => header.duration = whole(bytes) as u32,
            973 => header.options = bytes.to_vec(),
            _ => {}
        }
    }
    Some(header)
}

/// Reads a file of this client's; none for what is no replay, or is one of the Windows client's, which has
/// no state this client can begin from.
pub fn read(bytes: &[u8]) -> Option<File> {
    let header = header(bytes)?;
    let keys = header.keys();
    let entry = bytes.get(TABLE_AT + STATE * 10..TABLE_AT + STATE * 10 + 2)?;
    if i16::from_le_bytes([entry[0], entry[1]]) != STATE_ID {
        return None;
    }
    let state = section(bytes, STATE, keys, true)?;
    let mut file = File { header, ..File::default() };
    let short = |bytes: &[u8]| bytes.get(..2).map_or(0, |word| u16::from_le_bytes([word[0], word[1]]));
    for (id, bytes) in tags(&state) {
        match id {
            9010 => file.who.character = bytes.get(..4).map_or(0, |word| u32::from_le_bytes([word[0], word[1], word[2], word[3]])),
            9011 => file.who.job = short(bytes),
            9012 => file.who.level = short(bytes),
            9013 => file.who.told(bytes),
            9020 => file.before.push(bytes.to_vec()),
            _ => {}
        }
    }
    let packets = section(bytes, PACKETS, keys, false)?;
    let mut rest = packets.as_slice();
    while rest.len() >= 10 {
        let tick = u32::from_le_bytes([rest[4], rest[5], rest[6], rest[7]]);
        let size = usize::from(u16::from_le_bytes([rest[8], rest[9]]));
        let Some(packet) = rest.get(10..10 + size) else { break };
        let mut packet = packet.to_vec();
        cipher(keys.0, keys.1, &mut packet);
        file.packets.push((tick, packet));
        rest = &rest[10 + size..];
    }
    Some(file)
}

/// The number of a packet kept in a file: its first two bytes.
pub fn packet_id(packet: &[u8]) -> Option<u16> {
    packet.get(..2).map(|id| u16::from_le_bytes([id[0], id[1]]))
}

/// The name a recording is kept under when the player gives none: the client's `RagnarokReplay_%04d.rrf`
/// with the first number no file of `names` has.
pub fn next_name<'name>(names: impl IntoIterator<Item = &'name str> + Clone) -> String {
    (1..=9999u32)
        .map(|number| format!("RagnarokReplay_{number:04}.rrf"))
        .find(|name| !names.clone().into_iter().any(|taken| taken.eq_ignore_ascii_case(name)))
        .unwrap_or_else(|| "RagnarokReplay_9999.rrf".to_owned())
}

/// The name of a file the player named (`%s.rrf`).
pub fn named(typed: &str) -> String {
    format!("{}.rrf", typed.trim())
}

/// The time a recording has run, as its window writes it (`UpdateRecordTime`: `%03d:%02d:%02d`).
pub fn clock(milliseconds: u32) -> String {
    let seconds = milliseconds / 1000;
    format!("{:03}:{:02}:{:02}", seconds / 3600, seconds / 60 % 60, seconds % 60)
}

/// Hours, minutes and seconds of so many milliseconds (`Replay::Duration`).
pub fn parts(milliseconds: u32) -> (u32, u32, u32) {
    let seconds = milliseconds / 1000;
    (seconds / 3600, seconds / 60 % 60, seconds % 60)
}

/// The picture of the four that turns while a recording runs: one each quarter of a second.
pub fn turning(milliseconds: u32) -> usize {
    (milliseconds % 1000 / 250) as usize
}

/// The speeds of the player, slowest first. The buttons go between a quarter and four times
/// (`Replay::IncreaseSpeed`, `DecreaseSpeed`); the lines of the message table name seven
/// (`Replay::SpeedLevel`).
pub const SPEEDS: [f32; 7] = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];

/// The speed one step up or down, never under a quarter nor over four times.
pub fn stepped(speed: f32, faster: bool) -> f32 {
    let level = SPEEDS.iter().position(|one| *one == speed).unwrap_or(2);
    SPEEDS[if faster { (level + 1).min(4) } else { level.saturating_sub(1) }]
}

/// The line of the message table that words a speed (1956 to 1962; 1963 for one that is none of the seven).
pub fn speed_line(speed: f32) -> usize {
    SPEEDS.iter().position(|one| *one == speed).map_or(1963, |level| 1956 + level)
}

/// What the kinds of service are called in the list of files (`UISelectReplayDataWnd::UpdateInfo`); none for
/// a kind the client calls a "Data Error".
pub fn service_name(kind: i32) -> Option<&'static str> {
    ["Primary", "Sakray", "Local", "InstantEvent", "PVP", "PK"].get(usize::try_from(kind).ok()?).copied()
}

#[cfg(test)]
mod tests;
