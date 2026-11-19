//! Body states and health states as they show in the world (`crate::ailments`, the client's
//! `CGameActor::SetStatusStates`): the colour of the one ailing, the marks by them, the ice that breaks, the
//! sounds, the motion held still, and the two veils over the screen of the player's own character.
//!
//! Everyone's body as the effects on it leave it is kept here too (`crate::body`, `bodies`), with what a body
//! leaves behind: the footprints of one who runs, and the copies of a character that an effect leaves standing.
//!
//! The sounds are played at their full loudness; the client makes them fainter with the distance to the
//! player's own character (`PlayWave` with the place), which is not done here.

use super::*;
use crate::ailments::Ailing;

/// The pictures of the veils, among the client's pictures of effects.
pub(super) const PICTURES: [&str; 3] = ["poison_f.bmp", "fullb.tga", "white02.bmp"];

/// What someone's body state and health state have made of them.
#[derive(Clone)]
pub(super) struct Ail {
    states: Ailing,
    /// Whether its motion is held where it was.
    still: bool,
    /// When the states were last told: the marks' pictures run from then.
    since: f64,
    /// The ice that broke off them: when, and where they stood (in cells).
    broke: Option<(f64, (f32, f32))>,
    /// Its body as its states and the effects on it have left it (`crate::body`): what its sprite is drawn by,
    /// and where its colour is kept, the one its states gave it among that.
    pub(super) body: crate::body::Body,
    /// Up to when the passes of its body have been counted.
    passed: f64,
}

impl Default for Ail {
    fn default() -> Self {
        Self { states: Ailing::default(), still: false, since: 0.0, broke: None, body: crate::body::Body::default(), passed: 0.0 }
    }
}

/// The veils over the screen of the player's own character, each since when: poison's and blindness's
/// (`EF_POISON`, `EF_BLIND`; each lasts until its state goes).
#[derive(Default)]
pub(super) struct Veils {
    poison: Option<f64>,
    blind: Option<f64>,
}

impl Ail {
    /// Is told the states it is in now, at `time`, standing at `at`; `news` and `own` as `crate::ailments::change`
    /// has them. The sounds of the change are played. Gives back the effects to begin and to end.
    fn told(&mut self, now: Ailing, news: bool, own: bool, at: Option<(f32, f32)>, time: f64) -> (Vec<u32>, Vec<u32>) {
        let change = crate::ailments::change(self.states, self.body.state_colour(), self.still, now, news, own);
        if self.states != now {
            self.since = time;
        }
        (self.states, self.still) = (now, change.still);
        // The state's colour is written to the body, and locked there if it is one (`SetStatusStates`).
        self.body.stated(change.colour);
        if change.ice_breaks {
            self.broke = at.map(|at| (time, at));
        }
        for sound in change.sounds {
            sound::effect(sound, 1.0);
        }
        (change.begins, change.ends)
    }

    /// Whether its health state is not 0: a character that ails stands ready (`CPc::SetState(ST_STAND)`).
    pub(super) fn unwell(&self) -> bool {
        self.states.health != 0
    }

    /// Whether its motion is held where it is: its picture stays the one drawn last.
    pub(super) fn still(&self) -> bool {
        self.still
    }
}

/// The server tells someone's body state and health state (ZC_STATE_CHANGE; none named: the player's own
/// character): the client's `AM_ATTR_STATE_CHANGE`, with the word that it is news.
pub fn ailing(id: Option<u32>, body: u16, health: u16) {
    STATE.with_borrow_mut(|state| {
        let (time, now) = (now(), Ailing { body, health });
        match id {
            None => {
                let at = state.scene.as_ref().map(|scene| scene.player.at);
                let (begins, ends) = state.own_ail.told(now, true, true, at, time);
                // An effect is begun only if it is not there already (the client's `HasId`).
                for (effect, veil) in [(crate::ailments::EF_POISON, &mut state.veils.poison), (crate::ailments::EF_BLIND, &mut state.veils.blind)] {
                    if ends.contains(&effect) {
                        *veil = None;
                    } else if begins.contains(&effect) && veil.is_none() {
                        *veil = Some(time);
                    }
                }
                if let Some(scene) = &mut state.scene {
                    // (Its body stays the one the effects on it have touched; the state's colour is written to it.)
                    let mut body = scene.player.ail.body;
                    body.stated(state.own_ail.body.state_colour());
                    scene.player.ail = state.own_ail.clone();
                    scene.player.ail.body = body;
                }
            }
            Some(id) => {
                if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) {
                    let at = unit.at;
                    unit.ail.told(now, true, false, Some(at), time);
                }
            }
        }
    });
}

