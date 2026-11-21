//! The particle emitters a map places (smoke over a chimney, dust in the wind): the client's effect 974,
//! `CEmitterEffect` (Source/Contents/Effects/Particles/EmitterEffect.cpp), as rules that need no page: what an
//! emitter is before a map's script sets anything of it and what a setting changes (`Emitter`), and how its
//! particles are born, move, fade and end, a frame of the client's at a time (`Emitting`).
//!
//! A map's `.rsw` places the emitters (`CWorld::LoadMapEffects`) and the map's script of the `effecttool`
//! folder then sets each by its number among them (`EffectLists::LoadMapEffectScript`, `SetEmitterInfo`); the
//! gateway reads the script (`tables/emitters.rs`), `web/data/emitters.rs` asks for it, and the world keeps and
//! draws the emitters (`web/world/scenery/emitters.rs`).
//!
//! Places here are the client's: x east, y down, z north.

/// The effect an emitter is among those a map places (`EF_EMITTER`).
pub const EF_EMITTER: u32 = 974;
/// Within how many world units of the player's character an emitter goes on giving particles (`CEffect`'s
/// `range`).
pub const NEAR: f32 = 130.0;
/// What a frame is of a second (`timeScale`): a particle's life and an emitter's rate are told in seconds.
const STEP: f32 = 1.0 / 60.0;

/// What an emitter is set to. As it is made (`CEmitterEffect::CEmitterEffect`) it is `default()`; a map's
/// script sets fields of it (`set`).
#[derive(Clone, Debug, PartialEq)]
pub struct Emitter {
    /// Between which two speeds a particle's is drawn, along each axis (`velocityMin`, `velocityMax`).
    pub low: [f32; 3],
    pub high: [f32; 3],
    /// What is added to a particle's speed every second (`acceleration`).
    pub gravity: [f32; 3],
    /// How far to each side of the emitter a particle is born (`spread`).
    pub spread: [f32; 3],
    /// Red, green, blue and alpha, of 255.
    pub colour: [i32; 4],
    /// The least and the most of how many are born a second, of a particle's size and of its seconds.
    pub rate: [f32; 2],
    pub size: [f32; 2],
    pub life: [f32; 2],
    /// The particles' picture, under `data\texture\`.
    pub picture: String,
    /// What a particle's speed is taken times (`particleSpeed`): with none it stays where it was born.
    pub pace: f32,
    /// Direct3D's two blends, by their numbers (`sourceBlend`, `destinationBlend`).
    pub blend: [i32; 2],
    /// How many particles the emitter has (`count`).
    pub most: usize,
    /// Whether what stands before a particle hides it.
    pub depth: bool,
    /// Whether the particles are turned by `turn` (degrees about each axis) and not to the camera (the client's
    /// `billboard`, which the scripts call `billboard_off`).
    pub flat: bool,
    pub turn: [f32; 3],
    /// Whether the particles neither fade nor end (`fixedAlpha`).
    pub lasting: bool,
    /// The width and the height a particle's size is taken times.
    pub stretch: [f32; 2],
}

impl Default for Emitter {
    fn default() -> Self {
        Self {
            low: [-2.0, 2.0, -2.0],
            high: [2.0, -1.0, 2.0],
            gravity: [0.7, -2.0, 1.0],
            spread: [-1.0, 2.0, -1.0],
            colour: [255; 4],
            rate: [20.0, 100.0],
            size: [3.0, 8.0],
            life: [5.0, 10.0],
            picture: "smoke2.bmp".to_owned(),
            pace: 1.0,
            blend: [5, 2],
            most: 40,
            depth: true,
            flat: false,
            turn: [0.0; 3],
            lasting: false,
            stretch: [1.0, 1.0],
        }
    }
}

