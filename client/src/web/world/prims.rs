//! The effects the client draws by code, out of its primitives, under way in the world.
//!
//! In the client such an effect (`CRagEffect`) is a function run every tick of its life, which launches
//! primitives (`CEffectPrim`) and sets their figures; each tick a primitive moves by its speeds and is drawn by
//! its kind. The primitives themselves are in `crate::prim`; the effects made of them are in the files of
//! `prims/`, each named for what its effects are and listed once, in `files!` below: a file answers for its
//! effects through its `FILE`, and nothing else names a file. This file runs an effect on someone and hands
//! what it shows to the frame.
//!
//! An effect launched for a blow is told more than whom it is on: a place, mostly that of the one struck (the
//! client's `AM_SETPOS`) and of how many hits the blow is (`AM_SETCOUNT`); `Blow` (`world/launch.rs`) is that, and `Course`
//! what the effect keeps of it and remembers from tick to tick.
//!
//! The client measures the flat kinds in pixels of its screen. Here a pixel of them is a pixel of a sprite, as
//! for the effects played from files: they keep their size beside the characters when the camera moves. A
//! sprite of an effect the client draws as large as the sprites of those standing at that depth
//! (`RenderSegmentSprites2`, `CRenderer::ScaleDistance`), and so it is here; it draws the layers of the
//! sprite's frame one by one, each in the primitive's colour alone, and so does the frame here
//! (`crate::sprite::layer_quads`, `gpu/layers.rs`).

use super::*;
use crate::prim::{Master, Prim, Shape, Slot, cosine, faced, heading, pointing, sine};

/// The files of effects, each a module with a `FILE`: the one list of them. A new file is a name here.
macro_rules! files {
    ($($file:ident),*) => {
        $(mod $file;)*
        const FILES: &[&File] = &[$(&$file::FILE),*];
    };
}
files!(auras, blasts, blows, bodies, bolts, bursts, falls, grounds, gusts, heals, held, hits, links, marks, missiles, motes, pillars, places, played, prints, puffs, rays, shields, songs, souls, sounds, sprays, sprites, struck, trails, walls, waves, weather, winds);
mod launched;

