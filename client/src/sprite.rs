//! The client's sprites: SPR (the pictures), ACT (how they are laid out per action and frame) and PAL (a recolour),
//! and drawing a frame of them into an RGBA picture.
//!
//! Format facts follow the layouts documented by korangar's `ragnarok-formats` (MIT) and rustnarok-client's
//! `src/sprite.rs` (MIT OR Apache-2.0).

pub type Palette = [[u8; 4]; 256];

pub struct Picture {
    pub width: usize,
    pub height: usize,
    /// Palette indices for `Sprite::indexed`, RGBA for `Sprite::truecolor`; top row first.
    pub pixels: Vec<u8>,
}

pub struct Sprite {
    pub indexed: Vec<Picture>,
    pub truecolor: Vec<Picture>,
    pub palette: Palette,
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let slice = self.bytes.get(self.at..self.at + count).ok_or("the file ends too early")?;
        self.at += count;
        Ok(slice)
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    /// A count that the rest of the file could hold, so a broken file cannot ask for gigabytes.
    fn count(&mut self, each_at_least: usize) -> Result<usize, String> {
        let count = usize::try_from(self.i32()?).map_err(|_| "a negative count")?;
        if count.saturating_mul(each_at_least) > self.bytes.len() - self.at {
            return Err("a count larger than the file".to_owned());
        }
        Ok(count)
    }
}

/// A PAL file, or the palette at the end of a SPR: 256 colours of 4 bytes (red, green, blue, unused).
pub fn parse_palette(bytes: &[u8]) -> Option<Palette> {
    let bytes = bytes.get(..1024)?;
    let mut palette = [[0u8; 4]; 256];
    for (colour, raw) in palette.iter_mut().zip(bytes.chunks_exact(4)) {
        *colour = [raw[0], raw[1], raw[2], 255];
    }
    Some(palette)
}

pub fn parse_spr(bytes: &[u8]) -> Result<Sprite, String> {
    let mut reader = Reader { bytes, at: 0 };
    if reader.take(2)? != b"SP" {
        return Err("not a SPR file".to_owned());
    }
    let version = reader.u16()?;
    let indexed_count = usize::from(reader.u16()?);
    let truecolor_count = if version >= 0x200 { usize::from(reader.u16()?) } else { 0 };

    let mut indexed = Vec::with_capacity(indexed_count.min(4096));
    for _ in 0..indexed_count {
        let (width, height) = (usize::from(reader.u16()?), usize::from(reader.u16()?));
        let pixels = if version >= 0x201 {
            // Run-length coded: a zero byte is followed by how many zeros it stands for.
            let coded = { let size = usize::from(reader.u16()?); reader.take(size)? };
            let mut pixels = Vec::with_capacity(width * height);
            let mut bytes = coded.iter();
            while let Some(&byte) = bytes.next() {
                if byte == 0 {
                    let run = usize::from(*bytes.next().unwrap_or(&1)).max(1);
                    pixels.resize(pixels.len() + run, 0);
                } else {
                    pixels.push(byte);
                }
            }
            pixels.resize(width * height, 0);
            pixels
        } else {
            reader.take(width * height)?.to_vec()
        };
        indexed.push(Picture { width, height, pixels });
    }

    let mut truecolor = Vec::with_capacity(truecolor_count.min(4096));
    for _ in 0..truecolor_count {
        let (width, height) = (usize::from(reader.u16()?), usize::from(reader.u16()?));
        // Stored bottom row first, each pixel as alpha, blue, green, red.
        let raw = reader.take(width * height * 4)?;
        let mut pixels = vec![0u8; raw.len()];
        for row in 0..height {
            for column in 0..width {
                let from = ((height - 1 - row) * width + column) * 4;
                let to = (row * width + column) * 4;
                pixels[to..to + 4].copy_from_slice(&[raw[from + 3], raw[from + 2], raw[from + 1], raw[from]]);
            }
        }
        truecolor.push(Picture { width, height, pixels });
    }

    let palette = if version >= 0x101 { bytes.len().checked_sub(1024).and_then(|at| parse_palette(&bytes[at..])) } else { None };
    Ok(Sprite { indexed, truecolor, palette: palette.unwrap_or([[0, 0, 0, 255]; 256]) })
}

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
    /// Whether `picture` counts in the truecolor pictures.
    pub truecolor: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Frame {
    pub layers: Vec<Layer>,
    /// The point other sprites attach by: a head is moved so that its anchor lies on the body's.
    pub anchor: Option<(i32, i32)>,
}

