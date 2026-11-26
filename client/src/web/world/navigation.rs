//! The way the navigation marks on the ground (the client's `CNavigationManager::DrawRoute`): an arrow on
//! every other cell of the way, each pointing along it, and a mark turning on its last cell; and the ground of
//! the map one is on, for the navigation's own search of a way (`crate::navigation::Ground`).

use std::collections::{HashMap, HashSet};

use super::*;
use crate::map::Gat;

/// How far over the ground the marks lie, so that the ground does not cut through them (the client draws
/// them nearer by a hair of the depth instead: `GroundGeometry::DrawCellDecal`).
const OVER: f32 = 0.4;

/// The way to mark: its cells from where the character stood, which of the client's eight sets of arrows
/// marks it (`viewIcon`), and whether its end is the place sought itself (the mark there is white then, else
/// light blue: `drawDestination`).
pub struct Marked {
    pub cells: Vec<(i32, i32)>,
    pub icon: u8,
    pub at_goal: bool,
}

/// What the world keeps of the navigation's marks: the way, and the pictures read for it.
#[derive(Default)]
pub(super) struct Marks {
    way: Option<Marked>,
    pictures: HashMap<String, Rc<[wgpu::BindGroup; 2]>>,
    asked: HashSet<String>,
}

/// The picture of an arrow of a set, by the way it points as the client's search numbers the steps (0 north,
/// then round by the west: `CPathFinder::FindPath`), and of the mark at the way's end at one of its four
/// moments.
fn arrow(icon: u8, points: usize) -> String {
    format!("유저인터페이스\\navigation_interface3\\navi_grid{icon}_{points}.tga")
}

fn end_mark(moment: usize) -> String {
    format!("유저인터페이스\\navigation_interface3\\location{moment}.tga")
}

/// Which way a step points, as the client's search numbers it.
fn pointing(from: (i32, i32), to: (i32, i32)) -> usize {
    match ((to.0 - from.0).signum(), (to.1 - from.1).signum()) {
        (-1, 1) => 1,
        (-1, 0) => 2,
        (-1, -1) => 3,
        (0, -1) => 4,
        (1, -1) => 5,
        (1, 0) => 6,
        (1, 1) => 7,
        _ => 0,
    }
}

/// The navigation's way is marked on the ground, or with `None` no more. The pictures it needs are read the
/// first time each is wanted.
pub fn way_marked(way: Option<Marked>) {
    let wanted: Vec<String> = STATE.with_borrow_mut(|state| {
        let names: Vec<String> = way.iter().flat_map(|way| (0..8).map(|points| arrow(way.icon, points)).chain((1..=4).map(end_mark))).collect();
        state.way.way = way;
        names.into_iter().filter(|name| !state.way.pictures.contains_key(name) && state.way.asked.insert(name.clone())).collect()
    });
    for name in wanted {
        wasm_bindgen_futures::spawn_local(async move {
            let read = picture(&name).await;
            STATE.with_borrow_mut(|state| match (read, &state.gpu) {
                (Ok(image), Some(gpu)) => {
                    let binds = gpu.effect_binds(image.width, image.height, &image.rgba, &gpu.sampler);
                    state.way.pictures.insert(name, binds);
                }
                // Not read, or nothing to put it on yet: asked for again with the next way.
                _ => {
                    state.way.asked.remove(&name);
                }
            });
        });
    }
}

/// Asks something of the ground of the map one is on; nothing while no map is shown.
pub fn with_ground<Answer>(ask: impl FnOnce(&Gat) -> Answer) -> Option<Answer> {
    STATE.with_borrow(|state| state.scene.as_ref().map(|scene| ask(&scene.gat)))
}

/// A square lying on the ground about a cell, `reach` cells to each side of the cell's own (0 for the cell
/// itself), on the heights of that cell's own corners: its picture's top along the south side.
fn square(quads: &mut Quads, gat: &Gat, cell: (i32, i32), reach: f32, colour: [f32; 4]) {
    let heights = gat.corners(cell.0, cell.1).unwrap_or_default();
    let corner = |index: usize| {
        let (east, north) = ((index & 1) as f32, (index >> 1) as f32);
        let at = [(cell.0 as f32 + east + (east * 2.0 - 1.0) * reach) * CELL, heights[index] + OVER, (cell.1 as f32 + north + (north * 2.0 - 1.0) * reach) * CELL];
        (at, [east, north])
    };
    for index in [0, 1, 2, 2, 1, 3] {
        let (at, uv) = corner(index);
        quads.corners.extend(at);
        quads.corners.extend(uv);
        quads.corners.extend(colour);
    }
}

/// The marks of this frame (`DrawRoute`): those within ten cells of the character. An arrow lies on every
/// other cell of the way and points as the step from it goes; one of them in ten is brighter, the bright one
/// running along the way ten times a second. The mark on the last cell is two cells across (a quarter
/// of the client's ground square beyond the cell on every side) and shows its four pictures in turn, 150
/// milliseconds each.
pub(super) fn drawn(state: &State, time: f64, quads: &mut Quads) {
    let (Some(scene), Some(way)) = (&state.scene, &state.way.way) else { return };
    let Some(last) = way.cells.last().copied() else { return };
    let own = scene.player.at;
    let near = |cell: (i32, i32)| (cell.0 as f32 - own.0.round()).abs() < 10.0 && (cell.1 as f32 - own.1.round()).abs() < 10.0;
    if near(last) {
        let moment = (time / 150.0) as usize % 4 + 1;
        if let Some(binds) = state.way.pictures.get(&end_mark(moment)) {
            let colour = if way.at_goal { [1.0, 1.0, 1.0, 238.0 / 255.0] } else { [100.0 / 255.0, 170.0 / 255.0, 1.0, 238.0 / 255.0] };
            square(quads, &scene.gat, last, 0.5, colour);
            quads.picture_under(binds, false);
        }
    }
    let bright = (time % 1000.0 / 100.0) as usize;
    for index in (0..way.cells.len() - 1).step_by(2) {
        let cell = way.cells[index];
        if !near(cell) {
            continue;
        }
        let Some(binds) = state.way.pictures.get(&arrow(way.icon, pointing(cell, way.cells[index + 1]))) else { continue };
        square(quads, &scene.gat, cell, 0.0, [1.0, 1.0, 1.0, if bright == index % 10 { 238.0 / 255.0 } else { 153.0 / 255.0 }]);
        quads.picture_under(binds, false);
    }
}
