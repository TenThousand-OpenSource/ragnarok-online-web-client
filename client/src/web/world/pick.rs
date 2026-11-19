//! What the pointer is on: someone in the world, or a thing lying on the ground.

use super::*;

/// Who a point of the page is on, and where on the page their feet are. The client's way (`CGameActor::
/// SetRenderInfo`, `CActorPickNode::Pick`): each one drawn has the rectangle of its picture on the screen, made
/// at least 40 pixels each way (of a screen 480 high, as the rest of the client's measures here), so that a
/// small monster is not a small thing to hit; and of those the point is in, a monster comes before an NPC or a
/// companion, those before a player, and a pet last. Among the same kind it is the nearest.
pub(super) fn pointed_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32)) -> Option<(u32, (f32, f32))> {
    scene
        .units
        .iter()
        .filter_map(|(id, unit)| {
            // (Nor one whose sprite is not drawn: it hides, or fades out of sight.)
            if unit.dying.is_some() || unit.hidden() || unit.pose.vanishes.is_some() {
                return None;
            }
            let under = touched(gpu, scene, view, unit, pointer)?;
            let rank = match (&unit.look, unit.lively, unit.companion) {
                (_, _, true) => 50,
                (_, true, _) => 400,
                (Look::Creature(_), ..) => 200,
                (Look::Character(_), ..) => 100,
            };
            Some((rank, dot(subtract(on_ground(&scene.gat, unit.at), view.eye), view.forward), *id, under))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)))
        .map(|(_, _, id, feet)| (id, feet))
}

/// Where on the page the feet of someone are whose picture a point of the page is on (its rectangle made at
/// least 40 pixels each way, of a screen 480 high); nothing when the point is not on it.
pub(super) fn touched(gpu: &Gpu, scene: &Scene, view: &View, unit: &Mover, pointer: (f32, f32)) -> Option<(f32, f32)> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    let least = 40.0 * height / 480.0;
    let texture = gpu.sprite_textures.get(unit.shown.as_ref()?)?;
    let feet = on_ground(&scene.gat, unit.at);
    let inside = within(corners(view, feet, texture)?, least, (width, height), pointer);
    let under = view.project(feet)?;
    inside.then_some((under.0 * width, under.1 * height))
}

/// Two opposite corners of a picture standing with its feet at a place, as the view shows them (parts of the
/// screen); nothing for one behind the camera.
fn corners(view: &View, feet: Vector, texture: &gpu::SpriteTexture) -> Option<((f32, f32), (f32, f32))> {
    let corner = |x: f32, y: f32| view.project(
        along(along(feet, view.right, (x - texture.feet.0) * SPRITE_SCALE), view.up, (texture.feet.1 - y) * SPRITE_SCALE)
    );
    Some((corner(0.0, 0.0)?, corner(texture.size.0, texture.size.1)?))
}

/// Whether a point of the page is in the rectangle between two such corners, the page being so wide and
/// high: the rectangle grown evenly about its middle where it is smaller than the `least`.
fn within((low, high): ((f32, f32), (f32, f32)), least: f32, (width, height): (f32, f32), pointer: (f32, f32)) -> bool {
    let span = |low: f32, high: f32| {
        let (low, high) = (low.min(high), low.max(high));
        let short = (least - (high - low)).max(0.0) / 2.0;
        low - short..=high + short
    };
    span(low.0 * width, high.0 * width).contains(&pointer.0) && span(low.1 * height, high.1 * height).contains(&pointer.1)
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
                Some(texture) => corners(view, feet, texture)?,
                None => (under, under),
            };
            let inside = within((low, high), least, (width, height), pointer);
            inside.then(|| (dot(subtract(feet, view.eye), view.forward), *id))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}
