//! A Granny file describes itself: with every object comes its type, a list of members with their names and
//! kinds, and the objects are read by those names. So a member is asked for by its name and found wherever
//! this file's version of the type has it.
//!
//! The kinds of members and their sizes follow nwn2mdk (`gr2_file.cpp`, Apache-2.0) and opengr2
//! (`typeinfo.c`, MPL-2.0), checked against the client's files.

use super::file::Image;

/// An object in the file: where its type is described and where its bytes are.
#[derive(Clone, Copy)]
pub(super) struct Object<'a> {
    pub image: &'a Image,
    pub kind: usize,
    pub at: usize,
}

/// So many objects of one type, one after another.
#[derive(Clone, Copy)]
pub(super) struct Array<'a> {
    pub image: &'a Image,
    pub kind: usize,
    pub at: usize,
    pub count: usize,
}

/// A place with its turning and its stretching (Granny's `transform`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position: [f32; 3],
    /// A quaternion: x, y, z, w.
    pub orientation: [f32; 4],
    /// Three columns of a matrix of stretching and shearing.
    pub scale: [[f32; 3]; 3],
}

impl Transform {
    pub const IDENTITY: Self =
        Self { position: [0.0; 3], orientation: [0.0, 0.0, 0.0, 1.0], scale: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] };
}

/// What a member holds.
pub(super) enum Field<'a> {
    Numbers(Vec<f64>),
    Text(String),
    Object(Object<'a>),
    Array(Array<'a>),
    /// Objects each of its own place (and, for a list of mixed things, its own type).
    Objects(Vec<Object<'a>>),
    Transform(Transform),
    Nothing,
}

/// One member of a type: its kind (Granny's numbers for them), name, the type it refers to, how many of it
/// lie in a row (0 for one), and where in the object it lies.
pub(super) struct Member {
    pub kind: u32,
    pub name: String,
    pub of: Option<usize>,
    pub width: usize,
    pub at: usize,
}

/// A member's description is 32 bytes: kind, name, type referred to, width, and four words not read here.
const MEMBER: usize = 32;
const INLINE: u32 = 1;
const REFERENCE: u32 = 2;
const REFERENCE_TO_ARRAY: u32 = 3;
const ARRAY_OF_REFERENCES: u32 = 4;
const VARIANT_REFERENCE: u32 = 5;
const REFERENCE_TO_VARIANT_ARRAY: u32 = 7;
const STRING: u32 = 8;
const TRANSFORM: u32 = 9;
const REAL32: u32 = 10;
/// No type nests deeper than this in a file that is sound.
const DEEPEST: usize = 16;

/// The members of the type described at a place, each with where it lies in an object of the type.
pub(super) fn members(image: &Image, kind: usize) -> Result<Vec<Member>, String> {
    Ok(described(image, kind, 0)?.0)
}

/// How many bytes an object of a type takes.
pub(super) fn size(image: &Image, kind: usize) -> Result<usize, String> {
    Ok(described(image, kind, 0)?.1)
}

/// A type's members and the size of an object of it.
fn described(image: &Image, kind: usize, depth: usize) -> Result<(Vec<Member>, usize), String> {
    let mut members = Vec::new();
    let mut at = 0;
    for index in 0..256 {
        let mut reader = image.reader(kind + index * MEMBER);
        let number = reader.u32()?;
        if number == 0 {
            return Ok((members, at));
        }
        let name = image.text(kind + index * MEMBER + 4).unwrap_or_default();
        let of = image.pointer(kind + index * MEMBER + 8);
        reader.at += 8;
        let width = reader.u32()? as usize;
        let one = match number {
            INLINE if depth < DEEPEST => described(image, of.ok_or("a member with no type")?, depth + 1)?.1,
            REFERENCE | STRING | REAL32 | 19 | 20 | 22 => 4,
            REFERENCE_TO_ARRAY | ARRAY_OF_REFERENCES | VARIANT_REFERENCE => 8,
            REFERENCE_TO_VARIANT_ARRAY => 12,
            TRANSFORM => 68,
            11..=14 => 1,
            15..=18 | 21 => 2,
            other => return Err(format!("a member of a kind not known ({other})")),
        };
        members.push(Member { kind: number, name, of, width, at });
        at += one * width.max(1);
    }
    Err("a type with no end".to_owned())
}

impl<'a> Object<'a> {
    /// The member of a name, read; nothing when this type has none of that name.
    pub(super) fn get(&self, name: &str) -> Result<Field<'a>, String> {
        let members = members(self.image, self.kind)?;
        match members.iter().find(|member| member.name == name) {
            Some(member) => self.read(member),
            None => Ok(Field::Nothing),
        }
    }

    pub(super) fn read(&self, member: &Member) -> Result<Field<'a>, String> {
        let image = self.image;
        let at = self.at + member.at;
        let mut reader = image.reader(at);
        let typed = |kind: Option<usize>| kind.ok_or_else(|| format!("{}: a member with no type", member.name));
        Ok(match member.kind {
            INLINE => Field::Object(Object { image, kind: typed(member.of)?, at }),
            REFERENCE => match image.pointer(at) {
                Some(to) => Field::Object(Object { image, kind: typed(member.of)?, at: to }),
                None => Field::Nothing,
            },
            REFERENCE_TO_ARRAY => {
                let count = reader.u32()? as usize;
                self.array(typed(member.of)?, image.pointer(at + 4), count)?
            }
            ARRAY_OF_REFERENCES => {
                let count = reader.u32()? as usize;
                let (kind, list) = (typed(member.of)?, image.pointer(at + 4));
                let pointed = |index: usize| list.and_then(|list| image.pointer(list + index * 4));
                Field::Objects((0..count.min(4096)).filter_map(pointed).map(|at| Object { image, kind, at }).collect())
            }
            VARIANT_REFERENCE => match (image.pointer(at), image.pointer(at + 4)) {
                (Some(kind), Some(to)) => Field::Object(Object { image, kind, at: to }),
                _ => Field::Nothing,
            },
            REFERENCE_TO_VARIANT_ARRAY => {
                reader.at += 4;
                let count = reader.u32()? as usize;
                match image.pointer(at) {
                    Some(kind) => self.array(kind, image.pointer(at + 8), count)?,
                    None => Field::Nothing,
                }
            }
            STRING => image.text(at).map_or(Field::Nothing, Field::Text),
            TRANSFORM => {
                reader.at += 4;
                let (position, orientation) = (reader.floats()?, reader.floats()?);
                Field::Transform(Transform { position, orientation, scale: [reader.floats()?, reader.floats()?, reader.floats()?] })
            }
            number => Field::Numbers((0..member.width.max(1)).map(|_| plain(&mut reader, number)).collect::<Result<_, _>>()?),
        })
    }

    fn array(&self, kind: usize, at: Option<usize>, count: usize) -> Result<Field<'a>, String> {
        let Some(at) = at.filter(|_| count > 0) else { return Ok(Field::Nothing) };
        let array = Array { image: self.image, kind, at, count };
        if array.stride()?.saturating_mul(count) > self.image.bytes.len().saturating_sub(at) {
            return Err("an array past the file's end".to_owned());
        }
        Ok(Field::Array(array))
    }

    /// A member that is a number, or the first of several.
    pub(super) fn number(&self, name: &str) -> Result<f64, String> {
        match self.get(name)? {
            Field::Numbers(numbers) => Ok(numbers[0]),
            _ => Err(format!("{name}: no number")),
        }
    }

    pub(super) fn numbers(&self, name: &str) -> Result<Vec<f64>, String> {
        match self.get(name)? {
            Field::Numbers(numbers) => Ok(numbers),
            Field::Array(array) => array.numbers(),
            Field::Nothing => Ok(Vec::new()),
            _ => Err(format!("{name}: no numbers")),
        }
    }

    pub(super) fn text(&self, name: &str) -> Result<String, String> {
        match self.get(name)? {
            Field::Text(text) => Ok(text),
            _ => Ok(String::new()),
        }
    }

    /// A member that is an object, if it is there.
    pub(super) fn object(&self, name: &str) -> Result<Option<Object<'a>>, String> {
        match self.get(name)? {
            Field::Object(object) => Ok(Some(object)),
            _ => Ok(None),
        }
    }

    /// A member that is several objects, however the file keeps them.
    pub(super) fn objects(&self, name: &str) -> Result<Vec<Object<'a>>, String> {
        match self.get(name)? {
            Field::Objects(objects) => Ok(objects),
            Field::Array(array) => array.objects(),
            Field::Object(object) => Ok(vec![object]),
            _ => Ok(Vec::new()),
        }
    }

    pub(super) fn transform(&self, name: &str) -> Result<Transform, String> {
        match self.get(name)? {
            Field::Transform(transform) => Ok(transform),
            _ => Err(format!("{name}: no transform")),
        }
    }
}