/// The frames of each action. A character's actions come in eights, one per direction, south first and then
/// clockwise: standing is 0-7, sitting 16-23.
pub struct Act {
    pub actions: Vec<Vec<Frame>>,
}

pub fn parse_act(bytes: &[u8]) -> Result<Act, String> {
    let mut reader = Reader { bytes, at: 0 };
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
                let mut layer = Layer { x, y, picture, mirrored: reader.i32()? != 0, colour: [255; 4], scale: (1.0, 1.0), truecolor: false };
                if version >= 0x200 {
                    layer.colour = reader.take(4)?.try_into().unwrap();
                    let scale_x = reader.f32()?;
                    let scale_y = if version >= 0x204 { reader.f32()? } else { scale_x };
                    layer.scale = (scale_x, scale_y);
                    // The rotation, in degrees: not used by the sprites drawn so far.
                    reader.i32()?;
                    layer.truecolor = reader.i32()? != 0;
                    if version >= 0x205 {
                        reader.take(8)?;
                    }
                }
                layers.push(layer);
            }
            let mut frame = Frame { layers, anchor: None };
            if version >= 0x200 {
                // The frame's sound event.
                reader.i32()?;
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
    Ok(Act { actions })
}

/// An RGBA picture to draw into.
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, rgba: vec![0; width * height * 4] }
    }

    /// Draws one frame with its origin (the feet) at `origin`. `palette` replaces the sprite's own (hair and
    /// cloth colours).
    pub fn draw(&mut self, origin: (i32, i32), sprite: &Sprite, palette: Option<&Palette>, frame: &Frame) {
        let palette = palette.unwrap_or(&sprite.palette);
        for layer in &frame.layers {
            let pictures = if layer.truecolor { &sprite.truecolor } else { &sprite.indexed };
            let Some(picture) = usize::try_from(layer.picture).ok().and_then(|index| pictures.get(index)) else { continue };
            let width = (picture.width as f32 * layer.scale.0.abs()).round() as i32;
            let height = (picture.height as f32 * layer.scale.1.abs()).round() as i32;
            if width <= 0 || height <= 0 {
                continue;
            }
            // The picture's centre goes on the layer's point; an odd size leaves the extra pixel on the left and top.
            let left = origin.0 + layer.x - (width + 1) / 2;
            let top = origin.1 + layer.y - (height + 1) / 2;
            for row in 0..height {
                let to_y = top + row;
                if to_y < 0 || to_y >= self.height as i32 {
                    continue;
                }
                let from_y = (row as usize * picture.height / height as usize).min(picture.height - 1);
                for column in 0..width {
                    let to_x = left + column;
                    if to_x < 0 || to_x >= self.width as i32 {
                        continue;
                    }
                    let across = if layer.mirrored { width - 1 - column } else { column };
                    let from_x = (across as usize * picture.width / width as usize).min(picture.width - 1);
                    let from = from_y * picture.width + from_x;
                    let colour = if layer.truecolor {
                        picture.pixels[from * 4..from * 4 + 4].try_into().unwrap()
                    } else {
                        // Index 0 is the transparent colour.
                        match picture.pixels[from] {
                            0 => continue,
                            index => palette[usize::from(index)],
                        }
                    };
                    self.blend(to_x as usize, to_y as usize, colour, layer.colour);
                }
            }
        }
    }

    fn blend(&mut self, x: usize, y: usize, colour: [u8; 4], tint: [u8; 4]) {
        let alpha = u32::from(colour[3]) * u32::from(tint[3]) / 255;
        if alpha == 0 {
            return;
        }
        let pixel = &mut self.rgba[(y * self.width + x) * 4..][..4];
        let under = u32::from(pixel[3]);
        for channel in 0..3 {
            let over = u32::from(colour[channel]) * u32::from(tint[channel]) / 255;
            pixel[channel] = ((over * alpha + u32::from(pixel[channel]) * under * (255 - alpha) / 255) / (alpha + under * (255 - alpha) / 255).max(1)) as u8;
        }
        pixel[3] = (alpha + under * (255 - alpha) / 255) as u8;
    }
}

