//! A frame of a sprite as the client's effects draw it: not put together into one picture, but layer by layer,
//! each layer's own picture as a quad of its own (`CEffectPrim::RenderSegmentSprites2`,
//! `Contents/Effects/Primitives/EffectPrimitiveRendering.cpp`).
//!
//! What that function takes of a layer (`SpriteFrameCell`): its picture, its place, its two scales, whether it
//! is mirrored, and its turn. What it leaves out: the layer's colour, unless the primitive has `pattern &
//! 0x200` (`Prim::own_colours`): every layer is drawn in the primitive's colour and alpha, or with that pattern
//! in the layer's own (`LayerQuad::colour`). The layers are
//! drawn in the order the frame has them; one with no picture is skipped.
//!
//! - Place: the client keeps a layer's place as its picture's left top corner, worked out as the file is read
//!   (`RelativeCoordinate`, `Resource/Sprites/ActionResource.cpp`): the place in the file less the picture's
//!   half (rounded up) times the layer's scale, divided by that scale and cut to a whole number. The quad then
//!   reaches from that corner to that corner plus the picture's size, both multiplied by the layer's scale and
//!   by the primitive's (`record.scale`): the primitive's size scales a layer's place and size alike.
//! - Mirror: the quad's left and right change places; nothing else does.
//! - Turn: the layer's turn and the primitive's roll together, in whole degrees, about the quad's own middle
//!   (`CRenderObject::DrawQuad`, `Contents/Actors/GameObject.cpp`): a corner `(x, y)` from the middle goes to
//!   `(x cos - y sin, y cos + x sin)`, the screen's y pointing down.
//!
//! (`DrawQuad` also moves every corner half a pixel right and down, which is how its renderer puts a pixel's
//! middle; that is left to the renderer here.)

use super::*;

/// One layer of a frame as an effect draws it: which of the sprite's pictures, and the two triangles of its
/// quad, each corner as its place from the sprite's origin (x to the right, y down, in pixels of the sprite)
/// and where of the picture it shows.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerQuad {
    pub truecolor: bool,
    pub picture: usize,
    pub corners: [[f32; 4]; 6],
    /// The colour the file gives the layer (red, green, blue, alpha): what a primitive with `pattern & 0x200`
    /// draws it in.
    pub colour: [u8; 4],
}

/// `RelativeCoordinate`: a layer's place along one direction as the client keeps it, the picture being so many
/// pixels along it.
fn relative(position: i32, extent: i32, scale: f32) -> i32 {
    let centre = extent / 2 + extent % 2;
    ((position as f32 - centre as f32 * scale) / scale) as i32
}

/// The sine and cosine of a turn as the client's tables have them: of whole degrees.
fn turn(degrees: f32) -> (f32, f32) {
    degrees.trunc().to_radians().sin_cos()
}

/// The quads of a frame's layers, in the frame's order, the primitive being `scale` times its own size and
/// rolled by `roll` degrees. `size_of` gives the size of one of the sprite's pictures (whether it counts among
/// the truecolor ones, and its number); a layer whose picture the sprite lacks is left out.
pub fn layer_quads(frame: &Frame, size_of: impl Fn(bool, usize) -> Option<(usize, usize)>, scale: f32, roll: f32) -> Vec<LayerQuad> {
    let mut quads = Vec::new();
    for layer in &frame.layers {
        let Ok(picture) = usize::try_from(layer.picture) else { continue };
        let Some((wide, tall)) = size_of(layer.truecolor, picture) else { continue };
        let (wide, tall) = (wide as i32, tall as i32);
        // A file of version 2.5 says itself how large the picture is taken to be in placing it.
        let (placed_wide, placed_tall) = layer.size.unwrap_or((wide, tall));
        let (x, y) = (relative(layer.x, placed_wide, layer.scale.0), relative(layer.y, placed_tall, layer.scale.1));
        let (across, down) = (scale * layer.scale.0, scale * layer.scale.1);
        let (mut left, mut right) = (x as f32 * across, (wide + x) as f32 * across);
        let (top, bottom) = (y as f32 * down, (tall + y) as f32 * down);
        if layer.mirrored {
            (left, right) = (right, left);
        }
        let angle = layer.rotation as f32 + roll;
        let (half_wide, half_tall) = ((right - left) * 0.5, (bottom - top) * 0.5);
        let (middle_x, middle_y) = (left + half_wide, top + half_tall);
        let (sin, cos) = if angle == 0.0 { (0.0, 1.0) } else { turn(angle) };
        let corner = |x: f32, y: f32, u: f32, v: f32| [x * cos - y * sin + middle_x, y * cos + x * sin + middle_y, u, v];
        let (high_left, high_right) = (corner(-half_wide, -half_tall, 0.0, 0.0), corner(half_wide, -half_tall, 1.0, 0.0));
        let (low_left, low_right) = (corner(-half_wide, half_tall, 0.0, 1.0), corner(half_wide, half_tall, 1.0, 1.0));
        quads.push(LayerQuad { truecolor: layer.truecolor, picture, corners: [high_left, high_right, low_left, low_left, high_right, low_right], colour: layer.colour });
    }
    quads
}

