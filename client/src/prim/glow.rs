//! The kind 153 of the client's primitives: a sprite standing on the ground that comes up slowly and goes
//! again before its time is over, drawn with a rim of its own light (`PrimFadeAlpha`, drawn by
//! `RenderSegmentSpriteGlow`): the monsters that stand round one who sings.
//!
//! `RenderSegmentSpriteGlow` draws every layer of the sprite's frame twice, in the colour its file gives the
//! layer and at the primitive's alpha: once mixed in (or added, for a primitive told so: `Prim::adds`), and
//! once more a pixel larger on every side with its light added. Here the second drawing is as large as the
//! first.

use super::*;

impl Prim {
    /// Whether it is of the kind whose sprite is drawn a second time with its light added, each layer in its
    /// own colour at the primitive's alpha.
    pub fn glows(&self) -> bool {
        self.kind == Shape::SpriteGlow
    }

    /// `PrimFadeAlpha`: over its first 49 ticks it comes up by 5 a tick, to 150 at most; over its last 50 it
    /// goes down by 5 a tick. The client's effect stands it on the ground as it launches it
    /// (`C3dGround::Height` at its place); here it finds the ground at its first tick.
    pub(super) fn come_up_and_go(&mut self, ground: &dyn Fn(f32, f32) -> f32) {
        if self.count == 0 {
            let height = ground(self.put.0 + self.moved[0], self.deep + self.moved[2]);
            if height.is_finite() {
                self.moved[1] = height;
            }
        }
        let process = self.count + 1;
        if process < 50 {
            self.alpha = (self.alpha + 5.0).min(150.0);
        } else if process > self.duration - 50 {
            self.alpha = (self.alpha - 5.0).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A monster as `MonsterSpriteBurst` launches one: nothing of it shows at first.
    fn monster(lasts: i32) -> Prim {
        let mut prim = Prim::new(Shape::SpriteGlow, "몬스터/geographer");
        (prim.duration, prim.alpha, prim.size) = (lasts, 0.0, 0.7);
        prim.moved = [24.0, 0.0, 0.0];
        prim
    }

    #[test]
    fn it_comes_up_to_150_and_goes_down_over_its_last_fifty_ticks() {
        let mut prim = monster(400);
        let mut master = Master::left(400);
        assert!(prim.tick_on(&mut master, 4, &|_, _| 3.0));
        assert_eq!(prim.alpha, 5.0);
        for _ in 0..40 {
            prim.tick(400);
        }
        assert_eq!(prim.alpha, 150.0);
        // It stands on the ground found at its first tick, and shows its sprite there at its size.
        let seen = prim.seen();
        assert_eq!((seen.len(), seen[0].at, seen[0].scale), (1, [24.0, 3.0, 0.0], 0.7));
        assert!(prim.shows_sprite() && prim.glows());
        for _ in 41..350 {
            prim.tick(400);
        }
        assert_eq!(prim.alpha, 150.0);
        // Its 351st tick is the first of the last fifty.
        prim.tick(400);
        assert_eq!(prim.alpha, 145.0);
        for _ in 0..29 {
            prim.tick(400);
        }
        assert_eq!(prim.alpha, 0.0);
    }

    #[test]
    fn its_frames_go_on_every_so_many_ticks_and_start_over() {
        let mut prim = monster(400);
        prim.anim_speed = 4;
        let mut master = Master::left(400);
        for _ in 0..9 {
            prim.tick_on(&mut master, 3, &|_, _| 0.0);
        }
        // A frame at the ticks 0, 4 and 8: three frames on, back at the first.
        assert_eq!(prim.motion, 0);
        prim.tick_on(&mut master, 3, &|_, _| 0.0);
        assert_eq!(prim.motion, 0);
        for _ in 0..3 {
            prim.tick_on(&mut master, 3, &|_, _| 0.0);
        }
        assert_eq!(prim.motion, 1);
    }
}