impl Emitter {
    /// One field of the gateway's table is set (the fields are named in its `tables/emitters.rs`): to the
    /// numbers given, or for the picture the text. What is no field, or was given too few numbers, sets
    /// nothing.
    pub fn set(&mut self, field: &str, numbers: &[f64], text: Option<&str>) {
        fn filled<const COUNT: usize>(numbers: &[f64]) -> Option<[f32; COUNT]> {
            (numbers.len() >= COUNT).then(|| std::array::from_fn(|at| numbers[at] as f32))
        }
        let first = numbers.first().copied();
        let (three, two) = (filled::<3>(numbers), filled::<2>(numbers));
        match field {
            "low" => self.low = three.unwrap_or(self.low),
            "high" => self.high = three.unwrap_or(self.high),
            "gravity" => self.gravity = three.unwrap_or(self.gravity),
            "spread" => self.spread = three.unwrap_or(self.spread),
            "turn" => self.turn = three.unwrap_or(self.turn),
            "colour" if numbers.len() >= 4 => self.colour = std::array::from_fn(|part| numbers[part] as i32),
            "rate" => self.rate = two.unwrap_or(self.rate),
            "size" => self.size = two.unwrap_or(self.size),
            "life" => self.life = two.unwrap_or(self.life),
            "stretch" => self.stretch = two.unwrap_or(self.stretch),
            "picture" => self.picture = text.map_or_else(|| self.picture.clone(), str::to_owned),
            "pace" => self.pace = first.map_or(self.pace, |pace| pace as f32),
            "source" => self.blend[0] = first.map_or(self.blend[0], |blend| blend as i32),
            "destination" => self.blend[1] = first.map_or(self.blend[1], |blend| blend as i32),
            "most" => self.most = first.map_or(self.most, |most| most.max(0.0) as usize),
            "depth" => self.depth = first.map_or(self.depth, |depth| depth != 0.0),
            "flat" => self.flat = first.map_or(self.flat, |flat| flat != 0.0),
            "lasting" => self.lasting = first.map_or(self.lasting, |lasting| lasting != 0.0),
            _ => {}
        }
    }
}

/// `RandomRange`: a number from `low` to `high` in tenths; `lot(under)` is the client's `rand() % under`. (Told
/// a `low` over its `high` it gives one from `low` upwards, by as much as lies between them: the client's own
/// figures have such, the spread an emitter is made with.)
fn between(low: f32, high: f32, lot: &mut dyn FnMut(u32) -> u32) -> f32 {
    if low == high {
        return low;
    }
    let least = (low * 10.0) as i32;
    let span = (-least - (high * -10.0) as i32).abs() + 1;
    (lot(span as u32) as i32 + least) as f32 * 0.1
}

/// One particle of an emitter (`CParticle`, what `CEmitterEffect` uses of it).
#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    /// Its seconds; one of none never ends.
    life: f32,
    scale: f32,
    speed: [f32; 3],
    /// How far from the emitter it has come.
    offset: [f32; 3],
    /// How much of it shows, of 255.
    pub alpha: i32,
    /// The frames since it was born (`elapsedFrames`).
    frames: i32,
    /// Whether the emitter has it out now (`ParticlePool`).
    active: bool,
    /// Where it is drawn, from the emitter: the emitter's own place for one not out.
    pub at: [f32; 3],
    /// Half its width and half its height as it is drawn (the scale of its `transform`, on a quad of one to
    /// each side).
    pub half: [f32; 2],
}

impl Particle {
    /// As the client makes one (`ResetParticle`) and an emitter then begins it (`born`).
    fn new(emitter: &Emitter, lot: &mut dyn FnMut(u32) -> u32) -> Self {
        let (nothing, whole) = ([0.0; 3], [1.0; 2]);
        let mut particle = Self { life: 0.0, scale: 1.0, speed: nothing, offset: nothing, alpha: 255, frames: 0, active: false, at: nothing, half: whole };
        particle.born(emitter, lot);
        particle
    }

    /// `CEmitterEffect::InitParticle`: its life, its size and its speed are drawn, it stands within the spread
    /// about a place 10 over the emitter, and nothing of it shows yet.
    fn born(&mut self, emitter: &Emitter, lot: &mut dyn FnMut(u32) -> u32) {
        self.life = between(emitter.life[0], emitter.life[1], lot);
        self.scale = between(emitter.size[0], emitter.size[1], lot);
        self.speed = std::array::from_fn(|axis| between(emitter.low[axis], emitter.high[axis], lot));
        let x = between(-emitter.spread[0], emitter.spread[0], lot);
        let y = between(-emitter.spread[1], emitter.spread[1], lot) - 10.0;
        let z = between(-emitter.spread[2], emitter.spread[2], lot);
        self.offset = [x, y, z];
        self.alpha = 0;
        self.frames = 0;
    }

    /// `IsParticleAlive`.
    fn alive(&self) -> bool {
        self.life == 0.0 || self.life * 60.0 > self.frames as f32
    }

