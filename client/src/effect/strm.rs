//! The `.str` format ("STRM") as it lies in the file: an effect's layers, each with its textures and its keys
//! (`CEZ2STRParticle::Open`), and where a layer's pictures are.

use super::*;
use crate::bytes::Reader;

/// Where a picture of an effect file is, among the client's pictures of effects: in the file's own folder
/// (`CEZ2STRParticle::Open` keeps the folder of the file's name, and `ClipTexture` asks for every picture
/// with it before its name). `file` is the file's name in the folder of effects, with either slash.
pub fn texture_path(file: &str, texture: &str) -> String {
    match file.rfind(['/', '\\']) {
        Some(end) => format!("{}{texture}", &file[..=end]),
        None => texture.to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pub frame: i32,
    /// 0 sets the layer's state; anything else is added to it every frame.
    pub kind: u32,
    pub transform: Transform,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layer {
    /// The names of the layer's texture files.
    pub textures: Vec<String>,
    pub keys: Vec<Key>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Effect {
    pub frames_per_second: u32,
    /// The last frame.
    pub last_frame: i32,
    pub layers: Vec<Layer>,
}

/// Reads an effect file. `name` turns a texture's name as the file has it (EUC-KR bytes) into text.
pub fn parse(bytes: &[u8], name: impl Fn(&[u8]) -> String) -> Result<Effect, String> {
    let mut reader = Reader::new(bytes);
    if reader.take(4)? != b"STRM" {
        return Err("not an effect file".to_owned());
    }
    let _version = reader.u32()?;
    let frames_per_second = reader.u32()?;
    let last_frame = reader.i32()?;
    let layer_count = reader.u32()? as usize;
    reader.take(16)?;
    let mut layers = Vec::with_capacity(layer_count.min(256));
    for _ in 0..layer_count {
        let texture_count = reader.i32()?.max(0) as usize;
        let mut textures = Vec::with_capacity(texture_count.min(256));
        for _ in 0..texture_count {
            textures.push(name(reader.name(128)?));
        }
        let key_count = reader.i32()?.max(0) as usize;
        let mut keys = Vec::with_capacity(key_count.min(4096));
        for _ in 0..key_count {
            let (frame, kind) = (reader.i32()?, reader.u32()?);
            let [x, y, u, v, us, vs, _u2, _v2, _us2, _vs2] = reader.floats::<10>()?;
            let (ax, ay) = (reader.floats::<4>()?, reader.floats::<4>()?);
            let (aniframe, anitype, anidelta, rz) = (reader.f32()?, reader.u32()?, reader.f32()?, reader.f32()?);
            let colour = reader.floats::<4>()?;
            let (source_blend, destination_blend, _texture_mode) = (reader.u32()?, reader.u32()?, reader.u32()?);
            keys.push(Key { frame, kind, transform: Transform {
                x,
                y,
                u,
                v,
                us,
                vs,
                ax,
                ay,
                aniframe,
                anitype,
                anidelta,
                rz,
                colour,
                source_blend,
                destination_blend
            } });
        }
        layers.push(Layer { textures, keys });
    }
    Ok(Effect { frames_per_second: frames_per_second.max(1), last_frame, layers })
}
