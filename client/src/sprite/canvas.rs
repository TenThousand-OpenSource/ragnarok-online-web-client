//! Drawing a frame of a sprite into an RGBA picture: each layer's picture scaled, mirrored, turned and tinted as
//! the frame says, one over the other.
//!
//! A layer is turned about the middle of its picture by whole degrees, clockwise, as the client's
//! `CRenderObject::DrawQuad` turns a quad (`x cos - y sin`, `y cos + x sin`, the screen's y pointing down).

use super::*;

/// An RGBA picture to draw into.
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Layer {
    /// How large its picture is drawn, a picture of that size being scaled as the layer says.
    fn size(&self, picture: (usize, usize)) -> (i32, i32) {
        ((picture.0 as f32 * self.scale.0.abs()).round() as i32, (picture.1 as f32 * self.scale.1.abs()).round() as i32)
    }

    /// What of the frame it covers, from the frame's origin: its left, its top, how wide and how tall; nothing
    /// for a layer that shows nothing. The picture's centre goes on the layer's point; an odd size leaves the
    /// extra pixel on the left and top. One that is turned covers the box around its turned corners.
    pub fn reach(&self, picture: (usize, usize)) -> Option<(i32, i32, i32, i32)> {
        let (width, height) = self.size(picture);
        if width <= 0 || height <= 0 {
            return None;
        }
        let (left, top) = (self.x - (width + 1) / 2, self.y - (height + 1) / 2);
        if self.rotation % 360 == 0 {
            return Some((left, top, width, height));
        }
        let (sin, cos) = (self.rotation as f32).to_radians().sin_cos();
        let (half_wide, half_tall) = (width as f32 * 0.5, height as f32 * 0.5);
        // A hair less than the turned corners reach, so that a quarter turn covers no pixel more than its picture.
        let (wide, tall) = (half_wide * cos.abs() + half_tall * sin.abs() - 0.001, half_wide * sin.abs() + half_tall * cos.abs() - 0.001);
        let (middle_x, middle_y) = (left as f32 + half_wide, top as f32 + half_tall);
        let (low_x, low_y) = ((middle_x - wide).floor() as i32, (middle_y - tall).floor() as i32);
        Some((low_x, low_y, (middle_x + wide).ceil() as i32 - low_x, (middle_y + tall).ceil() as i32 - low_y))
    }

    /// Which pixel of its picture, scaled to `size` but not yet mirrored, shows at a place `x` and `y` from the
    /// frame's origin: for one that is turned, the place turned back about the picture's middle.
    fn source(&self, (width, height): (i32, i32), x: i32, y: i32) -> Option<(i32, i32)> {
        let (left, top) = (self.x - (width + 1) / 2, self.y - (height + 1) / 2);
        if self.rotation % 360 == 0 {
            return Some((x - left, y - top));
        }
        let (sin, cos) = (self.rotation as f32).to_radians().sin_cos();
        let (half_wide, half_tall) = (width as f32 * 0.5, height as f32 * 0.5);
        // The middle of the pixel, from the middle of the picture.
        let (x, y) = (x as f32 + 0.5 - (left as f32 + half_wide), y as f32 + 0.5 - (top as f32 + half_tall));
        let (across, down) = ((x * cos + y * sin + half_wide).floor() as i32, (y * cos - x * sin + half_tall).floor() as i32);
        (across >= 0 && across < width && down >= 0 && down < height).then_some((across, down))
    }
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
            let Some((left, top, wide, tall)) = layer.reach((picture.width, picture.height)) else { continue };
            let (width, height) = layer.size((picture.width, picture.height));
            for row in 0..tall {
                let to_y = origin.1 + top + row;
                if to_y < 0 || to_y >= self.height as i32 {
                    continue;
                }
                for column in 0..wide {
                    let to_x = origin.0 + left + column;
                    if to_x < 0 || to_x >= self.width as i32 {
                        continue;
                    }
                    let Some((across, down)) = layer.source((width, height), left + column, top + row) else { continue };
                    let across = if layer.mirrored { width - 1 - across } else { across };
                    let from_x = (across as usize * picture.width / width as usize).min(picture.width - 1);
                    let from_y = (down as usize * picture.height / height as usize).min(picture.height - 1);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(rotation: i32) -> Layer {
        Layer { x: 0, y: 0, picture: 0, mirrored: false, colour: [255; 4], scale: (1.0, 1.0), rotation, truecolor: false, size: None }
    }

    /// A sprite of one picture, 4 wide and 2 tall, its top row of colour 1 and its bottom row of colour 2.
    fn bar() -> Sprite {
        let mut palette = [[0, 0, 0, 255]; 256];
        (palette[1], palette[2]) = ([10, 0, 0, 255], [20, 0, 0, 255]);
        Sprite { indexed: vec![Picture { width: 4, height: 2, pixels: vec![1, 1, 1, 1, 2, 2, 2, 2] }], truecolor: Vec::new(), palette }
    }

    #[test]
    fn a_layer_covers_its_picture_about_its_point() {
        assert_eq!(layer(0).reach((4, 2)), Some((-2, -1, 4, 2)));
        assert_eq!(layer(360).reach((4, 2)), Some((-2, -1, 4, 2)));
        // Turned a quarter it stands on end about the same middle.
        assert_eq!(layer(90).reach((4, 2)), Some((-1, -2, 2, 4)));
        assert_eq!(Layer { scale: (0.0, 1.0), ..layer(0) }.reach((4, 2)), None);
    }

    #[test]
    fn a_layer_turned_a_quarter_is_drawn_on_end() {
        let mut canvas = Canvas::new(4, 4);
        canvas.draw((2, 2), &bar(), None, &Frame { layers: vec![layer(90)], anchor: None, sound: None });
        let red = |x: usize, y: usize| (canvas.rgba[(y * 4 + x) * 4], canvas.rgba[(y * 4 + x) * 4 + 3]);
        // Clockwise: the top row is now the right column, the bottom row the left one; nothing beside them.
        for y in 0..4 {
            assert_eq!((red(1, y), red(2, y)), ((20, 255), (10, 255)));
            assert_eq!((red(0, y).1, red(3, y).1), (0, 0));
        }
    }

    #[test]
    fn a_layer_not_turned_is_drawn_as_it_is() {
        let mut canvas = Canvas::new(4, 2);
        canvas.draw((2, 1), &bar(), None, &Frame { layers: vec![Layer { mirrored: true, ..layer(0) }], anchor: None, sound: None });
        assert_eq!(canvas.rgba.chunks_exact(4).map(|pixel| pixel[0]).collect::<Vec<_>>(), [10, 10, 10, 10, 20, 20, 20, 20]);
    }
}
