//! The hunter's falcon as it is drawn by its owner: which sprite it is, where it hovers as its owner walks,
//! how it flies at what a skill sends it to and comes back, which way it is seen and which picture shows
//! (drawn by `web/world/falcon.rs`).
//!
//! The client makes the falcon an effect of its own that follows its owner, as it does the cart
//! (`CGameActor::SendMsg` and `CPlayer::SendMsg`, `AM_MAKE_BIRD`: a `CMsgEffect` of the kind `MET_BIRD`, whose
//! sprite is named in `CGameMode::SpawnMsgEffect`, with `SetAnimationSpeed(6.0)`). It is there while the bit
//! `BIT` of its owner's effect state is set (`StateFlag04`, the server's `OPTION_FALCON`) and the owner does not
//! hide (`CRenderObject::SetState`-side code in `RenderObject.cpp`: the bits 2, 4 and 0x40 send
//! `AM_DELETE_BIRD`, and coming out sends `AM_MAKE_BIRD`, which makes it anew behind its owner). Nothing in
//! the client takes it away while its owner sits or lies dead.
//!
//! Everything here goes by the client's own ticks, 60 a second (`InitTimer(60)`, one `CMsgEffect::OnProcess`
//! a tick): `TICK_MS`. Places on the ground are in cells, heights in the world's measure (a cell is 5).

use crate::map::CELL;

/// The bit of the effect state that says its owner has a falcon (`StateFlag04`: bit 4).
pub const BIT: u32 = 0x10;

/// The bits of the effect state while which the client has taken the falcon away: its owner hides (2), is
/// cloaked (4) or is unseen as `StateFlag06` has it (0x40). The same three take the cart away.
pub const HIDDEN: u32 = 2 | 4 | 0x40;

/// A tick of the client's in milliseconds (`frameFreq = 1000 / 60`).
pub const TICK_MS: f64 = 16.0;

/// How high over the ground under it the falcon is drawn (`FollowOwner`: the terrain's height less 25, the
/// client's y pointing down).
pub const HEIGHT: f32 = 25.0;

/// How far behind its owner it is made, in cells (`AM_SET_MASTER_POINTER`: 5.0 of the world's measure).
const BEHIND: f32 = 1.0;

/// How many ticks a picture of its hovering shows (`SetAnimationSpeed(6.0)`).
const PICTURE_TICKS: u32 = 6;

/// Blitz Beat and Falcon Assault (`HT_BLITZBEAT`, `SN_FALCONASSAULT`): a hit of these sends the falcon of the
/// one who used them at the one struck (`CGameActor::ProcessSkillMessage` with `GetEffectCounter`, which names
/// the Wind Hawk's skills too: those come with later content).
pub fn strikes(skill: u16) -> bool {
    matches!(skill, 129 | 381)
}

/// Detect (`HT_DETECTING`): used on a place, it sends the falcon there (`CGameActor::PlayGroundSkillEffect`).
pub const DETECT: u16 = 130;

/// Spring Trap (`HT_SPRINGTRAP`): used on a trap, it sends the falcon to the trap (`CGameMode::ReceiveSkillUse`).
pub const SPRING_TRAP: u16 = 131;

/// The sprite of the falcon of one of that job, under `data/sprite/` (`AM_MAKE_BIRD`, `SpawnMsgEffect`): a
/// Sniper's (`JT_HUNTER_H`) is `매2`, a baby Hunter's (`JT_HUNTER_B`) the grown one's pictures by a table of
/// its own (`매.spr` with `베이비매.act`, written here as the two names `figure.rs` reads apart), everyone
/// else's `매`. (The client names more: the Ranger's owl, the Wind Hawk's, and three monsters for jobs that
/// were never opened; those come with later content.)
pub fn sprite(job: u16) -> &'static str {
    match job {
        4012 => "이팩트/매2",
        4034 => "이팩트/매@이팩트/베이비매",
        _ => "이팩트/매",
    }
}

/// A falcon: where it is, and what it is doing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Falcon {
    /// Where it is over the ground, in cells.
    pub at: (f32, f32),
    /// How high it is, in the world's measure, counted up.
    pub up: f32,
    /// How far it goes each tick while it is sent at something (`m_velocity`): cells east and north, and up.
    velocity: (f32, f32, f32),
    /// `processTick`: the ticks of the picture that shows while it hovers; while it is sent, those since.
    tick: u32,
    /// `activeMotion`: it has been sent at something.
    sent: bool,
    /// `m_finished`, which the flight sets at its 25th tick.
    over: bool,
    /// `motionInitialized`: it is by its owner; not while it is sent and until it has come back.
    home: bool,
    /// `curMotion`: the picture of its action that shows.
    picture: usize,
}

