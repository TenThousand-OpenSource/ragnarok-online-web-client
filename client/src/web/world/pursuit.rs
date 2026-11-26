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
    let waited = after_blow(state, time, &mut pursuit);
    journey(state, time, &mut pursuit);
    walk_on(state, view, size, time, &mut pursuit);
    fetch(state, time, &mut pursuit);
    chase(state, time, &mut pursuit);
    reach(state, time, &mut pursuit);
    // In the motion of a blow of its own the character asks for no walk: the client's player is sent on to
    // what it fights only out of `ST_ATTACK`, to a thing or a place for a skill only standing
    // (`CPlayer::UpdatePendingAction`), and a press on the ground waits (`after_blow`). Only the walk that
    // waited for a blow to end goes although the next blow has begun: the client asks for it as the first ends.
    if time < state.own_striking_until && pursuit.follow.map(|(_, goal)| goal) != waited {
        pursuit.follow = None;
    }
    // Whatever asks for the walk, it goes only if none went a moment ago.
    if pursuit.follow.is_some_and(|(_, goal)| !state.asks_walk(goal, time)) {
        pursuit.follow = None;
    }
    pursuit
}

impl State {
    /// Whether a walk to `goal` may be asked of the server now; if so it is noted as asked for. The client
    /// drops a request that comes within 350 milliseconds of the last (`crate::route::may_ask`), a press on
    /// the ground as any other.
    pub(super) fn asks_walk(&mut self, goal: (i32, i32), time: f64) -> bool {
        let may = crate::route::may_ask(time, self.walk_asked);
        if may {
            (self.walk_asked, self.asked) = (time, goal);
        }
        may
    }
}

/// A place pressed while the character's body was in a blow it deals is walked to as that blow ends, whether
/// or not the next has begun by then. The client keeps the cell (`AM_REQ_MOVE` of `CPlayer::SendMsg`,
/// `Source/Contents/Actors/Player.cpp:3896`: `isPreengageStateOfMove` while `IsSpecialState`, which `ST_ATTACK`
/// is) and asks for the walk as its body leaves that state (`CPlayer::SetState`, `:5579`). Gives the cell asked
/// for now, if any.
fn after_blow(state: &mut State, time: f64, pursuit: &mut Pursuit) -> Option<(i32, i32)> {
    let (goal, over) = state.walk_waits?;
    if time < over || !crate::route::may_ask(time, state.walk_asked) {
        return None;
    }
    state.walk_waits = None;
    pursuit.follow = state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal));
    Some(goal)
}

/// How long a journey waits, in milliseconds, for the server to set the character walking after a stretch
/// was asked for, before it takes the stretch as refused (this project's own).
const JOURNEY_PATIENCE: f64 = 1000.0;

/// A place pressed that is farther than the server walks in one go (its `max_walk_path`) is walked to in
/// stretches: arrived at the end of one, the character is sent on towards the place, until it is there or
/// the way is shut. The client asks once and does not go on; this is the project's own, and happens only with
/// its far walk on (`Options::far_walk`; on at first, the command `/farwalk` switches it).
fn journey(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    let (Some(scene), Some((to, asked))) = (&state.scene, state.journey) else { return };

    // Anything else the character is after ends it.
    if state.chase.is_some() || state.fetch.is_some() || state.reach.is_some() || state.aiming.is_some() {
        return state.journey = None;
    }
    let cell = scene.player.cell;
    if scene.player.walk.is_some() || cell != asked {
        // On its way; or the server has not yet answered the request (the character still stands where it
        // was: the journey waits); or it stopped short of the stretch's end (struck, or walked elsewhere, or
        // the server would not walk it): then it stays.
        if scene.player.walk.is_none() && time - state.walk_asked > JOURNEY_PATIENCE {
            state.journey = None;
        }
        return;
    }
    // The next stretch waits until a walk may be asked for again.
    if !crate::route::may_ask(time, state.walk_asked) {
        return;
    }
    match scene.goal_towards(cell, to, state.options.far_walk).filter(|next| *next != cell) {
        Some(next) => {
            state.journey = (next != to).then_some((to, next));
            pursuit.follow = state.hooks.as_ref().map(|hooks| (hooks.walk_to, next));
        }
        None => state.journey = None,
    }
}

