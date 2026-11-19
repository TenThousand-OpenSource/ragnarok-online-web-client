//! `.imf`: which of a character's body and head lies over the other in each frame of each action, and from
//! it the order a character's parts are drawn in (the client's `CImfRes` and `GetSpriteLayerOrder`, as
//! `SpriteLayerRenderer::Render` goes through it).

/// For each of the file's layers (0 the body, 1 the head), for each action, the priority of each frame.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Imf {
    layers: Vec<Vec<Vec<i32>>>,
}

/// `f32 version (1.01 at least), i32 checksum, i32 last layer`, then for each layer `i32 actions` and for each
/// action `i32 frames` and for each frame `i32 priority, i32 x, i32 y` (`CImfRes::Load`).
pub fn parse_imf(bytes: &[u8]) -> Result<Imf, String> {
    let mut reader = crate::bytes::Reader::new(bytes);
    let mut long = || reader.i32();
    if f32::from_bits(long()? as u32) < 1.01 {
        return Err("not an IMF file".to_owned());
    }
    long()?;
    let last = long()?;
    let mut layers = Vec::new();
    for _ in 0..=last.clamp(0, 14) {
        let mut actions = Vec::new();
        for _ in 0..long()?.clamp(0, 4096) {
            let mut frames = Vec::new();
            for _ in 0..long()?.clamp(0, 4096) {
                frames.push(long()?);
                long()?;
                long()?;
            }
            actions.push(frames);
        }
        layers.push(actions);
    }
    Ok(Imf { layers })
}

impl Imf {
    /// The layer that has a priority in a frame of an action (`CImfRes::GetLayer`); none when the file does
    /// not reach that far.
    fn layer(&self, priority: i32, action: usize, frame: usize) -> Option<usize> {
        self.layers.get(priority as usize)?.get(action)?.get(frame)?;
        self.layers.iter().position(|layer| layer.get(action).and_then(|frames| frames.get(frame)) == Some(&priority))
    }

    /// Whether the head is drawn before the body, and so lies under it, in a frame of an action: the layer of
    /// priority 1 is drawn first and that of priority 0 after it.
    pub fn head_under(&self, action: usize, frame: usize) -> bool {
        self.layer(1, action, frame) == Some(1) && self.layer(0, action, frame) == Some(0)
    }
}

/// A part of a character, as the client numbers its layers: 0 the body, 1 the head, 2 to 4 what is worn on the
/// head (low, top, middle), 5 the weapon, 6 the weapon's trail, 7 the shield.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Body,
    Head,
    Weapon,
    Trail,
    Shield,
}

/// The parts of a character but what it wears on its head, in the order they are drawn, the first lowest
/// (`GetSpriteLayerOrder`): seen from the front or a side of the front (the directions 0, 1, 6 and 7) the body
/// and the head, then the weapon, its trail and the shield; seen from behind the shield first, under
/// everything. `head_under` is what the job's `.imf` says of the frame. What is worn on the head is drawn
/// after all of these, by its own priorities (`GetLayerPriority`).
pub fn order(direction: usize, head_under: bool) -> [Part; 5] {
    let (first, second) = if head_under { (Part::Head, Part::Body) } else { (Part::Body, Part::Head) };
    match direction % 8 {
        0 | 1 | 6 | 7 => [first, second, Part::Weapon, Part::Trail, Part::Shield],
        _ => [Part::Shield, first, second, Part::Weapon, Part::Trail],
    }
}

/// The three things worn on the head as the packets list them (low, top, middle), each with its priority, in
/// the order they are drawn: the lowest priority first (the client keeps them in a map by priority;
/// `QueueQuad`). One of no priority (-1) is not drawn at all.
pub fn worn(priorities: [i32; 3]) -> Vec<usize> {
    let mut slots: Vec<usize> = (0..3).filter(|slot| priorities[*slot] != -1).collect();
    slots.sort_by_key(|slot| priorities[*slot]);
    slots
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file of two layers with one action of two frames: the body's priorities, then the head's.
    fn file(body: [i32; 2], head: [i32; 2]) -> Vec<u8> {
        let mut bytes = 1.01f32.to_le_bytes().to_vec();
        bytes.extend([0i32, 1].iter().flat_map(|number| number.to_le_bytes()));
        for layer in [body, head] {
            bytes.extend([1i32, 2].iter().flat_map(|number| number.to_le_bytes()));
            for priority in layer {
                bytes.extend([priority, 5, -9].iter().flat_map(|number| number.to_le_bytes()));
            }
        }
        bytes
    }

    #[test]
    fn the_file_says_which_of_body_and_head_is_under() {
        let imf = parse_imf(&file([1, 0], [0, 1])).unwrap();
        assert!(!imf.head_under(0, 0));
        assert!(imf.head_under(0, 1));
        // A frame the file does not have: the body under the head, as ever.
        assert!(!imf.head_under(0, 2) && !imf.head_under(3, 0));
        assert!(parse_imf(&[0; 8]).is_err());
    }

    #[test]
    fn the_shield_is_under_everything_from_behind() {
        assert_eq!(order(0, false), [Part::Body, Part::Head, Part::Weapon, Part::Trail, Part::Shield]);
        assert_eq!(order(7, true)[..2], [Part::Head, Part::Body]);
        assert_eq!(order(4, false), [Part::Shield, Part::Body, Part::Head, Part::Weapon, Part::Trail]);
    }

    #[test]
    fn what_is_worn_is_drawn_by_its_priority() {
        // The client's defaults: the middle 100, the top 200, the low 300.
        assert_eq!(worn([300, 200, 100]), [2, 1, 0]);
        assert_eq!(worn([150, -1, 100]), [2, 0]);
    }
}
