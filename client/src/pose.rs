//! How a character is posed, as the client's actors decide it: which action its state plays and how
//! (`CGameActor::SetState`, `CPc::SetState`, `WeaponActionBase`, `JobActionBase`), the frames of an action a
//! state runs through by itself (`CRenderObject::SetFrameSequence`), when it stands ready and not at ease,
//! when its weapon and shield are drawn at all (`IsReadyOrAttackAction`), how large a job is drawn
//! (`CSession::GetSkillFactor`), and which picture of a standing head shows (`headDir`).
//!
//! The world's side is `web/world/mover.rs`, which keeps the pose of everyone and picks the picture.

/// The first of a character's actions by what it does (`WeaponActionBase`): each has eight, a direction each.
pub const STANDS: usize = 0;
pub const WALKS: usize = 8;
pub const SITS: usize = 16;
pub const PICKS_UP: usize = 24;
pub const READY: usize = 32;
pub const THROWS: usize = 40;
pub const FLINCHES: usize = 48;
pub const LIES: usize = 64;
pub const STRIKES: usize = 80;
pub const STRIKES_OTHERWISE: usize = 88;
pub const CASTS: usize = 96;

/// One pass of an actor (`ProcessState`), in milliseconds: the frames of a sequence move on every so many
/// passes (`sequenceInterval`). The client counts passes, not time; a pass is taken here as `crate::body`
/// takes it, 16 milliseconds, which was not measured.
pub const PASS_MS: f64 = 16.0;

/// How a sequence of frames goes on when it has reached its last (`sequenceLoop` with the finish animation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Run {
    /// It holds the last frame until its time is up.
    Once,
    /// It begins again at its first frame (`ST_SING`: the finish animation is the action itself).
    Loops,
    /// It goes back and forth between its first and its last frame (`ST_DANCE`: no finish animation).
    Swings,
}

/// The frames of an action that a state runs through by itself (`SetFrameSequence(action, first, last, …,
/// duration, loop)` with the `sequenceInterval` set before it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sequence {
    pub action: usize,
    pub first: usize,
    pub last: usize,
    /// How many passes a frame shows.
    pub interval: u32,
    /// How long the whole lasts, in milliseconds; a sequence that loops lasts until another state begins.
    pub lasts: f64,
    pub run: Run,
}

impl Sequence {
    const fn once(action: usize, first: usize, last: usize, interval: u32, lasts: f64) -> Self {
        Self { action, first, last, interval, lasts, run: Run::Once }
    }

    /// The frame that shows after so many milliseconds.
    pub fn frame(&self, passed: f64) -> usize {
        let step = (passed.max(0.0) / (PASS_MS * f64::from(self.interval.max(1)))) as usize;
        let span = self.last.saturating_sub(self.first);
        match self.run {
            Run::Once => self.first + step.min(span),
            Run::Loops => self.first + step % (span + 1),
            // First to last, then back down to the first and up again (`--sequenceFrame`).
            Run::Swings if span == 0 => self.first,
            Run::Swings => {
                let at = step % (span * 2);
                self.first + if at <= span { at } else { span * 2 - at }
            }
        }
    }

    /// Whether its time is up.
    pub fn over(&self, passed: f64) -> bool {
        self.run == Run::Once && passed >= self.lasts
    }
}

/// What a character does for a state it is put in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Posed {
    /// An action played once through, each picture for as long as the sprite says (`SetAction`).
    Plays(usize),
    /// Frames of an action it runs through by itself.
    Runs(Sequence),
}

/// Bard, Dancer, Crusader and their baby selves, Gunslinger, Taekwon and the Star Gladiators with theirs: the
/// jobs that use a skill standing ready (`CPc::JobActionBase(ST_SKILL)` gives 32).
const USE_READY: [u16; 13] = [14, 19, 20, 24, 4037, 4042, 4043, 4046, 4047, 4048, 4225, 4226, 4228];
/// The jobs that use a skill in the first picture of the casting action, held for 400 milliseconds
/// (`CPc::SetState(ST_SKILL)`): the reborn Bard, Dancer and Crusader, the Monks, the Royal Guards and Suras.
const USE_HELD: [u16; 12] = [4020, 4021, 4015, 4066, 4073, 15, 4016, 4038, 4070, 4077, 4106, 4262];