    /// `CEmitterEffect::UpdateParticle`, for an emitter whose particles face the camera: one out whose life is
    /// over is taken back (one that lasts only once the emitter has stopped); another shows more by a frame's
    /// share of 255 each frame, and less by it in its last sixty; it moves by its speed, and its speed gains
    /// the gravity. Every particle, out or not, counts the frame.
    fn frame(&mut self, emitter: &Emitter, stopped: bool) {
        self.at = [0.0; 3];
        if self.active {
            if !self.alive() && (!emitter.lasting || stopped) {
                self.active = false;
            } else {
                if !emitter.lasting {
                    let (fade, value) = (STEP * 255.0, self.alpha as f32);
                    self.alpha = if 60.0 > self.life * 60.0 - self.frames as f32 { ((value - fade) as i32).max(0) } else { ((fade + value) as i32).min(255) };
                }
                for axis in 0..3 {
                    self.offset[axis] += self.speed[axis] * emitter.pace * STEP;
                }
                self.at = self.offset;
                for axis in 0..3 {
                    self.speed[axis] += STEP * emitter.gravity[axis];
                }
                self.half = [self.scale * emitter.stretch[0], self.scale * emitter.stretch[1]];
            }
        }
        self.frames += 1;
    }
}

/// An emitter at work: what it is set to, its particles, and what of a particle is still to be born
/// (`emitCarry`).
#[derive(Clone, Debug, PartialEq)]
pub struct Emitting {
    pub emitter: Emitter,
    particles: Vec<Particle>,
    carry: f32,
    /// Whether it gives no new particles, the player's character being far (`emissionStopped`).
    stopped: bool,
    /// Whether it neither lives nor is drawn, being far with no particle out (`outsideRange`).
    away: bool,
}

impl Emitting {
    /// An emitter as it is once the script has set it (`CEmitterEffect::Open`, then `ResetParticles`): all its
    /// particles begun and none out; of one that lasts and gives none they are all out, as much there as its
    /// colour says.
    pub fn new(emitter: Emitter, lot: &mut dyn FnMut(u32) -> u32) -> Self {
        let mut particles: Vec<Particle> = (0..emitter.most).map(|_| Particle::new(&emitter, lot)).collect();
        if emitter.lasting && emitter.rate == [0.0, 0.0] {
            for particle in &mut particles {
                (particle.active, particle.alpha) = (true, emitter.colour[3]);
            }
        }
        Self { emitter, particles, carry: 0.0, stopped: false, away: false }
    }

    /// One frame of the client's with the player's character so far away (`CEffect::Update`): near, it goes
    /// on; far, it gives no more, and once none of its particles is out it stands still
    /// (`CEmitterEffect::CheckRange`). Then so many are born as its rate gives a frame, what is left of one
    /// kept for the next, each the first particle that is not out, and every particle lives the frame
    /// (`CEmitterEffect::OnProcess`).
    pub fn frame(&mut self, away: f32, lot: &mut dyn FnMut(u32) -> u32) {
        if away < NEAR {
            (self.away, self.stopped) = (false, false);
        } else {
            self.stopped = true;
            if self.particles.iter().all(|particle| !particle.active) {
                self.away = true;
            }
        }
        if self.away {
            return;
        }
        if !self.stopped {
            let rate = between(self.emitter.rate[0], self.emitter.rate[1], lot) as i32 as f32 * STEP;
            let mut born = rate as i32;
            self.carry += rate - born as f32;
            if self.carry > 1.0 {
                let whole = self.carry as i32;
                born += whole;
                self.carry -= whole as f32;
            }
            for _ in 0..born {
                let Some(particle) = self.particles.iter_mut().find(|particle| !particle.active) else { break };
                particle.born(&self.emitter, lot);
                particle.active = true;
                if self.emitter.lasting {
                    particle.alpha = self.emitter.colour[3];
                }
            }
        }
        for particle in &mut self.particles {
            particle.frame(&self.emitter, self.stopped);
        }
    }

    /// The particles to draw now (`CEmitterEffect::Render` gives every one to the renderer, which leaves out
    /// what has no alpha: its test of the see-through queue): none of an emitter that stands still.
    pub fn shown(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter().filter(|particle| !self.away && particle.alpha > 0)
    }

