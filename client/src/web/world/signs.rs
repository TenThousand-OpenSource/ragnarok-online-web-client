//! The signs that stand over heads: of a chat room over the one it belongs to, of a shop over its vendor
//! (the client's `UIChatRoomTitle` and `UIMerchantShopTitle`, both 140 by 34, which an actor carries as its
//! `actorWindows[1]` and `[2]`). A sign is something of the page that is pressed, so it is HTML that its
//! system's window file writes (`hud/`); the world is told only whom it is over, keeps it over that one's
//! head frame after frame, and drops it when that one goes out of sight, as the client's actor takes its
//! windows with it.

use super::*;

/// Which of the two signs of an actor: each has its own place over the head.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Board {
    /// A chat room's (`AM_MAKE_CHAT_TITLE`).
    Room,
    /// A shop's, which stands 17 higher (`CGameActor::UpdateGaugePositions`).
    Shop,
    /// The mark over one of another team on a battleground (`UIWordDisplayWnd`): 24 square, its corner 5 of a
    /// screen 640 wide left of the feet and 100 of 480 over them, nearer when the camera is; not pressed.
    Foe,
}

/// The signs standing now.
#[derive(Default)]
pub(super) struct Signs {
    /// The visit to a map they are of (`State::entered`): another map has none of them.
    visit: u32,
    /// By owner (none is the player's own character) and kind: what the sign is made of, and whether its
    /// owner was seen on the map since it was set.
    boards: BTreeMap<(Option<u32>, Board), (String, bool)>,
    /// The page does not have them as they are now: they are written anew at the next frame.
    stale: bool,
}

/// The size of a sign (`AM_MAKE_CHAT_TITLE`: `SetSize(140, 34)`).
pub const SIGN: (i32, i32) = (140, 34);

/// Sets the sign of that kind over someone, or takes it away: `html` is what stands in its 140 by 34. `own`
/// is the player's own ID.
pub fn sign(owner: u32, own: u32, board: Board, html: Option<String>) {
    STATE.with_borrow_mut(|state| {
        if state.signs.visit != state.entered {
            state.signs = Signs { visit: state.entered, ..Signs::default() };
        }
        let key = ((owner != own).then_some(owner), board);
        match html {
            Some(html) => state.signs.boards.insert(key, (html, false)).is_some(),
            None => state.signs.boards.remove(&key).is_some(),
        };
        state.signs.stale = true;
    });
}

/// Puts every sign over its owner's head on the page: the middle of its width over the owner's feet, its top
/// 17 over the line the client's gauges stand on, 81 of a screen 480 high over the feet and nearer when the
/// camera is (`CGameActor::UpdateGaugePositions`); a shop's 17 higher. The signs are kept in the element the
/// list of windows has for them (`hud/windows/chat_rooms.rs`), which lies on the stage: their places are in
/// the stage's measure. They are written anew only when one comes or goes, so a double click finds the sign
/// it began on.
pub(super) fn shown(state: &mut State, view: &View, (width, height): (f32, f32)) {
    let Some(scene) = &state.scene else { return };
    let signs = &mut state.signs;
    if signs.visit != state.entered {
        *signs = Signs { visit: state.entered, stale: !signs.boards.is_empty(), ..Signs::default() };
    }
    // A sign goes with its owner, once that one was seen and is in sight no more.
    let before = signs.boards.len();
    signs.boards.retain(|(owner, _), (_, seen)| {
        let here = scene.place_of(*owner).is_some();
        *seen |= here;
        here || !*seen
    });
    signs.stale |= signs.boards.len() != before;
    let Some(page) = crate::web::by_id("signs") else { return };
    if signs.stale || page.child_element_count() != signs.boards.len() as u32 {
        signs.stale = false;
        page.set_inner_html(&signs.boards.values().map(|(html, _)| format!(r#"<div class="a" style="display:none">{html}</div>"#)).collect::<String>());
    }
    let scale = crate::web::stage_scale() as f32;
    let over = crate::bars::over(crate::bars::CAST_OVER, height, scene.camera.distance);
    let mut next = page.first_element_child();
    for ((owner, board), _) in &signs.boards {
        let Some(shown) = next else { break };
        next = shown.next_element_sibling();
        let feet = scene.place_of(*owner).and_then(|at| view.project(on_ground(&scene.gat, at)));
        let style = match feet {
            Some(feet) if *board == Board::Foe => format!(
                "left:{:.0}px;top:{:.0}px;width:24px;height:24px;pointer-events:none",
                (feet.0 * width - crate::bars::over(5.0, width * 480.0 / 640.0, scene.camera.distance)) / scale,
                (feet.1 * height - crate::bars::over(100.0, height, scene.camera.distance)) / scale
            ),
            Some(feet) => {
                let higher = if *board == Board::Shop { 34.0 } else { 17.0 };
                format!(
                    "left:{:.0}px;top:{:.0}px;width:{}px;height:{}px;pointer-events:auto",
                    feet.0 * width / scale - SIGN.0 as f32 / 2.0,
                    (feet.1 * height - over) / scale - higher,
                    SIGN.0,
                    SIGN.1
                )
            }
            None => "display:none".to_owned(),
        };
        if shown.get_attribute("style").as_deref() != Some(style.as_str()) {
            let _ = shown.set_attribute("style", &style);
        }
    }
}