/// Someone comes into sight in those states (the packets that show a unit: the client's `SetStatusStates`
/// without the word that it is news).
pub(super) fn seen(mover: &mut Mover, body: u16, health: u16) {
    let at = mover.at;
    mover.ail.told(Ailing { body, health }, false, false, Some(at), now());
}

/// The bodies of everyone at `time` (`crate::body`): the passes gone by since the last frame fade a colour no
/// state holds, the flags of the pass before are put back to nothing, and the effects that run now do to the
/// one each is on what they do at this tick (`touches`: on whom, what, and whether the effect lived a tick
/// since it was last asked). One an effect has turned faces the new way. A character that an effect has
/// running leaves its footprints (`footprints`). (A pass of an actor is taken as a tick of an effect; whether
/// the client counts them alike was not found.)
pub(super) fn bodies(state: &mut State, touches: &[(Option<u32>, crate::body::Touch, bool)], time: f64) {
    let Some(scene) = &mut state.scene else { return };
    let mut prints = Vec::new();
    for mover in std::iter::once(&mut scene.player).chain(scene.units.values_mut()) {
        let ticks = ((time - mover.ail.passed) / CAST_FRAME_MS).max(0.0) as u32;
        mover.ail.passed = if ticks > 255 { time } else { mover.ail.passed + f64::from(ticks) * CAST_FRAME_MS };
        mover.ail.body.pass(ticks);
        if mover.ail.body.runs() && matches!(mover.look, Look::Character(_)) {
            prints.extend((0..ticks.min(14)).filter_map(|_| mover.ail.body.stepped([mover.at.0, mover.at.1])).map(|print| (mover.at, print)));
        }
        mover.ail.body.clear();
    }
    for (on, touch, fresh) in touches {
        let Some(mover) = (match on {
            Some(id) => scene.units.get_mut(id),
            None => Some(&mut scene.player),
        }) else {
            continue;
        };
        mover.ail.body.touch(touch, *fresh);
        // (`map::direction` numbers the ways as the body counts its eighths: from the south round by the west.)
        mover.facing = (i32::from(mover.facing) + mover.ail.body.turned()).rem_euclid(8) as u8;
        // A colour for a character that is poisoned, which is locked as a state's is (`EF_BLACKBODY`).
        if let Some(colour) = touch.if_poisoned.filter(|_| mover.ail.states.health & 1 != 0 && matches!(mover.look, Look::Character(_))) {
            mover.ail.body.locked(colour);
        }
    }
    footprints(state, &prints, time);
}

/// The footprints of those who run (`CPc::UpdateFootprints`): each is an effect of its own launched on the
/// place the runner is at (`LaunchDummyEffect`, for 333 ticks), `EF_FOOT3` for a left foot and `EF_FOOT4` for a
/// right one, told how it is turned.
fn footprints(state: &mut State, prints: &[((f32, f32), crate::body::Print)], time: f64) {
    for (at, print) in prints {
        let spot = skill_units::spot(state, *at, 333, time);
        if let Some(footprint) = prims::begin(if print.left { 447 } else { 448 }, Some(spot), None, time, &mut state.bursts, &mut state.sounds_due) {
            footprint.tell([print.turned as f32, 0.0, 0.0]);
        }
    }
}

/// A copy of someone that an effect left standing (`crate::body::copies`, the client's `CHalluPC`): the
/// picture that one showed then, where it stands, its colour, and since when.
pub(super) struct Ghost {
    key: SpriteKey,
    at: Vector,
    colour: [u8; 3],
    since: f64,
}

/// The copies that effects leave standing at this frame (`prims::Asked::copies`: of whom, where from that
/// one's place, in which colour). Only a character is copied.
pub(super) fn copies_left(state: &mut State, copies: &[(Option<u32>, Vector, [u8; 3])], time: f64) {
    let Some(scene) = &state.scene else { return };
    for (of, from, colour) in copies {
        let Some(mover) = (match of {
            Some(id) => scene.units.get(id),
            None => Some(&scene.player),
        }) else {
            continue;
        };
        if let (Look::Character(_), Some(key)) = (&mover.look, &mover.shown) {
            state.copies.push(Ghost { key: key.clone(), at: along(on_ground(&scene.gat, mover.at), *from, 1.0), colour: *colour, since: time });
        }
    }
}

/// The copies standing now, added to the pictures of this frame; those that have faded out are forgotten.
pub(super) fn copies(copies: &mut Vec<Ghost>, gpu: &mut Gpu, time: f64, pictures: &mut Vec<Stood>) {
    let left = |copy: &Ghost| crate::body::copies::alpha(((time - copy.since) / CAST_FRAME_MS) as i32);
    copies.retain(|copy| left(copy).is_some());
    for copy in copies.iter().filter(|copy| gpu.has_sprite(&copy.key)) {
        let [red, green, blue] = copy.colour;
        pictures.push((copy.key.clone(), copy.at, 1.0, crate::body::Quad { colour: [red, green, blue, left(copy).unwrap_or(0)], ..crate::body::Quad::PLAIN }));
    }
}

