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
    if let Some(cell) = state.pointer.and_then(|pointer| view.pick(&scene.gat, (pointer.0 / page_width.max(1.0), pointer.1 / page_height.max(1.0)))).filter(|cell| scene.gat.walkable(cell.0, cell.1)) {
        let corner = |x: i32, y: i32| [x as f32 * CELL, scene.gat.height_at(x as f32, y as f32) + 0.4, y as f32 * CELL];
        let (x, y) = cell;
        for (corner_x, corner_y, u, v) in [(x, y + 1, 0.0, 0.0), (x + 1, y + 1, 1.0, 0.0), (x, y, 0.0, 1.0), (x, y, 0.0, 1.0), (x + 1, y + 1, 1.0, 0.0), (x + 1, y, 1.0, 1.0)] {
            marks.extend(corner(corner_x, corner_y));
            marks.extend([u, v]);
        }
    }
    (ring_corners, marks)
}

/// The numbers in the air, by the client's own figures. A number starts five times its size and shrinks to a
/// little over it, rises and falls back as it drifts, and fades: all over 70 of the client's frames. A miss only
/// rises, and fades over 80.
pub(super) fn numbers(state: &mut State, time: f64) -> Vec<(SpriteKey, Vector, f32, f32, f32)> {
    let Some(gpu) = &mut state.gpu else { return Vec::new() };
    state.flying.retain(|flying| (time - flying.began) / EFFECT_FRAME_MS <= if flying.flight == Flight::Miss { 80.0 } else { 70.0 });
    let mut numbers = Vec::new();
    for flying in &state.flying {
        let elapsed = ((time - flying.began) / EFFECT_FRAME_MS) as f32;
        // What rises and falls back as it drifts (the client's `MsgEffectDrift`).
        let drifting = || {
            let aside = elapsed / 3.0 + 3.0;
            let up = 12.0 - 8.0 + (2.0 - elapsed / 30.0) * elapsed;
            [flying.base[0] + flying.drift.0 * aside, flying.base[1] + up, flying.base[2] + flying.drift.1 * aside]
        };
        // Where it is, how large and how much of it is left: each kind by the client's own figures
        // (`CMsgEffect::OnProcess`).
        let (at, scale, alpha) = match flying.flight {
            Flight::Miss => (along(flying.base, [0.0, 1.0, 0.0], 12.0 + elapsed * 0.54), 1.0, (250.0 - elapsed * 3.0) / 255.0),
            Flight::Number => (drifting(), (5.0 - elapsed * 0.24).max(1.2), (250.0 - elapsed * 3.4) / 255.0),
            Flight::Critical => (drifting(), (5.0 - elapsed * 0.24).max(1.3), (250.0 - elapsed * 3.4) / 255.0),
            Flight::Mark => (drifting(), (3.0 - elapsed * 0.24 * 0.6).max(1.0), (255.0 - elapsed * 3.45) / 255.0),
        };
        // The mark runs through the pictures of its action, one every two frames, and stays on the last.
        if flying.flight == Flight::Mark {
            if let Some(SpriteKey::Creature(sprite, action, _)) = flying.pictures.first() {
                // The sprite's own name, without the colour it is tinted with.
                let last = figure::frames_of(sprite.split('#').next().unwrap_or(sprite), *action).map_or(0, |frames| frames.saturating_sub(1));
                let key = SpriteKey::Creature(sprite.clone(), *action, ((elapsed * 0.5) as usize).min(last));
                if gpu.has_sprite(&key) {
                    numbers.push((key, at, scale, alpha, 0.0));
                }
            }
            continue;
        }
        // Each digit stands 8 pixels of the sprite from the next, whatever its own width, the number's middle on
        // its place (the client's `CGameActor::MakeNumber`: the digit `index` places from the last is put
        // `(count - index * 2) * 4 - 4` across, and that grows with the number as it is drawn larger).
        let count = flying.pictures.len();
        for (place, key) in flying.pictures.iter().enumerate() {
            if gpu.has_sprite(key) {
                numbers.push((key.clone(), at, scale, alpha, crate::blow::digit_across(count, count - 1 - place)));
            }
        }
    }
    numbers
}
