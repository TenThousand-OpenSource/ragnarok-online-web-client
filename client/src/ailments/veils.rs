//! The two veils over the screen of the player's own character: the poison's (`EF_POISON`) and the
//! blindness's (`EF_BLIND`), as their panes and how much of each shows.

/// One pane of a veil over the screen: its picture among the client's pictures of effects, and its four
/// corners in `RenderTeiRect`'s order, each as how far across the screen and down it from the veil's middle
/// (in the world's measure) and where of the picture it shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pane {
    pub picture: &'static str,
    pub corners: [([f32; 2], [f32; 2]); 4],
}

/// A pane between four corners, the picture laid as `RenderTeiRect` lays it: its left bottom on the first
/// corner, then its right bottom, right top and left top.
fn pane(picture: &'static str, corners: [[f32; 2]; 4]) -> Pane {
    let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    Pane { picture, corners: std::array::from_fn(|index| (corners[index], uv[index])) }
}

/// The veil of `EF_POISON` (`CRagEffect::POISON`, drawn by `CEffectPrim::RenderGrid`): sixty panes of
/// `poison_f.bmp`, each 30 a side, ten across and six down, around a place 9 above the feet of the player's
/// own character, facing the screen. White, mixed in by its alpha. (The client holds the panes 10 off their
/// plane, which changes nothing of where they show on a screen they face.)
pub fn poison_veil() -> Vec<Pane> {
    let mut panes = Vec::with_capacity(60);
    for across in (-150..150).step_by(30) {
        for down in (-90..90).step_by(30) {
            let (left, top) = (across as f32, down as f32);
            panes.push(pane("poison_f.bmp", [[left, top], [left + 30.0, top], [left + 30.0, top + 30.0], [left, top + 30.0]]));
        }
    }
    panes
}

/// How much of the poison's veil shows after so many of the effect's ticks, of 255 (`CEffectPrim::PrimPoison`:
/// 5 more a tick until it is past 45).
pub fn poison_alpha(ticks: i32) -> f32 {
    (ticks.max(0) * 5).min(50) as f32
}

/// The veil of `EF_BLIND` (`CRagEffect::BLIND` with no flag, drawn by `CEffectPrim::RenderBlind`): around the
/// place 9 above the feet of the player's own character, four panes of `fullb.tga`, 65 a side, that meet in
/// the middle (the picture's left bottom there in each), and round them four panes of `white02.bmp` 100 wide
/// that close the frame. All are drawn in the colour 10, 10, 10 of 255, mixed in by their alpha.
pub fn blind_veil() -> Vec<Pane> {
    let (near, far) = (65.0, 165.0);
    let mut panes = Vec::with_capacity(8);
    for (across, down) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        panes.push(pane("fullb.tga", [[0.0, 0.0], [0.0, down * near], [across * near, down * near], [across * near, 0.0]]));
    }
    panes.push(pane("white02.bmp", [[near, -near], [far, -near], [far, far], [near, far]]));
    panes.push(pane("white02.bmp", [[near, near], [near, far], [-far, far], [-far, near]]));
    panes.push(pane("white02.bmp", [[-near, near], [-far, near], [-far, -far], [-near, -far]]));
    panes.push(pane("white02.bmp", [[-near, -near], [-near, -far], [far, -far], [far, -near]]));
    panes
}

/// The colour of the blindness's veil, each of 255.
pub const BLIND_COLOUR: [f32; 3] = [10.0; 3];

/// How much of the blindness's veil shows after so many of the effect's ticks, of 255
/// (`CEffectPrim::PrimFadeInFollow`: one more a tick).
pub fn blind_alpha(ticks: i32) -> f32 {
    ticks.clamp(0, 255) as f32
}
