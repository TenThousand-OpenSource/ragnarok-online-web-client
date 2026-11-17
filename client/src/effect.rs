//! The client's effect files (`.str`, "STRM"): layers of textured quads moved, turned, tinted and swapped from key
//! to key. The format and the way a layer is brought from one frame to the next are the client's own
//! (`CEZ2STRParticle` in the decompiled source): a key of kind 0 sets a layer's whole state, a key of another kind
//! is added to it every frame while that key is the current one.

/// The state of a layer at one frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transform {
    /// The middle of the quad, in pixels of a 640 by 480 screen whose point (320, 240) is the effect's place.
    pub x: f32,
    pub y: f32,
    /// Texture coordinates: the corner, and the size taken from there.
    pub u: f32,
    pub v: f32,
    pub us: f32,
    pub vs: f32,
    /// The corners around the middle, clockwise from the top left, in pixels.
    pub ax: [f32; 4],
    pub ay: [f32; 4],
    /// Which of the layer's textures shows (a whole number is taken of it), how it runs on, and by how much.
    pub aniframe: f32,
    pub anitype: u32,
    pub anidelta: f32,
    /// The turn, in 1024ths of a full turn.
    pub rz: f32,
    /// Red, green, blue and alpha, 0 to 255.
    pub colour: [f32; 4],
    /// How it is blended, as Direct3D numbers its blend factors (5 and 6 the usual see-through, 2 as the
    /// destination adds light).
    pub source_blend: u32,
    pub destination_blend: u32,
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

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let slice = self.at.checked_add(count).and_then(|end| self.bytes.get(self.at..end)).ok_or("the file ends too early")?;
        self.at += count;
        Ok(slice)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(self.u32()? as i32)
    }

    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_bits(self.u32()?))
    }

    fn floats<const N: usize>(&mut self) -> Result<[f32; N], String> {
        let mut floats = [0.0; N];
        for float in &mut floats {
            *float = self.f32()?;
        }
        Ok(floats)
    }
}

/// Reads an effect file. `name` turns a texture's name as the file has it (EUC-KR bytes) into text.
pub fn parse(bytes: &[u8], name: impl Fn(&[u8]) -> String) -> Result<Effect, String> {
    let mut reader = Reader { bytes, at: 0 };
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
            let raw = reader.take(128)?;
            let end = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
            textures.push(name(&raw[..end]));
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
            keys.push(Key { frame, kind, transform: Transform { x, y, u, v, us, vs, ax, ay, aniframe, anitype, anidelta, rz, colour, source_blend, destination_blend } });
        }
        layers.push(Layer { textures, keys });
    }
    Ok(Effect { frames_per_second: frames_per_second.max(1), last_frame, layers })
}

impl Layer {
    /// The layer's state at a frame, if it shows then: the client's own walk through the keys, one frame at a
    /// time from the start.
    pub fn at(&self, frame: i32) -> Option<Transform> {
        let count = self.keys.len();
        if count == 0 {
            return None;
        }
        let textures = self.textures.len() as f32;
        // The key the layer is on; `None` once it has run through them all.
        let mut cursor = Some(0usize);
        let mut transform = Transform::default();
        let mut shown = false;
        for now in 0..=frame {
            shown = false;
            let Some(mut on) = cursor else { break };
            let mut key = &self.keys[on];
            let elapsed = now - key.frame;
            if elapsed < 0 {
                continue;
            }
            // On to the next key when its frame has come; past the last one the layer is done.
            let due = if count == 1 { key.frame == now } else if on + 1 < count { elapsed != 0 && self.keys[on + 1].frame == now } else { key.frame == now };
            if due {
                on += 1;
                if on < count - 1 {
                    key = &self.keys[on];
                    cursor = Some(on);
                } else {
                    cursor = None;
                }
            }
            if key.kind == 0 {
                transform = key.transform;
                // A key to add that shares this key's frame starts with the next frame.
                if let Some(on) = cursor.filter(|_| count > 1) {
                    if on + 1 < count && self.keys[on + 1].frame == key.frame {
                        cursor = Some(on + 1);
                    }
                }
            } else {
                let step = &key.transform;
                transform.x += step.x;
                transform.y += step.y;
                transform.u += step.u;
                transform.v += step.v;
                transform.us += step.us;
                transform.vs += step.vs;
                for corner in 0..4 {
                    transform.ax[corner] += step.ax[corner];
                    transform.ay[corner] += step.ay[corner];
                    transform.colour[corner] += step.colour[corner];
                }
                transform.rz += step.rz;
                match step.anitype {
                    1 => transform.aniframe += step.aniframe,
                    2 => transform.aniframe = (transform.aniframe + step.anidelta).min((textures - 1.0).max(0.0)),
                    3 => {
                        transform.aniframe += step.anidelta;
                        if textures > 0.0 && transform.aniframe >= textures {
                            transform.aniframe -= (transform.aniframe / textures).floor() * textures;
                        }
                    }
                    4 => {
                        transform.aniframe -= step.anidelta;
                        if transform.aniframe < 0.0 {
                            transform.aniframe += textures;
                        }
                    }
                    _ => {}
                }
            }
            shown = true;
        }
        shown.then_some(transform)
    }
}

