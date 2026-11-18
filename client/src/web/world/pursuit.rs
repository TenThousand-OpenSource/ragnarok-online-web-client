//! What the player's own character does by itself between the player's presses: walking on while the button is
//! held, going to a thing to pick it up, running after the monster it fights and striking it, walking into reach
//! for a skill aimed from too far.

use super::*;

/// What came of it this frame, for the app to be told once the world is let go of: each a hook to call and what
/// to call it with.
#[derive(Default)]
pub(super) struct Pursuit {
    /// Walk to this cell.
    pub follow: Option<(fn(u16, u16), (i32, i32))>,
    /// Strike this monster (and go on striking).
    pub strike: Option<(fn(u32, bool), u32)>,
    /// Pick this thing up.
    pub take: Option<(fn(u32), u32)>,
    /// The aimed skill is in reach of whom or where it was aimed.
    pub used: Option<(fn(Option<u32>, (u16, u16)), Option<u32>, (u16, u16))>,
    /// Strike this monster once.
    pub once: Option<(fn(u32), u32)>,
}

/// Looks at what the character is after and decides what to ask for now. `view` and `size` are this frame's
/// camera and the page's size, for the place the pointer is held on. A later step's walk replaces an earlier one's.
pub(super) fn pursue(state: &mut State, view: &View, size: (f32, f32), time: f64) -> Pursuit {
    let mut pursuit = Pursuit::default();
    journey(state, &mut pursuit);
    walk_on(state, view, size, time, &mut pursuit);
    fetch(state, time, &mut pursuit);
    chase(state, time, &mut pursuit);
    reach(state, time, &mut pursuit);
    pursuit
}

/// A place pressed that is farther than the server walks in one go (its `max_walk_path`) is walked to in
/// stretches: arrived at the end of one, the character is sent on towards the place, until it is there or
/// the way is shut. The client asks once and stops where the server's stretch ends; going on is this
/// project's, for a camera that shows farther than the client's does.
fn journey(state: &mut State, pursuit: &mut Pursuit) {
    let (Some(scene), Some((to, asked))) = (&state.scene, state.journey) else { return };
    // Anything else the character is after ends it.
    if state.chase.is_some() || state.fetch.is_some() || state.reach.is_some() || state.aiming.is_some() {
        return state.journey = None;
    }
    let cell = scene.player.cell;
    if scene.player.walk.is_some() || cell != asked {
        // On its way; or stopped short of the stretch's end (struck, or walked elsewhere): then it stays.
        if scene.player.walk.is_none() {
            state.journey = None;
        }
        return;
    }
    match scene.gat.goal_towards(cell, to).filter(|next| *next != cell) {
        Some(next) => {
            state.journey = (next != to).then_some((to, next));
            pursuit.follow = state.hooks.as_ref().map(|hooks| (hooks.walk_to, next));
        }
        None => state.journey = None,
    }
}

/// While the left button is held on the ground, the character keeps walking to where the pointer is.
fn walk_on(state: &mut State, view: &View, (width, height): (f32, f32), time: f64, pursuit: &mut Pursuit) {
    let Some(scene) = &mut state.scene else { return };
    let follow = state.held.filter(|_| time - state.held_asked > 250.0).and_then(|held| {
        let picked = view.pick(&scene.gat, (held.0 / width.max(1.0), held.1 / height.max(1.0)))?;
        let cell = scene.player.cell;
        let goal = scene.gat.goal_towards(cell, picked)?;
        (goal != cell).then_some(goal)
    });
    if follow.is_some() {
        state.held_asked = time;
        // The pointer leads now, not the place pressed before.
        state.journey = None;
    }
    pursuit.follow = follow.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
}

