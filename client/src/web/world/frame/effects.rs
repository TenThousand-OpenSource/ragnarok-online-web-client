//! The effects of this frame, each kind turned into quads for the renderer: the effects played from the
//! client's files, the casts, the effects made of the client's primitives and the arrows in flight.

use super::*;

/// Where and how an effect file playing shows at a moment: the place in the world its corners are measured
/// from, how many pixels up the screen from there its middle is, the angle its corners are turned by about the
/// place, how much of it shows (0 to 1), and the frame of the file.
struct Showing {
    place: Vector,
    lift: f32,
    angle: f32,
    alpha: f32,
    frame: i32,
}

/// Where a place in the world is on the screen (x to the right, y down), as far as the angle between two such
/// goes: the client measures its angles between places projected onto its screen (`ProjectViewport`).
fn projected(view: &View, at: Vector) -> [f32; 2] {
    let from_eye = subtract(at, view.eye);
    let depth = dot(from_eye, view.forward).max(0.001);
    [dot(from_eye, view.right) / depth, -dot(from_eye, view.up) / depth]
}

/// What a playing file shows now: nothing once it is over; nothing to draw while it waits, or with the one
/// it is on out of sight. A plain file plays over the one it is on, its middle 80 pixels up the screen (the
/// client's `CEZ2STRScreenEffect::OnProcess`), a frame of its own clock at a time. One that is carried
/// (`crate::effect::carried`) measures as it begins where the two it was told of stand and the angle it is
/// turned by, and then lives a tick of the client's at a time on its way.
fn showing(playing: &mut Playing, scene: &Scene, view: &View, time: f64) -> Option<Option<Showing>> {
    use crate::effect::carried::{self, Turn, Way};
    let effect = &playing.shown.effect;
    let standing = |on: Option<u32>| scene.stands(on);
    let here = standing(playing.on);
    let Some(carrying) = &mut playing.carried else {
        let frame = ((time - playing.began) / 1000.0 * f64::from(effect.frames_per_second)) as i32;
        return (frame <= effect.last_frame).then(|| here.map(|place| Showing { place, lift: 80.0, angle: 0.0, alpha: 1.0, frame }));
    };
    let there = carrying.other.and_then(standing);
    if carrying.set.is_none() {
        let Some(here) = here else { return Some(None) };
        let there = there.unwrap_or(here);
        let angle = match carrying.how.turn {
            Turn::None => 0.0,
            Turn::By(angle) => angle,
            Turn::Towards(more) => carried::screen_angle(projected(view, here), projected(view, there)) + more,
            Turn::Flight => carried::steps_turn(here, there, crate::prim::camera_yaw(view.right)),
        };
        carrying.set = Some((here, there, angle));
    }
    let Some((from, to, angle)) = carrying.set else { return Some(None) };
    let tick = ((time - playing.began) / f64::from(carried::TICK_MS)) as i32;
    let Some(lived) = carrying.how.lived(tick) else { return Some(None) };
    let frame = carrying.how.frame(lived, effect.last_frame)?;
    let (place, alpha) = match carrying.how.way {
        Way::Stays => (here, 1.0),
        Way::There => (there.or(here), 1.0),
        Way::Steps => (Some(carried::steps(from, to, lived)?), 1.0),
        Way::Straight => carried::straight(from, to, lived).map(|(place, alpha)| (Some(place), alpha))?,
        // (The thrower's place is lifted 10 up the screen; the bow is to the side the other is on there.)
        Way::Bowed => match (here, there) {
            (Some(here), Some(there)) => (Some(carried::bowed(along(here, view.up, 10.0), there, dot(subtract(there, here), view.right) >= 0.0, lived)?), 1.0),
            _ => (None, 1.0),
        },
    };
    let offset = carrying.how.offset;
    Some(place.map(|place| Showing { place: [place[0] + offset[0], place[1] + offset[1], place[2] + offset[2]], lift: carrying.how.lift, angle, alpha, frame }))
}

