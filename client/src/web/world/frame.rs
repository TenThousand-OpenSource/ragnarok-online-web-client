//! One frame of the world: everyone is moved to where they are now, the effects advance, the picture is drawn,
//! and the page's overlays (names, bars, the pointer) follow.

use super::*;

/// How long one struck down takes to fade once it has fallen: the client's `CCorpse::OnProcess` takes 1 of 255
/// from it every 24 milliseconds.
const CORPSE_MS: f64 = 255.0 * 24.0;

mod effects;
mod marks;
mod sprites;

/// Draws one frame, and asks for the next while there is a world to draw: a list of steps and nothing else.
pub(super) fn frame() {
    due();
    if let Some((moved, pursuit)) = STATE.with_borrow_mut(step) {
        tell(moved, pursuit);
    }
    if STATE.with_borrow(|state| state.scene.is_some()) {
        next_frame();
    }
}

/// The effects and sounds whose moment has come begin.
fn due() {
    let (effects, sounds) = STATE.with_borrow_mut(|state| {
        let time = now();
        let effects: Vec<_> = state.effects_due.extract_if(.., |due| due.0 <= time).collect();
        let sounds: Vec<_> = state.sounds_due.extract_if(.., |due| due.0 <= time).collect();
        (effects, sounds)
    });
    for (_, name, on, carried) in effects {
        effect_carried(&name, on, carried);
    }
    for (_, name) in sounds {
        sound::effect(&name, 1.0);
    }
}

/// The cell the page is to be told the character is on now, if it changed, with the hook to tell it by.
type Moved = Option<(fn(u16, u16), (i32, i32))>;

/// The world at this moment: everyone moved, the camera, the picture, the page's overlays, what the player
/// is after. Gives back what the page is to be told; nothing without a world to draw.
fn step(state: &mut State) -> Option<(Moved, Pursuit)> {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return None };
    gpu.fit();
    let (time, yaw) = (now(), scene.camera.yaw);
    stride(state, time);
    let (own, feet, view) = camera_step(state, time, yaw)?;
    let casts = drawn(state, &view, own, time, yaw)?;

    let (width, height) = overlays(state, &view, time, feet, &casts)?;

    let pursuit = pursue(state, &view, (width, height), time);
    let scene = state.scene.as_mut()?;

    let cell = scene.player.cell;
    let changed = cell != scene.told;
    scene.told = cell;
    Some((state.hooks.as_ref().filter(|_| changed).map(|hooks| (hooks.moved, cell)), pursuit))
}

/// The player's own character where it is now (its place in cells and its picture), its feet in the world,
/// and the view of a camera that follows them.
fn camera_step(state: &mut State, time: f64, yaw: f32) -> Option<(((f32, f32), SpriteKey), Vector, View)> {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return None };
    let own = scene.player.now(time, yaw);
    let feet = on_ground(&scene.gat, own.0);
    if state.camera_fixed {
        scene.aim = None;
    }
    let aim = scene.follow(feet, time);
    // The camera goes towards what was asked of it, and the models whose parts turn are moved on.
    scene.ease_camera(time);
    scene.models.animate(gpu, aim, time);
    // A shake of the screen throws the camera about, a step every frame, and leaves it as it was.
    let (camera, aim) = camera::shaken(&mut state.quake, scene.camera, aim, time);
    Some((own, feet, camera.view(aim, gpu.config.width as f32 / gpu.config.height as f32)))
}

/// Everything of the world lives on to `time` and is drawn as that view sees it. Gives back the casts under
/// way, for the bars the page shows over them.
fn drawn(state: &mut State, view: &View, own: ((f32, f32), SpriteKey), time: f64, yaw: f32) -> Option<Vec<((f32, f32), f64, f64, u32, Option<u32>)>> {
    let landed = combat::landings(state, time);
    sprites::landings(state, time, &landed);
    earshot::due(state, time);
    let mut standing = sprites::standing(state, own, time, yaw);
    // The marks over NPCs that have a quest stand among them.
    quests::marks(state, time, &mut standing);
    // The carts merchants pull stand among them too.
    cart::drawn(state, &mut standing);
    falcon::drawn(state, time, &mut standing);
    let (rings, marks) = marks::ground(state, view, time);
    let numbers = marks::numbers(state, time);
    skill_units::step(state, time);
    // What the map places about itself, its weather and the night.
    scenery::step(state, time);
    let mut quads = Quads::default();
    skill_units::drawn(state, view, &mut quads);
    effects::files(state, view, time, &mut quads);
    scope::drawn(state, time, &mut quads);
    let casts = effects::casts(state, time, &mut quads);
    let asked = effects::bursts(state, view, time, &mut quads);
    // What the effects do to the bodies of those they are on, and the shakes of the screen they ask for.
    ailments::bodies(state, &asked.touches, time);
    ailments::copies_left(state, &asked.copies, time);
    if let Some(kind) = asked.quakes.last().filter(|_| state.options.quake) {
        state.quake = Some(crate::quake::Quake::begun(*kind, time));
    }
    effects::arrows(state, view, time, &mut quads);
    ailments::veils(state, view, time, &mut quads);
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return None };
    gpu.draw(scene, view, &mut standing, &rings, &marks, &numbers, &quads);
    Some(casts)
}

/// The page is told what the frame found: in the order the client's loop acts on them.
fn tell(moved: Moved, Pursuit { follow, strike, take, used, once }: Pursuit) {
    if let Some((strike_once, id)) = once {
        strike_once(id);
    }
    if let Some((tell, whom, cell)) = used {
        tell(whom, cell);
    }
    if let Some((take, id)) = take {
        take(id);
    }
    if let Some((press, id)) = strike {
        press(id, true);
    }
    if let Some((moved, cell)) = moved {
        moved(cell.0.max(0) as u16, cell.1.max(0) as u16);
    }
    if let Some((walk_to, goal)) = follow {
        walk_to(goal.0 as u16, goal.1 as u16);
    }
}

pub(super) fn next_frame() {
    STATE.with_borrow_mut(|state| {
        let frame = state.frame.get_or_insert_with(|| Closure::<dyn FnMut()>::new(frame));
        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(frame.as_ref().unchecked_ref());
        }
    });
}
