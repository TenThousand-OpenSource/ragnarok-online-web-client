//! One frame of the world: everyone is moved to where they are now, the effects advance, the picture is drawn,
//! and the page's overlays (names, bars, the pointer) follow.

use super::*;

/// Draws one frame, and asks for the next while there is a world to draw.
pub(super) fn frame() {
    // The effects and sounds whose moment has come.
    let (effects, sounds) = STATE.with_borrow_mut(|state| {
        let time = now();
        let effects: Vec<_> = state.effects_due.extract_if(.., |due| due.0 <= time).collect();
        let sounds: Vec<_> = state.sounds_due.extract_if(.., |due| due.0 <= time).collect();
        (effects, sounds)
    });
    for (_, name, on) in effects {
        effect(&name, on);
    }
    for (_, name) in sounds {
        sound::effect(&name, 1.0);
    }
    let told = STATE.with_borrow_mut(|state| {
        let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return None };
        gpu.fit();
        let (time, yaw) = (now(), scene.camera.yaw);
        let (position, key) = scene.player.now(time, yaw);
        let feet = on_ground(&scene.gat, position);
        if state.camera_fixed {
            scene.aim = None;
        }
        let aim = scene.follow(feet, time);
        let view = scene.camera.view(aim, gpu.config.width as f32 / gpu.config.height as f32);

        // Everyone to draw: each with the picture wanted now if it is ready, else the one drawn last.
        // The blows that land now: the one struck flinches, the flash and the number show, the blow sounds.
        let mut landed = Vec::new();
        state.landing.retain(|blow| {
            if blow.0 > time {
                return true;
            }
            landed.push((blow.1, blow.2, blow.3, blow.4.clone()));
            false
        });
        for (target, own, damage, heard) in landed {
            let Some(hurt) = (if target == own { Some(&mut scene.player) } else { scene.units.get_mut(&target) }) else { continue };
            if damage > 0 {
                if hurt.motion.is_none() && hurt.dying.is_none() && !hurt.dead {
                    hurt.motion = Some((if matches!(hurt.look, Look::Character(_)) { 48 } else { 24 }, time, 280.0));
                }
                state.sparks.push((along(on_ground(&scene.gat, hurt.at), [0.0, 1.0, 0.0], 7.0), time));
                if let Some(heard) = heard {
                    sound::effect(&heard, 1.0);
                }
            }
            state.hits.push((target, damage));
        }
        // Monsters struck down are gone once they have fallen.
        scene.units.retain(|_, unit| unit.dying.is_none_or(|fell| time - fell < 1000.0));
        let mut sprites = Vec::with_capacity(scene.units.len() + 1);
        let gat = &scene.gat;
        let feet_cell = position;
        let hide_players = state.options.hide_players;
        let mut own = true;
        for (mover, position, wanted) in std::iter::once((&mut scene.player, position, key)).chain(scene.units.values_mut().map(|unit| {
            let (position, key) = unit.now(time, yaw);
            (unit, position, key)
        })) {
            // The first is the player's own character.
            let is_own = std::mem::replace(&mut own, false);
            // A frame of a monster's sprite may start a sound: the nearer the monster, the louder.
            if let SpriteKey::Creature(path, action, frame) = &wanted {
                if mover.heard.replace((*action, *frame)) != Some((*action, *frame)) {
                    if let Some(name) = figure::sound_of(path, *action, *frame) {
                        let away = ((position.0 - feet_cell.0).powi(2) + (position.1 - feet_cell.1).powi(2)).sqrt();
                        sound::effect(&name, f64::from(1.0 - away / 18.0));
                    }
                }
            }
            if gpu.has_sprite(&wanted) {
                mover.shown = Some(wanted);
            }
            // Other players are left out when the player has it so (the client's Hide other Players).
            let hidden = hide_players && !is_own && matches!(mover.look, Look::Character(_));
            if let Some(shown) = mover.shown.as_ref().filter(|_| !hidden) {
                sprites.push((shown.clone(), on_ground(gat, position)));
            }
            // The emotion it shows: the pictures of that action, each for as long as the sprite says, once through.
            // Until the sprite is read its length is not known: it is given four seconds to come.
            if let Some((sprite, action, began)) = &mover.emotion {
                let passed = time - began;
                let picture = match figure::timing(sprite, *action) {
                    Some((pictures, each)) => Some(((passed / each.max(1.0)) as usize, pictures)).filter(|(at, pictures)| at < pictures).map(|(at, _)| at),
                    None => Some(0).filter(|_| passed < 4000.0),
                };
                match picture {
                    Some(picture) => {
                        let key = SpriteKey::Creature(sprite.clone(), *action, picture);
                        // It follows the one who shows it, 20 above their feet (the client's `CMsgEffect`, for
                        // `MET_EMOTION`); the rest of the way up is in the sprite's own pictures.
                        if gpu.has_sprite(&key) {
                            sprites.push((key, along(on_ground(gat, position), [0.0, 1.0, 0.0], 20.0)));
                        }
                    }
                    None => mover.emotion = None,
                }
            }
        }
        // What lies on the ground, once the item's picture is known.
        for (id, (_, item, at, identified)) in &scene.drops {
            // One not identified lies there as what the client's table shows it as until it is.
            let Some((_, resource)) = data::item_as(*item, *identified) else { continue };
            let key = SpriteKey::Creature(format!("아이템/{}", if resource.is_empty() { "사과" } else { resource.as_str() }), 0, 0);
            if gpu.has_sprite(&key) {
                // A thing that fell out of a monster starts 15 above the ground, is thrown up a little and
                // falls (the client's `CItem::OnProcess`, in its frames of 24 milliseconds).
                let above = scene.falls.get(id).map_or(0.0, |began| {
                    let elapsed = ((time - began) / EFFECT_FRAME_MS) as f32;
                    (15.0 + 0.6 * elapsed - 0.083 * elapsed * elapsed).max(0.0)
                });
                sprites.push((key, along(on_ground(gat, *at), [0.0, 1.0, 0.0], above)));
            }
        }
        // Last frame's place of the pointer in the world: its pictures turn at two of the client's frames each.
        if let Some(at) = state.target_at {
            let pictures = figure::timing("cursors", 3).map_or(1, |(count, _)| count.max(1));
            let key = SpriteKey::Creature("cursors".to_owned(), 3, (time / EFFECT_FRAME_MS / 2.0) as usize % pictures);
            if gpu.has_sprite(&key) {
                sprites.push((key, at));
            }
        }
        let mut ring_corners = Vec::new();
        for cell in scene.portals.values() {
            rings(on_ground(gat, (cell.0 as f32, cell.1 as f32)), time, &mut ring_corners);
        }
        // A blow that lands flashes: a ring of light opening for a moment where it struck.
        state.sparks.retain(|(_, struck)| time - struck < 180.0);
        for (at, struck) in &state.sparks {
            spark(*at, ((time - struck) / 180.0) as f32, &mut ring_corners);
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
        // The numbers in the air, by the client's own figures. A number starts five times its size and shrinks
        // to a little over it, rises and falls back as it drifts, and fades: all over 70 of the client's frames.
        // A miss only rises, and fades over 80.
        state.flying.retain(|flying| (time - flying.began) / EFFECT_FRAME_MS <= if flying.miss { 80.0 } else { 70.0 });
        let mut numbers = Vec::new();
        for flying in &state.flying {
            let elapsed = ((time - flying.began) / EFFECT_FRAME_MS) as f32;
            let (at, scale, alpha) = if flying.miss {
                (along(flying.base, [0.0, 1.0, 0.0], 12.0 + elapsed * 0.54), 1.0, (250.0 - elapsed * 3.0) / 255.0)
            } else {
                let aside = elapsed / 3.0 + 3.0;
                let up = 12.0 - 8.0 + (2.0 - elapsed / 30.0) * elapsed;
                ([flying.base[0] + flying.drift.0 * aside, flying.base[1] + up, flying.base[2] + flying.drift.1 * aside], (5.0 - elapsed * 0.24).max(1.2), (250.0 - elapsed * 3.4) / 255.0)
            };
            let mut across = 0.0;
            for key in &flying.pictures {
                if !gpu.has_sprite(key) {
                    continue;
                }
                let wide = gpu.sprite_textures[key].size.0;
                numbers.push((key.clone(), at, scale, alpha, across - (flying.pictures.len() as f32 - 1.0) * wide * 0.4));
                across += wide * 0.8;
            }
        }
        // The effects playing: every layer's quad as it is at this frame of the effect, around the one it plays
        // on, by the client's own figures (a pixel of the effect is a pixel of a sprite).
        state.effects.retain(|playing| ((time - playing.began) / 1000.0 * f64::from(playing.shown.effect.frames_per_second)) as i32 <= playing.shown.effect.last_frame);
        let mut effects = (Vec::new(), Vec::new());
        for playing in &state.effects {
            let place = match playing.on {
                None => Some(position),
                Some(id) => scene.units.get_mut(&id).map(|unit| unit.now(time, yaw).0),
            };
            let Some(place) = place else { continue };
            // The effect's middle stands 80 pixels over the place of the one it plays on (the client's
            // `CEZ2STRScreenEffect::OnProcess` moves it up the screen by that much).
            let middle = along(on_ground(&scene.gat, place), view.up, 80.0 * SPRITE_SCALE);
            let nearer = 1.0 - 25.0 / dot(subtract(middle, view.eye), view.forward).max(40.0);
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
                    let at = along(along(middle, view.right, x * SPRITE_SCALE), view.up, -y * SPRITE_SCALE);
                    effects.0.extend(along(view.eye, subtract(at, view.eye), nearer));
                    effects.0.extend(uv);
                    effects.0.extend(colour);
                }
                // Direct3D's "one" as the destination (2) adds the quad's light to what is there; so does "as
                // much as the destination's alpha" (7), the screen having none to speak of.
                effects.1.push((binds.clone(), matches!(transform.destination_blend, 2 | 7)));
            }
        }
        // The casts under way: the rings around each caster and the mark on the one aimed at.
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
                let quads = cast_rings(on_ground(&scene.gat, *at), frame, ((lasts / CAST_FRAME_MS) as f32).max(70.0), tint, &mut effects.0);
                effects.1.extend(std::iter::repeat_n((binds.clone(), adds), quads));
            }
            let aimed = target.and_then(|id| scene.units.get(&id)).map(|unit| unit.at);
            if let (Some(aimed), Some(binds)) = (aimed, gpu.effect_pictures.get("lockon128.tga")) {
                let quads = lock_on(on_ground(&scene.gat, aimed), frame, &mut effects.0);
                effects.1.extend(std::iter::repeat_n((binds.clone(), false), quads));
            }
        }
        // The bolts falling: each a picture turned along its fall, its tip ahead, until it reaches the ground.
        const BOLT_SPEED: f32 = 2.75;
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
            let (across, up) = (dot(way, view.right), dot(way, view.up));
            let length = (across * across + up * up).sqrt().max(0.001);
            // The picture's tip is at its right: that end leads.
            let (ahead, aside) = ([across / length, up / length], [-up / length, across / length]);
            let nearer = 1.0 - 10.0 / dot(subtract(at, view.eye), view.forward).max(20.0);
            for (x, y, u, v) in [(-1.0, 0.5, 0.0, 0.0), (0.0, 0.5, 1.0, 0.0), (-1.0, -0.5, 0.0, 1.0), (-1.0, -0.5, 0.0, 1.0), (0.0, 0.5, 1.0, 0.0), (0.0, -0.5, 1.0, 1.0)] {
                let (x, y) = (x * long, y * wide);
                let corner = along(along(at, view.right, x * ahead[0] + y * aside[0]), view.up, x * ahead[1] + y * aside[1]);
                effects.0.extend(along(view.eye, subtract(corner, view.eye), nearer));
                effects.0.extend([u, v]);
                effects.0.extend([1.0, 1.0, 1.0, 1.0]);
            }
            effects.1.push((binds.clone(), fire));
        }
        // The arrows in flight: each a picture turned along its way on the screen, from the archer to its target.
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
        if let Some((binds, (wide, tall))) = &gpu.arrow {
            for (leaves, from, target) in &state.arrows {
                if time < *leaves {
                    continue;
                }
                let Some(to) = scene.units.get(target).map(|unit| unit.at).or((*target != 0).then_some(scene.player.at)) else { continue };
                let to = along(on_ground(&scene.gat, to), [0.0, 1.0, 0.0], 10.0);
                let at = along(*from, subtract(to, *from), ((time - leaves) / ARROW_MS) as f32);
                // The way it flies as the screen shows it: the picture's head is at its bottom, and is turned to
                // lead.
                let way = subtract(*from, to);
                let (across, up) = (dot(way, view.right), dot(way, view.up));
                let length = (across * across + up * up).sqrt().max(0.001);
                let (cosine, sine) = (up / length, across / length);
                let nearer = 1.0 - 10.0 / dot(subtract(at, view.eye), view.forward).max(20.0);
                for (x, y, u, v) in [(-0.5, 0.5, 0.0, 0.0), (0.5, 0.5, 1.0, 0.0), (-0.5, -0.5, 0.0, 1.0), (-0.5, -0.5, 0.0, 1.0), (0.5, 0.5, 1.0, 0.0), (0.5, -0.5, 1.0, 1.0)] {
                    let (x, y) = (x * wide * SPRITE_SCALE, y * tall * SPRITE_SCALE);
                    let corner = along(along(at, view.right, x * cosine + y * sine), view.up, y * cosine - x * sine);
                    effects.0.extend(along(view.eye, subtract(corner, view.eye), nearer));
                    effects.0.extend([u, v]);
                    effects.0.extend([1.0, 1.0, 1.0, 1.0]);
                }
                effects.1.push((binds.clone(), false));
            }
        }
        gpu.draw(scene, &view, &mut sprites, &ring_corners, &marks, &numbers, &effects);

        let (width, height) = overlays(state, &view, time, feet, &casts)?;

        let Pursuit { follow, strike, take, used, once } = pursue(state, &view, (width, height), time);
        let scene = state.scene.as_mut()?;

        let cell = scene.player.cell;
        let changed = cell != scene.told;
        scene.told = cell;
        Some((state.hooks.as_ref().filter(|_| changed).map(|hooks| (hooks.moved, cell)), follow, strike, take, used, once))
    });
    if let Some((moved, follow, strike, take, used, once)) = told {
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
    if STATE.with_borrow(|state| state.scene.is_some()) {
        next_frame();
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

/// Shows a name on the page, its middle at a place, or takes it away. Monsters' names are pink, as the original's.
pub(super) fn name_shown(named: Option<(&str, bool, (f32, f32))>) {
    let Some(label) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("unit-name")) else { return };
    match named {
        Some((name, monster, (x, y))) => {
            if label.text_content().as_deref() != Some(name) {
                label.set_text_content(Some(name));
            }
            let colour = if monster { "#ffc3c3" } else { "#fff" };
            let _ = label.set_attribute("style", &format!("display:block;left:{x:.0}px;top:{y:.0}px;color:{colour}"));
        }
        None => {
            let _ = label.set_attribute("style", "display:none");
        }
    }
}