/// The effects playing from files: every layer's quad as it is at this frame of the effect, around the place
/// the file shows at (`showing`), by the client's own figures (a pixel of the effect is a pixel of a sprite);
/// the corners are turned about that place as the client's `CEZ2STRParticle::RenderScreen` turns them.
pub(super) fn files(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    let Some(scene) = &state.scene else { return };
    let mut shown = Vec::new();
    state.effects.retain_mut(|playing| {
        let Some(showing) = showing(playing, scene, view, time) else { return false };
        shown.extend(showing.map(|showing| (playing.shown.clone(), showing)));
        true
    });
    for (file, Showing { place, lift, angle, alpha, frame }) in shown {
        let nearer = nearer(view, along(place, view.up, lift * SPRITE_SCALE), 25.0);
        for (layer, pictures) in file.effect.layers.iter().zip(&file.pictures) {
            let Some(transform) = layer.at(frame) else { continue };
            let Some(Some(binds)) = pictures.get(transform.aniframe.max(0.0) as usize) else { continue };
            let mut colour = transform.colour.map(|part| (part / 255.0).clamp(0.0, 1.0));
            colour[3] *= alpha;
            if colour[3] <= 0.0 {
                continue;
            }
            let corners = transform.corners();
            for corner in [0, 1, 3, 3, 1, 2] {
                let ([x, y], uv) = corners[corner];
                let [x, y] = crate::effect::carried::turned([x, y - lift], angle);
                quads.corner(view, nearer, along(along(place, view.right, x * SPRITE_SCALE), view.up, -y * SPRITE_SCALE), uv, colour);
            }
            // Direct3D's "one" as the destination (2) adds the quad's light to what is there; so does "as
            // much as the destination's alpha" (7), the screen having none to speak of.
            quads.picture(binds, matches!(transform.destination_blend, 2 | 7));
        }
    }
}

/// The casts under way (where, since when, for how long, which effect, at whom): the rings around each caster
/// and the mark on the one aimed at. A cast whose time is up is over.
pub(super) fn casts(state: &mut State, time: f64, quads: &mut Quads) -> Vec<((f32, f32), f64, f64, u32, Option<u32>)> {
    let (Some(gpu), Some(scene)) = (&state.gpu, &mut state.scene) else { return Vec::new() };
    let mut casts = Vec::new();
    for mover in std::iter::once(&mut scene.player).chain(scene.units.values_mut()) {
        if mover.casting.is_some_and(|(began, lasts, ..)| time > began + lasts) {
            mover.casting = None;
        }
        if let Some((began, lasts, effect, target)) = mover.casting {
            casts.push((mover.at, began, lasts, effect, target));
        }
    }
    for (at, began, lasts, effect, target) in &casts {
        let frame = ((time - began) / CAST_FRAME_MS) as f32;
        let (name, tint, adds) = cast_look(*effect);
        if let Some(binds) = gpu.effect_pictures.get(name) {
            let count = cast_rings(on_ground(&scene.gat, *at), frame, ((lasts / CAST_FRAME_MS) as f32).max(70.0), tint, &mut quads.corners);
            quads.pictures.extend(std::iter::repeat_n((binds.clone(), adds), count));
        }
        let aimed = target.and_then(|id| scene.units.get(&id)).map(|unit| unit.at);
        if let (Some(aimed), Some(binds)) = (aimed, gpu.effect_pictures.get("lockon128.tga")) {
            let count = lock_on(on_ground(&scene.gat, aimed), frame, &mut quads.corners);
            quads.pictures.extend(std::iter::repeat_n((binds.clone(), false), count));
        }
    }
    casts
}