impl<'a> Array<'a> {
    pub(super) fn stride(&self) -> Result<usize, String> {
        size(self.image, self.kind)
    }

    pub(super) fn objects(&self) -> Result<Vec<Object<'a>>, String> {
        let stride = self.stride()?;
        Ok((0..self.count).map(|index| Object { image: self.image, kind: self.kind, at: self.at + index * stride }).collect())
    }

    /// An array of plain numbers (a type of one member), as numbers.
    pub(super) fn numbers(&self) -> Result<Vec<f64>, String> {
        let members = members(self.image, self.kind)?;
        let [member] = &members[..] else { return Err("an array of more than numbers".to_owned()) };
        let mut reader = self.image.reader(self.at);
        (0..self.count * member.width.max(1)).map(|_| plain(&mut reader, member.kind)).collect()
    }
}

/// A plain number of one of Granny's kinds.
fn plain(reader: &mut crate::bytes::Reader, kind: u32) -> Result<f64, String> {
    Ok(match kind {
        REAL32 => reader.f32()? as f64,
        11 | 13 => reader.u8()? as i8 as f64,
        12 | 14 => reader.u8()? as f64,
        15 | 17 => reader.u16()? as i16 as f64,
        16 | 18 => reader.u16()? as f64,
        19 => reader.i32()? as f64,
        20 => reader.u32()? as f64,
        other => return Err(format!("a number of a kind not known ({other})")),
    })
}
