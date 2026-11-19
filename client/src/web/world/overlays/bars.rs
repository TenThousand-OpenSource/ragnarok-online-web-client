//! The bars and words laid over the world: the life of the monsters being fought, the player's own life and
//! spirit, the casts under way, what was said over heads.

use super::*;
use crate::bars as figures;

pub(super) fn shown(state: &mut State, view: &View, time: f64, feet: Vector, casts: &[((f32, f32), f64, f64, u32, Option<u32>)], (width, height): (f32, f32)) {
    let Some(scene) = &state.scene else { return };
    // The life of the monsters being fought, as a bar under each; and of the members of the player's party,
    // whose life the server tells (`member_gauge`).
    let bars: String = scene
        .units
        .values()
        // (One that hides has no bar: the client's `AM_REMOVEGAGE`.)
        .filter(|unit| !unit.hidden())
        .filter_map(|unit| {
            let member = matches!(unit.look, Look::Character(_)) && !unit.companion;
            let (left, most) = unit.life.filter(|life| figures::shown(*life, member, state.options.monster_life))?;
            let under = view.project(on_ground(&scene.gat, unit.at))?;
            // 12 pixels of a screen 480 high under the feet, nearer when the camera is (the client's place for
            // a monster's gauge and a player's; the name stands at 20).
            let below = figures::below(figures::LIFE_BELOW, height, scene.camera.distance);
            let (x, y) = (figures::left(under.0, width), under.1 * height + below);
            if member {
                return Some(member_gauge(x, y, left, most));
            }
            let full = figures::monster(left, most);
            Some(format!(r#"<div class="life" style="left:{x:.0}px;top:{y:.0}px"><i style="width:{full}px"></i></div>"#))
        })
        .collect();
    // Under the player's own character: life in green over spirit in blue, as the original shows them.
    let mut bars = bars;
    // The life of the player's own homunculus and mercenary, under each (`ai.rs`).
    bars.push_str(&ai::gauges(state, scene, view, (width, height), member_gauge));
    if let Some(under) = view.project(feet).filter(|_| state.own[1] > 0) {
        let low = if figures::own_low(state.own[0], state.own[1]) { " low" } else { "" };
        bars.push_str(&format!(
            r#"<div class="life own" style="left:{:.0}px;top:{:.0}px"><i class="hp{low}" style="width:{}px"></i><i class="sp" style="width:{}px"></i></div>"#,
            figures::left(under.0, width),
            under.1 * height + figures::below(figures::LIFE_BELOW, height, scene.camera.distance),
            figures::own(state.own[0], state.own[1]),
            figures::own(state.own[2], state.own[3]),
        ));
    }
    // The bar of a cast, over the caster (the client's `UIRechargeGage`): 60 by 6, 81 pixels over the feet,
    // filling over the cast's time.
    // The bar a script fills over someone is the same gauge (`scripted.rs`).
    let scripted = scripted::gauges(&state.scripted, scene, state.entered, time);
    for (at, began, lasts) in casts.iter().map(|(at, began, lasts, ..)| (*at, *began, *lasts)).chain(scripted) {
        if let Some(under) = view.project(on_ground(&scene.gat, at)) {
            let full = figures::cast(time - began, lasts);
            let over = figures::over(figures::CAST_OVER, height, scene.camera.distance);
            bars.push_str(&format!(
                r#"<div class="cast" style="left:{:.0}px;top:{:.0}px"><i style="width:{full}px"></i></div>"#,
                figures::left(under.0, width),
                under.1 * height - over
            ));
        }
    }
    // The words of units on the ground stand over them as what is said does (`written.rs`).
    for (stands, words) in written::shown(&mut state.written, scene, time) {
        if let Some(over) = view.project(along(stands, [0.0, 1.0, 0.0], 21.0)) {
            bars.push_str(&said(&words, over, (width, height)));
        }
    }
    // What was said hangs over the head of whoever said it for a few seconds.
    for mover in std::iter::once(&scene.player).chain(scene.units.values()) {
        let Some((words, _)) = mover.says.as_ref().filter(|(_, when)| time - when <= 5000.0) else { continue };
        if let Some(over) = view.project(along(on_ground(&scene.gat, mover.at), [0.0, 1.0, 0.0], 21.0)) {
            bars.push_str(&said(words, over, (width, height)));
        }
    }
    if !bars.is_empty() || state.bars {
        state.bars = !bars.is_empty();
        if let Some(element) = crate::web::by_id("bars") {
            element.set_inner_html(&bars);
        }
    }
}

/// The bar of life under a member of the player's party (the client's `UIPcGage`, made on
/// ZC_NOTIFY_HP_TO_GROUPM: `AM_UPDATEGAGE`): 60 by 5 with a rim, shown whether the life is whole or not; what
/// is left is green, or red under a quarter, as wide as the whole hundredths of it of 60, less the rim.
fn member_gauge(x: f32, y: f32, left: i32, most: i32) -> String {
    let (full, low) = figures::member(left, most);
    let low = if low { r#" class="low""# } else { "" };
    format!(r#"<div class="life pc" style="left:{x:.0}px;top:{y:.0}px"><i{low} style="width:{full}px"></i></div>"#)
}

/// Words standing at a place of the screen (parts of a page so wide and high): what someone said, what is
/// written on a unit on the ground.
fn said(words: &str, over: (f32, f32), (width, height): (f32, f32)) -> String {
    let safe = words.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    format!(r#"<div class="say" style="left:{:.0}px;top:{:.0}px">{safe}</div>"#, over.0 * width, over.1 * height)
}
