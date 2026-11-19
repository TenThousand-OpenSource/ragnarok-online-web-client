//! Those who stand and walk on the map: where each is at a moment, and which picture of it shows.

use super::*;

/// Which of a character's two attacks shows when it strikes: the action at 88 for the weapons its job handles
/// in that way, the one at 80 for everything else (the client's `IsJobWeaponAllowed`). An Archer shoots with
/// the action at 80 and strikes with the other when it holds no bow; a Swordsman thrusts a spear with the one
/// at 88, a Mage and a Merchant stab with a dagger so.
pub(super) fn attack_action(figure: &Figure) -> usize {
    let kind = data::about(figure.weapon).and_then(|(_, facts)| facts.into_iter().find(|(key, _)| key == "SubType")).map(|(_, kind)| kind).unwrap_or_default();
    let kind = kind.as_str();
    // The reborn and the baby jobs fight as the ones they come from.
    let job = match figure.job {
        4001..=4022 => figure.job - 4001,
        4023..=4045 => figure.job - 4023,
        job => job,
    };
    let male = figure.sex == 1;
    let second = match job {
        1 | 7 | 13 | 14 | 21 => matches!(kind, "1hSpear" | "2hSpear"),
        2 | 5 => kind == "Dagger",
        3 => kind != "Bow",
        6 | 11 | 17 | 19 | 20 => kind == "Bow",
        8 => kind == "Book",
        9 => (kind == "Dagger" && male) || (matches!(kind, "Staff" | "2hStaff") && !male),
        10 | 18 => matches!(kind, "1hSword" | "1hAxe" | "2hAxe" | "Mace"),
        12 => kind == "Katar",
        15 => matches!(kind, "" | "Knuckle"),
        16 => matches!(kind, "2hSpear" | "Staff" | "Book" | "2hStaff"),
        0 | 23 => (kind == "Dagger" && !male) || (male && matches!(kind, "1hSword" | "2hSword" | "1hAxe" | "2hAxe" | "Mace" | "2hMace" | "Staff" | "2hStaff")),
        _ => false,
    };
    if second { 88 } else { 80 }
}

/// Where a point given in cells is in the world: cell centres, on the ground.
pub(super) fn on_ground(gat: &Gat, (x, y): (f32, f32)) -> Vector {
    // A character on cell 3 stands at 3.5 cells.
    [(x + 0.5) * CELL, gat.height_at(x + 0.5, y + 0.5), (y + 0.5) * CELL]
}

impl Scene {
    /// Where someone is now, in cells: no one named is the player's own character; nothing for one out of sight.
    pub(super) fn place_of(&self, who: Option<u32>) -> Option<(f32, f32)> {
        match who {
            Some(id) => self.units.get(&id).map(|unit| unit.at).or_else(|| self.spots.get(&id).copied()),
            None => Some(self.player.at),
        }
    }
}

/// Everyone on the map is moved to where the server's clock has them now (the client's actors each do it in
/// their `ProcessState`, by `CSession::GetServerTime`).
pub(super) fn stride(state: &mut State, time: f64) {
    let server = state.clock.now(time);
    let last = state.stepped.replace(server).unwrap_or(server);
    let Some(scene) = &mut state.scene else { return };
    scene.player.step(last, server);
    for unit in scene.units.values_mut() {
        unit.step(last, server);
    }
}

/// How someone is posed beyond its walk and its motions (`crate::pose`), and what else of it is drawn.
#[derive(Default)]
pub(super) struct Pose {
    /// A character stands ready (action 32) and not at ease: after a blow given or taken or a cast, until it
    /// walks or sits (`CPc::SetState(ST_STAND)`).
    ready: bool,
    /// The action of the motion under way: what it stands as once that is over follows from it.
    doing: Option<usize>,
    /// The frames of an action it runs through by itself, and since when (`SetFrameSequence`).
    pub(super) sequence: Option<(crate::pose::Sequence, f64)>,
    /// Which way a character's head is turned: 0 ahead, 1 to the right, 2 to the left (`headDir`).
    pub(super) head: u8,
    /// How far it has walked since it came into sight, in the world's measure (`CGameActor::dist`).
    walked: f32,
    /// It is out of sight and fades: when it is gone (`AM_OUTOFSIGHT`, `vanishTime`).
    pub(super) vanishes: Option<f64>,
    /// The puff of smoke of one that hides or comes out of hiding: since when, and where (in cells).
    pub(super) smoke: Option<(f64, (f32, f32))>,
    /// The window of a combo the server told of it last, in milliseconds (ZC_COMBODELAY; the client's
    /// `CSession::InsertPair`): its combos hold their frames for as long (`crate::pose::strikes_with`).
    pub(super) combo: f64,
    /// Its blade is caught: its motion is held where it was (`isMotionFreezed`, `combat::blade_caught`).
    pub(super) frozen: bool,
}