/// While the left button is held on the ground, the character keeps walking to where the pointer is: asked for
/// anew 600 milliseconds after the last time (the client's `CGameMode::ProcessGroundClick`).
fn walk_on(state: &mut State, view: &View, (width, height): (f32, f32), time: f64, pursuit: &mut Pursuit) {
    let Some(scene) = &mut state.scene else { return };
    let follow = state.held.filter(|_| time - state.held_asked > crate::route::HELD_APART).and_then(|held| {
        let picked = view.pick(&scene.gat, (held.0 / width.max(1.0), held.1 / height.max(1.0)))?;
        let cell = scene.player.cell;
        let goal = scene.goal_towards(cell, picked, state.options.far_walk)?;
        (goal != cell).then_some(goal)
    });
    if follow.is_some() {
        state.held_asked = time;
        // The pointer leads now, not the place pressed before.
        state.journey = None;
    }
    // Only a pointer that leads replaces the walk a journey asked for a step before.
    if let Some(goal) = follow {
        pursuit.follow = state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal));
    }
}

/// A thing to pick up is walked to, and picked up from beside it. With the project's far walk on, the way is
/// found round whatever stands between, to the nearest cell beside the thing (`Scene::step_into_reach`), and
/// the thing is let be when there is none; the client asks for the cell a step towards the thing and no more
/// (`CGameMode::StepTowardWalkableCell`).
fn fetch(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    let Some(scene) = &mut state.scene else { return };
    let Some((id, looked)) = state.fetch else { return };
    let beside = |from: (i32, i32), cell: (i32, i32)| (cell.0 - from.0).abs().max((cell.1 - from.1).abs()) <= 1;
    match scene.drops.get(&id) {
        None => state.fetch = None,
        Some(Lying { cell, .. }) if time - looked > 350.0 => {
            let own = scene.player.cell;
            state.fetch = Some((id, time));
            if beside(own, *cell) {
                pursuit.take = state.hooks.as_ref().map(|hooks| (hooks.take, id));
            } else if scene.player.walk.is_none() {
                let goal = match state.options.far_walk {
                    true => scene.step_into_reach(*cell, |from| beside(from, *cell)),
                    false => scene.goal_towards(own, *cell, false),
                };
                pursuit.follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
                if goal.is_none() && state.options.far_walk {
                    state.fetch = None;
                }
            }
        }
        Some(_) => {}
    }
}

