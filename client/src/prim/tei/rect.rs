//! A rectangle as the client's `RenderTeiRect` draws one, and the ways the kinds of `prim/tei.rs` build theirs,
//! each written once: a place across the screen (`to_screen`, the client's `Tei_TowardScreen`), a square that
//! faces the screen (`Part::facing`), a piece of a band of 21 (`Rect::piece`), and a whole band between a row
//! of edges (`Rect::band`, `edges`), which is how every wall of light of these kinds is drawn.

use super::*;

/// A rectangle as `RenderTeiRect` draws one: the picture, the six corners of its two triangles as places from
/// the effect's (east, up, north) with where of the picture each shows, its colour (each of 255) and alpha (of
/// 255), and whether its light is added to what is behind it (the client's `destBlend` of 2) or mixed in by
/// its alpha (6).
#[derive(Clone, Debug, PartialEq)]
pub struct Rect {
    pub picture: &'static str,
    pub corners: [([f32; 3], [f32; 2]); 6],
    pub colour: [u8; 3],
    pub alpha: f32,
    pub adds: bool,
}

impl Rect {
    /// `RenderTeiRect`: four corners in the client's directions, the first two at the picture's bottom (left,
    /// right) and the next two at its top (right, left), as the triangles (1, 2, 3) and (4, 3, 1); `across` is
    /// the part of the picture's width it shows. The alpha is the low byte of the part's.
    pub(super) fn new(picture: &'static str, alpha: i32, corners: [[f32; 3]; 4], across: (f32, f32), colour: [u8; 3], adds: bool) -> Self {
        let corner = |index: usize, u: f32, v: f32| ([corners[index][0], -corners[index][1], corners[index][2]], [u, v]);
        let (first, second, third, fourth) = (corner(0, across.0, 1.0), corner(1, across.1, 1.0), corner(2, across.1, 0.0), corner(3, across.0, 0.0));
        Self { picture, corners: [first, second, third, fourth, third, first], colour, alpha: (alpha & 255) as f32, adds }
    }

    /// `RenderTeiRect` as it draws one piece of a band of 21: the piece of that number shows its 21st of the
    /// picture's width, in the colour and the blend the primitive's `size` stands for.
    pub(super) fn piece(picture: &'static str, piece: i32, alpha: i32, corners: [[f32; 3]; 4], size: f32) -> Self {
        let (colour, adds) = sizes::by_size(size);
        Self::new(picture, alpha, corners, (piece as f32 / 21.0, (piece + 1) as f32 / 21.0), colour, adds)
    }

    /// A band between a row of edges, as the client's drawing functions of a wall build one: each edge its two
    /// ends (the foot and the top, the client's directions), and between each two edges a piece that shows its
    /// 21st of the picture (past the `most`-th piece, the `most`-th's) in the colour and the blend the
    /// primitive's `size` stands for (`Rect::piece`).
    pub(super) fn band(picture: &'static str, alpha: i32, size: f32, most: i32, edges: impl Iterator<Item = ([f32; 3], [f32; 3])>) -> Vec<Self> {
        let mut rects = Vec::new();
        let mut before: Option<([f32; 3], [f32; 3])> = None;
        for (order, (foot, top)) in edges.enumerate() {
            if let Some((foot_before, top_before)) = before {
                rects.push(Self::piece(picture, (order as i32).min(most) - 1, alpha, [foot_before, foot, top, top_before], size));
            }
            before = Some((foot, top));
        }
        rects
    }
}

/// The edges of a band that goes `round` degrees in steps of `step` (over nothing): each as its number, which
/// stays at `most` once it has come to it (the client's own count), and how many degrees on from the band's
/// beginning it stands. A band that `closes` has an edge at its very end too.
pub(super) fn edges(round: i32, step: i32, closes: bool, most: i32) -> impl Iterator<Item = (i32, i32)> {
    (0..if closes { round + 1 } else { round }).step_by(step.max(1) as usize).enumerate().map(move |(order, offset)| ((order as i32).min(most), offset))
}

/// A place so far across the screen and down it from a place of a part (the client's directions): what
/// `Tei_TowardScreen` makes of a point laid flat, its x across and its z down. `right` and `up` are the
/// screen's two directions in the world (east, up, north).
pub(super) fn to_screen(at: [f32; 3], across: f32, down: f32, right: [f32; 3], up: [f32; 3]) -> [f32; 3] {
    let off: [f32; 3] = std::array::from_fn(|axis| right[axis] * across - up[axis] * down);
    [at[0] + off[0], at[1] - off[1], at[2] + off[2]]
}

impl Part {
    /// A square that faces the screen about a place (the client's directions): its four corners `reach` from
    /// the place, the first `turn` degrees round from the screen's right towards its bottom and each next a
    /// quarter on. `right` and `up` are the screen's two directions in the world (east, up, north).
    pub(super) fn facing(place: [f32; 3], reach: f32, turn: i32, right: [f32; 3], up: [f32; 3]) -> [[f32; 3]; 4] {
        std::array::from_fn(|quarter| {
            let angle = ((turn + quarter as i32 * 90) % 360) as f32;
            to_screen(place, reach * cosine(angle), reach * sine(angle), right, up)
        })
    }
}