/// What a character of a job does for a state the client's table gives a skill (`skill_motions.txt`, the
/// states of `CGameActor::SetState`). `strike` is its attack for what it holds (`attack_action`: 80 or 88).
/// A state not known here is the motion of using a skill.
pub fn skilled(state: u8, job: u16, strike: usize) -> Posed {
    let held = |action, frame, lasts| Posed::Runs(Sequence::once(action, frame, frame, 10, lasts));
    match state {
        // ST_ATTACK, ST_ATTACK2: both ask what is held (`IsJobWeaponAllowed`).
        2 | 9 => Posed::Plays(strike),
        5 => Posed::Plays(PICKS_UP),
        11 => Posed::Plays(READY),
        // ST_THROW.
        12 => Posed::Plays(THROWS),
        // ST_AUTOCOUNTER: the attack for what is held, frozen in its first picture (`isMotionFreezed`)
        // until another state begins; taken here as the 3000 milliseconds of the readied shot, since the
        // state that ends it is the skill's own, told by the server.
        13 => held(strike, 0, 3000.0),
        16 => Posed::Runs(Sequence { action: CASTS, first: 1, last: 3, interval: 10, lasts: f64::INFINITY, run: Run::Swings }),
        17 => Posed::Runs(Sequence { action: CASTS, first: 1, last: 3, interval: 10, lasts: f64::INFINITY, run: Run::Loops }),
        // ST_READYSHOT.
        18 => held(STRIKES_OTHERWISE, 0, 3000.0),
        // The Taekwon stances and kicks (ST_R_STORM … ST_A_JUMPKICK, ST_SG_DOWN, ST_SG_TURN).
        21 => held(CASTS, 0, 2000.0),
        22 => held(CASTS, 2, 2000.0),
        23 => held(CASTS, 3, 2000.0),
        24 => held(CASTS, 4, 2000.0),
        25 => held(PICKS_UP, 1, 2000.0),
        27 => held(CASTS, 5, 3000.0),
        30 => held(CASTS, 1, 2000.0),
        33 => held(CASTS, 2, 1000.0),
        34 => held(CASTS, 3, 1000.0),
        // The ninja's and the gunslinger's (ST_NINJASKILL1, ST_NINJASKILL2, ST_GUNSKILL1 … ST_GUNREADY2).
        35 => Posed::Runs(Sequence::once(CASTS, 0, 1, 6, 500.0)),
        36 => Posed::Runs(Sequence::once(CASTS, 2, 5, 6, 1000.0)),
        37 => held(CASTS, 2, 500.0),
        38 => Posed::Runs(Sequence::once(CASTS, 0, 3, 6, 1000.0)),
        39 => Posed::Runs(Sequence::once(CASTS, 4, 5, 6, 500.0)),
        40 => Posed::Runs(Sequence::once(THROWS, 0, 2, 6, 500.0)),
        41 => held(CASTS, 1, 500.0),
        42 => held(THROWS, 0, 500.0),
        // ST_SKILL and whatever else: by the job.
        _ => used(job),
    }
}

/// The motion of using a skill (`CPc::SetState(ST_SKILL)`).
fn used(job: u16) -> Posed {
    match job {
        _ if USE_HELD.contains(&job) => Posed::Runs(Sequence::once(CASTS, 0, 0, 4, 400.0)),
        25 | 4222 => Posed::Runs(Sequence::once(CASTS, 2, 5, 4, 1000.0)),
        // (`JobActionBase` gives a Gunslinger 32 and `SetState` then runs frames of 96: the sequence wins.)
        24 | 4228 => Posed::Runs(Sequence::once(CASTS, 0, 3, 4, 1000.0)),
        _ if USE_READY.contains(&job) => Posed::Plays(READY),
        _ => Posed::Plays(CASTS),
    }
}

/// Whether a character stands ready, not at ease, once a motion of this action is over
/// (`CPc::SetState(ST_STAND)` coming from `ST_ATTACK`, `ST_DAMAGE` or `ST_BEGIN_SKILL`: action 32 and not 0).
/// It stands so until it walks or sits. One whose health state is not 0 stands ready after anything.
pub fn readies(action: usize) -> bool {
    matches!(action - action % 8, STRIKES | STRIKES_OTHERWISE | FLINCHES | CASTS)
}

/// Whether the weapon, its trail and the shield are drawn in an action (`IsReadyOrAttackAction`): only
/// standing ready and striking.
pub fn armed(action: usize) -> bool {
    matches!(action, 32..=39 | 80..=95)
}

/// How many times its sprite's size a job is drawn (`CSession::GetSkillFactor`): the baby jobs are smaller,
/// the first ones 0.75 and the second ones 0.82. (The baby selves of the jobs after 2010 are 0.82 too and
/// the Doram 0.8: named here only as far as the third jobs.)
pub fn size(job: u16) -> f32 {
    match job {
        4023..=4029 | 4045 | 4191 => 0.75,
        4030..=4044 | 4096..=4112 => 0.82,
        _ => 1.0,
    }
}

