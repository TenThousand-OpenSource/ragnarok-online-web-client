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
        let view = scene.camera.view(feet, gpu.config.width as f32 / gpu.config.height as f32);

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
        for (mover, position, wanted) in std::iter::once((&mut scene.player, position, key)).chain(scene.units.values_mut().map(|unit| {
            let (position, key) = unit.now(time, yaw);
            (unit, position, key)
        })) {
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
            if let Some(shown) = &mover.shown {
                sprites.push((shown.clone(), on_ground(gat, position)));
            }
        }
        // What lies on the ground, once the item's picture is known.
        for (id, (_, item, at)) in &scene.drops {
            let Some((_, resource)) = data::item(*item) else { continue };
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

        // Whoever the pointer is on: the nearest of those whose picture it touches. The name goes under the feet,
        // as the original puts it, and the pointer becomes the one for talking or for fighting.
        let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
        let over = state.pointer.and_then(|pointer| pointed_at(gpu, scene, &view, pointer)).and_then(|(id, feet)| scene.units.get(&id).map(|unit| (0.0, id, unit, feet)));
        // The client's pointer in the world (the third of its pointers, turning): at the feet of whoever the
        // pointer is on if it is a monster, or else of the monster being fought. Never at an NPC's.
        let marked = over.as_ref().filter(|(_, _, unit, _)| unit.lively).map(|(_, id, _, _)| *id).or(state.chase.map(|(id, ..)| id)).and_then(|id| scene.units.get(&id)).filter(|unit| unit.dying.is_none());
        state.target_at = marked.map(|unit| on_ground(&scene.gat, unit.at));
        state.over = over.as_ref().filter(|(_, _, unit, _)| matches!(unit.look, Look::Creature(_))).map(|(_, id, unit, _)| (*id, unit.lively));
        // The numbers of the hits rise from the heads of those hit, and are gone in a moment.
        for (target, damage) in std::mem::take(&mut state.hits) {
            let (hurt, own) = match scene.units.get(&target) {
                Some(unit) => (unit, false),
                None => (&scene.player, true),
            };
            // The client sends the number off by the quarter the one struck faces (`MsgEffectDrift`).
            let drift = match hurt.facing {
                0 | 1 => (-0.8, 0.8),
                2 | 3 => (-0.8, -0.8),
                4 | 5 => (0.8, -0.8),
                _ => (0.8, 0.8),
            };
            // Blows taken by the player's own character are red.
            let sprite = if own { "이팩트/숫자#ff0000" } else { "이팩트/숫자" };
            let pictures = if damage > 0 {
                damage.to_string().bytes().map(|digit| SpriteKey::Creature(sprite.to_owned(), 0, usize::from(digit - b'0'))).collect()
            } else {
                vec![SpriteKey::Creature("이팩트/msg".to_owned(), 0, 0)]
            };
            state.flying.push(Flying { began: time, base: on_ground(&scene.gat, hurt.at), drift, pictures, miss: damage <= 0 });
        }
        // The life of the monsters being fought, as a bar under each.
        let bars: String = scene
            .units
            .values()
            .filter_map(|unit| {
                let (left, most) = unit.life.filter(|(left, most)| *most > 0 && left < most)?;
                let under = view.project(on_ground(&scene.gat, unit.at))?;
                let full = (left.max(0) as f32 / most as f32 * 58.0).round();
                // 12 pixels of a screen 480 high under the feet, nearer when the camera is (the client's place for
                // a monster's gauge; its name stands at 20).
                let below = height * 12.0 / 480.0 * DISTANCE / scene.camera.distance;
                Some(format!(r#"<div class="life" style="left:{:.0}px;top:{:.0}px"><i style="width:{full}px"></i></div>"#, under.0 * width - 30.0, under.1 * height + below))
            })
            .collect();
        // Under the player's own character: life in green over spirit in blue, as the original shows them.
        let mut bars = bars;
        if let Some(under) = view.project(feet).filter(|_| state.own[1] > 0) {
            let part = |left: i64, most: i64| if most > 0 { (left.clamp(0, most) as f32 / most as f32 * 58.0).round() } else { 0.0 };
            let low = if state.own[0] * 4 < state.own[1] { " low" } else { "" };
            bars.push_str(&format!(
                r#"<div class="life own" style="left:{:.0}px;top:{:.0}px"><i class="hp{low}" style="width:{}px"></i><i class="sp" style="width:{}px"></i></div>"#,
                under.0 * width - 30.0,
                under.1 * height + height * 12.0 / 480.0 * DISTANCE / scene.camera.distance,
                part(state.own[0], state.own[1]),
                part(state.own[2], state.own[3]),
            ));
        }
        // The bar of a cast, over the caster (the client's `UIRechargeGage`): 60 by 6, 81 pixels over the feet,
        // filling over the cast's time.
        for (at, began, lasts, ..) in &casts {
            if let Some(under) = view.project(on_ground(&scene.gat, *at)) {
                let full = ((60.0 * ((time - began) / lasts.max(1.0)).clamp(0.0, 1.0) as f32) - 2.0).max(0.0).floor();
                let over = 81.0 * DISTANCE / scene.camera.distance * height / 480.0;
                bars.push_str(&format!(r#"<div class="cast" style="left:{:.0}px;top:{:.0}px"><i style="width:{full}px"></i></div>"#, under.0 * width - 30.0, under.1 * height - over));
            }
        }
        // What was said hangs over the head of whoever said it for a few seconds.
        for mover in std::iter::once(&scene.player).chain(scene.units.values()) {
            let Some((words, _)) = mover.says.as_ref().filter(|(_, when)| time - when <= 5000.0) else { continue };
            if let Some(over) = view.project(along(on_ground(&scene.gat, mover.at), [0.0, 1.0, 0.0], 21.0)) {
                let safe = words.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
                bars.push_str(&format!(r#"<div class="say" style="left:{:.0}px;top:{:.0}px">{safe}</div>"#, over.0 * width, over.1 * height));
            }
        }
        if !bars.is_empty() || state.bars {
            state.bars = !bars.is_empty();
            if let Some(element) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("bars")) {
                element.set_inner_html(&bars);
            }
        }
        // A portal is pointed at anywhere over its rings.
        let near = DISTANCE / scene.camera.distance * height / 480.0;
        let on_portal = state.pointer.is_some_and(|pointer| {
            scene.portals.values().any(|cell| {
                view.project(on_ground(&scene.gat, (cell.0 as f32, cell.1 as f32))).is_some_and(|middle| {
                    let (across, up) = ((pointer.0 - middle.0 * width).abs(), middle.1 * height - pointer.1);
                    across < 45.0 * near && (-25.0 * near..45.0 * near).contains(&up)
                })
            })
        });
        let pointing = if state.turning.is_some() { 5 } else { usize::MAX };
        let pointing = match &over {
            _ if pointing == 5 => 5,
            _ if state.aiming.is_some() => 6,
            Some((_, _, unit, _)) if unit.lively => 2,
            // A thing lying at an NPC's feet is still picked up.
            Some(_) if state.pointer.is_some_and(|pointer| lying_at(gpu, scene, &view, pointer).is_some()) => 4,
            Some((_, _, unit, _)) if matches!(unit.look, Look::Creature(_)) => 1,
            None if state.pointer.is_some_and(|pointer| lying_at(gpu, scene, &view, pointer).is_some()) => 4,
            None if on_portal => 3,
            _ => 0,
        };
        // The pointer's pictures run in turn, as the original's do (the hand over a thing to pick up grabs).
        let pointer_frame = state.pointers.get(pointing).map_or(0, |(pictures, shows)| (time / shows) as usize % pictures.len().max(1));
        if pointing != state.pointing || pointer_frame != state.pointer_frame {
            state.pointing = pointing;
            state.pointer_frame = pointer_frame;
            if let Some(pointer) = state.pointers.get(pointing).and_then(|(pictures, _)| pictures.get(pointer_frame)) {
                let _ = gpu.canvas.style().set_property("cursor", pointer);
            }
        }
        match &over {
            Some((_, id, unit, feet)) if !unit.name.is_empty() => {
                state.named = Some(*id);
                // 20 pixels of a screen 480 high under the feet, nearer when the camera is.
                let below = height * 20.0 / 480.0 * DISTANCE / scene.camera.distance;
                name_shown(Some((&unit.name, unit.lively, (feet.0, feet.1 + below))));
            }
            // With nothing pointed at, the name of the monster being fought stays (the client's lock-on).
            _ if state.chase.and_then(|(id, ..)| scene.units.get(&id)).is_some_and(|unit| !unit.name.is_empty() && unit.dying.is_none()) => {
                if let Some(unit) = state.chase.and_then(|(id, ..)| scene.units.get(&id)) {
                    if let Some(feet) = view.project(on_ground(&scene.gat, unit.at)) {
                        state.named = state.chase.map(|(id, ..)| id);
                        let below = height * 20.0 / 480.0 * DISTANCE / scene.camera.distance;
                        name_shown(Some((&unit.name, unit.lively, (feet.0 * width, feet.1 * height + below))));
                    }
                }
            }
            _ if state.named.take().is_some() => name_shown(None),
            _ => {}
        }

        // While the left button is held on the ground, the character keeps walking to where the pointer is.
        let follow = state.held.filter(|_| time - state.held_asked > 250.0).and_then(|held| {
            let picked = view.pick(&scene.gat, (held.0 / width.max(1.0), held.1 / height.max(1.0)))?;
            let cell = scene.player.cell;
            let goal = if scene.gat.path(cell, picked).is_empty() { scene.gat.reachable_towards(cell, picked)? } else { picked };
            (goal != cell).then_some(goal)
        });
        if follow.is_some() {
            state.held_asked = time;
        }
        let mut follow = follow.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));

        // A thing to pick up is walked to, and picked up from beside it.
        let mut take = None;
        if let Some((id, looked)) = state.fetch {
            match scene.drops.get(&id) {
                None => state.fetch = None,
                Some((cell, ..)) if time - looked > 350.0 => {
                    let own = scene.player.cell;
                    if (cell.0 - own.0).abs().max((cell.1 - own.1).abs()) <= 1 {
                        take = state.hooks.as_ref().map(|hooks| (hooks.take, id));
                    } else if scene.player.walk.is_none() {
                        let goal = if scene.gat.path(own, *cell).is_empty() { scene.gat.reachable_towards(own, *cell) } else { Some(*cell) };
                        follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                    }
                    state.fetch = Some((id, time));
                }
                Some(_) => {}
            }
        }
        // The monster being fought is run after when it is out of reach, and struck again once it is near.
        let mut strike = None;
        if let Some((id, looked, told)) = state.chase {
            match scene.units.get(&id) {
                None => state.chase = None,
                Some(unit) if time - looked > 400.0 => {
                    let cell = scene.player.cell;
                    // Near enough is what the server said the weapon reaches. Until it has said (it does when a
                    // blow is asked for from too far), the character stays where it is: a bow reaches far, and
                    // its archer is not to run up to the monster.
                    let range = state.strike_range.unwrap_or(30).max(1);
                    let far = (unit.cell.0 - cell.0).abs().max((unit.cell.1 - cell.1).abs()) > range;
                    if far {
                        // The way to the monster, as far as the cell from which the weapon reaches it.
                        let way = scene.gat.path(cell, unit.cell);
                        let goal = if way.is_empty() { scene.gat.reachable_towards(cell, unit.cell) } else { way.get(way.len().saturating_sub(1 + range as usize).min(way.len() - 1)).copied().filter(|goal| *goal != cell).or(Some(unit.cell)) };
                        follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                    } else if !told || (scene.player.walk.is_none() && time - state.own_struck > 1500.0 && time - state.strike_asked > 1500.0) {
                        // Near enough: the server is told to strike. It is told again when nothing came of it
                        // (it drops a request that reaches it while the character still ends something else,
                        // as after the player changed targets quickly), so the character does not stand idle.
                        strike = state.hooks.as_ref().map(|hooks| (hooks.press, id));
                        state.strike_asked = time;
                    }
                    state.chase = Some((id, time, !far));
                }
                Some(_) => {}
            }
        }
        // A skill aimed out of its reach: the character walks towards whom or where it was aimed, and uses it
        // as soon as it is near enough. The client's rule (`CPlayer::UpdatePendingAction`): the skill's reach in
        // cells of 5, and 5.5 more, is at least the distance.
        let mut used = None;
        if let Some((whom, cell, range, looked)) = state.reach {
            let aimed = match whom {
                Some(id) => scene.units.get(&id).filter(|unit| unit.dying.is_none()).map(|unit| (unit.cell, unit.at)),
                None => Some((cell, (cell.0 as f32, cell.1 as f32))),
            };
            match aimed {
                None => state.reach = None,
                Some((cell, at)) => {
                    let (across, along) = (at.0 - scene.player.at.0, at.1 - scene.player.at.1);
                    if (across * across + along * along).sqrt() * 5.0 <= range * 5.0 + 5.5 {
                        used = state.hooks.as_ref().map(|hooks| (hooks.aimed, whom, (cell.0.max(0) as u16, cell.1.max(0) as u16)));
                        state.reach = None;
                    } else if time - looked > 350.0 {
                        if scene.player.walk.is_none() {
                            let own = scene.player.cell;
                            let goal = if scene.gat.path(own, cell).is_empty() { scene.gat.reachable_towards(own, cell) } else { Some(cell) };
                            follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                        }
                        state.reach = Some((whom, cell, range, time));
                    }
                }
            }
        }

        let cell = scene.player.cell;
        let changed = cell != scene.told;
        scene.told = cell;
        Some((state.hooks.as_ref().filter(|_| changed).map(|hooks| (hooks.moved, cell)), follow, strike, take, used))
    });
    if let Some((moved, follow, strike, take, used)) = told {
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

/// Who a point of the page is on: the nearest of those whose picture it touches, and where on the page their
/// feet are.
pub(super) fn pointed_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32)) -> Option<(u32, (f32, f32))> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
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
            let inside = (low.0 * width..=high.0 * width).contains(&pointer.0) && (low.1 * height..=high.1 * height).contains(&pointer.1);
            let under = view.project(feet)?;
            inside.then(|| (dot(subtract(feet, view.eye), view.forward), *id, (under.0 * width, under.1 * height)))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id, feet)| (id, feet))
}

/// The thing on the ground a point of the page is on.
pub(super) fn lying_at(gpu: &Gpu, scene: &Scene, view: &View, pointer: (f32, f32)) -> Option<u32> {
    let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
    scene.drops.iter().find_map(|(id, (_, _, at))| {
        let place = view.project(on_ground(&scene.gat, *at))?;
        let (across, up) = ((pointer.0 - place.0 * width).abs(), place.1 * height - pointer.1);
        (across < 16.0 && (-8.0..26.0).contains(&up)).then_some(*id)
    })
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
