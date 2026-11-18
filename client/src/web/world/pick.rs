//! What the pointer is on: someone in the world, or a thing lying on the ground.

use super::*;

/// Who a point of the page is on, and where on the page their feet are. The client's way (`CGameActor::
/// SetRenderInfo`, `CActorPickNode::Pick`): each one drawn has the rectangle of its picture on the screen, made
/// at least 40 pixels each way (of a screen 480 high, as the rest of the client's measures here), so that a
/// small monster is not a small thing to hit; and of those the point is in, a monster comes before an NPC or a
/// companion, those before a player, and a pet last. Among the same kind it is the nearest.
pub(super) fn pointed_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32)) -> Option<(u32, (f32, f32))> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    let least = 40.0 * height / 480.0;
    scene
        .units
        .iter()
        .filter_map(|(id, unit)| {
            if unit.dying.is_some() {
                return None;
            }
            let texture = gpu.sprite_textures.get(unit.shown.as_ref()?)?;
            let feet = on_ground(&scene.gat, unit.at);
            let corner = |x: f32, y: f32| view.project(along(along(feet, view.right, (x - texture.feet.0) * SPRITE_SCALE), view.up, (texture.feet.1 - y) * SPRITE_SCALE));
            let (low, high) = (corner(0.0, 0.0)?, corner(texture.size.0, texture.size.1)?);
            // The rectangle, grown evenly about its middle where it is smaller than the least.
            let span = |low: f32, high: f32| {
                let (low, high) = (low.min(high), low.max(high));
                let short = (least - (high - low)).max(0.0) / 2.0;
                low - short..=high + short
            };
            let inside = span(low.0 * width, high.0 * width).contains(&pointer.0) && span(low.1 * height, high.1 * height).contains(&pointer.1);
            let rank = match (&unit.look, unit.lively, unit.companion) {
                (_, _, true) => 50,
                (_, true, _) => 400,
                (Look::Creature(_), ..) => 200,
                (Look::Character(_), ..) => 100,
            };
            let under = view.project(feet)?;
            inside.then(|| (rank, dot(subtract(feet, view.eye), view.forward), *id, (under.0 * width, under.1 * height)))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)))
        .map(|(_, _, id, feet)| (id, feet))
}

/// The thing on the ground a point of the page is on. The client's way (`CItem::SetRenderInfo`): the rectangle
/// of the thing's picture on the screen, made at least 34 pixels each way (of a screen 480 high), so that a small
/// thing is not a small thing to hit; of those the point is in, the nearest.
pub(super) fn lying_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32)) -> Option<u32> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    let least = 34.0 * height / 480.0;
    scene
        .drops
        .iter()
        .filter_map(|(id, Lying { item, at, identified, .. })| {
            let feet = on_ground(&scene.gat, *at);
            let under = view.project(feet)?;
            // The picture's own rectangle, where the picture is known; a point at the thing's place otherwise.
            let resource = data::item_as(*item, *identified).map(|(_, resource)| resource).unwrap_or_default();
            let key = SpriteKey::Creature(format!("아이템/{}", if resource.is_empty() { "사과" } else { resource.as_str() }), 0, 0);
            let (low, high) = match gpu.sprite_textures.get(&key) {
                Some(texture) => {
                    let corner = |x: f32, y: f32| view.project(along(along(feet, view.right, (x - texture.feet.0) * SPRITE_SCALE), view.up, (texture.feet.1 - y) * SPRITE_SCALE));
                    (corner(0.0, 0.0)?, corner(texture.size.0, texture.size.1)?)
                }
                None => (under, under),
            };
            let span = |low: f32, high: f32| {
                let (low, high) = (low.min(high), low.max(high));
                let short = (least - (high - low)).max(0.0) / 2.0;
                low - short..=high + short
            };
            let inside = span(low.0 * width, high.0 * width).contains(&pointer.0) && span(low.1 * height, high.1 * height).contains(&pointer.1);
            inside.then(|| (dot(subtract(feet, view.eye), view.forward), *id))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}
