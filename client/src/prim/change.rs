//! What a primitive may be told to become at a tick of its life (the client's `changePoint` and the `change…`
//! fields beside it), the speeds along the three directions some kinds go by, and the ring on the screen that
//! widens:
//!
//! - a wedge (`PP_2DFLASH`) takes at that tick a new speed of length and a new turning (`CEffectPrim::OnProcess`:
//!   `heightSpeed = changeHeightSpeed`, `longSpeed = changeSpeed`, `longAccel = changeAccel`);
//! - a spike (`PP_3DQUADHORN`) a new speed and acceleration (`AdvanceEffectPrimitive`);
//! - a sprite (`PP_3DPARTICLE`) new speeds along the three directions and how they change (`Prim3DParticle`:
//!   `speed3d = changeSpeed3d`, `deltaPosAccel = changeAccel3d`); with its `pattern & 0x400` it goes by those
//!   speeds and not along its own forward;
//! - a picture in the world (`PP_3DTEXTURE`) and a crossed one (`PP_3DCROSSTEXTURE`) take whatever of them was
//!   given (`MoveEffectPrimitive`: `Prim::take_given`);
//! - a ring on the ground (`PP_3DCIRCLE`) a new speed of widening and how that changes (`Prim3DCircle`:
//!   `radiusSpeed = changeSpeed`, `radiusAccel = changeAccel`);
//! - `PP_2DCIRCLE` in `CEffectPrim::OnProcess`: the ring widens by a speed that changes.

use super::*;

/// What the client has in a `change…` field nobody set: `MoveEffectPrimitive` leaves alone what still has it,
/// and the other kinds take it as it is.
pub const UNSET: f32 = 0.0001;

/// The client's `changePoint` and what a primitive becomes there. The three directions are the client's own:
/// x east, y down, z north.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Change {
    pub at: i32,
    pub speed: f32,
    pub accel: f32,
    pub width_speed: f32,
    pub width_accel: f32,
    pub height_speed: f32,
    pub height_accel: f32,
    pub speed3: [f32; 3],
    pub accel3: [f32; 3],
}

impl Change {
    /// A change at that tick with every field as the client starts it.
    pub fn at(at: i32) -> Self {
        Self {
            at,
            speed: UNSET,
            accel: UNSET,
            width_speed: UNSET,
            width_accel: UNSET,
            height_speed: UNSET,
            height_accel: UNSET,
            speed3: [0.0; 3],
            accel3: [0.0; 3]
        }
    }
}

impl Prim {
    /// What is done before a primitive's own step of a tick: at the tick of its change a wedge, a spike, a ring
    /// on the ground and a sprite take what each takes of it, and a sprite that goes by its three speeds goes
    /// by them. (The client does not send such a sprite along its own forward as well; none of the effects
    /// here gives one a speed of that kind.)
    pub(super) fn shift(&mut self) {
        let due = self.changes.filter(|change| change.at == self.count);
        match self.kind {
            Shape::Flash2d => {
                if let Some(change) = due {
                    (self.height_speed, self.long_speed, self.long_accel) = (change.height_speed, change.speed, change.accel);
                }
            }
            Shape::QuadHorn3d => {
                if let Some(change) = due {
                    (self.speed, self.accel) = (change.speed, change.accel);
                }
            }
            Shape::Circle3d => {
                if let Some(change) = due {
                    (self.radius_speed, self.radius_accel) = (change.speed, change.accel);
                }
            }
            Shape::Particle3d => {
                if let Some(change) = due {
                    (self.speed3, self.accel3) = (change.speed3, change.accel3);
                }
                if self.by_axes {
                    self.drift();
                }
            }
            _ => {}
        }
    }

    /// `MoveEffectPrimitive`, its first step, for a picture in the world and a crossed one: at the tick of its
    /// change it takes each speed that was given, and leaves alone what was not. It changes once: the client
    /// puts every field back to the value of one not given.
    pub(super) fn take_given(&mut self) {
        let Some(change) = self.changes.filter(|change| change.at == self.count) else { return };
        let given = |now: &mut f32, new: f32| {
            if new != UNSET {
                *now = new;
            }
        };
        given(&mut self.width_speed, change.width_speed);
        given(&mut self.height_speed, change.height_speed);
        given(&mut self.speed, change.speed);
        given(&mut self.width_accel, change.width_accel);
        given(&mut self.height_accel, change.height_accel);
        given(&mut self.accel, change.accel);
        (0..3).for_each(|axis| given(&mut self.speed3[axis], change.speed3[axis]));
        self.changes = None;
    }

    /// Its three speeds change by what they change by, and it goes by them (the client's y points down).
    pub(super) fn drift(&mut self) {
        self.speed3 = std::array::from_fn(|axis| self.speed3[axis] + self.accel3[axis]);
        self.moved = [self.moved[0] + self.speed3[0], self.moved[1] - self.speed3[1], self.moved[2] + self.speed3[2]];
    }