impl Transform {
    /// The quad's corners on the screen, in pixels from the effect's place (x to the right, y down), each with
    /// its texture coordinates: top left, top right, bottom right, bottom left.
    pub fn corners(&self) -> [([f32; 2], [f32; 2]); 4] {
        // 1024 to a full turn.
        let (sine, cosine) = (self.rz / 1024.0 * std::f32::consts::TAU).sin_cos();
        let (right, bottom) = (self.u + self.us, self.v + self.vs);
        let coordinates = [[self.u, self.v], [right, self.v], [right, bottom], [self.u, bottom]];
        std::array::from_fn(|corner| {
            let (x, y) = (self.ax[corner], self.ay[corner]);
            ([x * cosine - y * sine + self.x - 320.0, x * sine + y * cosine + self.y - 240.0], coordinates[corner])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(frame: i32, kind: u32, x: f32) -> Key {
        Key { frame, kind, transform: Transform { x, ..Transform::default() } }
    }

    fn file(layers: &[(&[&str], &[Key])]) -> Vec<u8> {
        let mut bytes = b"STRM".to_vec();
        for value in [0x94u32, 60, 10, layers.len() as u32] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend([0; 16]);
        for (textures, keys) in layers {
            bytes.extend((textures.len() as u32).to_le_bytes());
            for texture in *textures {
                let mut name = texture.as_bytes().to_vec();
                name.resize(128, 0);
                bytes.extend(name);
            }
            bytes.extend((keys.len() as u32).to_le_bytes());
            for key in *keys {
                bytes.extend(key.frame.to_le_bytes());
                bytes.extend(key.kind.to_le_bytes());
                let transform = &key.transform;
                for float in [transform.x, transform.y, transform.u, transform.v, transform.us, transform.vs, 0.0, 0.0, 0.0, 0.0] {
                    bytes.extend(float.to_le_bytes());
                }
                for float in transform.ax.iter().chain(&transform.ay) {
                    bytes.extend(float.to_le_bytes());
                }
                bytes.extend(transform.aniframe.to_le_bytes());
                bytes.extend(transform.anitype.to_le_bytes());
                bytes.extend(transform.anidelta.to_le_bytes());
                bytes.extend(transform.rz.to_le_bytes());
                for float in transform.colour {
                    bytes.extend(float.to_le_bytes());
                }
                for value in [transform.source_blend, transform.destination_blend, 0] {
                    bytes.extend(value.to_le_bytes());
                }
            }
        }
        bytes
    }

    #[test]
    fn reads_layers_textures_and_keys() {
        let keys = [key(0, 0, 320.0), key(0, 1, 2.0), key(5, 0, 400.0)];
        let effect = parse(&file(&[(&["a.bmp", "b.bmp"], &keys)]), |raw| String::from_utf8_lossy(raw).into_owned()).unwrap();
        assert_eq!((effect.frames_per_second, effect.last_frame, effect.layers.len()), (60, 10, 1));
        assert_eq!(effect.layers[0].textures, ["a.bmp", "b.bmp"]);
        assert_eq!(effect.layers[0].keys, keys);
        assert!(parse(b"nope", |_| String::new()).is_err());
    }

    /// A key that sets, a key that adds from the same frame on, and the key that ends the layer.
    #[test]
    fn a_layer_is_set_then_added_to_frame_by_frame_then_gone() {
        let layer = Layer { textures: vec![], keys: vec![key(2, 0, 100.0), key(2, 1, 5.0), key(6, 0, 0.0)] };
        assert_eq!(layer.at(1), None);
        assert_eq!(layer.at(2).map(|transform| transform.x), Some(100.0));
        assert_eq!(layer.at(3).map(|transform| transform.x), Some(105.0));
        assert_eq!(layer.at(5).map(|transform| transform.x), Some(115.0));
        // On the last key's frame the layer still shows what it had; after it, nothing.
        assert!(layer.at(6).is_some());
        assert_eq!(layer.at(7), None);
    }

    #[test]
    fn corners_are_around_the_middle_and_turn_with_it() {
        let flat = Transform { x: 320.0, y: 240.0, us: 1.0, vs: 1.0, ax: [-10.0, 10.0, 10.0, -10.0], ay: [-5.0, -5.0, 5.0, 5.0], ..Transform::default() };
        assert_eq!(flat.corners()[0], ([-10.0, -5.0], [0.0, 0.0]));
        assert_eq!(flat.corners()[2], ([10.0, 5.0], [1.0, 1.0]));
        // A quarter turn (256 of 1024) sends the top left corner to the top right.
        let turned = Transform { rz: 256.0, ..flat };
        let [x, y] = turned.corners()[0].0;
        assert!((x - 5.0).abs() < 1e-3 && (y + 10.0).abs() < 1e-3, "{x} {y}");
    }
}