/// Where a head goes for its anchor to lie on the body's.
pub fn head_offset(body: &Frame, head: &Frame) -> (i32, i32) {
    match (body.anchor, head.anchor) {
        (Some(body), Some(head)) => (body.0 - head.0, body.1 - head.1),
        _ => (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A SPR 2.1 with one 3x2 indexed picture (`pixels`, run-length coded here) and a palette where colour N is
    /// (N, N+1, N+2).
    fn spr(pixels: &[u8; 6]) -> Vec<u8> {
        let mut coded = Vec::new();
        let mut index = 0;
        while index < pixels.len() {
            if pixels[index] == 0 {
                let run = pixels[index..].iter().take_while(|&&pixel| pixel == 0).count();
                coded.extend([0, run as u8]);
                index += run;
            } else {
                coded.push(pixels[index]);
                index += 1;
            }
        }
        let mut file = b"SP".to_vec();
        file.extend(0x201u16.to_le_bytes());
        file.extend(1u16.to_le_bytes());
        file.extend(0u16.to_le_bytes());
        file.extend(3u16.to_le_bytes());
        file.extend(2u16.to_le_bytes());
        file.extend((coded.len() as u16).to_le_bytes());
        file.extend(coded);
        for colour in 0..=255u8 {
            file.extend([colour, colour.wrapping_add(1), colour.wrapping_add(2), 0]);
        }
        file
    }

    /// An ACT 2.5 with one action of one frame of one layer, and an anchor.
    fn act(x: i32, y: i32, mirrored: bool, anchor: (i32, i32)) -> Vec<u8> {
        let mut file = b"AC".to_vec();
        file.extend(0x205u16.to_le_bytes());
        file.extend(1u16.to_le_bytes());
        file.extend([0u8; 10]);
        file.extend(1i32.to_le_bytes());
        file.extend([0u8; 32]);
        file.extend(1i32.to_le_bytes());
        for value in [x, y, 0, i32::from(mirrored)] {
            file.extend(value.to_le_bytes());
        }
        file.extend([255u8; 4]);
        file.extend(1.0f32.to_le_bytes());
        file.extend(1.0f32.to_le_bytes());
        file.extend(0i32.to_le_bytes());
        file.extend(0i32.to_le_bytes());
        file.extend(3i32.to_le_bytes());
        file.extend(2i32.to_le_bytes());
        file.extend((-1i32).to_le_bytes());
        file.extend(1i32.to_le_bytes());
        file.extend([0u8; 4]);
        file.extend(anchor.0.to_le_bytes());
        file.extend(anchor.1.to_le_bytes());
        file.extend(0i32.to_le_bytes());
        file
    }

    #[test]
    fn reads_a_run_length_coded_sprite() {
        let sprite = parse_spr(&spr(&[0, 0, 5, 7, 0, 0])).unwrap();
        assert_eq!(sprite.indexed.len(), 1);
        assert_eq!((sprite.indexed[0].width, sprite.indexed[0].height), (3, 2));
        assert_eq!(sprite.indexed[0].pixels, [0, 0, 5, 7, 0, 0]);
        assert_eq!(sprite.palette[5], [5, 6, 7, 255]);
        assert!(parse_spr(b"BM").is_err());
        assert!(parse_spr(&spr(&[1; 6])[..12]).is_err());
    }

    #[test]
    fn reads_an_action_file() {
        let act = parse_act(&act(4, -9, true, (1, -30))).unwrap();
        assert_eq!(act.actions.len(), 1);
        let frame = &act.actions[0][0];
        assert_eq!(frame.anchor, Some((1, -30)));
        assert_eq!(frame.layers, [Layer { x: 4, y: -9, picture: 0, mirrored: true, colour: [255; 4], scale: (1.0, 1.0), truecolor: false }]);
        assert!(parse_act(b"SP").is_err());
    }

    /// A count that the file cannot hold is an error, not an allocation.
    #[test]
    fn refuses_impossible_counts() {
        let mut file = act(0, 0, false, (0, 0));
        file[16..20].copy_from_slice(&i32::MAX.to_le_bytes());
        assert!(parse_act(&file).is_err());
    }

    fn pixel(canvas: &Canvas, x: usize, y: usize) -> [u8; 4] {
        canvas.rgba[(y * canvas.width + x) * 4..][..4].try_into().unwrap()
    }

    #[test]
    fn draws_a_layer_centred_on_its_point() {
        let sprite = parse_spr(&spr(&[1, 2, 3, 4, 0, 6])).unwrap();
        let frame = &parse_act(&act(0, 0, false, (0, 0))).unwrap().actions[0][0];
        let mut canvas = Canvas::new(8, 8);
        canvas.draw((4, 4), &sprite, None, frame);
        // 3x2 around (4, 4): columns 2-4, rows 3-4.
        assert_eq!(pixel(&canvas, 2, 3), [1, 2, 3, 255]);
        assert_eq!(pixel(&canvas, 4, 3), [3, 4, 5, 255]);
        assert_eq!(pixel(&canvas, 2, 4), [4, 5, 6, 255]);
        // Index 0 is transparent, and nothing is drawn outside the picture.
        assert_eq!(pixel(&canvas, 3, 4), [0, 0, 0, 0]);
        assert_eq!(pixel(&canvas, 5, 3), [0, 0, 0, 0]);
    }

    #[test]
    fn mirrors_and_recolours() {
        let sprite = parse_spr(&spr(&[1, 2, 3, 4, 0, 6])).unwrap();
        let frame = &parse_act(&act(0, 0, true, (0, 0))).unwrap().actions[0][0];
        let mut other = [[0u8; 4]; 256];
        other[3] = [200, 100, 50, 255];
        other[1] = [9, 9, 9, 255];
        let mut canvas = Canvas::new(8, 8);
        canvas.draw((4, 4), &sprite, Some(&other), frame);
        assert_eq!(pixel(&canvas, 2, 3), [200, 100, 50, 255]);
        assert_eq!(pixel(&canvas, 4, 3), [9, 9, 9, 255]);
    }

    /// Parts of a picture that fall outside the canvas are cut off, on every side.
    #[test]
    fn clips_at_the_edges() {
        let sprite = parse_spr(&spr(&[1, 2, 3, 4, 5, 6])).unwrap();
        let frame = &parse_act(&act(0, 0, false, (0, 0))).unwrap().actions[0][0];
        for origin in [(0, 0), (7, 7), (-50, 3), (3, 500)] {
            let mut canvas = Canvas::new(8, 8);
            canvas.draw(origin, &sprite, None, frame);
        }
        let mut canvas = Canvas::new(8, 8);
        canvas.draw((0, 0), &sprite, None, frame);
        assert_eq!(pixel(&canvas, 0, 0), [6, 7, 8, 255]);
    }

    #[test]
    fn a_head_follows_the_bodys_anchor() {
        let body = Frame { layers: Vec::new(), anchor: Some((2, -40)) };
        let head = Frame { layers: Vec::new(), anchor: Some((-1, -10)) };
        assert_eq!(head_offset(&body, &head), (3, -30));
        assert_eq!(head_offset(&body, &Frame::default()), (0, 0));
    }
}
