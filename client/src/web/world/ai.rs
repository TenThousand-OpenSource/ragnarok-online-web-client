//! What the scripts of a homunculus and a mercenary are told of the map (`crate::ai::Seen`: who is in sight,
//! where, doing what), the commands the player gives the two with the mouse, and the life under each of them.

use super::*;
use crate::ai::{Seen, motion};

/// What the world keeps of the player's homunculus and mercenary, as the app tells it.
#[derive(Default)]
pub(super) struct Owned {
    /// The units that are the two (0 for none), the homunculus first, and what each has left of its life
    /// and how much it has.
    units: [u32; 2],
    life: [(i64, i64); 2],
}

/// The app says which units are the player's homunculus and mercenary (0 for none), and the life of each.
pub fn companions(units: [u32; 2], life: [(i64, i64); 2]) {
    STATE.with_borrow_mut(|state| state.ai = Owned { units, life });
}

/// The job of someone in sight: the app asks it of the homunculus and the mercenary, which it knows only by
/// their IDs. None for one out of sight, or one struck down.
pub fn unit_job(id: u32) -> Option<u16> {
    STATE.with_borrow(|state| state.scene.as_ref()?.units.get(&id).filter(|unit| unit.dying.is_none()).map(|unit| unit.job))
}

/// Whether a cell of the map can be walked on.
pub fn walkable(x: i32, y: i32) -> bool {
    STATE.with_borrow(|state| state.scene.as_ref().is_some_and(|scene| scene.gat.walkable(x, y)))
}

/// What someone is doing, as the client's scripts are told it (the client's `render.state`): dead, walking,
/// casting, in a blow or flinching from one (a creature's flinch is the action at 24, a character's the one
/// at 48, where its pick-up is at 24), sitting, or standing.
fn doing(unit: &Mover, time: f64) -> i32 {
    let character = matches!(unit.look, Look::Character(_));
    if unit.dead || unit.dying.is_some() {
        motion::DEAD
    } else if unit.walk.is_some() {
        motion::MOVE
    } else if unit.casting.is_some() {
        motion::CASTING
    } else {
        match unit.motion.filter(|(_, began, lasts)| time >= *began && time - began < *lasts) {
            Some((48, ..)) if character => motion::DAMAGE,
            Some((24, ..)) if !character => motion::DAMAGE,
            Some((24, ..)) => 5,
            Some((96, ..)) if character => motion::SKILL,
            Some(_) => motion::ATTACK,
            None if unit.sitting => motion::SIT,
            None => motion::STAND,
        }
    }
}

/// Everyone the scripts may ask about: the player's own character (under the ID `own`), and everyone else in
/// sight. `targets` says whom each last struck (the app keeps it as the server tells of blows). A monster is
/// an enemy, and so is a player who is fought on this map (`pvp.rs`); the hidden states are known of players
/// only: the world keeps no state of a monster's.
pub fn sight(own: u32, targets: &HashMap<u32, u32>) -> Option<(Seen, Vec<Seen>)> {
    STATE.with_borrow(|state| {
        let scene = state.scene.as_ref()?;
        let time = now();
        let seen = |id: u32, unit: &Mover, enemy: bool| Seen { id, cell: unit.cell, motion: doing(unit, time), target: targets.get(&id).copied().unwrap_or(0), job: unit.job, hidden: crate::ai::hidden(unit.state), enemy };
        let others = scene.units.iter().map(|(id, unit)| seen(*id, unit, (unit.lively && !unit.companion) || state.pvp.foe(*id, unit))).collect();
        Some((seen(own, &scene.player, false), others))
    })
}

/// A button of the mouse went down on the world with Alt held: the right one commands the homunculus (0), the
/// left one the mercenary (1), if the player has that one out (the client's `CGameMode`, where it handles a
/// press on an actor and on the ground: on someone who can be fought, to fight it; on the ground, to walk
/// there). Tells the app whom or where, and whether Shift was held (the command is then kept for later).
/// False when there is no one to command: the press is then a press as any other.
pub(super) fn commanded(event: &MouseEvent, at: (f32, f32), which: usize) -> bool {
    let told = STATE.with_borrow(|state| {
        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
        scene.units.get(&state.ai.units[which]).filter(|_| state.ai.units[which] != 0)?;
        let view = scene.camera.view(scene.looked_at(), gpu.config.width as f32 / gpu.config.height as f32);
        let fought = |id: &u32| scene.units.get(id).is_some_and(|unit| unit.dying.is_none() && ((unit.lively && !unit.companion) || state.pvp.foe(*id, unit)));
        let whom = pointed_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32)).map(|(id, _)| id).filter(fought);
        let cell = match whom.and_then(|id| scene.units.get(&id)) {
            Some(unit) => unit.cell,
            None => view.pick(&scene.gat, at)?,
        };
        state.hooks.as_ref().map(|hooks| (hooks.commanded, whom, (cell.0.max(0) as u16, cell.1.max(0) as u16)))
    });
    let Some((commanded, whom, cell)) = told else { return false };
    commanded(which == 1, whom, cell, event.shift_key());
    true
}

/// The life of the player's homunculus and mercenary, as a bar under each, drawn as a party member's is
/// (`gauge` is that drawing). The client sends `AM_UPDATEGAGE` to each of the two as its script is loaded
/// (`CGameActor::OnProcess`), which for a player makes a `UIPcGage`; what it makes for these two was not
/// found in the decompilation, and the party member's bar is this client's choice for it.
pub(super) fn gauges(state: &State, scene: &Scene, view: &View, (width, height): (f32, f32), gauge: fn(f32, f32, i32, i32) -> String) -> String {
    let mut bars = String::new();
    for (unit, (left, most)) in state.ai.units.iter().zip(state.ai.life) {
        let Some(mover) = scene.units.get(unit).filter(|_| *unit != 0 && most > 0) else { continue };
        let Some(under) = view.project(on_ground(&scene.gat, mover.at)) else { continue };
        let below = height * 12.0 / 480.0 * DISTANCE / scene.camera.distance;
        let small = |number: i64| number.clamp(0, i64::from(i32::MAX)) as i32;
        bars.push_str(&gauge(under.0 * width - 30.0, under.1 * height + below, small(left), small(most)));
    }
    bars
}
