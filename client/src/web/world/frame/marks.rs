//! What is laid on the ground and hung in the air this frame: the mark of the cell under the pointer, the rings
//! of the portals, the numbers of the blows.

use super::*;

/// The rings of the portals, and the mark of the ground cell under the pointer, as the original marks where a
/// press would walk to.
pub(super) fn ground(state: &State, view: &View, time: f64) -> (Vec<f32>, Vec<f32>) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return Default::default() };
    let gat = &scene.gat;
    let mut ring_corners = Vec::new();
    for cell in scene.portals.values() {
        rings(on_ground(gat, (cell.0 as f32, cell.1 as f32)), time, &mut ring_corners);
    }
    // The ground cell under the pointer is marked, as the original marks where a press would walk to.
    let mut marks = Vec::new();
    let (page_width, page_height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    // Only a cell that can be walked on is marked (the client skips walls and cliffs).
    if let Some(cell) = state.pointer
        .and_then(|pointer| view.pick(&scene.gat, (pointer.0 / page_width.max(1.0), pointer.1 / page_height.max(1.0))))
        .filter(|cell| scene.gat.walkable(cell.0, cell.1))
    {
        // The mark lies on the cell's own four corners (`GroundGeometry::DrawGrid` → `CellRectangle`: the heights
        // of that cell of the `.gat`, not those of the cells beside it, which under a bridge are the river's bed),
        // the picture's top along the cell's south side (`g_groundGridUV`). The client draws it nearer by a hair
        // of the depth instead of lifting it; here it is lifted 0.4.
        let (x, y) = cell;
        let heights = scene.gat.corners(x, y).unwrap_or_default();
        let corner = |index: usize| {
            let (east, north) = ((index & 1) as f32, (index >> 1) as f32);
            [(x as f32 + east) * CELL, heights[index] + 0.4, (y as f32 + north) * CELL, east, north]
        };
        for index in [0, 1, 2, 2, 1, 3] {
            marks.extend(corner(index));
        }
    }
    (ring_corners, marks)
}

/// The numbers and words in the air, each where its kind has it at its age, as large and as faint
/// (`crate::blow::flight`, the client's `CMsgEffect::OnProcess`): over its owner while that one is in sight.
pub(super) fn numbers(state: &mut State, time: f64) -> Vec<(SpriteKey, Vector, f32, f32, f32)> {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return Vec::new() };
    let mut numbers = Vec::new();
    state.flying.retain_mut(|flying| {
        let elapsed = ((time - flying.began) / EFFECT_FRAME_MS) as f32;
        if elapsed > flying.flight.frames() {
            return false;
        }
        if let Some(stands) = scene.place_of(flying.on) {
            flying.base = on_ground(&scene.gat, stands);
        }
        let place = flying.flight.at(elapsed);
        let at = [flying.base[0] + flying.drift.0 * place.aside, flying.base[1] + place.up, flying.base[2] + flying.drift.1 * place.aside];
        // A word runs through the pictures of its action, as its kind does; one played once is gone after
        // its last.
        if let [SpriteKey::Creature(sprite, action, _)] = flying.pictures.as_slice() {
            if matches!(flying.flight, Flight::Mark | Flight::Lucky) {
                // (The sprite's own name, without the colour it is tinted with; until it is read, its first.)
                let count = figure::frames_of(sprite.split('#').next().unwrap_or(sprite), *action).unwrap_or(1);
                let Some(picture) = flying.flight.picture(elapsed, count) else { return false };
                let key = SpriteKey::Creature(sprite.clone(), *action, picture);
                if gpu.has_sprite(&key) {
                    numbers.push((key, at, place.scale, place.alpha, 0.0));
                }
                return true;
            }
        }
        // Each digit stands 8 pixels of the sprite from the next, whatever its own width, the number's middle on
        // its place (the client's `CGameActor::SpawnDamageDigits`: the digit `index` places from the last is put
        // `(count - index * 2) * 4 - 4` across, and that grows with the number as it is drawn larger).
        let count = flying.pictures.len();
        for (index, key) in flying.pictures.iter().enumerate() {
            if gpu.has_sprite(key) {
                numbers.push((key.clone(), at, place.scale, place.alpha, crate::blow::digit_across(count, count - 1 - index)));
            }
        }
        true
    });
    numbers
}
