//! The effects of this frame, each kind turned into quads for the renderer: the effects played from the
//! client's files, the casts, First Aid's glitter, the effects made of the client's primitives, the bolts
//! falling and the arrows in flight.

use super::*;

/// The effects playing from files: every layer's quad as it is at this frame of the effect, around the one it
/// plays on, by the client's own figures (a pixel of the effect is a pixel of a sprite).
pub(super) fn files(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    let Some(scene) = &state.scene else { return };
    state.effects.retain(|playing| ((time - playing.began) / 1000.0 * f64::from(playing.shown.effect.frames_per_second)) as i32 <= playing.shown.effect.last_frame);
    for playing in &state.effects {
        let Some(place) = scene.place_of(playing.on) else { continue };
        // The effect's middle stands 80 pixels over the place of the one it plays on (the client's
        // `CEZ2STRScreenEffect::OnProcess` moves it up the screen by that much).
        let middle = along(on_ground(&scene.gat, place), view.up, 80.0 * SPRITE_SCALE);
        let nearer = nearer(view, middle, 25.0);
        let frame = ((time - playing.began) / 1000.0 * f64::from(playing.shown.effect.frames_per_second)) as i32;
        for (layer, pictures) in playing.shown.effect.layers.iter().zip(&playing.shown.pictures) {
            let Some(transform) = layer.at(frame) else { continue };
            let Some(Some(binds)) = pictures.get(transform.aniframe.max(0.0) as usize) else { continue };
            let colour = transform.colour.map(|part| (part / 255.0).clamp(0.0, 1.0));
            if colour[3] <= 0.0 {
                continue;
            }
            let corners = transform.corners();
            for corner in [0, 1, 3, 3, 1, 2] {
                let ([x, y], uv) = corners[corner];
                quads.corner(view, nearer, along(along(middle, view.right, x * SPRITE_SCALE), view.up, -y * SPRITE_SCALE), uv, colour);
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

/// First Aid's glitter over whoever used it, for as long as it shows (180 of the client's frames).
pub(super) fn aids(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    state.aids.retain(|(_, began, ..)| time - began < 180.0 * CAST_FRAME_MS);
    let Some(binds) = gpu.effect_pictures.get("pikapika2.bmp") else { return };
    for (on, began, turned, pulse) in &state.aids {
        let Some(at) = scene.place_of(*on) else { continue };
        if aid_glitter(on_ground(&scene.gat, at), view.right, view.up, ((time - began) / CAST_FRAME_MS) as f32, *turned, *pulse, &mut quads.corners) {
            quads.picture(binds, true);
        }
    }
}

/// The effects made of the client's primitives: each is told, before its first tick, which way the one it is on
/// faces, lives its ticks, and is drawn around that one, a touch nearer than where it stands.
pub(super) fn bursts(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    for burst in state.bursts.iter_mut().filter(|burst| burst.begins <= time) {
        let facing = match burst.on {
            Some(id) => scene.units.get(&id).map_or(0, |unit| unit.facing),
            None => scene.player.facing,
        };
        // The step on the ground that facing is (as `map::direction` numbers them, from the south round by the west).
        let step = [(0.0, -1.0), (-1.0, -1.0), (-1.0, 0.0), (-1.0, 1.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (1.0, -1.0)][usize::from(facing % 8)];
        let length = f32::hypot(step.0, step.1);
        burst.face((step.0 / length, step.1 / length));
    }
    state.bursts.retain_mut(|burst| burst.begins > time || burst.live(time));
    for burst in state.bursts.iter().filter(|burst| burst.begins <= time) {
        let Some(at) = scene.place_of(burst.on) else { continue };
        let place = on_ground(&scene.gat, at);
        let nearer = nearer(view, place, 25.0);
        burst.draw(place, view.right, view.up, |picture, corners, colour| {
            let Some(binds) = gpu.effect_pictures.get(picture) else { return };
            for (at, uv) in corners {
                quads.corner(view, nearer, at, uv, colour);
            }
            quads.picture(binds, false);
        });
    }
}

/// The bolts falling: each a picture turned along its fall, its tip ahead, until it reaches the ground.
pub(super) fn bolts(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    const BOLT_SPEED: f32 = 2.75;
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    let mut falling = Vec::new();
    state.bolts.retain(|bolt| {
        if time < bolt.leaves {
            return true;
        }
        let Some(on) = scene.units.get(&bolt.target).map(|unit| unit.at).or((scene.units.is_empty() || bolt.target == 0).then_some(scene.player.at)) else { return false };
        let to = on_ground(&scene.gat, on);
        let from = [to[0] + bolt.from[0], to[1] + bolt.from[1], to[2] + bolt.from[2]];
        let way = subtract(to, from);
        let length = dot(way, way).sqrt().max(0.001);
        let gone = ((time - bolt.leaves) / CAST_FRAME_MS) as f32 * BOLT_SPEED;
        if gone >= length {
            return false;
        }
        falling.push((along(from, way, gone / length), way, bolt.fire, ((time - bolt.leaves) / CAST_FRAME_MS / 2.0) as usize));
        true
    });
    for (at, way, fire, frame) in falling {
        let (name, long, wide) = if fire { (FIRE_BOLT[frame % FIRE_BOLT.len()], 14.0, 3.5) } else { ("icearrow.tga", 11.5, 3.8) };
        let Some(binds) = gpu.effect_pictures.get(name) else { continue };
        // The picture's tip is at its right: that end leads.
        let (ahead, aside) = on_screen(view, way);
        let nearer = nearer(view, at, 10.0);
        for (x, y, u, v) in [(-1.0, 0.5, 0.0, 0.0), (0.0, 0.5, 1.0, 0.0), (-1.0, -0.5, 0.0, 1.0), (-1.0, -0.5, 0.0, 1.0), (0.0, 0.5, 1.0, 0.0), (0.0, -0.5, 1.0, 1.0)] {
            let (x, y) = (x * long, y * wide);
            quads.corner(view, nearer, along(along(at, view.right, x * ahead[0] + y * aside[0]), view.up, x * ahead[1] + y * aside[1]), [u, v], [1.0; 4]);
        }
        quads.picture(binds, fire);
    }
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
    if !state.arrows.is_empty() && gpu.arrow.is_none() {
        let key = SpriteKey::Creature(ARROW.to_owned(), 0, 0);
        if let Some(picture) = figure::picture(&key) {
            let canvas = &picture.canvas;
            let texture = gpu.texture(canvas.width as u32, canvas.height as u32, &canvas.rgba);
            let binds = [0, 1].map(|which| gpu.textured(&gpu.effect_pipelines[which].get_bind_group_layout(1), &texture, &gpu.pixel_sampler));
            gpu.arrow = Some((Rc::new(binds), (canvas.width as f32, canvas.height as f32)));
        }
    }
    let Some((binds, (wide, tall))) = &gpu.arrow else { return };
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
        quads.picture(binds, false);
    }
}