/// A thing to pick up is walked to, and picked up from beside it.
fn fetch(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    let Some(scene) = &mut state.scene else { return };
    let Some((id, looked)) = state.fetch else { return };
    match scene.drops.get(&id) {
        None => state.fetch = None,
        Some(Lying { cell, .. }) if time - looked > 350.0 => {
            let own = scene.player.cell;
            if (cell.0 - own.0).abs().max((cell.1 - own.1).abs()) <= 1 {
                pursuit.take = state.hooks.as_ref().map(|hooks| (hooks.take, id));
            } else if scene.player.walk.is_none() {
                let goal = scene.gat.goal_towards(own, *cell);
                pursuit.follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
            }
            state.fetch = Some((id, time));
        }
        Some(_) => {}
    }
}

/// The monster being fought is run after when it is out of reach, and struck again once it is near.
fn chase(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    let Some(scene) = &mut state.scene else { return };
    let Some((id, looked, told)) = state.chase else { return };
    match scene.units.get(&id) {
        None => state.chase = None,
        Some(unit) if time - looked > 400.0 => {
            let cell = scene.player.cell;
            // Near enough is what the server said the weapon reaches (it says on entering the map, when
            // that changes, and when a blow is asked for from too far). Until it has said, the
            // character stays where it is: a bow reaches far, and its archer is not to run up to the
            // monster. The client measures straight across (the reach and a little more); here it is
            // by cells, as the server does, so that the server never finds the blow asked from too far.
            let range = state.strike_range.unwrap_or(30).max(1);
            let far = (unit.cell.0 - cell.0).abs().max((unit.cell.1 - cell.1).abs()) > range;
            if far {
                // The way to the monster, as far as the cell from which the weapon reaches it.
                let way = scene.gat.path(cell, unit.cell);
                let near = way.get(way.len().saturating_sub(1 + range as usize).min(way.len().saturating_sub(1))).copied().filter(|goal| *goal != cell).unwrap_or(unit.cell);
                let goal = scene.gat.goal_towards(cell, near);
                pursuit.follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
            } else if !told || (scene.player.walk.is_none() && time - state.own_struck > 1500.0 && time - state.strike_asked > 1500.0) {
                // Near enough: the server is told to strike. It is told again when nothing came of it
                // (it drops a request that reaches it while the character still ends something else,
                // as after the player changed targets quickly), so the character does not stand idle.
                pursuit.strike = state.hooks.as_ref().map(|hooks| (hooks.press, id));
                state.strike_asked = time;
                // One blow only, the button having been let go: asked for as that, and the fight is over.
                if state.fight == 2 {
                    pursuit.once = state.hooks.as_ref().map(|hooks| (hooks.strike_once, id));
                    pursuit.strike = None;
                }
            }
            state.chase = if pursuit.once.is_some() { None } else { Some((id, time, !far)) };
            if pursuit.once.is_some() {
                state.fight = 0;
            }
        }
        Some(_) => {}
    }
}

/// A skill aimed out of its reach: the character walks towards whom or where it was aimed, and uses it
/// as soon as it is near enough. The client's rule (`CPlayer::UpdatePendingAction`): the skill's reach in
/// cells of 5, and 5.5 more, is at least the distance.
fn reach(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    let Some(scene) = &mut state.scene else { return };
    let Some((whom, cell, range, looked)) = state.reach else { return };
    let aimed = match whom {
        Some(id) => scene.units.get(&id).filter(|unit| unit.dying.is_none()).map(|unit| (unit.cell, unit.at)),
        None => Some((cell, (cell.0 as f32, cell.1 as f32))),
    };
    match aimed {
        None => state.reach = None,
        Some((cell, at)) => {
            let (across, along) = (at.0 - scene.player.at.0, at.1 - scene.player.at.1);
            if (across * across + along * along).sqrt() * 5.0 <= range * 5.0 + 5.5 {
                pursuit.used = state.hooks.as_ref().map(|hooks| (hooks.aimed, whom, (cell.0.max(0) as u16, cell.1.max(0) as u16)));
                state.reach = None;
            } else if time - looked > 350.0 {
                if scene.player.walk.is_none() {
                    let own = scene.player.cell;
                    let goal = scene.gat.goal_towards(own, cell);
                    pursuit.follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                }
                state.reach = Some((whom, cell, range, time));
            }
        }
    }
}
