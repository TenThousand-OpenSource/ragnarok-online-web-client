//! Blows and casts: what the app tells the world when someone strikes, casts, aims or is hit. (An effect by
//! its number or its file is `launch.rs`; the effects of states `state_effects.rs`.)

use super::*;

mod landing;

pub use landing::number_over;
pub(super) use landing::{ARROW, ARROW_MS, Flying, Landing, Struck, landings};

/// A blow or a skill that strikes, as the app tells it: who strikes whom (`own` is the player's own ID, who
/// is not among the others), the speed of the swing and the time the one struck flinches as the server sent
/// them, the hits it is made of (`crate::blow::landing`), whether each lands with the ring of a plain blow
/// (`EF_HIT1`: a skill's does not, it shows what the client has for that skill), and whether the player's own
/// character is under a state that wards blows off. Of a skill: which it is (the striker is posed by it,
/// `crate::pose::strikes_with`), and the effect that follows it on the one struck, which the client's message
/// of its first hit carries (`effectId`: the one struck flashes by it, `crate::blow::signs::flash`).
pub struct Strike {
    pub source: u32,
    pub target: u32,
    pub own: u32,
    pub lasts: f64,
    pub flinch: f64,
    pub hits: Vec<crate::blow::landing::Hit>,
    pub ring: bool,
    pub guarded: bool,
    pub skill: Option<u16>,
    pub follow: Option<u32>,
}

/// Someone strikes someone: the one turns to the other and swings, and each hit of the blow lands in its
/// time: the other flinches, the hit sounds, and its numbers rise (`landing.rs`). Gives back how long from now
/// the first hit lands (milliseconds).
pub fn struck(strike: Strike) -> f64 {
    let Strike { source, target, own, lasts, flinch, hits, ring, guarded, skill, follow } = strike;
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return 0.0 };
        let time = now();
        let place = |scene: &Scene, id: u32| scene.one(id, own).map(|unit| unit.cell);
        let (from, to) = (place(scene, source), place(scene, target));
        let mut lands = flinch.clamp(0.0, 1500.0);
        if source == own {
            state.own_struck = time;
            // The blow's own motion (the server's `amotion`, sent with the blow): until it is over the character
            // does not walk (see `striking`).
            state.own_striking_until = time + lasts.max(0.0);
        }
        if let Some(striker) = scene.one_mut(source, own) {
            // Characters have two attacks, and which one shows depends on the job and on what is held (see
            // `attack_action`); monsters have one.
            let action = match &striker.look {
                Look::Character(figure) => attack_action(figure),
                Look::Creature(_) => 16,
            };
            striker.motion = Some((action, time, lasts.clamp(250.0, 2000.0)));
            lands = lands_after(striker, action, lasts, flinch);
            // A monk's combos and fists pose a character in frames of their own, some for as long as the
            // window of the combo the server told last (`CGameActor::ProcessSkillMessage`).
            if let (Look::Character(figure), Some(sequence)) = (&striker.look, skill.and_then(|skill| crate::pose::strikes_with(skill, striker.pose.combo))) {
                figure::prepare(figure, sequence.action);
                (striker.motion, striker.pose.sequence) = (None, Some((sequence, time)));
            }
            if let (Some(from), Some(to)) = (from, to) {
                if from != to {
                    striker.facing = crate::map::direction(to.0 - from.0, to.1 - from.1);
                }
            }
        }
        // Each hit lands a moment into the swing, the later ones their time after the first.
        for (index, hit) in hits.into_iter().enumerate() {
            let when = time + lands + hit.after.clamp(0.0, 8000.0);
            // (Only the first hit's message carries the effect that follows: the later ones carry none.)
            state.blows.push((when, target, Landing { own, striker: source, hit, ring, guarded, follow: follow.filter(|_| index == 0) }));
        }
        lands
    })
}