impl Pose {
    /// What a motion that is over, or a walk begun, leaves of the pose. `doing` is the action of the motion
    /// under way now, `unwell` whether its health state is not 0 (one who ails stands ready after anything).
    fn settle(&mut self, doing: Option<usize>, walking: bool, unwell: bool) {
        if let (Some(was), None) = (self.doing, doing) {
            self.ready = crate::pose::readies(was) || unwell;
        }
        self.doing = doing;
        // `CPc::SetState(ST_MOVE)`: the head looks ahead again; and one who walked stands at ease after it
        // unless it ails.
        if walking {
            (self.ready, self.head, self.sequence) = (unwell, 0, None);
        }
    }

    /// Stands at ease again, whatever it did (it sits down, lies dead, rises).
    pub(super) fn at_ease(&mut self) {
        (self.ready, self.sequence, self.doing) = (false, None, None);
    }
}

impl Mover {
    pub(super) fn new(look: Look, cell: (i32, i32)) -> Self {
        let job = if let Look::Character(figure) = &look { figure.job } else { 0 };
        Self { look, job, state: 0, name: String::new(), lively: false, companion: false, guild: (0, 0), cell, at: (cell.0 as f32, cell.1 as f32), facing: 0, walk: None, strode: 0.0, speed: DEFAULT_SPEED, shown: None, sitting: false, dead: false, dying: None, says: None, emotion: None, life: None, heard: None, motion: None, ail: Default::default(), casting: None, pose: Pose::default() }
    }

    /// Dresses a character as its job is drawn in the state it is in: on its mount, in its machine.
    pub(super) fn dress(&mut self) {
        if let Look::Character(figure) = &mut self.look {
            figure.job = crate::mounts::drawn_job(self.job, self.state);
        }
    }

    /// Starts the walk the server tells of (the client's `AM_MOVE`): along the path this map gives, each cell
    /// at its time of the server's clock from the time the walk set out at (`crate::route`). The walker is
    /// not put anywhere: it goes on from where it is drawn. A walk of no step leaves it standing.
    pub(super) fn walk(&mut self, gat: &Gat, clock: &Clock, told: &Walked) {
        let (time, from, to) = (now(), (i32::from(told.from.0), i32::from(told.from.1)), (i32::from(told.to.0), i32::from(told.to.1)));
        let path = gat.path(from, to);
        // The server walks its own path. Should this map disagree with it, the walker goes straight there.
        let path = if path.is_empty() && from != to { vec![to] } else { path };
        let cells: Vec<_> = std::iter::once(from).chain(path).collect();
        // Where within its cell it sets out, in sixteenths from the cell's corner (`CPathFinder::FindPath`).
        let set_out = (from.0 as f32 + (f32::from(told.within.0) - 8.0) / 16.0, from.1 as f32 + (f32::from(told.within.1) - 8.0) / 16.0);
        let route = Route::found(clock.of(time, told.started), clock.now(time), set_out, &cells, self.speed as i32);
        // The legs begin anew with a walk begun from standing.
        if route.is_some() && self.walk.is_none() {
            self.strode = time;
        }
        self.walk = route;
    }

    /// Moves a walker to where it is at the server's time `server`, `last` being the server's time of the frame
    /// before (the `ST_MOVE` arm of the client's `CGameActor::ProcessState`).
    pub(super) fn step(&mut self, last: f64, server: f64) {
        let Some(route) = &self.walk else { return };
        let step = route.step(self.at, last, server);
        // The way gone, in the world's measure: the legs go by it (`CGameActor::ProcessState` adds it to `dist`).
        self.pose.walked += ((step.at.0 - self.at.0).powi(2) + (step.at.1 - self.at.1).powi(2)).sqrt() * CELL;
        self.at = step.at;
        self.facing = step.facing.unwrap_or(self.facing);
        self.cell = if step.arrived { route.goal() } else { (step.at.0.round() as i32, step.at.1.round() as i32) };
        if step.arrived {
            self.walk = None;
        }
    }

    /// Puts it on a cell at once, wherever it was drawn (it comes into sight standing, or leaps: the client's
    /// `AM_STANDENTRY` and ZC_HIGHJUMP write the cell's middle as its place).
    pub(super) fn put(&mut self, cell: (i32, i32)) {
        (self.cell, self.at) = (cell, (cell.0 as f32, cell.1 as f32));
    }

    /// The server names the cell it stands on (ZC_STOPMOVE): its walk is over and it is drawn as
    /// `crate::route::stopped` says, facing as it did.
    pub(super) fn stand(&mut self, cell: (i32, i32)) {
        self.walk = None;
        (self.cell, self.at) = (cell, crate::route::stopped(self.at, cell));
    }

