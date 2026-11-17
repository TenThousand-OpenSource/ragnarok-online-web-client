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

impl Mover {
    pub(super) fn new(look: Look, cell: (i32, i32)) -> Self {
        Self { look, name: String::new(), lively: false, companion: false, cell, at: (cell.0 as f32, cell.1 as f32), facing: 0, walk: None, speed: DEFAULT_SPEED, shown: None, sitting: false, dead: false, dying: None, says: None, emotion: None, life: None, heard: None, motion: None, casting: None }
    }

    /// Starts a walk from one cell to another, along the path this map gives.
    pub(super) fn walk(&mut self, gat: &Gat, from: (i32, i32), to: (i32, i32)) {
        let path = gat.path(from, to);
        // The server walks its own path. Should this map disagree with it, the walker goes straight there.
        let path = if path.is_empty() && from != to { vec![to] } else { path };
        self.walk = Some((Walk::new(from, &path, self.speed), now()));
    }

    /// Where it is now, in cells, and which picture of it to show to a camera turned by `yaw` degrees.
    pub(super) fn now(&mut self, now: f64, yaw: f32) -> ((f32, f32), SpriteKey) {
        let (position, walking, elapsed) = match &self.walk {
            Some((walk, started)) => {
                let elapsed = now - started;
                let (position, facing) = walk.at(elapsed);
                self.facing = facing;
                if elapsed >= walk.duration() {
                    self.cell = walk.goal();
                    self.walk = None;
                    (position, false, 0.0)
                } else {
                    self.cell = (position.0.round() as i32, position.1.round() as i32);
                    (position, true, elapsed)
                }
            }
            None => ((self.cell.0 as f32, self.cell.1 as f32), false, 0.0),
        };
        // The legs keep the pace of the walk: a slower walk shows each picture longer.
        let frame = if walking {
            (elapsed / (WALK_FRAME_MS * self.speed / DEFAULT_SPEED)) as usize % WALK_FRAMES
        } else if self.lively {
            (now / IDLE_FRAME_MS) as usize
        } else {
            0
        };
        // The sprite's directions are numbered as the game's, seen by a camera looking north: a camera turned
        // around it sees it from another side.
        let turned = (f32::from(self.facing) * 45.0 - yaw).rem_euclid(360.0);
        // A motion that is yet to begin waits its turn; one under way ends when its time is up or a walk begins.
        // One struck down stays in the last picture of its fall.
        let motion = self.motion.filter(|(_, began, lasts)| !walking && now >= *began && (now - began < *lasts || self.dying.is_some()));
        if motion.is_none() && !self.motion.is_some_and(|(_, began, _)| began > now) {
            self.motion = None;
        }
        // A motion plays its pictures once over the time it takes, and holds the last.
        let frame = match motion {
            Some((action, began, lasts)) => {
                let pictures = match &self.look {
                    Look::Character(figure) => figure::body_frames(figure.job, figure.sex, action),
                    Look::Creature(path) => figure::frames_of(path, action),
                }
                .unwrap_or(5)
                .max(1);
                (((now - began) / lasts * pictures as f64) as usize).min(pictures - 1)
            }
            None => frame,
        };
        if walking {
            self.sitting = false;
        }
        let resting = match (&self.look, self.dead, self.sitting) {
            (Look::Character(_), true, _) => 64,
            (Look::Character(_), _, true) => 16,
            _ => 0,
        };
        let action = motion.map_or(if walking { WALKING } else { resting }, |(action, ..)| action) + ((turned / 45.0).round() as usize % 8);
        self.at = position;
        let key = match &self.look {
            Look::Character(figure) => SpriteKey::Player(Figure { action, frame: if walking || motion.is_some() { frame } else { 0 }, ..figure.clone() }),
            Look::Creature(path) => SpriteKey::Creature(path.clone(), action, frame),
        };
        (position, key)
    }
}