/// The server tells the window of someone's combo (ZC_COMBODELAY, the client's `ReceiveActorSequenceDeadline`):
/// it is kept for that one's next combos, and the frames it runs through now last so long from now
/// (`CRenderObject::SetSequenceDeadline`).
pub fn combo_window(who: u32, own: u32, milliseconds: f64) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let Some(mover) = scene.one_mut(who, own) else { return };
        let time = now();
        mover.pose.combo = milliseconds;
        if let Some((sequence, began)) = &mut mover.pose.sequence {
            *sequence = sequence.until(time - *began, milliseconds);
        }
    });
}

/// One catches another's blade, or lets it go (ZC_BLADESTOP, the client's `ReceiveActorPairFreeze`). Caught,
/// and only with both in sight: the one who catches is held in the fifth picture of its casting action, the
/// other's motion is held where it was, and both are grey (64, 64, 64), the colour locked as a state's is.
/// Let go: each that is in sight is white again and moves as before.
pub fn blade_caught(who: u32, whose: u32, own: u32, held: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let time = now();
        let there = |scene: &Scene, id: u32| id == own || scene.units.contains_key(&id);
        if held && !(there(scene, who) && there(scene, whose)) {
            return;
        }
        for (id, catches) in [(who, true), (whose, false)] {
            let Some(mover) = scene.one_mut(id, own) else { continue };
            mover.ail.body.locked(if held { [64; 3] } else { crate::body::WHITE });
            if !catches {
                mover.pose.frozen = held;
            } else if held && matches!(mover.look, Look::Character(_)) {
                (mover.motion, mover.pose.sequence) = (None, Some((crate::pose::CATCHES, time)));
            } else if mover.pose.sequence.is_some_and(|(sequence, _)| sequence == crate::pose::CATCHES) {
                mover.pose.sequence = None;
            }
        }
    });
}

/// Whether the client shows the other hand's damage of a blow that someone strikes (`crate::blow::signs`).
pub fn two_handed(who: u32, own: u32) -> bool {
    STATE.with_borrow(|state| {
        let Some(scene) = &state.scene else { return false };
        let striker = scene.one(who, own);
        striker.is_some_and(|striker| crate::blow::signs::two_handed(striker.job, matches!(striker.look, Look::Character(_))))
    })
}

/// How long into a swing its blow lands (milliseconds), as the client has it (`crate::blow`): by the frame of
/// the striker's sprite that the blow is on and the speed the server sent with the blow. Until the sprite is
/// read that frame is not known: then it is `otherwise`.
fn lands_after(striker: &Mover, action: usize, speed: f64, otherwise: f64) -> f64 {
    let frame = match &striker.look {
        Look::Character(figure) => figure::body_strike_frame(figure.job, figure.sex, action),
        Look::Creature(path) => figure::strike_frame(path, action),
    };
    frame.map_or(otherwise, |frame| crate::blow::lands_after(frame, speed, striker.job)).clamp(0.0, 1500.0)
}

/// The action of a character bending down for a thing on the ground (the client's `ST_PICKUP`).
const PICKING_UP: usize = 24;

/// Someone picks a thing up (the server's notice of it, `DMG_PICKUP_ITEM`): they turn to the thing and bend down
/// for it once, each picture for as long as the sprite says, and stand again (the client's `ST_PICKUP`). The
/// thing itself is gone from the ground when the server says so. `own` is the player's own ID.
pub fn picks_up(id: u32, own: u32, thing: u32) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let lies = scene.drops.get(&thing).map(|lying| lying.cell);
        let Some(mover) = scene.one_mut(id, own) else { return };
        let Look::Character(figure) = &mover.look else { return };
        figure::prepare(figure, PICKING_UP);
        let lasts = figure::body_timing(figure.job, figure.sex, PICKING_UP).map_or(450.0, |(pictures, each)| pictures as f64 * each);
        if let Some(cell) = lies.filter(|cell| *cell != mover.cell) {
            mover.facing = crate::map::direction(cell.0 - mover.cell.0, cell.1 - mover.cell.1);
        }
        // The thing is asked for from a cell away, so the server may say it is taken while the character still
        // walks its last step (after a monster killed from afar, as with a spell): it bends down when it gets
        // there, not while walking, when the bending would not be seen at all.
        let time = now();
        let begins = mover.arrives(&state.clock, time);
        mover.motion = Some((PICKING_UP, begins, lasts.max(250.0)));
    });
}