    /// `PP_2DCIRCLE` in `CEffectPrim::OnProcess`: its radius changes by a speed that changes; it is filled from
    /// its edge inwards by its inner size, or to its middle with the first pattern or while it is narrower
    /// than that.
    pub(super) fn widen(&mut self) {
        self.radius_speed += self.radius_accel;
        self.radius += self.radius_speed;
        self.size = if self.first_pattern || self.radius < self.inner { self.radius } else { self.inner };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::close;

    #[test]
    fn a_ring_on_the_screen_widens_and_is_filled_by_its_inner_size_once_wider_than_that() {
        // As the ring of Sonic Blow: 7 a tick, slowing.
        let mut ring = Prim::new(Shape::Circle2d, "");
        (ring.duration, ring.inner, ring.radius_speed, ring.radius_accel) = (40, 10.0, 7.0, -0.175);
        ring.tick(100);
        assert!(close(ring.radius, 6.825) && close(ring.size, 6.825));
        ring.tick(100);
        assert!(close(ring.radius, 13.475) && close(ring.size, 10.0));
        // One of the first pattern is filled to its middle however wide.
        let mut disc = Prim::new(Shape::Circle2d, "");
        (disc.first_pattern, disc.radius, disc.radius_speed) = (true, 370.0, -18.5);
        disc.tick(100);
        assert!(close(disc.radius, 351.5) && close(disc.size, 351.5));
    }

    #[test]
    fn a_wedge_takes_a_new_length_speed_and_turning_at_its_tick() {
        let mut wedge = Prim::new(Shape::Flash2d, "");
        (wedge.height_speed, wedge.long_speed, wedge.long_accel) = (5.0, 4.0, -0.1);
        wedge.changes = Some(Change { speed: 2.0, accel: 0.0, height_speed: 0.0, ..Change::at(2) });
        wedge.tick(100);
        wedge.tick(100);
        assert!(close(wedge.height, 10.0) && close(wedge.longitude, 3.9 + 3.8));
        wedge.tick(100);
        assert!(close(wedge.height, 10.0) && close(wedge.longitude, 7.7 + 2.0));
    }

    #[test]
    fn a_spike_takes_a_new_speed_at_its_tick() {
        // As the spike of Earth Spike: up at 1 a tick, quickening, then back at 1.2 from its twelfth tick.
        let mut spike = Prim::new(Shape::QuadHorn3d, "");
        (spike.latitude, spike.speed, spike.accel) = (90.0, 1.0, 0.01);
        spike.changes = Some(Change { speed: -1.2, accel: 0.0, ..Change::at(1) });
        spike.tick(100);
        let risen = spike.moved[1];
        assert!(close(risen, 1.01));
        spike.tick(100);
        assert!(close(spike.moved[1], risen - 1.2));
    }

    #[test]
    fn a_ring_on_the_ground_takes_a_new_speed_of_widening_at_its_tick() {
        // As the ring of Bowling Bash: slower and slower, and from its tick steadily.
        let mut ring = Prim::new(Shape::Circle3d, "");
        (ring.radius, ring.radius_speed, ring.radius_accel) = (8.0, 0.7, -0.1);
        ring.changes = Some(Change { speed: 0.08, accel: 0.0, ..Change::at(2) });
        ring.tick(100);
        ring.tick(100);
        assert!(close(ring.radius, 8.0 + 0.6 + 0.5));
        ring.tick(100);
        ring.tick(100);
        assert!(close(ring.radius, 9.1 + 0.08 + 0.08) && close(ring.radius_speed, 0.08));
    }

    #[test]
    fn a_crossed_picture_takes_the_speed_it_was_given_at_its_tick_and_keeps_the_rest() {
        // As a streak of Blitz Beat: back faster and faster, and from its tick slowly.
        let mut streak = Prim::new(Shape::CrossTexture3d, "");
        (streak.speed, streak.accel, streak.width_speed) = (-1.2, -0.1, 0.5);
        streak.changes = Some(Change { speed: -0.3, accel: 0.001, ..Change::at(1) });
        streak.tick(100);
        assert!(close(streak.speed, -1.3));
        streak.tick(100);
        // Its speed of width was not given anew and stays.
        assert!(close(streak.speed, -0.299) && close(streak.width, 1.0) && streak.changes.is_none());
    }

    #[test]
    fn a_sprite_goes_by_its_three_speeds_and_takes_new_ones_at_its_tick() {
        // As a flame of Sightrasher: outwards at 1 a tick, then at half that, drawn aside.
        let mut flame = Prim::new(Shape::Particle3d, "");
        (flame.by_axes, flame.speed3) = (true, [1.0, 0.0, -1.0]);
        flame.changes = Some(Change { speed3: [0.5, 0.0, -0.5], accel3: [-0.004, 0.5, 0.004], ..Change::at(2) });
        flame.tick(100);
        flame.tick(100);
        assert_eq!(flame.moved, [2.0, 0.0, -2.0]);
        flame.tick(100);
        // The client's y points down: a speed of 0.5 along it sinks.
        assert!(close(flame.moved[0], 2.496) && close(flame.moved[1], -0.5) && close(flame.moved[2], -2.496));
        // One without the pattern keeps its speeds to itself.
        let mut still = Prim::new(Shape::Particle3d, "");
        still.speed3 = [1.0, 1.0, 1.0];
        still.tick(100);
        assert_eq!(still.moved, [0.0; 3]);
    }
}
