//! The bars and words laid over the world: the life of the monsters being fought, the player's own life and
//! spirit, the casts under way, what was said over heads.

use super::*;

pub(super) fn shown(state: &mut State, view: &View, time: f64, feet: Vector, casts: &[((f32, f32), f64, f64, u32, Option<u32>)], (width, height): (f32, f32)) {
    let Some(scene) = &state.scene else { return };
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
        if let Some(element) = crate::web::by_id("bars") {
            element.set_inner_html(&bars);
        }
    }
}