/// The player's own character is about to pick something up: the pictures of its bending down are made ready
/// while it walks there.
pub fn ready_to_pick_up() {
    STATE.with_borrow(|state| {
        if let Some(Look::Character(figure)) = state.scene.as_ref().map(|scene| &scene.player.look) {
            figure::prepare(figure, PICKING_UP);
        }
    });
}

/// Someone starts to cast a skill (the client's `NotifySkillCasting`): they turn to the one aimed at and stand
/// casting, the bar over them fills over the cast's time, the rings of the cast's element turn around them and
/// the mark closes in on the one aimed at.
pub fn cast(source: u32, own: u32, target: u32, cell: (u16, u16), milliseconds: f64, effect: u32) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let place = |scene: &Scene, id: u32| scene.one(id, own).map(|unit| unit.cell);
        let towards = if target != 0 { place(scene, target) } else { Some((i32::from(cell.0), i32::from(cell.1))) };
        let Some(caster) = scene.one_mut(source, own) else { return };
        let time = now();
        if let Some(towards) = towards.filter(|towards| *towards != caster.cell) {
            caster.facing = crate::map::direction(towards.0 - caster.cell.0, towards.1 - caster.cell.1);
        }
        caster.casting = Some((time, milliseconds, effect, (target != 0 && target != source).then_some(target)));
        // Characters stand in their casting action for as long.
        if matches!(caster.look, Look::Character(_)) {
            caster.motion = Some((96, time, milliseconds.max(250.0)));
        }
        // The sound of a spell begun, from where the caster stands.
        earshot::heard(scene, "effect/ef_beginspell.wav", (source != own).then_some(source));
    });
}

/// A cast is over or broken off (the server's ZC_DISPEL, or the skill landing): the bar, the rings and the mark
/// go, and the caster stands as before.
pub fn cast_over(source: u32, own: u32, broken: bool) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let Some(caster) = scene.one_mut(source, own) else { return };
        if caster.casting.take().is_some() && broken {
            caster.motion = None;
        }
    });
}

/// An arrow is shot at someone, to arrive after `delay` milliseconds (the client's `CArrowEffect`): it leaves
/// 10 above the archer's feet and flies for 8 of the client's frames.
pub fn arrow(source: u32, target: u32, own: u32, delay: f64) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &state.scene else { return };
        let Some(archer) = scene.one(source, own) else { return };
        // Only one who holds a bow shoots (the client asks its weapon table whether the weapon uses arrows).
        let Look::Character(figure) = &archer.look else { return };
        if !data::about(figure.weapon).is_some_and(|(_, facts)| facts.iter().any(|(key, kind)| key == "SubType" && kind == "Bow")) {
            return;
        }
        let from = along(on_ground(&scene.gat, archer.at), [0.0, 1.0, 0.0], 10.0);
        state.arrows.push((now() + (delay - ARROW_MS).max(0.0), from, target));
    });
}

/// The server says how far the character's weapon reaches, in cells (ZC_ATTACK_RANGE): it does on entering the
/// map and whenever that changes (another weapon, a skill that lengthens a bow's reach).
pub fn weapon_reaches(range: i32) {
    STATE.with_borrow_mut(|state| state.strike_range = Some(range.max(1)));
}