    /// When its walk is over, as `now` counts (`time` is now); now, for one that stands.
    pub(super) fn arrives(&self, clock: &Clock, time: f64) -> f64 {
        self.walk.as_ref().map_or(time, |route| time + (route.ends() - clock.now(time)).max(0.0))
    }

    /// How many pictures an action of its sprite has and how long each shows, once the sprite has been read.
    fn pace(&self, action: usize) -> Option<(usize, f64)> {
        match &self.look {
            Look::Character(figure) => figure::body_timing(figure.job, figure.sex, action),
            Look::Creature(path) => figure::timing(path, action),
        }
    }

    /// The picture of an action that goes round and round, each for as long as the sprite says (the
    /// client's `MT_LOOP`); the first until the sprite has been read.
    fn looped(&self, action: usize, now: f64) -> usize {
        self.pace(action).map_or(0, |(pictures, each)| (now / each.max(1.0)) as usize % pictures.max(1))
    }

    /// The action it rests in when no motion is under way, and the picture of it: walking, the legs go by
    /// the way gone (`CGameActor::ProcessMotionWithDist`: 0.37 × 4 pictures to a unit of the world, over the
    /// action's speed); a monster, an NPC or a pet goes through its standing pictures; a character lies in
    /// the last picture of its fall, sits or stands with its head turned, runs through the frames a state
    /// gave it, or stands ready.
    fn rests(&mut self, walking: bool, now: f64) -> (usize, usize) {
        use crate::pose::{LIES, READY, SITS, STANDS, WALKS};
        if walking {
            // (The speed of an action is kept here in milliseconds, 25 to one of the file's units.)
            let (pictures, each) = self.pace(WALKS).unwrap_or((WALK_FRAMES, WALK_FRAME_MS));
            return (WALKS, (f64::from(self.pose.walked) * 0.37 * 4.0 / (each / 25.0).max(1.0)) as usize % pictures.max(1));
        }
        if matches!(self.look, Look::Creature(_)) {
            return (STANDS, self.looped(STANDS, now));
        }
        // (One whose fall is yet to begin, the blow still on its way, stands until then.)
        if self.dead && self.motion.is_none() {
            return (LIES, self.pace(LIES).map_or(0, |(pictures, _)| pictures.saturating_sub(1)));
        }
        if self.sitting {
            return (SITS, crate::pose::head_frame(self.pose.head));
        }
        if let Some((sequence, began)) = self.pose.sequence {
            if !sequence.over(now - began) {
                return (sequence.action, sequence.frame(now - began));
            }
            self.pose.sequence = None;
        }
        if self.pose.ready {
            return (READY, self.looped(READY, now));
        }
        (STANDS, crate::pose::head_frame(self.pose.head))
    }

    /// Where it is now, in cells (as `step` left it), and which picture of it to show to a camera turned by
    /// `yaw` degrees.
    pub(super) fn now(&mut self, now: f64, yaw: f32) -> ((f32, f32), SpriteKey) {
        let (position, walking) = (self.at, self.walk.is_some());
        // The sprite's directions are numbered as the game's, seen by a camera looking north: a camera turned
        // around it sees it from another side.
        let turned = (f32::from(self.facing) * 45.0 - yaw).rem_euclid(360.0);
        // A motion that is yet to begin waits its turn; one under way ends when its time is up or a walk begins.
        // One struck down stays in the last picture of its fall.
        let motion = self.motion.filter(|(_, began, lasts)| !walking && now >= *began && (now - began < *lasts || self.dying.is_some()));
        if motion.is_none() && !self.motion.is_some_and(|(_, began, _)| began > now) {
            self.motion = None;
        }
        if walking {
            self.sitting = false;
        }
        if matches!(self.look, Look::Character(_)) {
            self.pose.settle(motion.map(|(action, ..)| action), walking, self.ail.unwell());
        }
        let (action, frame) = match motion {
            // A motion plays its pictures once over the time it takes, and holds the last.
            Some((action, began, lasts)) => {
                let pictures = self.pace(action).map_or(5, |(pictures, _)| pictures).max(1);
                (action, (((now - began) / lasts * pictures as f64) as usize).min(pictures - 1))
            }
            None => self.rests(walking, now),
        };
        let action = action + ((turned / 45.0).round() as usize % 8);
        // An effect may hold a character that stands in one picture of an action (`crate::body`, the client's
        // `SetFrameSequence`).
        let held = self.ail.body.held().filter(|_| !walking && !self.dead).map(|(held, frame)| (held + action % 8, frame));
        let key = match &self.look {
            Look::Character(figure) => {
                let (action, frame) = held.unwrap_or((action, frame));
                SpriteKey::Player(Figure { action, frame, ..figure.clone() })
            }
            Look::Creature(path) => SpriteKey::Creature(path.clone(), action, frame),
        };
        (position, key)
    }
}