/// The effects made of the client's primitives: each is told, before its first tick, which way the one it is on
/// faces and where the place it aims at is, lives its ticks, and is drawn around that one (or where that one
/// was as it began), a touch nearer than where it stands. The sounds they start on the way are played. Gives
/// back what they ask of the world around them (`prims::Asked`); the effects they end are ended here.
pub(super) fn bursts(state: &mut State, view: &View, time: f64, quads: &mut Quads) -> prims::Asked {
    // The aura of the highest level is on those who show it now (`aura.rs`).
    aura::kept(state, time);
    let mut asked = prims::Asked::default();
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return asked };
    // The effects that a state of someone's keeps going (Sight), begun and begun again while it holds.
    let states: Vec<(Option<u32>, u32)> = std::iter::once((None, scene.player.state)).chain(scene.units.iter().map(|(id, unit)| (Some(*id), unit.state))).collect();
    let (kept, heard) = state.kept.look(&states, time, CAST_FRAME_MS, prims::lasts);
    for (effect, on) in kept {
        prims::begin(effect, on, None, time, &mut state.bursts, &mut state.sounds_due);
    }
    state.sounds_due.extend(heard.into_iter().map(|sound| (time, sound.to_owned())));
    let standing = |on: Option<u32>| scene.stands(on);
    for burst in state.bursts.iter_mut().filter(|burst| burst.begins <= time) {
        let facing = match burst.on {
            Some(id) => scene.units.get(&id).map_or(0, |unit| unit.facing),
            None => scene.player.facing,
        };
        // The step on the ground that facing is (as `map::direction` numbers them, from the south round by the west).
        let step = [(0.0, -1.0), (-1.0, -1.0), (-1.0, 0.0), (-1.0, 1.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (1.0, -1.0)][usize::from(facing % 8)];
        let length = f32::hypot(step.0, step.1);
        let aim = burst.aims_at().and_then(|at| Some(subtract(standing(at)?, standing(burst.on)?)));
        burst.begin((step.0 / length, step.1 / length), aim);
        // One that follows the other it was told of is told where that one is now.
        burst.other_at(aim.filter(|_| burst.aims_at().is_some()));
        burst.seen_from(crate::prim::camera_yaw(view.right));
        burst.named_at(name_offset(scene, burst.on));
        if let Some(place) = standing(burst.on) {
            burst.grounded(place[1]);
        }
    }
    let mut sounds = Vec::new();
    state.bursts.retain_mut(|burst| {
        if burst.begins > time {
            return true;
        }
        // The ground around its place, for what falls on it; with the one it is on out of sight, none.
        let place = standing(burst.on).map(|at| burst.place(at));
        let mut started = Vec::new();
        let left = burst.live(time, &|east, north| place.map_or(f32::NEG_INFINITY, |place| scene.gat.height_at((place[0] + east) / CELL, (place[2] + north) / CELL) - place[1]), &mut started, &mut asked);
        // Its sounds are heard from where the one it is on stands (`earshot.rs`).
        sounds.extend(started.into_iter().map(|sound| (time, sound, burst.on)));
        left
    });
    for (on, effect) in asked.ends.drain(..) {
        state.bursts.retain(|burst| !burst.is(effect, on));
    }
    // The effects their primitives asked for, each on the same one and told its place from that one's.
    for (on, effect, place) in asked.launches.drain(..) {
        if let Some(burst) = prims::begin(effect, on, None, time, &mut state.bursts, &mut state.sounds_due) {
            burst.tell(place);
        }
    }
    state.heard.extend(sounds);
    // The camera's yaw as the client has it (`crate::prim::camera_yaw` says from where): none looking north, a
    // quarter looking east.
    let yaw = crate::prim::camera_yaw(view.right);
    for burst in state.bursts.iter_mut().filter(|burst| burst.begins <= time) {
        let Some(place) = standing(burst.on).map(|at| burst.place(at)) else { continue };
        let nearer = nearer(view, place, 25.0);
        burst.draw(place, (view.right, view.up, yaw), |piece| match piece {
            prims::Piece::Picture(picture, corners, colours, adds) => {
                let Some(binds) = gpu.effect_pictures.get(picture) else { return };
                for ((at, uv), colour) in corners.into_iter().zip(colours) {
                    quads.corner(view, nearer, at, uv, colour);
                }
                quads.picture(binds, adds);
            }
            // A frame of a sprite, across the screen about its place: each layer of the frame a quad of its
            // own picture, in the frame's order, all in the one colour (`RenderSegmentSprites2`).
            prims::Piece::Sprite { sprite, action, frame, at, scale, roll, colour, own, lit, adds, rim } => {
                let Some(sheet) = gpu::sheet(sprite) else { return };
                let Some(shown) = sheet.act.actions.get(action).and_then(|frames| frames.get(frame % frames.len().max(1))) else { return };
                for layer in crate::sprite::layer_quads(shown, |truecolor, picture| sheet.sprite.size_of(truecolor, picture), scale, roll) {
                    let Some(binds) = gpu.effect_layer(sprite, &sheet, layer.truecolor, layer.picture) else { continue };
                    // (With the client's `pattern & 0x200` a layer is in the colour its file gives it.)
                    let colour = if own { layer.colour.map(|part| f32::from(part) / 255.0) } else { colour };
                    // (The kind 153: the layer's own red, green and blue at the primitive's alpha.)
                    let colour = if lit { [f32::from(layer.colour[0]) / 255.0, f32::from(layer.colour[1]) / 255.0, f32::from(layer.colour[2]) / 255.0, colour[3]] } else { colour };
                    // (The kind 167: a colour made of the layer's own, and for its second drawing a larger quad.)
                    let Some(colour) = rim.map_or(Some(colour), |rim| rim.colour(layer.colour)) else { continue };
                    for [x, y, u, v] in rim.map_or(layer.corners, |rim| rim.widened(layer.corners)) {
                        quads.corner(view, nearer, along(along(at, view.right, x * SPRITE_SCALE), view.up, -y * SPRITE_SCALE), [u, v], colour);
                    }
                    quads.picture(&binds, adds);
                }
            }
        });
    }
    asked
}

