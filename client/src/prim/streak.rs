//! `PP_CHEMICAL3` (56): streaks of light that lie flat, 6 above the ground, along the way from the effect's
//! place to the place it aims at, and run that way (`PrimChemical3`, drawn by `RenderGroundBar`).
//!
//! The client makes one primitive of up to four streaks, its four parts, each in a colour picked by a number
//! the effect gives the part (`flag1[0]`). Here a streak that has a colour of its own is a primitive of its
//! own, in that colour, with the one part: what is drawn is the same. Of the number only this is kept, in the
//! kind: whether it is 10 or more, a streak that comes up and runs faster and fades slower (`quick`).
//!
//! A part's figures, as the client names them: `riseAngle` is the way it lies, in whole degrees of
//! `Get_Angle` (`rise`); `maxHeight` half its width (`tall`); `distance` (`out`) where along its length the
//! effect's place is, 100 in every effect; `height[0]` how far to the side of that place it lies, `height[1]`
//! where its tail is along the way, `height[2]` how long it is and `height[3]` how fast it runs (`heights`).
//!
//! `RenderGroundBar` has more numbers than are drawn here (30 and on: darker colours, added to what is behind
//! and drawn over everything; 125: white): no effect ported gives them.

use super::*;

impl Slot {
    /// A streak as the effects launch one, towards a place so far east and north of the effect's own: so far to
    /// the side of the way there, its tail at `tail` along it, so long, running so fast, and of that half
    /// width. Nothing of it shows yet.
    pub fn streak((east, north): (f32, f32), side: f32, tail: f32, length: f32, speed: f32, half_width: f32) -> Self {
        let mut heights = [0.0; 21];
        heights[..4].copy_from_slice(&[side, tail, length, speed]);
        Self { heights, ..Self::wall(0, 100.0, angle_of(east, north), half_width, 0) }
    }

    /// `PrimChemical3`: over its first ten ticks it comes up by 12 a tick (a quick one by 20); it runs on by
    /// its speed (a quick one's speed grows by 0.05 a tick first), and having run 200 begins again from
    /// nothing; from its 21st tick it goes down by 4 a tick (a quick one by 2) to nothing.
    fn run_as_streak(&mut self, quick: bool) {
        self.process += 1;
        if self.process <= 10 {
            self.alpha += if quick { 20 } else { 12 };
        }
        if quick {
            self.heights[3] += 0.05;
        }
        self.heights[1] += self.heights[3];
        if self.heights[1] >= 200.0 {
            self.heights[1] -= 200.0;
        }
        if self.process > 20 {
            self.alpha = (self.alpha - if quick { 2 } else { 4 }).max(0);
        }
    }

    /// `RenderGroundBar`: a rectangle 6 above `middle`. Its middle line lies `heights[0]` to the right of the
    /// way it points (a quarter turn back from it, as the client's angles go), and it reaches half its width
    /// to both sides of that line, from its tail to its head along the way, both counted from 100 behind the
    /// middle. The picture's left is at its head and its bottom on the side a quarter turn on from the way.
    fn bar(&self, picture: &'static str, middle: [f32; 3]) -> Lit {
        let way = self.rise as f32;
        let (back, front) = (if self.rise >= 90 { way - 90.0 } else { way + 270.0 }, if self.rise < 270 { way + 90.0 } else { way - 270.0 });
        let base = (self.heights[0] * cosine(back), self.heights[0] * sine(back));
        let far = (self.tall * cosine(back) + base.0, self.tall * sine(back) + base.1);
        let near = (self.tall * cosine(front) + base.0, self.tall * sine(front) + base.1);
        let along = |reach: f32| ((reach - self.out) * cosine(way), (reach - self.out) * sine(way));
        let (tail, head) = (along(self.heights[1]), along(self.heights[1] + self.heights[2]));
        let corner = |side: (f32, f32), end: (f32, f32), u: f32, v: f32| ([middle[0] + side.0 + end.0, middle[1] + 6.0, middle[2] + side.1 + end.1], [u, v]);
        let (first, second, third, fourth) = (corner(near, head, 0.0, 1.0), corner(near, tail, 1.0, 1.0), corner(far, tail, 1.0, 0.0), corner(far, head, 0.0, 0.0));
        Lit { picture, corners: [first, second, third, fourth, third, first], alpha: self.alpha as f32 }
    }
}