/// Which picture of a standing or sitting head shows for the way the head is turned (`headDir` as the
/// packets carry it: 0 ahead, 1 to the right, 2 to the left; `curMotion = headDir`).
pub fn head_frame(turned: u8) -> usize {
    usize::from(turned.min(2))
}

/// One who stands or sits looks towards a place (`CPlayer::SendMsg(AM_REQ_CHANGE_DIR)`: a press on the ground
/// with Shift held, or while sitting): it faces `facing` with its head turned `head`, and the place lies in
/// the direction `towards` (directions as the sprites count them, eight round). A place a little to one side
/// turns the head alone, and a second look that way the body after it; one farther round turns the body to
/// an eighth short of it and the head the rest; one ahead or straight behind turns the body with the head
/// ahead. Gives back the direction it faces then and how its head is turned.
pub fn looks(facing: u8, head: u8, towards: u8) -> (u8, u8) {
    let towards = towards % 8;
    match (towards + 8 - facing % 8) % 8 {
        1 if head == 1 => (towards, 0),
        1 => (facing, 1),
        7 if head == 2 => (towards, 0),
        7 => (facing, 2),
        2 | 3 => ((towards + 7) % 8, 1),
        5 | 6 => ((towards + 1) % 8, 2),
        _ => (towards, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dance_swings_and_a_song_loops() {
        let Posed::Runs(dance) = skilled(16, 20, 80) else { panic!() };
        let frames: Vec<usize> = (0..9).map(|step| dance.frame(f64::from(step) * 160.0)).collect();
        assert_eq!(frames, [1, 2, 3, 2, 1, 2, 3, 2, 1]);
        assert!(!dance.over(1e9));
        let Posed::Runs(song) = skilled(17, 19, 80) else { panic!() };
        let frames: Vec<usize> = (0..7).map(|step| song.frame(f64::from(step) * 160.0)).collect();
        assert_eq!(frames, [1, 2, 3, 1, 2, 3, 1]);
    }

    #[test]
    fn a_sequence_holds_its_last_frame_until_its_time_is_up() {
        let Posed::Runs(ninja) = skilled(7, 25, 80) else { panic!() };
        assert_eq!((ninja.frame(0.0), ninja.frame(64.0), ninja.frame(900.0)), (2, 3, 5));
        assert!(!ninja.over(999.0) && ninja.over(1000.0));
    }

    #[test]
    fn a_skill_is_used_as_the_job_uses_it() {
        assert_eq!(skilled(7, 9, 80), Posed::Plays(CASTS));
        assert_eq!(skilled(7, 19, 80), Posed::Plays(READY));
        assert_eq!(skilled(12, 3, 80), Posed::Plays(THROWS));
        assert_eq!(skilled(9, 1, 88), Posed::Plays(88));
        assert!(matches!(skilled(7, 15, 80), Posed::Runs(Sequence { first: 0, last: 0, lasts, .. }) if lasts == 400.0));
        assert!(matches!(skilled(18, 11, 80), Posed::Runs(Sequence { action: STRIKES_OTHERWISE, .. })));
    }

    #[test]
    fn one_who_fought_stands_ready_and_only_then_shows_the_weapon() {
        assert!(readies(83) && readies(48) && readies(96) && readies(95));
        assert!(!readies(24) && !readies(0) && !readies(40));
        assert!(armed(32) && armed(39) && armed(80) && armed(95));
        assert!(!armed(0) && !armed(8) && !armed(40) && !armed(79) && !armed(96));
    }

    #[test]
    fn a_look_to_one_side_turns_the_head_first() {
        assert_eq!(looks(0, 0, 1), (0, 1));
        assert_eq!(looks(0, 1, 1), (1, 0));
        assert_eq!(looks(0, 0, 7), (0, 2));
        assert_eq!(looks(0, 2, 7), (7, 0));
        assert_eq!(looks(0, 0, 2), (1, 1));
        assert_eq!(looks(0, 0, 6), (7, 2));
        assert_eq!((looks(3, 1, 3), looks(3, 2, 7)), ((3, 0), (7, 0)));
    }

    #[test]
    fn babies_are_smaller() {
        assert_eq!((size(4023), size(4045), size(4030), size(4044), size(7), size(4008)), (0.75, 0.75, 0.82, 0.82, 1.0, 1.0));
    }
}
