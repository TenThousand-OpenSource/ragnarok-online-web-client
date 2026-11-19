//! ACT: how a sprite's pictures are laid out in each frame of each action, the frames' sounds and how long
//! they show; and what is read off the frames: where a head goes on a body, which frames' sounds are due.
//!
//! Format facts follow the layouts documented by korangar's `ragnarok-formats` (MIT) and rustnarok-client's
//! `src/sprite.rs` (MIT OR Apache-2.0).

use crate::bytes::Reader;

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// Where the picture's centre goes, from the frame's origin (the feet).
    pub x: i32,
    pub y: i32,
    /// Which picture; negative for none.
    pub picture: i32,
    pub mirrored: bool,
    pub colour: [u8; 4],
    pub scale: (f32, f32),
    /// How far the picture is turned about its middle, in degrees, clockwise.
    pub rotation: i32,
    /// Whether `picture` counts in the truecolor pictures.
    pub truecolor: bool,
    /// How large the file says the picture is (files of version 2.5 on): what its place is taken from.
    pub size: Option<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Frame {
    pub layers: Vec<Layer>,
    /// The point other sprites attach by: a head is moved so that its anchor lies on the body's.
    pub anchor: Option<(i32, i32)>,
    /// Which of the file's sounds starts with this frame.
    pub sound: Option<usize>,
}

/// The frames of each action. A character's actions come in eights, one per direction, south first and then
/// clockwise: standing is 0-7, sitting 16-23.
pub struct Act {
    pub actions: Vec<Vec<Frame>>,
    /// The names of the sound files the frames name by number.
    pub sounds: Vec<String>,
    /// How long a frame of each action shows, in units of 25 milliseconds (4 where the file does not say).
    pub delays: Vec<f32>,
}

pub fn parse_act(bytes: &[u8]) -> Result<Act, String> {
    let mut reader = Reader::new(bytes);
    if reader.take(2)? != b"AC" {
        return Err("not an ACT file".to_owned());
    }
    let version = reader.u16()?;
    let action_count = usize::from(reader.u16()?);
    reader.take(10)?;

    let mut actions = Vec::with_capacity(action_count.min(1024));
    for _ in 0..action_count {
        let frame_count = reader.count(36)?;
        let mut frames = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            // Two unused rectangles.
            reader.take(32)?;
            let layer_count = reader.count(16)?;
            let mut layers = Vec::with_capacity(layer_count);
            for _ in 0..layer_count {
                let (x, y, picture) = (reader.i32()?, reader.i32()?, reader.i32()?);
                let mut layer = Layer { x, y, picture, mirrored: reader.i32()? != 0, colour: [255; 4], scale: (1.0, 1.0), rotation: 0, truecolor: false, size: None };
                if version >= 0x200 {
                    layer.colour = reader.take(4)?.try_into().unwrap();
                    let scale_x = reader.f32()?;
                    let scale_y = if version >= 0x204 { reader.f32()? } else { scale_x };
                    layer.scale = (scale_x, scale_y);
                    layer.rotation = reader.i32()?;
                    layer.truecolor = reader.i32()? != 0;
                    if version >= 0x205 {
                        layer.size = Some((reader.i32()?, reader.i32()?));
                    }
                }
                layers.push(layer);
            }
            let mut frame = Frame { layers, anchor: None, sound: None };
            if version >= 0x200 {
                // The frame's sound event: -1 for none.
                frame.sound = usize::try_from(reader.i32()?).ok();
            }
            if version >= 0x203 {
                for index in 0..reader.count(16)? {
                    reader.take(4)?;
                    let point = (reader.i32()?, reader.i32()?);
                    reader.i32()?;
                    if index == 0 {
                        frame.anchor = Some(point);
                    }
                }
            }
            frames.push(frame);
        }
        actions.push(frames);
    }
    // The sounds' names, 40 bytes each. A file cut short here still has its pictures.
    let mut sounds = Vec::new();
    if version >= 0x201 {
        if let Ok(count) = reader.count(40) {
            for _ in 0..count {
                let Ok(name) = reader.name(40) else { break };
                sounds.push(String::from_utf8_lossy(name).into_owned());
            }
        }
    }
    let mut delays = Vec::new();
    if version >= 0x202 {
        for _ in 0..actions.len() {
            let Ok(delay) = reader.f32() else { break };
            delays.push(delay);
        }
    }
    Ok(Act { actions, sounds, delays })
}

/// Where a head goes for its anchor to lie on the body's.
pub fn head_offset(body: &Frame, head: &Frame) -> (i32, i32) {
    match (body.anchor, head.anchor) {
        (Some(body), Some(head)) => (body.0 - head.0, body.1 - head.1),
        _ => (0, 0),
    }
}

/// The frames of an action whose sounds are due as an actor's picture goes on to `frame` of `action`, having
/// last sounded at `last` (an action without its side, and a frame of it), the action having `count` frames
/// (`CRenderObject::PlayFrameSounds`): those passed since, up to the new one; of an action just begun, those
/// after its first; and when the frames began anew, those that were left to its end. None while the picture
/// stays.
pub fn frames_sounding(last: Option<(usize, usize)>, action: usize, frame: usize, count: usize) -> std::ops::Range<usize> {
    let before = last.filter(|(was, _)| *was == action).map_or(0, |(_, frame)| frame);
    match before.cmp(&frame) {
        std::cmp::Ordering::Less => before + 1..frame + 1,
        std::cmp::Ordering::Equal => 0..0,
        std::cmp::Ordering::Greater => before.min(count.saturating_sub(1)) + 1..count,
    }
}
