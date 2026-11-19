//! An effect drawn by code under way on someone (`Burst`): how one is begun, what it is told before and while
//! it lives, and how it lives a tick of the client's at a time. What it shows is `drawn.rs`.

use super::*;

/// An effect of the client's under way on someone.
pub(in crate::web::world) struct Burst {
    /// On whom: no one named is the player's own character.
    pub(in crate::web::world) on: Option<u32>,
    /// When it begins, and how many of its ticks have been lived through.
    pub(in crate::web::world) begins: f64,
    ticks: i32,
    /// Which effect it is, by the client's number, the file it is of, and how many ticks it lives (the
    /// client's `EffectLifetime`).
    effect: u32,
    file: &'static File,
    lasts: i32,
    pub(super) prims: Vec<Prim>,
    pub(super) course: Course,
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
pub(in crate::web::world) fn burst(effect: u32, on: Option<u32>, blow: Option<Blow>, begins: f64) -> Option<(Burst, &'static str)> {
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
pub(in crate::web::world) fn begin<'a>(effect: u32, on: Option<u32>, blow: Option<Blow>, begins: f64, bursts: &'a mut Vec<Burst>, sounds: &mut Vec<(f64, String)>) -> Option<&'a mut Burst> {
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
    pub(in crate::web::world) fn live(&mut self, time: f64, ground: &dyn Fn(f32, f32) -> f32, sounds: &mut Vec<String>, asked: &mut Asked) -> bool {
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
            asked.copies.extend(self.course.copies.drain(..).map(|(at, colour, kind)| (self.on, at, colour, kind)));
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
    pub(in crate::web::world) fn is(&self, effect: u32, on: Option<u32>) -> bool {
        self.effect == effect && self.on == on
    }

    /// Tells it where the other one it was told of is now, from the one it is on.
    pub(in crate::web::world) fn other_at(&mut self, other: Option<Vector>) {
        self.course.other = other;
    }

    /// Whose place it aims at, when it was told one.
    pub(in crate::web::world) fn aims_at(&self) -> Option<Option<u32>> {
        self.aims_at
    }

    /// Tells it how high over the feet the name of the one it is on stands.
    pub(in crate::web::world) fn named_at(&mut self, lift: f32) {
        self.course.lift = lift;
    }

    /// Tells it the four figures an effect placed by a map is told (`AM_SETPARAM`).
    pub(in crate::web::world) fn figured(&mut self, figures: [f32; 4]) {
        self.course.figures = figures;
    }

    /// Tells it how high the ground is where the one it is on stands now.
    pub(in crate::web::world) fn grounded(&mut self, height: f32) {
        self.course.ground = height;
    }

    /// Tells one just begun the place it aims at, from the one it is on: what the client tells an effect that
    /// a primitive launches (`AM_SETPOS`). It keeps that place whatever it is told at its first tick.
    pub(in crate::web::world) fn tell(&mut self, place: Vector) {
        self.told = Some(place);
    }

    /// Tells it where the one it is on stands as it is launched: an effect that goes by how far that one has
    /// gone since (`Course::walked`) counts from here, not from where that one is at its first tick.
    pub(in crate::web::world) fn stood(&mut self, place: Vector) {
        self.began_at.get_or_insert(place);
    }

    /// Tells it, before its first tick, which way the one it is on faces (a step of one on the ground) and
    /// where the place it aims at is from that one's (none: its own place): the client's effect looks at its
    /// parent's angle as it starts, and is told the place as it is launched.
    pub(in crate::web::world) fn begin(&mut self, facing: (f32, f32), aim: Option<Vector>) {
        if !self.begun {
            self.begun = true;
            self.course.facing = facing.0.atan2(-facing.1).to_degrees();
            self.course.shared.aim = self.told.or(aim).unwrap_or_default();
        }
    }

    /// Tells it the camera's yaw, the client's degrees, before it lives on.
    pub(in crate::web::world) fn seen_from(&mut self, yaw: f32) {
        self.course.yaw = yaw;
    }

    /// Where it is drawn, the one it is on being at `place` now: there, or for an effect the client leaves
    /// where it began (one that does not take its parent's place every tick), where that one was at first.
    /// It is told here how far it has gone since it began.
    pub(in crate::web::world) fn place(&mut self, place: Vector) -> Vector {
        let began_at = *self.began_at.get_or_insert(place);
        let place = if (self.file.stays)(self.effect) { began_at } else { place };
        self.course.walked = subtract(place, began_at);
        place
    }
}