/// What a file of effects answers for the effects it has.
struct File {
    /// How many ticks an effect lives (the client's `EffectLifetime`, or what its function makes of it) and
    /// the sound it starts with (none: empty); nothing for an effect that is not the file's. These functions
    /// are named `known` in every file: `tools/client-tables/skill_effects.py` reads the numbers they name to
    /// tell which effects are drawn here.
    known: fn(u32) -> Option<(i32, &'static str)>,
    /// Whether an effect is one that is told a place: one the client has the striker launch and then tells
    /// the place of the one struck (`CSession::PlaySkillEffect`: `source->LaunchEffect` and `AM_SETPOS`).
    thrown: fn(u32) -> bool,
    /// Whether an effect stays where it began: one whose function does not take its parent's place every
    /// tick. (Asked only of the effects for which it matters here.)
    stays: fn(u32) -> bool,
    /// One tick of an effect, as its course says: it launches what it launches at that tick.
    run: fn(u32, &mut Course, &mut Vec<Prim>),
    /// The pictures its effects are drawn with, among the client's pictures of effects, and the sprites they
    /// show, among the client's sprites: lists of the file's.
    pictures: &'static [&'static [&'static str]],
    sprites: &'static [&'static [&'static str]],
}

/// What a file answers whose effects are none of them thrown, or none of them stay.
fn none(_effect: u32) -> bool {
    false
}

/// Whether an effect that is told a place is one the client launches on the one struck and tells the place of
/// the striker (`target->LaunchEffect` and `AM_SETPOS` with `source->render.position`, in its handlers of
/// skills): `EF_BLOODDRAIN`, `EF_HITLINE5`, `EF_DARKPIERCING`, `EF_BLOODDRAIN2`. (Asked only of a blow's
/// effects: one of a skill that strikes nobody is told the other of the two whichever it is on.)
fn aims_back(effect: u32) -> bool {
    matches!(effect, 216 | 430 | 1030 | 1052)
}

/// The file an effect is of, how many ticks it lives and the sound it starts with; nothing for an effect not
/// ported yet.
fn file_of(effect: u32) -> Option<(&'static File, i32, &'static str)> {
    FILES.iter().find_map(|file| (file.known)(effect).map(|(lasts, sound)| (*file, lasts, sound)))
}

/// How many ticks an effect lives (the client's `EffectLifetime`); none for one not ported.
pub(super) fn lasts(effect: u32) -> i32 {
    file_of(effect).map_or(0, |(_, lasts, _)| lasts)
}

/// The pictures the effects are drawn with and the sprites they show: what the world reads ahead of their use
/// (`world/loading.rs`).
pub(super) fn pictures() -> impl Iterator<Item = &'static str> {
    FILES.iter().flat_map(|file| file.pictures.iter().flat_map(|list| list.iter().copied()))
}

pub(super) fn sprites() -> impl Iterator<Item = &'static str> {
    FILES.iter().flat_map(|file| file.sprites.iter().flat_map(|list| list.iter().copied()))
}

/// What an effect shows, a piece at a time.
pub(super) enum Piece {
    /// Two triangles of one of the client's pictures of effects: the picture, each of the six corners as its
    /// place in the world and where of the picture it shows, the colour of each corner, and whether its light
    /// is added to what is behind it.
    Picture(&'static str, [(Vector, [f32; 2]); 6], [[f32; 4]; 6], bool),
    /// A frame of the first action of a sprite, facing the screen: the sprite's path among the client's
    /// sprites, the frame, where in the world the sprite's origin is, how many times its own size it is, how
    /// far it is turned (the client's degrees), the colour every layer of the frame is drawn in, and whether
    /// its light is added to what is behind it.
    /// `action` is which action of the sprite the frame is of, and `own` whether every layer is drawn in the
    /// colour its file gives it instead of `colour`; `lit`, whether every layer is in the red, green and blue
    /// its file gives it at the alpha of `colour` (`RenderSegmentSpriteGlow`). With a `rim` every layer is in
    /// the colour and of the size that makes of the layer's own (`RenderSegmentSprites`, `crate::prim::Rim`).
    Sprite { sprite: &'static str, action: usize, frame: usize, at: Vector, scale: f32, roll: f32, colour: [f32; 4], own: bool, lit: bool, adds: bool, rim: Option<crate::prim::Rim> },
}

/// A whole number from 0 up to, not reaching, `under` (the client's `rand() % under`), as the figures of a
/// primitive take it.
fn lot(under: u32) -> f32 {
    super::lot(under) as f32
}

/// What an effect remembers from tick to tick: the fields of the client's `CRagEffect` that its functions use.
pub(super) struct Course {
    /// The tick it is at (`stateCnt`) of those it lives (`duration`), and the way the one it is on faced as it
    /// began, in the client's degrees (0 south, 90 east).
    pub(super) tick: i32,
    pub(super) lasts: i32,
    pub(super) facing: f32,
    /// What its primitives share with it: the place it aims at (`deltaPos`) and whether one has landed (`flag`).
    pub(super) shared: Master,
    /// How many it was told (`count`): the hits of the blow.
    pub(super) count: i32,
    /// Two places it keeps for itself (`deltaPos3`, `deltaPos2`) and two turns (`longitude`, `latitude`), each
    /// used as the effect's function says.
    pub(super) reach: [f32; 3],
    pub(super) step: [f32; 3],
    pub(super) longitude: f32,
    pub(super) latitude: f32,
    /// How far the one it is on has gone since it began (east, up, north): what a primitive that stays in the
    /// world where it was launched is told (`Prim::left`).
    pub(super) walked: [f32; 3],
    /// The camera's yaw now, the client's degrees (`crate::prim::camera_yaw`): what an effect that picks a
    /// sprite's action by the way it points across the screen looks at as it launches it.
    pub(super) yaw: f32,
    /// The sounds it starts at this tick, among the client's sounds.
    pub(super) sounds: Vec<String>,
    /// What it does at this tick to the body of the one it is on (`crate::body`: `SetColor` on `state.parent`,
    /// its `render.flags` and `render.angle`): written again at every tick it does so.
    pub(super) touch: crate::body::Touch,
    /// The shake of the screen it asks for at this tick (`MM_QUAKE` with that number: `crate::quake`).
    pub(super) quake: Option<u32>,
    /// The effects it ends at this tick on the one it is on, by the client's numbers (`AM_DELETE_EFFECT`).
    pub(super) ends: Vec<u32>,
    /// Where the other one it was told of is now, from the one it is on (none told, or out of sight: nothing):
    /// what an effect that follows a second actor looks at every tick (the client's `state.target`).
    pub(super) other: Option<[f32; 3]>,
    /// The copies of the one it is on that it leaves standing at this tick (the client's `CHalluPC`): where
    /// each stands from where that one is now (east, up, north), and the colour it is in.
    pub(super) copies: Vec<([f32; 3], [u8; 3])>,
    /// How high over the feet of the one it is on that one's name stands (`CGameActor::GetNameOffset`), which
    /// the signs of Fury and its like are lifted by (`Prim::lifted`).
    pub(super) lift: f32,
    /// The four figures it was told (`param`, the client's `AM_SETPARAM`: those of an effect a map places),
    /// nothing unless told; and how high the ground is where the one it is on stands, by the world's own
    /// measure (`Burst::figured`, `Burst::grounded`).
    pub(super) figures: [f32; 4],
    pub(super) ground: f32,
}

impl Course {
    /// Starts a sound at this tick, among the client's sounds.
    pub(super) fn sound(&mut self, sound: &str) {
        self.sounds.push(sound.to_owned());
    }

    /// Starts those of an effect's sounds whose tick this is: `sounds` has each with the tick it starts at.
    pub(super) fn starts(&mut self, sounds: &[(i32, &str)]) {
        let tick = self.tick;
        self.sounds.extend(sounds.iter().filter(|(at, _)| *at == tick).map(|(_, sound)| (*sound).to_owned()));
    }
}

/// What effects under way ask of the world around them in a frame: what each does to the body of the one it is
/// on (and whether it lived a tick since it was last asked), the shakes of the screen, the effects to end
/// on someone, and the effects to begin on someone at a place so far east, up and north of that one
/// (`launches`: what a primitive asks for where it is, `Prim::asks`).
#[derive(Default)]
pub(super) struct Asked {
    pub(super) touches: Vec<(Option<u32>, crate::body::Touch, bool)>,
    pub(super) quakes: Vec<u32>,
    pub(super) ends: Vec<(Option<u32>, u32)>,
    pub(super) launches: Vec<(Option<u32>, u32, Vector)>,
    /// The copies to leave standing: of whom, where from that one's place, and in which colour.
    pub(super) copies: Vec<(Option<u32>, [f32; 3], [u8; 3])>,
}

/// An effect of the client's under way on someone.
pub(super) struct Burst {
    /// On whom: no one named is the player's own character.
    pub(super) on: Option<u32>,
    /// When it begins, and how many of its ticks have been lived through.
    pub(super) begins: f64,
    ticks: i32,
    /// Which effect it is, by the client's number, the file it is of, and how many ticks it lives (the
    /// client's `EffectLifetime`).
    effect: u32,
    file: &'static File,
    lasts: i32,
    prims: Vec<Prim>,
    course: Course,
    /// Whether it has been told, before its first tick, how the one it is on faces and where it aims.
    begun: bool,
    /// Whose place it aims at, when it was told one.
    aims_at: Option<Option<u32>>,
    /// Where the one it is on stood as it began.
    began_at: Option<Vector>,
    /// What its last tick did to the body of the one it is on, and whether that has been handed on yet.
    touch: crate::body::Touch,
    fresh: bool,
    /// The place it was told to aim at as it was begun, from the one it is on (`Burst::tell`).
    told: Option<Vector>,
}

/// The effect of that number of the client's, to begin at `begins` on someone, and the sound it starts with (none:
/// empty); nothing for an effect not ported yet. One launched for a blow is told of the blow: if it is one the
/// client has the striker launch and tells a place (its file's `thrown`), it is on the striker, whoever `on` is,
/// and aims at the place it is told. (The client lets it be told of 50 hits at most.)
pub(super) fn burst(effect: u32, on: Option<u32>, blow: Option<Blow>, begins: f64) -> Option<(Burst, &'static str)> {
    let (file, lasts, sound) = file_of(effect)?;
    let course = Course { tick: 0, lasts, facing: 0.0, shared: Master::left(lasts), count: blow.map_or(0, |blow| blow.hits.min(50) as i32), reach: [0.0; 3], step: [0.0; 3], longitude: 0.0, latitude: 0.0, walked: [0.0; 3], yaw: 0.0, sounds: Vec::new(), touch: Default::default(), quake: None, ends: Vec::new(), other: None, copies: Vec::new(), lift: 0.0, figures: [0.0; 4], ground: 0.0 };
    let thrown = blow.filter(|_| (file.thrown)(effect));
    // One the client launches on the one struck and tells where the striker stands stays where it is and
    // aims back.
    let (on, aims_at) = match thrown.filter(|blow| aims_back(effect) && blow.striker != on) {
        Some(blow) => (on, Some(blow.striker)),
        None => (thrown.map_or(on, |blow| blow.striker), thrown.map(|blow| blow.place_of)),
    };
    Some((Burst { on, begins, ticks: 0, effect, file, lasts, prims: Vec::new(), course, begun: false, aims_at, began_at: None, touch: Default::default(), fresh: false, told: None }, sound))
}

/// The effect of that number begins at `begins` on someone, among the effects under way (`bursts`), and the
/// sound it starts with is due then (`sounds`): the one way an effect drawn by code is launched, whoever
/// launches it. Gives back the effect, to be told more (`Burst::tell`, `Burst::stood`); nothing for an effect
/// not ported yet.
pub(super) fn begin<'a>(effect: u32, on: Option<u32>, blow: Option<Blow>, begins: f64, bursts: &'a mut Vec<Burst>, sounds: &mut Vec<(f64, String)>) -> Option<&'a mut Burst> {
    let (burst, sound) = burst(effect, on, blow, begins)?;
    if !sound.is_empty() {
        sounds.push((begins, sound.to_owned()));
    }
    bursts.push(burst);
    bursts.last_mut()
}

impl Burst {
    /// Lives up to `time`, a tick of the client's at a time. `ground` tells how high the ground is at a place so
    /// far east and north of the one it is on, from where that one stands. Gives back whether anything of it
    /// is left; the sounds it starts on the way are put into `sounds`.
    /// What it asks of the world on the way (its parent's body, a shake, effects ended) is put into `asked`.
    pub(super) fn live(&mut self, time: f64, ground: &dyn Fn(f32, f32) -> f32, sounds: &mut Vec<String>, asked: &mut Asked) -> bool {
        let due = ((time - self.begins) / CAST_FRAME_MS) as i32;
        while self.ticks < due && self.ticks < self.lasts {
            // The effect's own tick first: it may launch primitives, which live from this tick on.
            self.course.tick = self.ticks;
            self.course.shared.left = self.lasts - self.ticks;
            let turned = if self.fresh { self.touch.turn } else { 0.0 };
            self.course.touch = Default::default();
            (self.file.run)(self.effect, &mut self.course, &mut self.prims);
            sounds.append(&mut self.course.sounds);
            // What this tick does to the body; of ticks lived at once the turns add up.
            (self.touch, self.fresh) = (self.course.touch, true);
            self.touch.turn += turned;
            asked.quakes.extend(self.course.quake.take());
            asked.ends.extend(self.course.ends.drain(..).map(|effect| (self.on, effect)));
            asked.copies.extend(self.course.copies.drain(..).map(|(at, colour)| (self.on, at, colour)));
            // A sprite's frames are counted once the sprite has been read; until then it stays on its first.
            let (shared, other, lift) = (&mut self.course.shared, self.course.other, self.course.lift);
            self.prims.retain_mut(|prim| {
                // A tube from the one the effect is on to another actor follows that one.
                prim.link_to(other);
                prim.lifted(lift);
                let frames = if prim.shows_sprite() { gpu::frames_of(prim.picture, prim.action).unwrap_or(0) } else { 0 };
                prim.tick_on(shared, frames, ground)
            });
            // What its primitives ask for where they are (`RenderSMA3`, `RenderLaunchEffects`: the client
            // launches one at every frame it draws such a primitive; here one a tick), from where the one it
            // is on stands now.
            for prim in &self.prims {
                let from = prim.left.map_or([0.0; 3], |left| subtract(left, self.course.walked));
                asked.launches.extend(prim.asks().into_iter().map(|(effect, at)| (self.on, effect, along(from, at, 1.0))));
            }
            self.ticks += 1;
        }
        // Its flags hold between its ticks; its colour and its turn are those of a tick just lived.
        if self.touch != Default::default() {
            asked.touches.push((self.on, self.touch, std::mem::take(&mut self.fresh)));
        }
        self.ticks < self.lasts
    }