/// The picture of a sprite's action that shows after so long, each for as long as the sprite says, over and
/// over; the first until the sprite has been read.
fn turning(sprite: &str, action: usize, passed: f64) -> usize {
    figure::timing(sprite, action).map_or(0, |(pictures, each)| (passed / each.max(1.0)) as usize % pictures.max(1))
}

/// The marks by someone standing at `feet` and the ice breaking off them, added to the pictures of this frame.
/// `turned` is which of the eight ways the one marked is seen from, as its own sprite is.
pub(super) fn marks(gpu: &mut Gpu, gat: &Gat, mover: &mut Mover, feet: Vector, turned: usize, time: f64, pictures: &mut Vec<Stood>) {
    for mark in crate::ailments::marks(mover.ail.states) {
        let (sprite, first) = mark.sprite();
        // The curse turns with the one cursed (`CMsgEffect::FollowOwner`: its action is the first and the way
        // its owner faces; taken here as the way the owner's own sprite is seen from).
        let action = first + if mark.faces() { turned % 8 } else { 0 };
        let key = SpriteKey::Creature(sprite.to_owned(), action, turning(sprite, action, time - mover.ail.since));
        if gpu.has_sprite(&key) {
            pictures.push((key, along(feet, [0.0, 1.0, 0.0], mark.above()), mark.alpha(), crate::body::Quad::PLAIN));
        }
    }
    // The ice that broke: its pictures once through, where the one frozen stood. Until the sprite is read its
    // length is not known: it is given four seconds to come.
    if let Some((broke, at)) = mover.ail.broke {
        if !played_once(gpu, crate::ailments::ICE_BREAKS, (broke, on_ground(gat, at)), time, pictures) {
            mover.ail.broke = None;
        }
    }
}

/// An action of a sprite played once through at a place since a moment, 1.2 pictures to each of the client's
/// frames (`CMsgEffect::OnProcess` for the ice that breaks and the puff of smoke alike), added to the
/// pictures of this frame. Gives back whether it is still going. Until the sprite is read its length is not
/// known: it is given four seconds to come.
pub(super) fn played_once(gpu: &mut Gpu, (sprite, action): (&str, usize), (began, at): (f64, Vector), time: f64, pictures: &mut Vec<Stood>) -> bool {
    let picture = ((time - began) / EFFECT_FRAME_MS * crate::ailments::ICE_BREAK_PACE) as usize;
    match figure::frames_of(sprite, action) {
        Some(frames) if picture >= frames => false,
        None if time - began > 4000.0 => false,
        known => {
            let key = SpriteKey::Creature(sprite.to_owned(), action, if known.is_some() { picture } else { 0 });
            if gpu.has_sprite(&key) && known.is_some() {
                pictures.push((key, at, 1.0, crate::body::Quad::PLAIN));
            }
            true
        }
    }
}

/// The veils over the screen of the player's own character: each pane a quad facing the screen around a place
/// 9 above its feet. The client's primitives for them carry the render flag 0x200, which the numbers of blows
/// carry too and which is taken here to mean over everything: they are drawn just before the eye, so that
/// nothing of the world cuts through them. (That reading of the flag was not verified.)
pub(super) fn veils(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    let middle = along(on_ground(&scene.gat, scene.player.at), [0.0, 1.0, 0.0], 9.0);
    let nearer = NEAR * 1.5 / dot(subtract(middle, view.eye), view.forward).max(NEAR * 1.5);
    let ticks = |since: f64| ((time - since) / CAST_FRAME_MS) as i32;
    let poison = state.veils.poison.map(|since| (crate::ailments::poison_veil(), [255.0; 3], crate::ailments::poison_alpha(ticks(since))));
    let blind = state.veils.blind.map(|since| (crate::ailments::blind_veil(), crate::ailments::BLIND_COLOUR, crate::ailments::blind_alpha(ticks(since))));
    for (panes, colour, alpha) in poison.into_iter().chain(blind) {
        let colour = [colour[0] / 255.0, colour[1] / 255.0, colour[2] / 255.0, alpha / 255.0];
        for pane in panes {
            let Some(binds) = gpu.effect_pictures.get(pane.picture) else { continue };
            // `RenderTeiRect`'s two triangles: the first three corners, then the fourth, the third and the first.
            for index in [0, 1, 2, 3, 2, 0] {
                let ([across, down], uv) = pane.corners[index];
                quads.corner(view, nearer, along(along(middle, view.right, across), view.up, -down), uv, colour);
            }
            quads.picture(binds, false);
        }
    }
}