    /// The colour its particles are drawn in, of 1, but for their alpha (`CParticle::SetColor`).
    pub fn colour(&self) -> [f32; 3] {
        std::array::from_fn(|part| self.emitter.colour[part].clamp(0, 255) as f32 / 255.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prontera's smoke as its script sets it (`effecttool\prontera.lub`), born six a second.
    fn smoke() -> Emitter {
        let mut emitter = Emitter::default();
        for (field, numbers) in [
            ("most", &[20.0][..]), ("low", &[-3.0, -5.0, -3.0]), ("high", &[5.0, 0.0, 5.0]), ("gravity", &[0.7, -2.0, 0.7]), ("spread", &[0.0; 3]),
            ("colour", &[5.0, 15.0, 20.0, 50.0]), ("rate", &[6.0, 6.0]), ("size", &[15.0, 20.0]), ("life", &[4.0, 4.0]), ("pace", &[0.0]),
            ("source", &[9.0]), ("destination", &[2.0]), ("depth", &[1.0]), ("flat", &[0.0]), ("turn", &[0.0; 3])
        ] {
            emitter.set(field, numbers, None);
        }
        emitter.set("picture", &[], Some("effect\\smoke1.bmp"));
        emitter
    }

    #[test]
    fn a_setting_changes_its_field_and_nothing_else() {
        let emitter = smoke();
        assert_eq!((emitter.most, emitter.blend, emitter.colour, emitter.picture.as_str()), (20, [9, 2], [5, 15, 20, 50], "effect\\smoke1.bmp"));
        assert_eq!((emitter.low, emitter.rate, emitter.pace, emitter.depth, emitter.flat), ([-3.0, -5.0, -3.0], [6.0, 6.0], 0.0, true, false));
        // What the script does not set is as the client makes an emitter.
        assert_eq!((emitter.stretch, emitter.lasting), ([1.0, 1.0], false));
        let mut other = Emitter::default();
        other.set("gravity", &[1.0], None);
        other.set("picture", &[1.0], None);
        other.set("unknown", &[1.0, 2.0, 3.0], None);
        assert_eq!(other, Emitter::default());
        other.set("lasting", &[1.0], None);
        other.set("stretch", &[2.0, 0.5, 9.0], None);
        assert_eq!((other.lasting, other.stretch), (true, [2.0, 0.5]));
    }

    #[test]
    fn a_number_is_drawn_in_tenths_between_two() {
        assert_eq!(between(4.0, 4.0, &mut |_| panic!("nothing is drawn")), 4.0);
        // From 15 to 20: one of fifty-one tenths.
        assert_eq!(between(15.0, 20.0, &mut |under| { assert_eq!(under, 51); 0 }), 15.0);
        assert!((between(15.0, 20.0, &mut |under| under - 1) - 20.0).abs() < 1e-4);
        assert!((between(-3.0, 5.0, &mut |_| 12) + 1.8).abs() < 1e-4);
        // The spread an emitter is made with, from 1 to -1: from 1 upwards by the two between them.
        assert!((between(1.0, -1.0, &mut |under| { assert_eq!(under, 21); 20 }) - 3.0).abs() < 1e-4);
    }

    #[test]
    fn a_particle_shows_more_each_frame_and_fades_in_its_last_sixty() {
        let mut emitting = Emitting::new(smoke(), &mut |_| 0);
        assert_eq!((emitting.particles.len(), emitting.shown().count()), (20, 0));
        // Six a second: the first is born on the frame that fills one, about the tenth.
        let mut waited = 0;
        while emitting.shown().count() == 0 {
            emitting.frame(0.0, &mut |_| 0);
            waited += 1;
        }
        assert!((10..=11).contains(&waited), "{waited}");
        let first = |emitting: &Emitting| emitting.particles[0].clone();
        assert_eq!((first(&emitting).active, first(&emitting).alpha, first(&emitting).frames), (true, 4, 1));
        // It stands 10 over the emitter (y is down) and does not move: Prontera's smoke has no pace.
        assert_eq!((first(&emitting).at, first(&emitting).half), ([0.0, -10.0, 0.0], [15.0, 15.0]));
        // 4 more a frame, up to 255.
        for _ in 0..62 {
            emitting.frame(0.0, &mut |_| 0);
        }
        assert_eq!(first(&emitting).alpha, 252);
        emitting.frame(0.0, &mut |_| 0);
        assert_eq!(first(&emitting).alpha, 255);
        // Four seconds are 240 frames: with less than sixty of them left it has 5 less a frame.
        for _ in 0..117 {
            emitting.frame(0.0, &mut |_| 0);
        }
        assert_eq!((first(&emitting).frames, first(&emitting).alpha), (181, 255));
        emitting.frame(0.0, &mut |_| 0);
        assert_eq!(first(&emitting).alpha, 250);
        for _ in 0..58 {
            emitting.frame(0.0, &mut |_| 0);
        }
        assert_eq!((first(&emitting).frames, first(&emitting).alpha, first(&emitting).active), (240, 0, true));
        // Its life over, it is taken back, and is the first to be born again.
        emitting.frame(0.0, &mut |_| 0);
        assert!(!first(&emitting).active);
        assert_eq!(first(&emitting).at, [0.0; 3]);
    }

    #[test]
    fn a_particle_moves_by_its_speed_times_the_pace_and_gains_the_gravity() {
        // Morocc's dust goes at nine times its speed.
        let mut emitter = smoke();
        emitter.set("pace", &[9.0], None);
        emitter.set("rate", &[60.0, 60.0], None);
        let mut emitting = Emitting::new(emitter, &mut |_| 0);
        emitting.frame(0.0, &mut |_| 0);
        let particle = emitting.particles[0].clone();
        // Its speed was drawn the lowest: (-3, -5, -3); a frame is a sixtieth of a second.
        let moved = [-3.0 * 9.0 / 60.0, -10.0 - 5.0 * 9.0 / 60.0, -3.0 * 9.0 / 60.0];
        assert!(particle.at.iter().zip(moved).all(|(at, moved)| (at - moved).abs() < 1e-4), "{:?}", particle.at);
        let gained = [-3.0 + 0.7 / 60.0, -5.0 - 2.0 / 60.0, -3.0 + 0.7 / 60.0];
        assert!(particle.speed.iter().zip(gained).all(|(speed, gained)| (speed - gained).abs() < 1e-4), "{:?}", particle.speed);
    }

    #[test]
    fn an_emitter_has_no_more_out_than_it_has_particles() {
        let mut emitter = smoke();
        emitter.set("rate", &[600.0, 600.0], None);
        let mut emitting = Emitting::new(emitter, &mut |_| 0);
        for _ in 0..5 {
            emitting.frame(0.0, &mut |_| 0);
        }
        assert_eq!(emitting.particles.iter().filter(|particle| particle.active).count(), 20);
        assert_eq!(emitting.shown().count(), 20);
    }

    #[test]
    fn far_from_the_player_it_gives_no_more_and_then_stands_still() {
        let mut emitting = Emitting::new(smoke(), &mut |_| 0);
        while emitting.shown().count() == 0 {
            emitting.frame(129.0, &mut |_| 0);
        }
        // Far: the one out lives its life to the end, and no other is born.
        for _ in 0..200 {
            emitting.frame(130.0, &mut |_| 0);
        }
        assert_eq!((emitting.particles.iter().filter(|particle| particle.active).count(), emitting.away), (1, false));
        for _ in 0..40 {
            emitting.frame(130.0, &mut |_| 0);
        }
        assert!(emitting.particles.iter().all(|particle| !particle.active));
        // With none out it stands still: its particles count no more frames.
        emitting.frame(130.0, &mut |_| 0);
        let frames = emitting.particles[1].frames;
        emitting.frame(130.0, &mut |_| 0);
        assert_eq!((emitting.away, emitting.particles[1].frames, emitting.shown().count()), (true, frames, 0));
        // Near again, it goes on.
        emitting.frame(0.0, &mut |_| 0);
        assert_eq!((emitting.away, emitting.particles[1].frames), (false, frames + 1));
    }

    #[test]
    fn one_that_lasts_and_gives_none_has_all_its_particles_out_from_the_first() {
        let mut emitter = smoke();
        for (field, numbers) in [("lasting", &[1.0][..]), ("rate", &[0.0, 0.0]), ("stretch", &[2.0, 0.5])] {
            emitter.set(field, numbers, None);
        }
        let mut emitting = Emitting::new(emitter, &mut |_| 0);
        assert_eq!(emitting.shown().count(), 20);
        for _ in 0..500 {
            emitting.frame(0.0, &mut |_| 0);
        }
        // Past their four seconds they are there as before, as much as the emitter's colour says.
        assert_eq!(emitting.shown().count(), 20);
        assert_eq!((emitting.particles[0].alpha, emitting.particles[0].half), (50, [30.0, 7.5]));
        assert_eq!(emitting.colour(), [5.0 / 255.0, 15.0 / 255.0, 20.0 / 255.0]);
    }
}