impl Falcon {
    /// A falcon as it is made: a cell behind its owner, who looks the game's way `facing` (0 south, then by
    /// eighths through west) (`AM_SET_MASTER_POINTER`, as the cart is), over the ground at `ground`.
    pub fn behind(owner: (f32, f32), facing: u8, ground: f32) -> Self {
        Falcon { at: crate::carts::behind(owner, facing, BEHIND), up: ground + HEIGHT, velocity: (0.0, 0.0, 0.0), tick: 0, sent: false, over: false, home: true, picture: 0 }
    }

    /// It is sent at a place (`CMsgEffect::SendMsg`, `AM_REQ_ATTACK`): a tenth of the way there each tick. Its
    /// count of ticks is left as it is, as the client leaves it.
    pub fn send(&mut self, to: (f32, f32), ground: f32) {
        self.velocity = ((to.0 - self.at.0) / 10.0, (to.1 - self.at.1) / 10.0, (ground - self.up) / 10.0);
        (self.sent, self.home) = (true, false);
    }

    /// One tick (`CMsgEffect::OnProcess` with `FollowOwner`). `ground` tells how high the ground is at a
    /// place, `pictures` how many pictures an action of its hovering has.
    pub fn tick(&mut self, owner: (f32, f32), ground: impl Fn((f32, f32)) -> f32, pictures: usize) {
        // The flight is over once its 35th tick has gone by: from then it comes back.
        if self.over && self.sent && self.tick > 35 {
            (self.sent, self.over) = (false, false);
            return;
        }
        if self.sent {
            self.fly();
            return;
        }
        self.follow(owner);
        self.up = ground(self.at) + HEIGHT;
        // The next picture every sixth tick, and none while it comes back: the ninth is asked for then,
        // which no action of the sprite has, so the first shows.
        let mut tick = self.tick;
        if tick % PICTURE_TICKS == 0 {
            if self.home {
                self.picture += 1;
            }
            tick = 0;
        }
        if self.picture >= pictures.max(1) {
            self.picture = 0;
        }
        self.tick = tick + 1;
    }

    /// A tick of the flight at what it was sent to: it goes on along its line, level between the tenth tick
    /// and the 25th, and from the 25th its rise is turned about and made 1.9 times as large every tick,
    /// which throws it far up and down until the flight is over (as the client's code has it).
    fn fly(&mut self) {
        self.at.0 += self.velocity.0;
        self.at.1 += self.velocity.1;
        if !(self.tick > 10 && self.tick < 25) {
            self.up += self.velocity.2;
            if self.tick >= 25 {
                self.over = true;
                self.velocity.2 *= -1.9;
            }
        }
        self.tick += 1;
        self.picture = 0;
    }

    /// A tick of `CMsgEffect::FollowOwner`. Coming back from a flight it closes a 25th of the way each tick
    /// until it is within 25 of the world's measure; by its owner it flies 1 of that measure a tick while
    /// it is farther than 12, a 20th of the way while farther than 3, and stays where it is within that.
    fn follow(&mut self, owner: (f32, f32)) {
        let (dx, dy) = (owner.0 - self.at.0, owner.1 - self.at.1);
        let away = (dx * dx + dy * dy).sqrt() * CELL;
        let share = if !self.home {
            if away <= 25.0 {
                self.home = true;
            }
            self.picture = 8;
            1.0 / 25.0
        } else if away > 12.0 {
            1.0 / away
        } else if away > 3.0 {
            1.0 / 20.0
        } else {
            return;
        };
        self.at = (self.at.0 + dx * share, self.at.1 + dy * share);
    }

    /// Which of the sprite's actions shows to a camera turned by `yaw` degrees: hovering it looks at its
    /// owner (`Face`, `Get8Dir2`: the first eight actions, numbered as the cart's are); sent at something it
    /// looks away from its owner, in the eight actions of its flight after those.
    pub fn action(&self, owner: (f32, f32), yaw: f32) -> usize {
        let towards = crate::carts::Pulled { at: self.at, rolled: 0.0 }.action(owner, yaw);
        if self.sent { (towards + 4) % 8 + 8 } else { towards }
    }

