//! Reader for the client's GRF archives: version 0x200 ("Master of Magic") and 0x300 ("Event Horizon", 64-bit
//! offsets, used by archives over 4 GB).
//!
//! The layout facts for 0x300 were checked against rustnarok-client's `src/grf.rs` (MIT OR Apache-2.0).

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

use flate2::read::ZlibDecoder;

const HEADER_SIZE: u64 = 46;
const FLAG_FILE: u8 = 0x01;
/// Header-and-interval DES ("mixcrypt") and header-only DES, used by old official archives.
const FLAG_MIXCRYPT: u8 = 0x02;
const FLAG_DES_HEADER: u8 = 0x04;

struct Entry {
    archive: usize,
    compressed_size: u32,
    aligned_size: u32,
    size: u32,
    flags: u8,
    offset: u64,
}

pub struct Grf {
    archives: Vec<Mutex<File>>,
    entries: HashMap<String, Entry>,
    /// `data\resnametable.txt`: files asked for under one name and stored under another (`iz_int01.gat` is
    /// `iz_int.gat`), by path under `data\` (maps) or `data\texture\` (minimaps). Seasonal maps are switched here.
    aliases: HashMap<String, String>,
}

impl Grf {
    /// Opens the archives a client's DATA.INI lists (`<n>=<file>`), lowest number first: it wins where two archives
    /// have the same file. Archives that do not open are skipped with a note on stderr.
    pub fn open_client_dir(dir: &Path) -> io::Result<Self> {
        let ini = std::fs::read(dir.join("DATA.INI"))?;
        let mut listed: Vec<(u32, String)> = String::from_utf8_lossy(&ini)
            .lines()
            .filter_map(|line| {
                let (number, name) = line.split_once('=')?;
                Some((number.trim().parse().ok()?, name.trim().to_owned()))
            })
            .collect();
        listed.sort();

        let mut grf = Self { archives: Vec::new(), entries: HashMap::new(), aliases: HashMap::new() };
        for (_, name) in listed {
            match grf.add_archive(&dir.join(&name)) {
                Ok(count) => eprintln!("grf: {name}: {count} files"),
                Err(error) => eprintln!("grf: {name}: skipped ({error})"),
            }
        }
        if grf.archives.is_empty() {
            return Err(invalid("no archive listed in DATA.INI could be opened"));
        }
        grf.load_aliases();
        Ok(grf)
    }

    fn add_archive(&mut self, path: &Path) -> io::Result<usize> {
        let mut file = File::open(path)?;
        let mut header = [0u8; HEADER_SIZE as usize];
        file.read_exact(&mut header)?;
        if !header.starts_with(b"Master of Magic") && !header.starts_with(b"Event Horizon") {
            return Err(invalid("not a GRF file"));
        }
        let version = le32(&header[42..]);
        // 0x300 widens the table offset and every entry offset to 64 bits and puts a zero word before the table sizes.
        let (table_offset, table_pad, fields_len) = match version {
            0x200 => (u64::from(le32(&header[30..])), 0, 17),
            0x300 => (le64(&header[30..]), 4, 21),
            _ => return Err(invalid("only GRF versions 0x200 and 0x300 are supported")),
        };

        file.seek(SeekFrom::Start(HEADER_SIZE + table_offset + table_pad))?;
        let mut sizes = [0u8; 8];
        file.read_exact(&mut sizes)?;
        let mut packed = vec![0; le32(&sizes) as usize];
        file.read_exact(&mut packed)?;
        let table = inflate(&packed, le32(&sizes[4..]))?;

        let archive = self.archives.len();
        let mut count = 0;
        let mut rest = &table[..];
        while let Some(name_end) = rest.iter().position(|&byte| byte == 0) {
            let fields = rest.get(name_end + 1..name_end + 1 + fields_len).ok_or_else(|| invalid("truncated file table"))?;
            let flags = fields[12];
            if flags & FLAG_FILE != 0 {
                let (name, _) = encoding_rs::EUC_KR.decode_without_bom_handling(&rest[..name_end]);
                let entry = Entry {
                    archive,
                    compressed_size: le32(fields),
                    aligned_size: le32(&fields[4..]),
                    size: le32(&fields[8..]),
                    flags,
                    offset: if version == 0x300 { le64(&fields[13..]) } else { u64::from(le32(&fields[13..])) },
                };
                self.entries.entry(normalize(&name)).or_insert(entry);
                count += 1;
            }
            rest = &rest[name_end + 1 + fields_len..];
        }
        self.archives.push(Mutex::new(file));
        Ok(count)
    }

    /// Reads `data\resnametable.txt` (EUC-KR, one `from#to#` per line, `//` comments), as the official client does.
    fn load_aliases(&mut self) {
        let Ok(text) = self.read("data\\resnametable.txt") else { return };
        let (text, _) = encoding_rs::EUC_KR.decode_without_bom_handling(&text);
        self.aliases = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('#');
                let (from, to) = (fields.next()?.trim(), fields.next()?.trim());
                (!from.is_empty() && !to.is_empty() && !from.starts_with("//")).then(|| (normalize(from), normalize(to)))
            })
            .collect();
        eprintln!("grf: {} name aliases", self.aliases.len());
    }

    /// The path a file is stored under: its own when it is there, else the one resnametable.txt gives for it.
    fn stored_path(&self, path: &str) -> String {
        let path = normalize(path);
        if self.entries.contains_key(&path) {
            return path;
        }
        for base in ["data\\texture\\", "data\\"] {
            if let Some(alias) = path.strip_prefix(base).and_then(|rest| self.aliases.get(rest)) {
                return format!("{base}{alias}");
            }
        }
        path
    }

    /// The names resnametable.txt gives to files stored under other names, as paths under `data\`.
    pub fn alias_names(&self) -> impl Iterator<Item = &str> {
        self.aliases.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Every file path, lowercase with backslashes.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// How many files carry each flag byte: shows whether an archive uses the legacy DES schemes.
    pub fn flag_counts(&self) -> Vec<(u8, usize)> {
        let mut counts: HashMap<u8, usize> = HashMap::new();
        for entry in self.entries.values() {
            *counts.entry(entry.flags).or_default() += 1;
        }
        let mut counts: Vec<_> = counts.into_iter().collect();
        counts.sort();
        counts
    }

    /// Reads a file by client path (`data\texture\...`); case and slash direction do not matter.
    pub fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        let entry = self.entries.get(&self.stored_path(path)).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.to_owned()))?;
        if entry.flags & (FLAG_MIXCRYPT | FLAG_DES_HEADER) != 0 {
            return Err(io::Error::new(io::ErrorKind::Unsupported, format!("{path}: DES-encrypted entry (flags {:#x})", entry.flags)));
        }
        let mut data = vec![0; entry.aligned_size as usize];
        {
            let mut file = self.archives[entry.archive].lock().unwrap();
            file.seek(SeekFrom::Start(HEADER_SIZE + entry.offset))?;
            file.read_exact(&mut data)?;
        }
        data.truncate(entry.compressed_size as usize);
        inflate(&data, entry.size)
    }
}

fn inflate(data: &[u8], size: u32) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(size as usize);
    ZlibDecoder::new(data).read_to_end(&mut out)?;
    Ok(out)
}

fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[..4].try_into().unwrap())
}

fn le64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes[..8].try_into().unwrap())
}

pub fn normalize(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}
