//! RSW: a map's water with its waves, its sunlight, where each model stands and how it plays its animation, and
//! the effects and sounds the map places (the client's `C3dWorldRes::Load`, Source/Resource/World/WorldResource.cpp).

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Water {
    /// The height of the surface, in the world's space (pointing up).
    pub level: f32,
    /// Which of the water textures.
    pub kind: i32,
    /// How many of the client's frames each picture of the water shows.
    pub frame_ticks: u32,
    /// The waves: how high they go, how many degrees their turn goes on a frame of the client's, and how many
    /// degrees lie between one cell of the ground and the next (`waveHeight`, `waveSpeed`, `wavePitch`; the
    /// client keeps the last two as whole numbers).
    pub wave_height: f32,
    pub wave_speed: i32,
    pub wave_pitch: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    /// The direction to the sun, in the world's space.
    pub direction: Vector,
    pub diffuse: Vector,
    pub ambient: Vector,
}

/// A model standing on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    /// The model's file, by its path under `data\model\`.
    pub model: String,
    /// In the file's space: from the map's middle, y down.
    pub position: Vector,
    /// Degrees around each axis.
    pub rotation: Vector,
    pub scale: Vector,
    /// How its animation plays (the client's `C3dActor::playback`: 1 once and it stays at its end, 2 over and
    /// over, anything else not at all) and how fast (`animationSpeed`, which the client takes as 1 when it is
    /// not over nothing and up to 100).
    pub playback: i32,
    pub speed: f32,
}

/// An effect the map places (the client's `WorldEffectRecord`): which of the client's effects, where (in the
/// file's space: from the map's middle, y down), how many of the client's frames lie between two launches, and
/// the four figures the effect is told.
#[derive(Debug, Clone, PartialEq)]
pub struct MapEffect {
    pub effect: u32,
    pub position: Vector,
    pub delay: f32,
    pub figures: [f32; 4],
}

/// A sound the map places (the client's `WorldSoundRecord`): the file among the client's sounds, where (in the
/// file's space), how loud, the size of what sounds (`width`, `height`), how far it is heard, and how many
/// seconds lie between two playings.
#[derive(Debug, Clone, PartialEq)]
pub struct MapSound {
    pub file: String,
    pub position: Vector,
    pub volume: f32,
    pub size: (i32, i32),
    pub range: f32,
    pub cycle: f32,
}

pub struct Rsw {
    pub water: Water,
    pub light: Light,
    pub placements: Vec<Placement>,
    pub effects: Vec<MapEffect>,
    pub sounds: Vec<MapSound>,
}

impl Rsw {
    /// Reads a RSW of versions 1.2 to 2.5. Names are EUC-KR and given to `name` to decode.
    pub fn parse(bytes: &[u8], name: impl Fn(&[u8]) -> String) -> Result<Self, String> {
        let mut reader = Reader::new(bytes);
        if reader.take(4)? != b"GRSW" {
            return Err("not a RSW file".to_owned());
        }
        reader.version = reader.u16()?.swap_bytes();
        let version = reader.version;
        if !(0x0102..=0x0205).contains(&version) {
            return Err(format!("RSW version {}.{} is not supported", version >> 8, version & 0xff));
        }
        if version >= 0x0205 {
            reader.take(4)?;
        }
        if version >= 0x0202 {
            reader.take(1)?;
        }
        // The names of the map's other files: its settings, ground, walkability and, later, source.
        reader.take(if version >= 0x0104 { 160 } else { 120 })?;
        let water = water(&mut reader)?;
        let mut light = Light { direction: sun(45, 45), diffuse: [1.0; 3], ambient: [0.3; 3] };
        if version >= 0x0105 {
            let (longitude, latitude) = (reader.i32()?, reader.i32()?);
            light = Light { direction: sun(longitude, latitude), diffuse: reader.vector()?, ambient: reader.vector()? };
        }
        if version >= 0x0107 {
            reader.f32()?;
        }
        if version >= 0x0106 {
            reader.take(16)?;
        }

        let count = reader.count(4)?;
        let (mut placements, mut effects, mut sounds) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..count {
            match reader.i32()? {
                1 => placements.push(placement(&mut reader, &name)?),
                // A light: name, position, colour, range. The client reads it and leaves it.
                2 => {
                    reader.take(80 + 12 + 12 + 4)?;
                }
                3 => sounds.push(sound(&mut reader, &name)?),
                4 => {
                    reader.take(80)?;
                    let position = reader.vector()?;
                    let (effect, delay) = (reader.i32()?, reader.f32()?);
                    let figures = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
                    effects.extend(u32::try_from(effect).ok().map(|effect| MapEffect { effect, position, delay, figures }));
                }
                other => return Err(format!("an object of unknown kind {other}")),
            }
        }
        Ok(Self { water, light, placements, effects, sounds })
    }
}

/// The water: its level, then (from 1.8) its kind and waves, then (from 1.9) how long a picture shows. An older
/// file has the client's own waves: 1 high, 2 degrees a frame, 50 degrees a cell.
fn water(reader: &mut Reader) -> Result<Water, String> {
    let mut water = Water { level: 0.0, kind: 0, frame_ticks: 3, wave_height: 1.0, wave_speed: 2, wave_pitch: 50 };
    if reader.version >= 0x0103 {
        // Heights point down in the file.
        water.level = -reader.f32()?;
    }
    if reader.version >= 0x0108 {
        water.kind = reader.i32()?;
        (water.wave_height, water.wave_speed, water.wave_pitch) = (reader.f32()?, reader.f32()? as i32, reader.f32()? as i32);
    }
    if reader.version >= 0x0109 {
        water.frame_ticks = (reader.i32()?.max(1)) as u32;
    }
    Ok(water)
}

/// A model: from 1.3 its own name, whether and how fast its animation plays and how it blocks; then its file,
/// the name of a node, and where it stands.
fn placement(reader: &mut Reader, name: &impl Fn(&[u8]) -> String) -> Result<Placement, String> {
    let (mut playback, mut speed) = (0, 1.0);
    if reader.version >= 0x0103 {
        reader.take(40)?;
        (playback, speed) = (reader.i32()?, reader.f32()?);
        reader.take(4)?;
        if !(speed > 0.0 && speed <= 100.0) {
            speed = 1.0;
        }
    }
    let model = name(reader.name(80)?);
    reader.take(80)?;
    Ok(Placement { model, position: reader.vector()?, rotation: reader.vector()?, scale: reader.vector()?, playback, speed })
}

/// A sound: name, file, position, volume, size, range and, from 2.0, how often it plays (4 seconds before).
fn sound(reader: &mut Reader, name: &impl Fn(&[u8]) -> String) -> Result<MapSound, String> {
    reader.take(80)?;
    let file = name(reader.name(80)?);
    let position = reader.vector()?;
    let (volume, size, range) = (reader.f32()?, (reader.i32()?, reader.i32()?), reader.f32()?);
    let cycle = if reader.version >= 0x0200 { reader.f32()? } else { 4.0 };
    Ok(MapSound { file, position, volume, size, range, cycle })
}

/// The direction to the sun from its longitude and latitude in degrees, in the world's space.
fn sun(longitude: i32, latitude: i32) -> Vector {
    let (longitude, latitude) = ((longitude as f32).to_radians(), (latitude as f32).to_radians());
    // In the file's space the light comes from (-cos lon sin lat, -cos lat, -sin lon sin lat), with y down.
    [-longitude.cos() * latitude.sin(), latitude.cos(), -longitude.sin() * latitude.sin()]
}