    /// Whether it is that effect of the client's on that one: what ends a running effect by its number
    /// (`AM_DELETE_EFFECT`).
    pub(super) fn is(&self, effect: u32, on: Option<u32>) -> bool {
        self.effect == effect && self.on == on
    }

    /// Tells it where the other one it was told of is now, from the one it is on.
    pub(super) fn other_at(&mut self, other: Option<Vector>) {
        self.course.other = other;
    }

    /// Whose place it aims at, when it was told one.
    pub(super) fn aims_at(&self) -> Option<Option<u32>> {
        self.aims_at
    }

    /// Tells it how high over the feet the name of the one it is on stands.
    pub(super) fn named_at(&mut self, lift: f32) {
        self.course.lift = lift;
    }

    /// Tells it the four figures an effect placed by a map is told (`AM_SETPARAM`).
    pub(super) fn figured(&mut self, figures: [f32; 4]) {
        self.course.figures = figures;
    }

    /// Tells it how high the ground is where the one it is on stands now.
    pub(super) fn grounded(&mut self, height: f32) {
        self.course.ground = height;
    }

    /// Tells one just begun the place it aims at, from the one it is on: what the client tells an effect that
    /// a primitive launches (`AM_SETPOS`). It keeps that place whatever it is told at its first tick.
    pub(super) fn tell(&mut self, place: Vector) {
        self.told = Some(place);
    }

