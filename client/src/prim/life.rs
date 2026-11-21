//! How a primitive lives: one tick of it, kind by kind (`CEffectPrim::OnProcess` and the functions it hands
//! the kinds to), and how it fades (`ProcessAlpha`). What a kind does that takes more than a few lines is in
//! the file of that kind.

use super::*;

impl Prim {
    /// One tick of its life. `left` is how many ticks the effect it belongs to still has. Gives back whether it
    /// lives on.
    pub fn tick(&mut self, left: i32) -> bool {
        self.tick_on(&mut Master::left(left), 0, &|_, _| f32::NEG_INFINITY)
    }

    /// One tick of its life, for a kind that needs to know more of the world: what it shares with its effect,
    /// how many frames the action of its sprite that plays has (0 while that is not known), and how high the ground
    /// is at a place so far east and north of the effect's, measured from the effect's place.
    pub fn tick_on(&mut self, master: &mut Master, frames: usize, ground: &dyn Fn(f32, f32) -> f32) -> bool {
        // The kinds made of parts neither fade as the others do nor show a sprite: each part has its own alpha.
        if self.has_slots() {
            self.tick_slots();
            self.count += 1;
            return self.count <= self.duration;
        }
        // (A ghost fades and lives by `PrimGhost` alone.)
        if self.kind == Shape::Ghost {
            return self.haunt(master, frames);
        }
        self.shift();
        match self.kind {
            Shape::Texture2d | Shape::Quad2d => {
                self.slide();
                // (`Move2DPrimitive` ends as `MoveEffectPrimitive` does: with the next of its pictures.)
                if !self.next_picture() {
                    return false;
                }
            }
            Shape::Flash2d => self.bounce(),
            Shape::Circle2d => self.widen(),
            Shape::Texture3d => {
                if !self.plate_flight(master) {
                    return false;
                }
            }
            Shape::Cylinder3d => self.drill(),
            Shape::Circle3d => self.ring_out(),
            Shape::Sphere3d => self.swell(),
            Shape::CrossTexture3d => {
                if !self.cross_flight(master, ground) {
                    return false;
                }
            }
            Shape::Particle3d => {
                self.fly();
                if self.reached(master) {
                    return false;
                }
            }
            Shape::ParticleGravity3d => self.fall(ground),
            Shape::ParticleOrbit3d => self.orbit(),
            Shape::ParticleSpline3d => self.bow(),
            Shape::QuadHorn3d => self.advance(),
            Shape::SpriteGlow => self.come_up_and_go(ground),
            Shape::Kouenka => self.drift_as_petal(),
            Shape::Fourfold => self.spread_fourfold(),
            Shape::Roamer => self.roam(),
            Shape::Petal { slow } => self.fall_as_petal(slow),
            Shape::ParticleUp | Shape::Casting3d | Shape::Walls(_) | Shape::Tei(_) | Shape::Streak { .. } | Shape::Cast(_) => {}
            Shape::Ghost | Shape::CastingLow => {}
        }
        if self.spike_fades() {
            self.fade(master.left);
        }
        // A sprite whose frames do not start over ends with its last one.
        let shown = !self.shows_sprite() || self.trail_and_frame(frames);
        self.count += 1;
        shown && self.count <= self.duration
    }

    /// `ProcessAlpha`: one that flickers turns back at nothing and at its most; one that would outlive its
    /// effect fades with it; at its own time to fade it sets off to be gone by its last tick (neither flickers
    /// any more); and it stays between nothing and its most.
    pub(super) fn fade(&mut self, left: i32) {
        if self.flickers && (if self.alpha_speed < 0.0 { self.alpha <= 0.0 } else { self.alpha >= self.max_alpha }) {
            self.alpha_speed = -self.alpha_speed;
        }
        if self.duration - self.count > left + 1 {
            self.flickers = false;
            self.alpha_speed = -(self.alpha / left.max(1) as f32);
        }
        if self.count == self.fade_out {
            self.flickers = false;
            self.alpha_speed = -(self.alpha / (self.duration - self.count) as f32);
        }
        self.alpha = (self.alpha + self.alpha_speed).clamp(0.0, self.max_alpha);
    }

    /// Goes its speed along its own forward, turned as it is (the client's `Forward` with its `transform`).
    pub(super) fn go_forward(&mut self) {
        let step = self.turned([0.0, 0.0, self.speed]);
        self.moved = [self.moved[0] + step[0], self.moved[1] + step[1], self.moved[2] + step[2]];
    }
}
