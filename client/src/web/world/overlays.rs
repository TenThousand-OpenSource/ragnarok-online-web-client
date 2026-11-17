//! What is laid over the picture of the world once it is drawn: the pointer's mark and its picture, the name of
//! whoever it is on, the numbers of the blows, the bars of life and of casts, what was said over heads.

use super::*;

/// Works those out for this frame and puts them on the page. `feet` is where the player's own character stands
/// and `casts` the casts under way (where, since when, for how long, which effect, at whom), as the frame found
/// them while drawing. Gives back the page's size, which what follows needs too.
pub(super) fn overlays(state: &mut State, view: &View, time: f64, feet: Vector, casts: &[((f32, f32), f64, f64, u32, Option<u32>)]) -> Option<(f32, f32)> {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return None };
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
        // A miss is shown only when the player has it so (the client's `/miss`).
        if damage > 0 || state.options.misses {
            state.flying.push(Flying { began: time, base: on_ground(&scene.gat, hurt.at), drift, pictures, miss: damage <= 0 });
        }
    }
    // The life of the monsters being fought, as a bar under each.
    let bars: String = scene
        .units
        .values()
        .filter_map(|unit| {
            let (left, most) = unit.life.filter(|(left, most)| state.options.monster_life && *most > 0 && left < most)?;
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
    for (at, began, lasts, ..) in casts {
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
        // A skill being aimed: the pointer of a skill, and its other picture over a monster it can go to
        // (the client's cursor actions 10 and 11).
        Some((_, _, unit, _)) if state.aiming == Some(false) && unit.lively => 7,
        _ if state.aiming.is_some() => 6,
        // Over a pet the pointer stays plain, as in the client.
        Some((_, _, unit, _)) if unit.companion => 0,
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
    Some((width, height))
}
