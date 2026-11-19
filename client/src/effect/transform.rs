//! A layer of an effect file at a frame: its state, how the client brings it there from key to key
//! (`CEZ2STRParticle`: a key of kind 0 sets the whole state, a key of another kind is added to it every frame
//! while it is the current one), and the quad it comes to on the screen.

use super::*;

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