/// The monster being fought is run after when it is out of reach, and struck again once it is near. The
/// client (`CPlayer::UpdatePendingAction`) sends its player on no sooner than 450 milliseconds after it last
/// did, and only to another cell than the one it is bound for. Here it is also sent on to the same cell when
/// it stands (a blow stopped it, or the server did not walk it): whether the client does was not found.
///
/// In reach is what the server will let be struck (`Gat::reaches`): within what it said the weapon reaches
/// (it says on entering the map, when that changes, and when a blow is asked for from too far; until it has
/// said, a bow's archer is not to run up to the monster) and with nothing between that stops a shot. The
/// client measures the distance alone, so behind a pillar its character stands and asks in vain (the server
/// answers nothing); with the project's far walk on, the character goes round to where it does reach
/// (`close_in`), and lets the monster be when there is no such place.
fn chase(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    /// The wait between two walks of the client's second way of chasing.
    const CHASE_APART_SECOND: f64 = 1200.0;
    /// The wait between two requests to strike (`attackReqTime` of `CPlayer::UpdatePendingAction`).
    const STRIKES_APART: f64 = 1200.0;
    let Some(scene) = &mut state.scene else { return };
    let Some((id, looked, told)) = state.chase else { return };
    match scene.units.get(&id) {
        None => state.chase = None,
        // One struck down is asked nothing of, and left alone until it is gone from the map: then the fight
        // is over (`UpdatePendingAction`: `target->render.state == ST_DEAD` returns before anything is asked).
        Some(unit) if unit.dying.is_some() => {}
        Some(unit) if time - looked > crate::route::CHASE_APART => {
            let cell = scene.player.cell;
            let range = state.strike_range.unwrap_or(30).max(1);
            let far = match state.options.far_walk {
                true => !scene.gat.reaches(cell, unit.cell, range),
                // The client's own measure is straight across (the reach and a little more); by cells here.
                false => (unit.cell.0 - cell.0).abs().max((unit.cell.1 - cell.1).abs()) > range,
            };
            // The client's second way of chasing (`PT_ATTACK_2`, chosen in its `AM_PROCEEDATTACK` when four
            // thirds of the player's own time for a cell is still less than the target's): it is then sent
            // on only every 1200 milliseconds, not every 450.
            let second = scene.player.speed * 4.0 / 3.0 < unit.speed;
            let mut lost = false;
            if far && second && time - state.walk_asked <= CHASE_APART_SECOND {
                // (It waits for its time.)
            } else if far {
                let goal = close_in(scene, unit.cell, range, state.asked, state.options.far_walk);
                lost = goal.is_none();
                let goal = goal.flatten().filter(|goal| *goal != state.asked || scene.player.walk.is_none());
                pursuit.follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
            } else if (!told && time - state.strike_asked > STRIKES_APART)
                || (scene.player.walk.is_none() && time - state.own_struck > 1500.0 && time - state.strike_asked > 1500.0)
            {
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
            let asked = pursuit.strike.is_some() || (told && !far);
            state.chase = if pursuit.once.is_some() || lost { None } else { Some((id, time, asked)) };
            if pursuit.once.is_some() || lost {
                state.fight = 0;
            }
        }
        Some(_) => {}
    }
}

/// Where the character is sent to come into reach of one on `target` with a weapon of `range` cells; `asked`
/// is the cell it was last sent to. The outer None: there is no place to strike from, and the fight is given
/// up. The inner None: nothing is asked for now.
///
/// With the project's far walk (`own`): to the nearest cell, by the steps walked, from which the server
/// lets the target be struck, or a stretch towards it (`Scene::step_into_reach`); one already walking to a
/// cell that reaches walks on. Without: as it was before, the cell of the server's way to the monster from
/// which the weapon is near enough, which is nothing when a wall is between (the client asks for the cell
/// `range` cells from the target on the straight line to it, `CGameMode::FindWalkableCellToward`, and its
/// character stands behind what is in the way just the same).
fn close_in(scene: &Scene, target: (i32, i32), range: i32, asked: (i32, i32), own: bool) -> Option<Option<(i32, i32)>> {
    let cell = scene.player.cell;
    if own {
        if scene.player.walk.is_some() && scene.gat.reaches(asked, target, range) {
            return Some(None);
        }
        return scene.step_into_reach(target, |from| scene.gat.reaches(from, target, range)).map(Some);
    }
    let way = scene.gat.path(cell, target);
    let near = way.get(way.len().saturating_sub(1 + range as usize).min(way.len().saturating_sub(1)))
        .copied()
        .filter(|goal| *goal != cell)
        .unwrap_or(target);
    Some(scene.goal_towards(cell, near, false))
}

/// A skill aimed out of its reach: the character walks towards whom or where it was aimed, and uses it
/// as soon as it is near enough. The client's rule (`CPlayer::UpdatePendingAction`): the skill's reach in
/// cells of 5, and 5.5 more, is at least the distance; it then asks, and the server answers nothing when a
/// wall is between. With the project's far walk on, in reach is what the server lets the skill be used from
/// (`Gat::reaches`: its range and a clear line), the character goes round to the nearest such cell
/// (`Scene::step_into_reach`), and the skill is let be when there is none.
fn reach(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    let Some(scene) = &mut state.scene else { return };
    let Some((whom, cell, range, looked)) = state.reach else { return };
    // In the motion of a blow of its own the character uses no skill yet: it waits here, as in the client
    // (`crate::skills::aim::asked_now`).
    if !crate::skills::aim::asked_now(time, state.own_striking_until) {
        return;
    }
    let aimed = match whom {
        // Someone, or a trap standing on the ground (a skill for a trap is aimed at the trap's own ID).
        Some(id) => scene.units.get(&id).filter(|unit| unit.dying.is_none()).map(|unit| (unit.cell, unit.at)).or_else(
            || state.grounds.trap(id).map(|at| ((at.0 as i32, at.1 as i32), at))
        ),
        None => Some((cell, (cell.0 as f32, cell.1 as f32))),
    };
    let Some((cell, at)) = aimed else { return state.reach = None };
    let own = scene.player.cell;
    let near = match state.options.far_walk {
        // One still walking is a cell before or behind where the server has it: a cell to spare then.
        true => scene.gat.reaches(own, cell, range as i32 - i32::from(scene.player.walk.is_some())),
        false => {
            let (across, along) = (at.0 - scene.player.at.0, at.1 - scene.player.at.1);
            (across * across + along * along).sqrt() * 5.0 <= range * 5.0 + 5.5
        }
    };
    if near {
        pursuit.used = state.hooks.as_ref().map(|hooks| (hooks.aimed, whom, (cell.0.max(0) as u16, cell.1.max(0) as u16)));
        state.reach = None;
    } else if time - looked > 350.0 {
        state.reach = Some((whom, cell, range, time));
        if scene.player.walk.is_none() {
            let goal = match state.options.far_walk {
                true => scene.step_into_reach(cell, |from| scene.gat.reaches(from, cell, range as i32)),
                false => scene.goal_towards(own, cell, false),
            };
            pursuit.follow = goal.and_then(|goal| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal)));
            if goal.is_none() && state.options.far_walk {
                state.reach = None;
            }
        }
    }
}