    /// Which picture of its action shows.
    pub fn picture(&self) -> usize {
        self.picture
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAT: fn((f32, f32)) -> f32 = |_| 0.0;

    #[test]
    fn a_job_has_its_falcon() {
        assert_eq!(sprite(11), "이팩트/매");
        assert_eq!(sprite(4012), "이팩트/매2");
        assert_eq!(sprite(4034), "이팩트/매@이팩트/베이비매");
        assert!(strikes(129) && strikes(381) && !strikes(130));
    }

    #[test]
    fn it_is_made_behind_its_owner_and_over_the_ground() {
        let falcon = Falcon::behind((10.0, 10.0), 0, 3.0);
        assert!((falcon.at.0 - 10.0).abs() < 1e-5 && (falcon.at.1 - 11.0).abs() < 1e-5);
        assert_eq!(falcon.up, 28.0);
        assert_eq!(falcon.action((10.0, 10.0), 0.0), 0);
    }

    #[test]
    fn it_stays_near_and_follows_one_who_walks_away() {
        let mut falcon = Falcon::behind((10.0, 10.0), 0, 0.0);
        // 5 away: a 20th of the way each tick, until it is within 3.
        falcon.tick((10.0, 10.0), FLAT, 8);
        assert!((falcon.at.1 - 10.95).abs() < 1e-5);
        for _ in 0..200 {
            falcon.tick((10.0, 10.0), FLAT, 8);
        }
        let rests = falcon.at;
        assert!(((rests.1 - 10.0) * CELL - 3.0).abs() < 0.2);
        falcon.tick((10.0, 10.0), FLAT, 8);
        assert_eq!(falcon.at, rests);
        // Four cells away it flies 1 of the world's measure a tick, a fifth of a cell.
        falcon.tick((10.0, 6.0), FLAT, 8);
        assert!((rests.1 - falcon.at.1 - 0.2).abs() < 1e-4);
        // And it keeps to the ground's height under it.
        falcon.tick((10.0, 6.0), |_| 7.0, 8);
        assert_eq!(falcon.up, 32.0);
    }

    #[test]
    fn its_pictures_go_round_every_sixth_tick() {
        let mut falcon = Falcon::behind((10.0, 10.0), 0, 0.0);
        let shown: Vec<usize> = (0..14).map(|_| {
            falcon.tick((10.0, 10.0), FLAT, 8);
            falcon.picture()
        }).collect();
        // The first tick moves it on, then every sixth after.
        assert_eq!(shown, [1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3]);
        for _ in 0..(6 * 5) {
            falcon.tick((10.0, 10.0), FLAT, 8);
        }
        assert_eq!(falcon.picture(), 0);
    }

    #[test]
    fn sent_at_something_it_flies_there_past_it_and_comes_back() {
        let mut falcon = Falcon::behind((10.0, 10.0), 0, 0.0);
        falcon.tick((10.0, 10.0), FLAT, 8);
        let from = falcon.at;
        // Sent ten cells east, to the ground there: its count of ticks is at 1.
        falcon.send((from.0 + 10.0, from.1), 0.0);
        assert_eq!(falcon.action((10.0, 10.0), 0.0), 8 + 4);
        // Ten ticks take it there and down to the ground.
        for _ in 0..10 {
            falcon.tick((10.0, 10.0), FLAT, 8);
        }
        assert!((falcon.at.0 - from.0 - 10.0).abs() < 1e-4 && falcon.up.abs() < 1e-3);
        assert_eq!(falcon.picture(), 0);
        // It goes on level until the 25th tick of its count.
        for _ in 0..14 {
            falcon.tick((10.0, 10.0), FLAT, 8);
        }
        assert!((falcon.at.0 - from.0 - 24.0).abs() < 1e-3 && falcon.up.abs() < 1e-3);
        // Eleven more ticks throw it up and down, and the next ends the flight.
        for _ in 0..11 {
            falcon.tick((10.0, 10.0), FLAT, 8);
        }
        assert!((falcon.at.0 - from.0 - 35.0).abs() < 1e-3 && falcon.sent);
        falcon.tick((10.0, 10.0), FLAT, 8);
        assert!(!falcon.sent && !falcon.home);
        // It comes back a 25th of the way a tick, over the ground again, showing its first picture.
        let far = falcon.at.0 - 10.0;
        falcon.tick((10.0, 10.0), FLAT, 8);
        assert!((falcon.at.0 - 10.0 - far * 24.0 / 25.0).abs() < 1e-3);
        assert_eq!((falcon.up, falcon.picture()), (HEIGHT, 0));
        assert!(falcon.action((10.0, 10.0), 0.0) < 8);
        // Within 25 of the world's measure it is by its owner again.
        for _ in 0..200 {
            falcon.tick((10.0, 10.0), FLAT, 8);
        }
        assert!(falcon.home && (falcon.at.0 - 10.0) * CELL <= 3.1);
    }
}