/// The server says a blow was asked for from too far (ZC_ATTACK_FAILURE_FOR_DISTANCE), and how far the weapon
/// reaches: the character walks up to the monster it fights and strikes from there, as the client does.
pub fn too_far(target: u32, range: i32) {
    STATE.with_borrow_mut(|state| {
        state.strike_range = Some(range.max(1));
        if let Some((id, _, told)) = &mut state.chase {
            if *id == target {
                // Looked after at once, and the blow asked for again once it stands near enough.
                *told = false;
            }
        }
        if let Some(chase) = &mut state.chase {
            chase.1 = 0.0;
        }
    });
}

/// The character stops going after anything: the monster it fought, the thing it meant to pick up, the place
/// it meant to use a skill from.
pub fn stand_down() {
    STATE.with_borrow_mut(|state| {
        state.chase = None;
        state.fetch = None;
        state.reach = None;
    });
}

/// A skill starts or stops being aimed: at a place (`Some(true)`), at someone (`Some(false)`), or not at all.
pub fn aim(at: Option<bool>, range: u16) {
    STATE.with_borrow_mut(|state| {
        state.aiming = at;
        state.aim_range = f32::from(range);
        state.reach = None;
    });
    aim_shown();
}

/// Whether the skill being aimed needs Shift held to go on a monster (a skill of help, with the client's No
/// Shift option off).
pub fn aim_guarded(guarded: bool) {
    STATE.with_borrow_mut(|state| state.aim_guarded = guarded);
}

/// What is written by the pointer while a skill is aimed: its name and the level it will be used at.
pub fn aim_words(words: &str) {
    STATE.with_borrow_mut(|state| state.aim_words = words.to_owned());
    aim_shown();
}

/// Someone uses a skill that strikes nobody (the server's ZC_USE_SKILL): a character does what the client's
/// table gives the skill as a state (`crate::pose::skilled`, the client's `SetState`): a motion played once, or
/// frames of an action it runs through by itself (a dance, a song, a shot readied); one who sits stays seated.
/// A monster goes through its attack once (`CGameActor::ActionBase` gives 16 for every state but standing,
/// walking, being struck and dying).
pub fn uses_skill(who: u32, own: u32, body: u8) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        let Some(mover) = scene.one_mut(who, own) else { return };
        let time = now();
        let figure = match &mover.look {
            Look::Character(figure) => figure,
            Look::Creature(path) => {
                if mover.lively && mover.dying.is_none() {
                    let lasts = figure::timing(path, 16).map_or(600.0, |(pictures, each)| pictures as f64 * each);
                    mover.motion = Some((16, time, lasts.max(250.0)));
                }
                return;
            }
        };
        if mover.sitting || mover.dead {
            return;
        }
        match crate::pose::skilled(body, mover.job, attack_action(figure)) {
            crate::pose::Posed::Plays(action) => {
                figure::prepare(figure, action);
                let lasts = figure::body_timing(figure.job, figure.sex, action).map_or(600.0, |(pictures, each)| pictures as f64 * each);
                (mover.motion, mover.pose.sequence) = (Some((action, time, lasts.max(250.0))), None);
            }
            crate::pose::Posed::Runs(sequence) => {
                figure::prepare(figure, sequence.action);
                (mover.motion, mover.pose.sequence) = (None, Some((sequence, time)));
            }
        }
    });
}

/// Whether the player's own character is in the motion of a blow it deals. The original client asks for no walk
/// during it (rAthena's `attack_walk_delay` is off for players because, as its comment says, "the client already
/// blocks sending movement requests during this time"): one cannot strike and walk on at once.
pub fn striking() -> bool {
    STATE.with_borrow(|state| now() < state.own_striking_until)
}

/// Whether the player's own character is walking now.
pub fn walking() -> bool {
    STATE.with_borrow(|state| state.scene.as_ref().is_some_and(|scene| scene.player.walk.is_some()))
}

/// Whether a skill is being aimed.
pub fn aiming() -> bool {
    STATE.with_borrow(|state| state.aiming.is_some())
}