    /// Tells it where the one it is on stands as it is launched: an effect that goes by how far that one has
    /// gone since (`Course::walked`) counts from here, not from where that one is at its first tick.
    pub(super) fn stood(&mut self, place: Vector) {
        self.began_at.get_or_insert(place);
    }

    /// Tells it, before its first tick, which way the one it is on faces (a step of one on the ground) and
    /// where the place it aims at is from that one's (none: its own place): the client's effect looks at its
    /// parent's angle as it starts, and is told the place as it is launched.
    pub(super) fn begin(&mut self, facing: (f32, f32), aim: Option<Vector>) {
        if !self.begun {
            self.begun = true;
            self.course.facing = facing.0.atan2(-facing.1).to_degrees();
            self.course.shared.aim = self.told.or(aim).unwrap_or_default();
        }
    }

    /// Tells it the camera's yaw, the client's degrees, before it lives on.
    pub(super) fn seen_from(&mut self, yaw: f32) {
        self.course.yaw = yaw;
    }

    /// Where it is drawn, the one it is on being at `place` now: there, or for an effect the client leaves
    /// where it began (one that does not take its parent's place every tick), where that one was at first.
    /// It is told here how far it has gone since it began.
    pub(super) fn place(&mut self, place: Vector) -> Vector {
        let began_at = *self.began_at.get_or_insert(place);
        let place = if (self.file.stays)(self.effect) { began_at } else { place };
        self.course.walked = subtract(place, began_at);
        place
    }

