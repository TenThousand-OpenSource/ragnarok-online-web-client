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

/// The frames a character runs through as a skill of its strikes (`CGameActor::ProcessSkillMessage`, of the
/// skills a classic server has: the monk's). `combo` is the window of a combo the server told of that one
/// last, in milliseconds (ZC_COMBODELAY, kept by `CSession::InsertPair` and asked with
/// `FindActorSkillAnimationValue`): the combos hold their frame for as long. None for any other skill, whose
/// striker swings as for a plain blow. (`CH_PALMSTRIKE` asks for frames that last one millisecond: left out.)
pub fn strikes_with(skill: u16, combo: f64) -> Option<Sequence> {
    Some(match skill {
        // MO_TRIPLEATTACK, and MO_EXTREMITYFIST for three and a half seconds.
        263 => Sequence::once(STRIKES_OTHERWISE, 0, 7, 4, combo),
        271 => Sequence::once(STRIKES_OTHERWISE, 0, 7, 4, 3500.0),
        // MO_INVESTIGATE, MO_FINGEROFFENSIVE.
        266 | 267 => Sequence::once(CASTS, 3, 4, 10, 1100.0),
        // MO_CHAINCOMBO, MO_COMBOFINISH, CH_TIGERFIST.
        272 => Sequence::once(CASTS, 5, 5, 4, combo),
        273 => Sequence::once(CASTS, 1, 2, 6, combo),
        371 => Sequence::once(CASTS, 4, 4, 6, combo),
        _ => return None,
    })
}

/// The picture of one who has caught a blade, held until the blade is let go (`ReceiveActorPairFreeze`:
/// `SetFrameSequence(96, 4, 4, 96, 4, 1, 99999999, 0)`).
pub const CATCHES: Sequence = Sequence::once(CASTS, 4, 4, 10, 99_999_999.0);