/// How high over the feet the name of someone stands (none named: the player's own character), by the job that
/// one is drawn as (`crate::mounts::name_offset`, the client's `GetNameOffset`): nothing for a monster or an
/// NPC, or for one out of sight.
fn name_offset(scene: &Scene, on: Option<u32>) -> f32 {
    let mover = match on {
        Some(id) => scene.units.get(&id),
        None => Some(&scene.player),
    };
    mover.filter(|mover| matches!(mover.look, Look::Character(_))).map_or(0.0, |mover| crate::mounts::name_offset(crate::mounts::drawn_job(mover.job, mover.state), data::is_doram(mover.job)))
}

/// The way something goes as the screen shows it, a step of one, and the way across it.
fn on_screen(view: &View, way: Vector) -> ([f32; 2], [f32; 2]) {
    let (across, up) = (dot(way, view.right), dot(way, view.up));
    let length = (across * across + up * up).sqrt().max(0.001);
    ([across / length, up / length], [-up / length, across / length])
}

/// The arrows in flight: each a picture turned along its way on the screen, from the archer to its target.
pub(super) fn arrows(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return };
    state.arrows.retain(|(leaves, ..)| time - leaves <= ARROW_MS);
    if state.arrows.is_empty() {
        return;
    }
    let Some(EffectSprite { binds, size: (wide, tall), .. }) = gpu.effect_sprite(&SpriteKey::Creature(ARROW.to_owned(), 0, 0)) else { return };
    for (leaves, from, target) in &state.arrows {
        if time < *leaves {
            continue;
        }
        let Some(to) = scene.units.get(target).map(|unit| unit.at).or((*target != 0).then_some(scene.player.at)) else { continue };
        let to = along(on_ground(&scene.gat, to), [0.0, 1.0, 0.0], 10.0);
        let at = along(*from, subtract(to, *from), ((time - leaves) / ARROW_MS) as f32);
        // The way it flies as the screen shows it: the picture's head is at its bottom, and is turned to lead.
        let ([sine, cosine], _) = on_screen(view, subtract(*from, to));
        let nearer = nearer(view, at, 10.0);
        for (x, y, u, v) in [(-0.5, 0.5, 0.0, 0.0), (0.5, 0.5, 1.0, 0.0), (-0.5, -0.5, 0.0, 1.0), (-0.5, -0.5, 0.0, 1.0), (0.5, 0.5, 1.0, 0.0), (0.5, -0.5, 1.0, 1.0)] {
            let (x, y) = (x * wide * SPRITE_SCALE, y * tall * SPRITE_SCALE);
            quads.corner(view, nearer, along(along(at, view.right, x * cosine + y * sine), view.up, y * cosine - x * sine), [u, v], [1.0; 4]);
        }
        quads.picture(&binds, false);
    }
}
