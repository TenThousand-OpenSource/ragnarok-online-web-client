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

/// What the character does now about something it was sent after, of which `reached` says from which cells
/// it is in reach (`crate::map::order`). A walk under way counts as leading into reach only when it is the
/// one last asked for, to `asked`: a press that gives the order forgets what was asked for before
/// (`State::ordered`), so the walk the character was on when told is always ended or replaced.
fn ordered(scene: &Scene, asked: (i32, i32), reached: impl Fn((i32, i32)) -> bool) -> crate::map::Order {
    let bound = scene.player.walk.as_ref().is_some_and(|walk| walk.goal() == asked && reached(asked));
    crate::map::order(reached(scene.player.cell), scene.player.walk.is_some(), bound)
}

impl State {
    /// Whether a walk may be asked for at this moment at all: none was asked for a moment ago
    /// (`asks_walk`), and the character is not in the motion of a blow of its own (`pursue`). An order that
    /// needs a walk first waits for this and looks again the next frame.
    fn may_walk(&self, time: f64) -> bool {
        crate::route::may_ask(time, self.walk_asked) && time >= self.own_striking_until
    }
}

/// A thing to pick up is walked to, and picked up from beside it. With the project's far walk on, the way is
/// found round whatever stands between, to the nearest cell beside the thing (`Scene::step_into_reach`), and
/// the thing is let be when there is none; the client asks for the cell a step towards the thing and no more
/// (`CGameMode::StepTowardWalkableCell`). A walk under way that leads elsewhere is replaced at once
/// (`crate::map::order`).
fn fetch(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    use crate::map::Order;
    let Some(scene) = &state.scene else { return };
    let Some((id, looked)) = state.fetch else { return };
    let Some(cell) = scene.drops.get(&id).map(|lying| lying.cell) else { return state.fetch = None };
    if time - looked <= 350.0 {
        return;
    }
    let beside = |from: (i32, i32)| (cell.0 - from.0).abs().max((cell.1 - from.1).abs()) <= 1;
    let (order, own) = (ordered(scene, state.asked, beside), scene.player.cell);
    let walks = |goal: (i32, i32)| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal));
    match order {
        Order::Wait => {}
        Order::Halt | Order::Go if !state.may_walk(time) => return,
        Order::Ask | Order::Halt => {
            pursuit.follow = Some(own).filter(|_| order == Order::Halt).and_then(walks);
            pursuit.take = state.hooks.as_ref().map(|hooks| (hooks.take, id));
        }
        Order::Go => {
            let goal = match state.options.far_walk {
                true => scene.step_into_reach(cell, beside),
                false => scene.goal_towards(own, cell, false),
            };
            pursuit.follow = goal.and_then(walks);
            if goal.is_none() && state.options.far_walk {
                return state.fetch = None;
            }
        }
    }
    state.fetch = Some((id, time));
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
///
/// A monster pressed while the character walks somewhere takes the place of that walk at once
/// (`crate::map::order`): struck from where the character is when it is in reach there, the walk ended
/// first; else the walk into reach replaces the walk under way.
fn chase(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    use crate::map::Order;
    /// The wait between two walks of the client's second way of chasing.
    const CHASE_APART_SECOND: f64 = 1200.0;
    let Some(scene) = &state.scene else { return };
    let Some((id, looked, told)) = state.chase else { return };
    let Some(unit) = scene.units.get(&id) else { return state.chase = None };
    // One struck down is asked nothing of, and left alone until it is gone from the map: then the fight
    // is over (`UpdatePendingAction`: `target->render.state == ST_DEAD` returns before anything is asked).
    if unit.dying.is_some() {
        return;
    }
    let (cell, target) = (scene.player.cell, unit.cell);
    let range = state.strike_range.unwrap_or(30).max(1);
    let own_way = state.options.far_walk;
    let order = ordered(scene, state.asked, |from| match own_way {
        true => scene.gat.reaches(from, target, range),
        // The client's own measure is straight across (the reach and a little more); by cells here.
        false => (target.0 - from.0).abs().max((target.1 - from.1).abs()) <= range,
    });
    // The client's second way of chasing (`PT_ATTACK_2`, chosen in its `AM_PROCEEDATTACK` when four
    // thirds of the player's own time for a cell is still less than the target's): it is then sent
    // on only every 1200 milliseconds, not every 450.
    let second = scene.player.speed * 4.0 / 3.0 < unit.speed;
    let walks = |goal: (i32, i32)| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal));
    // The first walk of a fight just ordered waits for neither of the client's times between two walks of
    // a chase: those begin with a walk of the chase itself (`tickOfMoveForAttack`), not with the walk the
    // character was on when it was told to fight (`State::ordered` has forgotten that one).
    let fresh = state.asked == (i32::MIN, i32::MIN);
    match order {
        // In reach, or bound for it: looked at every frame, so the blow is asked for the moment it can be.
        Order::Wait => return,
        Order::Ask | Order::Halt if !strikes_now(state, scene, told, time) => return,
        Order::Halt if !state.may_walk(time) => return,
        Order::Ask | Order::Halt => {
            // Near enough: the server is told to strike, the walk that led elsewhere ended first.
            pursuit.follow = Some(cell).filter(|_| order == Order::Halt).and_then(walks);
            state.strike_asked = time;
            // One blow only, the button having been let go: asked for as that, and the fight is over.
            if state.fight == 2 {
                pursuit.once = state.hooks.as_ref().map(|hooks| (hooks.strike_once, id));
                return (state.chase, state.fight) = (None, 0);
            }
            pursuit.strike = state.hooks.as_ref().map(|hooks| (hooks.press, id));
            state.chase = Some((id, looked, true));
        }
        Order::Go if !fresh && time - looked <= crate::route::CHASE_APART => {}
        Order::Go if !fresh && second && time - state.walk_asked <= CHASE_APART_SECOND => {}
        // (A walk that cannot be asked for yet is looked at again the next frame.)
        Order::Go if !state.may_walk(time) => {}
        Order::Go => {
            let Some(goal) = close_in(scene, target, range, own_way) else {
                // No place to strike from: the fight is given up.
                return (state.chase, state.fight) = (None, 0);
            };
            pursuit.follow = goal.filter(|goal| *goal != state.asked || scene.player.walk.is_none()).and_then(walks);
            // Out of reach, it is to be told to strike anew when it comes into reach.
            state.chase = Some((id, time, false));
            if fresh && pursuit.follow.is_none() {
                // (Nothing to ask for: the fight is no longer one just ordered.)
                state.asked = cell;
            }
        }
    }
}

