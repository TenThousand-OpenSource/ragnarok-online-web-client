//! What the pointer is on: someone in the world, a thing lying on the ground, or a trap standing on it.

use super::*;

/// Who a point of the page is on, and where on the page their feet are. The client's way (`CGameActor::
/// SetRenderInfo`, `CActorPickNode::Pick`): each one drawn has the rectangle of its picture on the screen, made
/// at least 40 pixels each way (of a screen 480 high, as the rest of the client's measures here), so that a
/// small monster is not a small thing to hit; and of those the point is in, a monster comes before an NPC or a
/// homunculus, those before a player, and a pet or a mercenary last; with `players_first` (a skill of help aimed and Shift
/// up, or Shift held and none aimed: `State::players_first`) a player comes before a monster and an NPC last
/// (`crate::skills::aim::rank`). Among the same kind it is the nearest.
pub(super) fn pointed_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32), players_first: bool) -> Option<(u32, (f32, f32))> {
    use crate::skills::aim::{Under, rank};
    scene
        .units
        .iter()
        .filter_map(|(id, unit)| {
            // (Nor one whose sprite is not drawn: it hides, or fades out of sight.)
            if unit.dying.is_some() || unit.hidden() || unit.pose.vanishes.is_some() {
                return None;
            }
            let feet = touched(gpu, scene, view, unit, pointer)?;
            let under = match (&unit.look, unit.lively, unit.companion) {
                (_, _, true) if crate::skills::aim::homunculus(unit.job) => Under::Homunculus,
                (_, _, true) => Under::Pet,
                (_, true, _) => Under::Monster,
                (Look::Creature(_) | Look::Model(_) | Look::Unseen, ..) => Under::Npc,
                (Look::Character(_), ..) => Under::Player,
            };
            Some((rank(under, players_first), dot(subtract(on_ground(&scene.gat, unit.at), view.eye), view.forward), *id, feet))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)))
        .map(|(_, _, id, feet)| (id, feet))
}

/// Where on the page the feet of someone are whose picture a point of the page is on (its rectangle made at
/// least 40 pixels each way, of a screen 480 high); nothing when the point is not on it.
pub(super) fn touched(gpu: &Gpu, scene: &Scene, view: &View, unit: &Mover, pointer: (f32, f32)) -> Option<(f32, f32)> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    let least = 40.0 * height / 480.0;
    if matches!(unit.look, Look::Model(_)) {
        return granny::touched(scene, view, unit, (width, height), pointer);
    }
    let feet = on_ground(&scene.gat, unit.at);
    let under = view.project(feet)?;
    // A hidden NPC has no picture: its rectangle is the place of its feet, as wide and half as high as the
    // size the server gives it, which is none (`crate::creatures::Shape::Hidden`), and is grown as any other.
    let rectangle = match unit.look {
        Look::Unseen => (under, under),
        _ => corners(view, feet, gpu.sprite_textures.get(unit.shown.as_ref()?)?)?,
    };
    let inside = within(rectangle, least, (width, height), pointer);
    inside.then_some((under.0 * width, under.1 * height))
}

/// Two opposite corners of what is drawn of a picture standing with its feet at a place, as the view shows
/// them (parts of the screen); nothing for one behind the camera. The box is that of what the picture shows,
/// not of the whole picture: a character's is put together on a sheet much larger than itself, and the
/// client's box is the rectangles of the layers it draws (`SpriteTexture::drawn`).
fn corners(view: &View, feet: Vector, texture: &gpu::SpriteTexture) -> Option<((f32, f32), (f32, f32))> {
    let corner = |x: f32, y: f32| view.project(
        along(along(feet, view.right, (x - texture.feet.0) * SPRITE_SCALE), view.up, (texture.feet.1 - y) * SPRITE_SCALE)
    );
    let (low, high) = texture.drawn;
    Some((corner(low.0, low.1)?, corner(high.0, high.1)?))
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

/// The trap a point of the page is on, and the trap's cell. The client's way (`CSkill::Render`,
/// `CSkill::SetRenderInfo`, for the units it follows on the screen, `render.trackBounds`: every trap of the
/// Hunter's): a rectangle about the place of the unit on the screen, made at least 34 pixels each way (of a
/// screen 480 high); of those the point is in, the nearest (`CActorPickNode::Pick`). The client's rectangle is
/// 4 of its units wide and 2 high over the place before it is grown, so its middle lies a unit above the
/// place: here it is grown about the place itself.
pub(super) fn trap_at(gpu: &Gpu, scene: &Scene, grounds: &skill_units::Grounds, view: &View, pointer: (f32, f32)) -> Option<(u32, (f32, f32))> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    let least = 34.0 * height / 480.0;
    grounds
        .traps()
        .filter_map(|(id, cell)| {
            let place = on_ground(&scene.gat, cell);
            let under = view.project(place)?;
            within((under, under), least, (width, height), pointer).then(|| (dot(subtract(place, view.eye), view.forward), id, cell))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id, cell)| (id, cell))
}
