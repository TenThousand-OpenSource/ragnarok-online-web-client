//! What an effect drawn by code shows now, a piece at a time (`Piece`), for the frame to draw
//! (`world/frame/effects.rs`).

use super::*;

/// What an effect shows, a piece at a time.
pub(in crate::web::world) enum Piece {
    /// Two triangles of one of the client's pictures of effects: the picture, each of the six corners as its
    /// place in the world and where of the picture it shows, the colour of each corner, and whether its light
    /// is added to what is behind it.
    Picture(&'static str, [(Vector, [f32; 2]); 6], [[f32; 4]; 6], bool),
    /// A frame of the first action of a sprite, facing the screen: the sprite's path among the client's
    /// sprites, the frame, where in the world the sprite's origin is, how many times its own size it is, how
    /// far it is turned (the client's degrees), the colour every layer of the frame is drawn in, and whether
    /// its light is added to what is behind it.
    /// `action` is which action of the sprite the frame is of, and `own` whether every layer is drawn in the
    /// colour its file gives it instead of `colour`; `lit`, whether every layer is in the red, green and blue
    /// its file gives it at the alpha of `colour` (`RenderSegmentSpriteGlow`). With a `rim` every layer is in
    /// the colour and of the size that makes of the layer's own (`RenderSegmentSprites`, `crate::prim::Rim`).
    Sprite {
        sprite: &'static str,
        action: usize,
        frame: usize,
        at: Vector,
        scale: f32,
        roll: f32,
        colour: [f32; 4],
        own: bool,
        lit: bool,
        adds: bool,
        rim: Option<crate::prim::Rim>
    },
}

impl Burst {
    /// What it shows now, a piece at a time: around `place`, the feet of the one it is on; the flat kinds across
    /// the screen along `right` and `up`. A sprite is in the primitive's colour and at the alpha of each of
    /// its drawings (the client's `record.color`). `yaw` is the camera's yaw, the client's degrees: a sprite
    /// that points a way in the world is turned on the screen by that way less the yaw (`Prim::rolled`). A
    /// primitive that stays in the world where it was launched is drawn around where the effect was then.
    pub(in crate::web::world) fn draw(&self, place: Vector, (right, up, yaw): (Vector, Vector, f32), mut draw: impl FnMut(Piece)) {
        for prim in &self.prims {
            let colour = |alpha: f32| [prim.colour[0] / 255.0, prim.colour[1] / 255.0, prim.colour[2] / 255.0, alpha.trunc() / 255.0];
            let place = prim.left.map_or(place, |left| std::array::from_fn(|axis| place[axis] - self.course.walked[axis] + left[axis]));
            // (The kind 167 is drawn at each of its parts, every layer twice in colours made of its own.)
            // The primitive's sprite as a piece: where, how large, its colour, how it is coloured and lit and
            // laid on (`own`, `lit`, `adds`), and its rim.
            let sprite = |at, scale, colour, (own, lit, adds): (bool, bool, bool), rim| Piece::Sprite {
                sprite: prim.picture,
                action: prim.action,
                frame: prim.motion,
                at,
                scale,
                roll: prim.rolled(yaw),
                colour,
                own,
                lit,
                adds,
                rim
            };
            for (at, scale, rim) in prim.fourfold() {
                draw(sprite(along(place, at, 1.0), scale, [1.0; 4], (true, false, prim.adds), Some(rim)));
            }
            for seen in prim.seen().into_iter().filter(|seen| prim.shows_sprite() && !prim.is_fourfold() && seen.alpha >= 1.0) {
                let at = [place[0] + seen.at[0], place[1] + seen.at[1], place[2] + seen.at[2]];
                draw(sprite(at, seen.scale, colour(seen.alpha), (prim.own_colours, prim.glows(), prim.adds), None));
                // (The kind 153 is drawn once more, its light added: `RenderSegmentSpriteGlow`.)
                if prim.glows() {
                    draw(sprite(at, seen.scale, colour(seen.alpha), (false, true, true), None));
                }
            }
            for lit in prim.lit(right, up).into_iter().filter(|lit| lit.alpha >= 1.0) {
                let corners = lit.corners.map(|(at, uv)| ([place[0] + at[0], place[1] + at[1], place[2] + at[2]], uv));
                draw(Piece::Picture(lit.picture, corners, [colour(lit.alpha); 6], prim.adds));
            }
            for rect in prim.rects(right, up).into_iter().filter(|rect| rect.alpha >= 1.0) {
                let corners = rect.corners.map(|(at, uv)| ([place[0] + at[0], place[1] + at[1], place[2] + at[2]], uv));
                draw(Piece::Picture(
                    rect.picture,
                    corners,
                    [[
                        f32::from(rect.colour[0]) / 255.0,
                        f32::from(rect.colour[1]) / 255.0,
                        f32::from(rect.colour[2]) / 255.0,
                        rect.alpha / 255.0
                    ]; 6],
                    rect.adds
                ));
            }
            // (A kite has a corner that shows whatever its alpha is.)
            if prim.alpha < 1.0 && !prim.glares(1) {
                continue;
            }
            for pair in prim.solid(place).chunks_exact(6) {
                draw(Piece::Picture(prim.shown(), std::array::from_fn(|index| pair[index]), [colour(prim.alpha); 6], prim.adds));
            }
            for pair in prim.corners().chunks_exact(6) {
                let corner = |index: usize| (
                    along(along(place, right, pair[index][0] * SPRITE_SCALE), up, -pair[index][1] * SPRITE_SCALE),
                    [pair[index][2], pair[index][3]]
                );
                let colours = std::array::from_fn(|index| if prim.glares(index) { [1.0; 4] } else { colour(prim.alpha) });
                draw(Piece::Picture(prim.picture, std::array::from_fn(corner), colours, prim.adds));
            }
        }
    }
}