impl Sequence {
    /// The same frames, to last so long from now on, `passed` milliseconds into them
    /// (`CRenderObject::SetSequenceDeadline`: the deadline of the sequence under way is put anew).
    pub fn until(self, passed: f64, more: f64) -> Self {
        Self { lasts: passed.max(0.0) + more, ..self }
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

/// The picture of the head, and of what is worn on it, that a Taekwon or a Star Gladiator shows in place of
/// the frame's own in some frames of its kicks (`CPc::SelectStanceFrame`, for the layers 1 to 4): an action
/// and the frame of it, which is the first of a standing head seen from another side. In the second attack
/// (88 to 95) a woman's fifth and sixth frames and a man's fourth and fifth are replaced, in the casting
/// action (96 and on) the second frame. `sex` is 1 for a man. None: the frame's own. (The client leaves one
/// who rides out: such a one is drawn as another job here. Of the jobs it names, the Star Emperors and their
/// like are Renewal's and not named here.)
pub fn stance_head(job: u16, sex: u8, action: usize, frame: usize) -> Option<(usize, usize)> {
    if !matches!(job, 4046..=4048 | 4225 | 4226 | 4238) {
        return None;
    }
    let stands = match action {
        88..=95 => {
            // Which of the two replaced frames it is: the first or the second.
            let which = frame.checked_sub(if sex == 1 { 3 } else { 4 }).filter(|which| *which < 2)?;
            [[6, 5], [6, 7], [2, 1], [2, 3]][(action - 88) / 2][which]
        }
        96..=97 if frame == 1 => 0,
        98..=99 if frame == 1 => 6,
        100.. if frame == 1 => 2,
        _ => return None,
    };
    Some((stands, 0))
}

/// The kind of weapon whose sprite a job shows for a weapon of a kind that has no sprite of its own for that
/// job (`CPc::MapMotion`, by the client's numbers of the kinds, `Weapon_IDs`): a thief swings an axe or a
/// mace as a sword, an assassin and a rogue a mace and a two-handed axe so (a rogue a one-handed axe as a
/// dagger), a mage holds a two-handed spear as a staff. (The third jobs the client names are Renewal's.)
pub fn weapon_shown(job: u16, kind: u16) -> u16 {
    match (job, kind) {
        (6 | 4007 | 4029, 6..=8) => 2,
        (12 | 17 | 4013 | 4018 | 4035 | 4040, 7 | 8) => 2,
        (17 | 4018 | 4040, 6) => 1,
        (2 | 9 | 16 | 4003 | 4010 | 4017 | 4025 | 4032 | 4039, 5) => 10,
        _ => kind,
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

/// The bits of the state someone is in (the client's `StateFlag01`, `02` and `06`) that take its sprite
/// away: hiding, cloaking, and being invisible.
pub const HIDES: u32 = 1 << 1;
pub const CLOAKS: u32 = 1 << 2;
pub const UNSEEN_BIT: u32 = 1 << 6;

/// Whether the sprite of one in such a state is not drawn (`render.enabled`, `render.visible`): it hides,
/// cloaks or is invisible.
pub fn hidden(state: u32) -> bool {
    state & (HIDES | CLOAKS | UNSEEN_BIT) != 0
}

/// Whether one is down, so that no shadow lies under it: it lies dead, or is a character that sits; not one
/// that is still falling as it dies (`dying`).
pub fn down(character: bool, dying: bool, dead: bool, sitting: bool) -> bool {
    !dying && (dead || (character && sitting))
}

/// Whether a shadow lies under one (`CPc::Render`, `CRenderObject::DrawShadow`): not under one who is `down`
/// or vanishes, nor under another who hides, cloaks or is invisible; under the player's own character that
/// hides it stays (`shadowVisible` is taken from the others only), not under one cloaked or invisible, whose
/// `visible` is off.
pub fn shadowed(down: bool, vanishes: bool, own: bool, state: u32) -> bool {
    !down && !vanishes && if own { state & (CLOAKS | UNSEEN_BIT) == 0 } else { !hidden(state) }
}

/// Whether one whose state changes from `was` to `now` begins to hide or comes out of hiding
/// (`CGameActor::SetStatusStates`): either puffs smoke where it stands.
pub fn hiding_changed(was: u32, now: u32) -> bool {
    (was ^ now) & HIDES != 0
}

/// Whether the client's `EF_BASH` shows on one of a job in such a state: on one that hides, unless a ninja
/// (jobs 25 and 4222).
pub fn bashed_hiding(state: u32, job: u16) -> bool {
    state & HIDES != 0 && !matches!(job, 25 | 4222)
}

/// How long one falls dead before it lies in the last picture of its fall, in milliseconds (`ST_DEAD`): the
/// pictures of the fall, each for as long as the sprite says (`timing`: how many and how long each), no
/// shorter than a quarter of a second; 700 for one whose sprite's timing is not known.
pub fn fall_lasts(timing: Option<(usize, f64)>) -> f64 {
    timing.map_or(700.0, |(pictures, each)| pictures as f64 * each).max(250.0)
}

/// The direction the sprites count (from the south, turning one way) for a direction as the server counts
/// it (0 north, turning left; ZC_CHANGE_DIRECTION and the packets of those that stand).
pub fn facing_of(direction: u8) -> u8 {
    (12 - direction % 8) % 8
}

/// The direction the server is told for one the sprites count as `facing` (0 to 7; the other way of
/// `facing_of`).
pub fn direction_of(facing: u8) -> u8 {
    (12 - facing) % 8
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
    fn a_combo_holds_its_frames_for_the_window_the_server_told() {
        let chain = strikes_with(272, 650.0).unwrap();
        assert_eq!((chain.action, chain.frame(0.0), chain.frame(600.0)), (CASTS, 5, 5));
        assert!(!chain.over(649.0) && chain.over(650.0));
        let triple = strikes_with(263, 300.0).unwrap();
        assert_eq!((triple.action, triple.frame(0.0), triple.frame(64.0), triple.frame(10_000.0)), (STRIKES_OTHERWISE, 0, 1, 7));
        assert_eq!(strikes_with(271, 300.0).unwrap().lasts, 3500.0);
        assert_eq!(strikes_with(5, 300.0), None);
        // A new window told 200 milliseconds into the frames keeps them 500 more.
        let longer = chain.until(200.0, 500.0);
        assert!(!longer.over(699.0) && longer.over(700.0));
        assert!(!CATCHES.over(1e7) && CATCHES.frame(5000.0) == 4);
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
    fn a_taekwons_head_in_its_kicks() {
        // A woman's fifth and sixth frames, a man's fourth and fifth, by the side it is seen from.
        assert_eq!((stance_head(4046, 0, 88, 4), stance_head(4046, 0, 89, 5), stance_head(4046, 0, 88, 3)), (Some((6, 0)), Some((5, 0)), None));
        assert_eq!((stance_head(4047, 1, 90, 3), stance_head(4047, 1, 91, 4), stance_head(4047, 1, 90, 5)), (Some((6, 0)), Some((7, 0)), None));
        assert_eq!((stance_head(4046, 1, 93, 4), stance_head(4046, 0, 95, 5)), (Some((1, 0)), Some((3, 0))));
        // The second frame of the casting action.
        assert_eq!(
            (stance_head(4046, 1, 96, 1), stance_head(4046, 1, 99, 1), stance_head(4046, 1, 101, 1), stance_head(4046, 1, 96, 2)),
            (Some((0, 0)), Some((6, 0)), Some((2, 0)), None)
        );
        // No other job, no other action.
        assert_eq!((stance_head(15, 1, 96, 1), stance_head(4046, 1, 80, 4)), (None, None));
    }

    #[test]
    fn a_weapon_without_a_sprite_for_the_job_is_shown_as_another_kind() {
        assert_eq!((weapon_shown(6, 6), weapon_shown(6, 8), weapon_shown(6, 1)), (2, 2, 1));
        assert_eq!((weapon_shown(12, 6), weapon_shown(17, 6), weapon_shown(12, 7)), (6, 1, 2));
        assert_eq!((weapon_shown(9, 5), weapon_shown(1, 5), weapon_shown(4010, 5)), (10, 5, 10));
    }

    #[test]
    fn babies_are_smaller() {
        assert_eq!((size(4023), size(4045), size(4030), size(4044), size(7), size(4008)), (0.75, 0.75, 0.82, 0.82, 1.0, 1.0));
    }

    #[test]
    fn one_that_hides_cloaks_or_is_invisible_is_not_drawn() {
        assert!(hidden(2) && hidden(4) && hidden(0x40) && hidden(0x46) && hidden(0xffff_ffff));
        assert!(!hidden(0) && !hidden(1) && !hidden(8) && !hidden(0x20) && !hidden(0x80));
    }

    #[test]
    fn a_shadow_lies_under_those_that_stand_and_are_seen() {
        // Down: one that lies dead, a character that sits; not a monster that "sits", nor one still falling.
        assert!(down(false, false, true, false) && down(true, false, false, true) && down(true, false, true, true));
        assert!(!down(false, false, false, true) && !down(true, true, true, true) && !down(true, false, false, false));
        assert!(shadowed(false, false, false, 0) && shadowed(false, false, true, 0) && shadowed(false, false, false, 1));
        assert!(!shadowed(true, false, false, 0) && !shadowed(false, true, true, 0));
        // Another that hides, cloaks or is invisible has none; the player's own that hides keeps it.
        assert!(!shadowed(false, false, false, HIDES) && !shadowed(false, false, false, CLOAKS));
        assert!(!shadowed(false, false, false, UNSEEN_BIT));
        assert!(shadowed(false, false, true, HIDES) && !shadowed(false, false, true, CLOAKS));
        assert!(!shadowed(false, false, true, UNSEEN_BIT) && !shadowed(false, false, true, HIDES | CLOAKS));
    }

    #[test]
    fn hiding_and_coming_out_puff_smoke_and_hiding_shows_the_bash() {
        assert!(hiding_changed(0, 2) && hiding_changed(2, 0) && hiding_changed(6, 4) && hiding_changed(0x40, 0x42));
        assert!(!hiding_changed(2, 2) && !hiding_changed(0, 4) && !hiding_changed(2, 6) && !hiding_changed(0, 0));
        assert!(bashed_hiding(2, 12) && bashed_hiding(6, 0) && bashed_hiding(2, 4035));
        assert!(!bashed_hiding(2, 25) && !bashed_hiding(2, 4222) && !bashed_hiding(0, 12) && !bashed_hiding(4, 12));
    }

    #[test]
    fn a_fall_lasts_as_long_as_its_pictures() {
        assert_eq!((fall_lasts(None), fall_lasts(Some((4, 100.0))), fall_lasts(Some((7, 62.5)))), (700.0, 400.0, 437.5));
        // No shorter than a quarter of a second, whatever the sprite says.
        assert_eq!((fall_lasts(Some((2, 100.0))), fall_lasts(Some((0, 100.0))), fall_lasts(Some((3, -5.0)))), (250.0, 250.0, 250.0));
        assert_eq!(fall_lasts(Some((1, f64::NAN))), 250.0);
    }

    #[test]
    fn the_servers_directions_and_the_sprites_turn_into_each_other() {
        assert_eq!([0, 1, 2, 3, 4, 5, 6, 7].map(facing_of), [4, 3, 2, 1, 0, 7, 6, 5]);
        assert_eq!([4, 3, 2, 1, 0, 7, 6, 5].map(direction_of), [0, 1, 2, 3, 4, 5, 6, 7]);
        // A direction past the eighth is taken round; a facing is not, up to 12.
        assert_eq!((facing_of(8), facing_of(13), facing_of(255)), (4, 7, 5));
        assert_eq!((direction_of(8), direction_of(12)), (4, 0));
    }
}