/// Whether the server is told to strike now, the monster being in reach: it was not told yet (`told`) and
/// the client's wait between two requests is over (1200 milliseconds, `attackReqTime` of
/// `CPlayer::UpdatePendingAction`; a press on another monster ends the wait, `input/press.rs`); or it was
/// told and nothing came of it for a second and a half (it drops a request that reaches it while the
/// character still ends something else, as after the player changed targets quickly), so that the character
/// does not stand idle.
fn strikes_now(state: &State, scene: &Scene, told: bool, time: f64) -> bool {
    const STRIKES_APART: f64 = 1200.0;
    (!told && time - state.strike_asked > STRIKES_APART)
        || (scene.player.walk.is_none() && time - state.own_struck > 1500.0 && time - state.strike_asked > 1500.0)
}

/// Where the character is sent to come into reach of one on `target` with a weapon of `range` cells. The
/// outer None: there is no place to strike from. The inner None: nothing is asked for now.
///
/// With the project's far walk (`own`): to the nearest cell, by the steps walked, from which the server
/// lets the target be struck, or a stretch towards it (`Scene::step_into_reach`). Without: as it was before,
/// the cell of the server's way to the monster from which the weapon is near enough, which is nothing when
/// a wall is between (the client asks for the cell `range` cells from the target on the straight line to
/// it, `CGameMode::FindWalkableCellToward`, and its character stands behind what is in the way just the
/// same).
fn close_in(scene: &Scene, target: (i32, i32), range: i32, own: bool) -> Option<Option<(i32, i32)>> {
    let cell = scene.player.cell;
    if own {
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
/// (`Scene::step_into_reach`), and the skill is let be when there is none. A walk under way that leads
/// elsewhere is ended or replaced at once (`crate::map::order`).
fn reach(state: &mut State, time: f64, pursuit: &mut Pursuit) {
    use crate::map::Order;
    let Some(scene) = &state.scene else { return };
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
    let (own, own_way) = (scene.player.cell, state.options.far_walk);
    let order = match own_way {
        true => ordered(scene, state.asked, |from| scene.gat.reaches(from, cell, range as i32)),
        // The client's own measure, from where the character is; it is sent on only when it stands.
        false => {
            let (across, along) = (at.0 - scene.player.at.0, at.1 - scene.player.at.1);
            let near = (across * across + along * along).sqrt() * 5.0 <= range * 5.0 + 5.5;
            if near { Order::Ask } else if scene.player.walk.is_some() { Order::Wait } else { Order::Go }
        }
    };
    let walks = |goal: (i32, i32)| state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal));
    match order {
        Order::Wait => {}
        Order::Halt if !state.may_walk(time) => {}
        Order::Ask | Order::Halt => {
            pursuit.follow = Some(own).filter(|_| order == Order::Halt).and_then(walks);
            pursuit.used = state.hooks.as_ref().map(|hooks| (hooks.aimed, whom, (cell.0.max(0) as u16, cell.1.max(0) as u16)));
            state.reach = None;
        }
        Order::Go if time - looked <= 350.0 || !state.may_walk(time) => {}
        Order::Go => {
            let goal = match own_way {
                true => scene.step_into_reach(cell, |from| scene.gat.reaches(from, cell, range as i32)),
                false => scene.goal_towards(own, cell, false),
            };
            pursuit.follow = goal.and_then(walks);
            state.reach = (goal.is_some() || !own_way).then_some((whom, cell, range, time));
        }
    }
}