impl Prim {
    /// One tick of the streaks of a primitive of this kind.
    pub(super) fn tick_streaks(&mut self, quick: bool) {
        self.slots.iter_mut().filter(|slot| slot.live).for_each(|slot| slot.run_as_streak(quick));
    }

    /// The quads of its streaks now: those of which anything shows.
    pub(super) fn lit_streaks(&self) -> Vec<Lit> {
        let middle = self.middle([0.0; 3]);
        self.slots.iter().filter(|slot| slot.live && slot.alpha > 0).map(|slot| slot.bar(self.picture, middle)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|axis| (a[axis] - b[axis]).abs() < 0.001)
    }

    fn streaks(quick: bool, slot: Slot) -> Prim {
        let mut prim = Prim::new(Shape::Streak { quick }, "line");
        prim.slots = vec![slot];
        prim
    }

    #[test]
    fn a_streak_comes_up_runs_begins_again_and_goes() {
        let mut prim = streaks(false, Slot::streak((1.0, 0.0), -15.0, 196.0, 30.0, 1.5, 0.4));
        assert!(prim.lit(RIGHT, UP).is_empty());
        for _ in 0..10 {
            prim.tick(300);
        }
        // Ten ticks at 12; it has run 15 and begun again past 200.
        assert_eq!(prim.slots[0].alpha, 120);
        assert!((prim.slots[0].heights[1] - 11.0).abs() < 0.001);
        // It stays so for ten ticks more and then goes down by 4 a tick: thirty ticks to nothing.
        for _ in 0..10 {
            prim.tick(300);
        }
        assert_eq!(prim.slots[0].alpha, 120);
        for _ in 0..30 {
            prim.tick(300);
        }
        assert_eq!(prim.slots[0].alpha, 0);
        assert!(prim.lit(RIGHT, UP).is_empty());
    }

    #[test]
    fn a_quick_streak_comes_up_by_20_runs_faster_and_faster_and_goes_by_2() {
        let mut prim = streaks(true, Slot::streak((1.0, 0.0), 15.0, 0.0, 40.0, 1.0, 0.5));
        for _ in 0..21 {
            prim.tick(300);
        }
        assert_eq!(prim.slots[0].alpha, 198);
        // Its speed has grown by 0.05 at each of the 21 ticks before it was gone by: 21 + 0.05 * (1 + ... + 21).
        assert!((prim.slots[0].heights[3] - 2.05).abs() < 0.001);
        assert!((prim.slots[0].heights[1] - 32.55).abs() < 0.001);
    }

    const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
    const UP: [f32; 3] = [0.0, 1.0, 0.0];

    /// A streak towards the east (the client's angle 0), 15 to the side, its tail 110 along (10 past the
    /// middle), 20 long and 1 wide: a quarter turn back from east is the client's 270, towards -z, the south.
    #[test]
    fn a_streak_lies_along_the_way_it_points_beside_the_middle() {
        let mut prim = streaks(false, Slot::streak((5.0, 0.0), 15.0, 109.0, 20.0, 1.0, 0.5));
        prim.tick(300);
        let lit = prim.lit(RIGHT, UP);
        let corners = lit[0].corners;
        assert_eq!(lit[0].alpha, 12.0);
        // Head and tail on the near side (north of the line), then tail and head on the far side.
        assert!(close(corners[0].0, [30.0, 6.0, -14.5]) && corners[0].1 == [0.0, 1.0]);
        assert!(close(corners[1].0, [10.0, 6.0, -14.5]) && corners[1].1 == [1.0, 1.0]);
        assert!(close(corners[2].0, [10.0, 6.0, -15.5]) && corners[2].1 == [1.0, 0.0]);
        assert!(close(corners[3].0, [30.0, 6.0, -15.5]) && corners[3].1 == [0.0, 0.0]);
        // A streak whose tail is behind the middle reaches back from it.
        let mut behind = streaks(false, Slot::streak((0.0, 5.0), 0.0, 49.0, 20.0, 1.0, 0.5));
        behind.tick(300);
        let corners = behind.lit(RIGHT, UP)[0].corners;
        assert!(close(corners[1].0, [-0.5, 6.0, -50.0]));
        assert!(close(corners[0].0, [-0.5, 6.0, -30.0]));
    }
}