impl Sprite {
    /// How large one of its pictures is.
    pub fn size_of(&self, truecolor: bool, picture: usize) -> Option<(usize, usize)> {
        let picture = if truecolor { self.truecolor.get(picture) } else { self.indexed.get(picture) }?;
        Some((picture.width, picture.height))
    }

    /// One of its pictures by itself as red, green, blue and alpha, the top row first: a picture of the palette
    /// in the palette's colours, its colour 0 showing nothing.
    pub fn rgba_of(&self, truecolor: bool, picture: usize) -> Option<Vec<u8>> {
        if truecolor {
            return self.truecolor.get(picture).map(|picture| picture.pixels.clone());
        }
        let picture = self.indexed.get(picture)?;
        Some(picture.pixels.iter().flat_map(|&index| if index == 0 { [0; 4] } else { self.palette[usize::from(index)] }).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(x: i32, y: i32) -> Layer {
        Layer { x, y, picture: 0, mirrored: false, colour: [255; 4], scale: (1.0, 1.0), rotation: 0, truecolor: false, size: None }
    }

    /// The quads of a frame of those layers, every picture being 6 wide and 4 tall.
    fn quads(layers: Vec<Layer>, scale: f32, roll: f32) -> Vec<LayerQuad> {
        layer_quads(&Frame { layers, anchor: None, sound: None }, |_, picture| (picture < 2).then_some((6, 4)), scale, roll)
    }

    /// The four corners of a quad as (left top, right top, left bottom, right bottom), places only.
    fn places(quad: &LayerQuad) -> [(f32, f32); 4] {
        [0, 1, 2, 5].map(|corner| (quad.corners[corner][0], quad.corners[corner][1]))
    }

    fn close(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 0.001 && (a.1 - b.1).abs() < 0.001
    }

    #[test]
    fn a_layer_is_put_by_its_pictures_left_top_corner() {
        // 6 by 4 at (10, -7): the corner is 10 - 3 and -7 - 2, and the quad reaches the picture's size from it.
        let quad = &quads(vec![layer(10, -7)], 1.0, 0.0)[0];
        assert_eq!(places(quad), [(7.0, -9.0), (13.0, -9.0), (7.0, -5.0), (13.0, -5.0)]);
        // The picture's corners are on the quad's: left top first.
        assert_eq!(quad.corners.map(|corner| (corner[2], corner[3])), [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)]);
        // An odd size leaves the extra pixel before the place: half of 5 rounded up is 3.
        let odd = layer_quads(&Frame { layers: vec![layer(0, 0)], anchor: None, sound: None }, |_, _| Some((5, 5)), 1.0, 0.0);
        assert_eq!(places(&odd[0])[0], (-3.0, -3.0));
        // A file of version 2.5 says the size the place is taken from.
        let told = quads(vec![Layer { size: Some((10, 8)), ..layer(10, -7) }], 1.0, 0.0);
        assert_eq!(places(&told[0]), [(5.0, -11.0), (11.0, -11.0), (5.0, -7.0), (11.0, -7.0)]);
    }

    #[test]
    fn the_scales_of_the_layer_and_of_the_primitive_scale_place_and_size_alike() {
        // The layer twice as wide and half as tall: the corner is (10 - 3 * 2) / 2 = 2 and (-7 - 2 * 0.5) / 0.5
        // = -16, both then multiplied by the layer's scale; the primitive three times its size on top.
        let quad = &quads(vec![Layer { scale: (2.0, 0.5), ..layer(10, -7) }], 3.0, 0.0)[0];
        assert_eq!(places(quad), [(12.0, -24.0), (48.0, -24.0), (12.0, -18.0), (48.0, -18.0)]);
        // The place in units of the scale is cut to a whole number: (10 - 3 * 4) / 4 = -0.5 is 0.
        let cut = &quads(vec![Layer { scale: (4.0, 1.0), ..layer(10, 0) }], 1.0, 0.0)[0];
        assert_eq!(places(cut)[0], (0.0, -2.0));
    }

    #[test]
    fn a_mirrored_layer_has_its_left_and_right_changed() {
        let quad = &quads(vec![Layer { mirrored: true, ..layer(10, -7) }], 1.0, 0.0)[0];
        // The picture's left edge is drawn at the right.
        assert_eq!(places(quad), [(13.0, -9.0), (7.0, -9.0), (13.0, -5.0), (7.0, -5.0)]);
    }

    #[test]
    fn a_layer_is_turned_about_its_own_middle_by_its_turn_and_the_roll() {
        // A quarter turn clockwise about the middle (10, -7): the left top corner (-3, -2) goes to (2, -3).
        let turned = &quads(vec![Layer { rotation: 90, ..layer(10, -7) }], 1.0, 0.0)[0];
        let expected = [(12.0, -10.0), (12.0, -4.0), (8.0, -10.0), (8.0, -4.0)];
        assert!(places(turned).iter().zip(expected).all(|(got, want)| close(*got, want)), "{:?}", places(turned));
        // The primitive's roll is added to the layer's turn: 60 and 30 make the same quarter.
        let rolled = &quads(vec![Layer { rotation: 60, ..layer(10, -7) }], 1.0, 30.0)[0];
        assert!(places(rolled).iter().zip(expected).all(|(got, want)| close(*got, want)));
        // Whole degrees only, as the client's tables have them.
        let nearly = &quads(vec![Layer { rotation: 0, ..layer(10, -7) }], 1.0, 0.9)[0];
        assert_eq!(places(nearly), [(7.0, -9.0), (13.0, -9.0), (7.0, -5.0), (13.0, -5.0)]);
    }

    #[test]
    fn layers_come_in_the_frames_order_and_one_without_a_picture_is_left_out() {
        let layers = vec![Layer { picture: 1, ..layer(0, 0) }, Layer { picture: -1, ..layer(0, 0) }, Layer { picture: 7, ..layer(0, 0) }, Layer { truecolor: true, ..layer(4, 0) }];
        let quads = quads(layers, 1.0, 0.0);
        assert_eq!(quads.iter().map(|quad| (quad.truecolor, quad.picture)).collect::<Vec<_>>(), [(false, 1), (true, 0)]);
        // A layer's own colour changes nothing of its quad.
        let tinted = self::quads(vec![Layer { colour: [10, 20, 30, 128], ..layer(4, 0) }], 1.0, 0.0);
        assert_eq!(tinted[0].corners, self::quads(vec![layer(4, 0)], 1.0, 0.0)[0].corners);
        // It is handed on for a primitive that draws its layers in their own colours.
        assert_eq!(tinted[0].colour, [10, 20, 30, 128]);
    }

    #[test]
    fn a_picture_of_the_palette_is_given_in_the_palettes_colours() {
        let mut palette = [[0, 0, 0, 255]; 256];
        palette[3] = [30, 40, 50, 255];
        let sprite = Sprite { indexed: vec![Picture { width: 2, height: 1, pixels: vec![0, 3] }], truecolor: vec![Picture { width: 1, height: 1, pixels: vec![1, 2, 3, 4] }], palette };
        assert_eq!(sprite.size_of(false, 0), Some((2, 1)));
        assert_eq!(sprite.rgba_of(false, 0), Some(vec![0, 0, 0, 0, 30, 40, 50, 255]));
        assert_eq!(sprite.rgba_of(true, 0), Some(vec![1, 2, 3, 4]));
        assert_eq!(sprite.rgba_of(false, 1), None);
    }
}
