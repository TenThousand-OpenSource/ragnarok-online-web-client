//! A Granny file as it lies: the header, the table of sections, each section unpacked, and the pointers
//! between them (the file's "fix-ups") — all sections laid one after another as one image in which a pointer
//! is a place.
//!
//! The layout (file format 6 of Granny 2.1, the 32-bit little-endian kind the client's files are) follows
//! the open readers nwn2mdk (`gr2_file.cpp`, Apache-2.0) and opengr2 (`gr2.c`, MPL-2.0), checked against the
//! client's files.

use super::oodle;
use crate::bytes::Reader;
use std::collections::HashMap;

/// The first 16 bytes of a 32-bit little-endian Granny file of this age.
const MAGIC: [u8; 16] = [0xb8, 0x67, 0xb0, 0xca, 0xf8, 0x6d, 0xb1, 0x0f, 0x84, 0x72, 0x8c, 0x7e, 0x5e, 0x19, 0x00, 0x1e];
/// The file's own header comes after the magic, a size, a format and two words kept free.
const HEADER: usize = 0x20;

/// The file's sections as one image, the pointers in it, and where its root object and that object's type
/// are.
pub(super) struct Image {
    pub bytes: Vec<u8>,
    /// Where a pointer lies, to where it points. A place that is not here holds no pointer (nothing).
    pub pointers: HashMap<usize, usize>,
    pub root: usize,
    pub root_type: usize,
}

/// One row of the table of sections.
struct Section {
    packing: u32,
    at: usize,
    packed: usize,
    size: usize,
    stops: [usize; 2],
    fixups_at: usize,
    fixups: usize,
}

impl Image {
    pub(super) fn read(file: &[u8]) -> Result<Self, String> {
        if file.get(..16) != Some(&MAGIC[..]) {
            return Err("not a Granny file of the kind the client has".to_owned());
        }
        let mut header = Reader::new(file);
        header.at = HEADER;
        let version = header.u32()?;
        if version != 6 {
            return Err(format!("a Granny file of format {version}, not 6"));
        }
        let (_size, _crc) = (header.u32()?, header.u32()?);
        let sections_at = HEADER + header.u32()? as usize;
        let count = header.u32()? as usize;
        let root_type = (header.u32()? as usize, header.u32()? as usize);
        let root = (header.u32()? as usize, header.u32()? as usize);
        header.at = sections_at;
        if count > 64 {
            return Err("too many sections".to_owned());
        }
        let sections = (0..count).map(|_| section(&mut header)).collect::<Result<Vec<_>, _>>()?;
        let mut bytes = Vec::new();
        let mut bases = Vec::new();
        for section in &sections {
            bases.push(bytes.len());
            bytes.extend(unpacked(file, section)?);
        }
        let place = |(section, offset): (usize, usize)| -> Result<usize, String> {
            let base = *bases.get(section).ok_or("a pointer into a section that is none")?;
            let place = base + offset;
            if offset > sections[section].size || place > bytes.len() {
                return Err("a pointer past its section".to_owned());
            }
            Ok(place)
        };
        let mut pointers = HashMap::new();
        for (index, section) in sections.iter().enumerate() {
            let mut fixups = Reader::new(file);
            fixups.at = section.fixups_at;
            for _ in 0..section.fixups {
                let from = place((index, fixups.u32()? as usize))?;
                pointers.insert(from, place((fixups.u32()? as usize, fixups.u32()? as usize))?);
            }
        }
        Ok(Self { root: place(root)?, root_type: place(root_type)?, bytes, pointers })
    }

    pub(super) fn reader(&self, at: usize) -> Reader<'_> {
        Reader { bytes: &self.bytes, at: at.min(self.bytes.len()) }
    }

    /// The pointer lying at a place, if one does.
    pub(super) fn pointer(&self, at: usize) -> Option<usize> {
        self.pointers.get(&at).copied()
    }

    /// The text a pointer at a place leads to (it ends with a zero).
    pub(super) fn text(&self, at: usize) -> Option<String> {
        let from = self.pointer(at)?;
        let length = self.bytes[from..].iter().position(|&byte| byte == 0)?;
        Some(String::from_utf8_lossy(&self.bytes[from..from + length]).into_owned())
    }
}

fn section(header: &mut Reader) -> Result<Section, String> {
    let mut word = || header.u32().map(|word| word as usize);
    let (packing, at, packed, size, _alignment) = (word()? as u32, word()?, word()?, word()?, word()?);
    let stops = [word()?, word()?];
    let (fixups_at, fixups) = (word()?, word()?);
    let (_marshalling_at, _marshalling) = (word()?, word()?);
    if size > 64 << 20 || stops[0] > stops[1] || stops[1] > size {
        return Err("a section of a size that cannot be".to_owned());
    }
    Ok(Section { packing, at, packed, size, stops, fixups_at, fixups })
}

/// A section's bytes: as they lie in the file, or unpacked.
fn unpacked(file: &[u8], section: &Section) -> Result<Vec<u8>, String> {
    let packed = file.get(section.at..section.at.saturating_add(section.packed)).ok_or("a section past the file's end")?;
    match section.packing {
        _ if section.size == 0 => Ok(Vec::new()),
        0 if section.packed == section.size => Ok(packed.to_vec()),
        1 => oodle::unpack(packed, [section.stops[0], section.stops[1], section.size]),
        other => Err(format!("a section packed in a way not read here ({other})")),
    }
}