    /// What it shows now, a piece at a time: around `place`, the feet of the one it is on; the flat kinds across
    /// the screen along `right` and `up`. A sprite is in the primitive's colour and at the alpha of each of
    /// its drawings (the client's `record.color`). `yaw` is the camera's yaw, the client's degrees: a sprite
    /// that points a way in the world is turned on the screen by that way less the yaw (`Prim::rolled`). A
    /// primitive that stays in the world where it was launched is drawn around where the effect was then.
    pub(super) fn draw(&self, place: Vector, (right, up, yaw): (Vector, Vector, f32), mut draw: impl FnMut(Piece)) {
        for prim in &self.prims {
            let colour = |alpha: f32| [prim.colour[0] / 255.0, prim.colour[1] / 255.0, prim.colour[2] / 255.0, alpha.trunc() / 255.0];
            let place = prim.left.map_or(place, |left| std::array::from_fn(|axis| place[axis] - self.course.walked[axis] + left[axis]));
            // (The kind 167 is drawn at each of its parts, every layer twice in colours made of its own.)
            for (at, scale, rim) in prim.fourfold() {
                draw(Piece::Sprite { sprite: prim.picture, action: prim.action, frame: prim.motion, at: along(place, at, 1.0), scale, roll: prim.rolled(yaw), colour: [1.0; 4], own: true, lit: false, adds: prim.adds, rim: Some(rim) });
            }
            for seen in prim.seen().into_iter().filter(|seen| prim.shows_sprite() && !prim.is_fourfold() && seen.alpha >= 1.0) {
                let at = [place[0] + seen.at[0], place[1] + seen.at[1], place[2] + seen.at[2]];
                draw(Piece::Sprite { sprite: prim.picture, action: prim.action, frame: prim.motion, at, scale: seen.scale, roll: prim.rolled(yaw), colour: colour(seen.alpha), own: prim.own_colours, lit: prim.glows(), adds: prim.adds, rim: None });
                // (The kind 153 is drawn once more, its light added: `RenderSegmentSpriteGlow`.)
                if prim.glows() {
                    draw(Piece::Sprite { sprite: prim.picture, action: prim.action, frame: prim.motion, at, scale: seen.scale, roll: prim.rolled(yaw), colour: colour(seen.alpha), own: false, lit: true, adds: true, rim: None });
                }
            }
            for lit in prim.lit(right, up).into_iter().filter(|lit| lit.alpha >= 1.0) {
                let corners = lit.corners.map(|(at, uv)| ([place[0] + at[0], place[1] + at[1], place[2] + at[2]], uv));
                draw(Piece::Picture(lit.picture, corners, [colour(lit.alpha); 6], prim.adds));
            }
            for rect in prim.rects(right, up).into_iter().filter(|rect| rect.alpha >= 1.0) {
                let corners = rect.corners.map(|(at, uv)| ([place[0] + at[0], place[1] + at[1], place[2] + at[2]], uv));
                draw(Piece::Picture(rect.picture, corners, [[f32::from(rect.colour[0]) / 255.0, f32::from(rect.colour[1]) / 255.0, f32::from(rect.colour[2]) / 255.0, rect.alpha / 255.0]; 6], rect.adds));
            }
            // (A kite has a corner that shows whatever its alpha is.)
            if prim.alpha < 1.0 && !prim.glares(1) {
                continue;
            }
            for pair in prim.solid(place).chunks_exact(6) {
                draw(Piece::Picture(prim.shown(), std::array::from_fn(|index| pair[index]), [colour(prim.alpha); 6], prim.adds));
            }
            for pair in prim.corners().chunks_exact(6) {
                let corner = |index: usize| (along(along(place, right, pair[index][0] * SPRITE_SCALE), up, -pair[index][1] * SPRITE_SCALE), [pair[index][2], pair[index][3]]);
                let colours = std::array::from_fn(|index| if prim.glares(index) { [1.0; 4] } else { colour(prim.alpha) });
                draw(Piece::Picture(prim.picture, std::array::from_fn(corner), colours, prim.adds));
            }
        }
    }
}
