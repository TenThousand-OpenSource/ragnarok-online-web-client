//! The kind 33 of the client's primitives: squares facing the screen that come up, stand a while over the whole
//! view and go, each then put anew somewhere round the player (`PrimFieldFireflies`, drawn by
//! `RenderFieldPanel`): the clouds and the stars `CRagEffect::SkyCloud` puts round a map in the sky.
//!
//! A part's `heights[7]` is the kind its effect gave it (1 and 4: clouds; 2: clouds under the map; 5: dark
//! clouds; 9: stars), `turned` the count from which it goes, `heights[0..4]` its two wandering turns, and
//! `heights[8]` the turn by which its size swings. The ported kinds are those; the client has more.

use super::*;

/// The mark in a part's `flags` by which it asks its effect to put it anew round where the player is now
/// (`PUT_ANEW`), which the client does in the same function: the effect here knows where its place is by then
/// (`web/world/prims/weather.rs`).
pub const PUT_ANEW: usize = 5;

impl Part {
    /// `PrimFieldFireflies`, for one part. It counts on; a star shows the next of its six pictures every 14
    /// ticks. Over its first 80 ticks its alpha comes up, by 2 a tick for the kinds 2 and 5 and by 3 for the
    /// others; past the count it was given to stand, it goes down by 1 a tick, and at nothing it starts over
    /// and asks to be put anew. Its two turns wander a degree a tick, and it drifts by their sines: 0.05 a
    /// tick east and north, 0.035 for the kind 5, and for the kind 4 always eastwards, 0.2 a tick. The turn of
    /// its swing goes on a degree a tick.
    pub(super) fn drift_as_cloud(&mut self) {
        let kind = self.heights[7] as i32;
        self.process += 1;
        if self.flags[2] == 23 && self.process % 14 == 0 {
            self.flags[0] = if self.flags[0] >= 5 { 0 } else { self.flags[0] + 1 };
        }
        if self.process < 80 {
            self.alpha += if matches!(kind, 2 | 5) { 2 } else { 3 };
        }
        if self.process > self.turned {
            self.alpha -= 1;
            if self.alpha <= 0 {
                (self.process, self.alpha) = (0, 0);
                self.flags[PUT_ANEW] = 1;
            }
        }
        self.wander_by(0, 1.0, 1.0);
        self.wander_by(2, 1.0, 1.0);
        let (east, north) = (sine(self.heights[0].trunc()), sine(self.heights[2].trunc()));
        let place = &mut self.at[0];
        match kind {
            4 => (place[0], place[2]) = (place[0] + east.abs() * 0.2, place[2] + north * 0.05),
            5 => (place[0], place[2]) = (place[0] + east * 0.035, place[2] + north * 0.035),
            _ => (place[0], place[2]) = (place[0] + east * 0.05, place[2] + north * 0.05),
        }
        self.heights[8] += 1.0;
        if self.heights[8] >= 360.0 {
            self.heights[8] -= 360.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cloud of the kind 1 as `SkyCloud` launches one, that stands for 300 ticks.
    fn cloud() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Sky), "cloud1.tga");
        prim.duration = 99_999_999;
        let mut part = Part { round: 45, turned: 300, out: 40.0, seed: 9, ..Part::default() };
        (part.heights[7], part.at[0]) = (1.0, [100.0, 45.0, -60.0]);
        part.heights[..4].copy_from_slice(&[90.0, 200.0, 270.0, 100.0]);
        prim.parts.push(part);
        prim
    }

    #[test]
    fn a_cloud_comes_up_stands_goes_and_asks_to_be_put_anew() {
        let mut prim = cloud();
        prim.tick(99_999_999);
        let part = prim.parts[0].clone();
        // Up by 3; its turns go a degree towards their marks, and it drifts by their sines.
        assert_eq!((part.alpha, part.heights[0], part.heights[2], part.heights[8]), (3, 91.0, 269.0, 1.0));
        assert!((part.at[0][0] - (100.0 + 0.05 * sine(91.0))).abs() < 1e-4 && (part.at[0][2] - (-60.0 + 0.05 * sine(269.0))).abs() < 1e-4);
        // It is a white square facing the screen, 40 and a little from its middle to a corner.
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(rects.len() == 1 && rects[0].alpha == 3.0 && !rects[0].adds && rects[0].colour == [255; 3]);
        for _ in 1..300 {
            prim.tick(99_999_999);
        }
        // 79 ticks of coming up; then it stands until its 300th.
        assert_eq!(prim.parts[0].alpha, 237);
        for _ in 0..237 {
            prim.tick(99_999_999);
        }
        assert_eq!((prim.parts[0].alpha, prim.parts[0].process, prim.parts[0].flags[PUT_ANEW]), (0, 0, 1));
    }

    #[test]
    fn a_star_runs_through_its_pictures() {
        let mut prim = cloud();
        (prim.parts[0].heights[7], prim.parts[0].flags[2]) = (9.0, 23);
        for _ in 0..14 {
            prim.tick(99_999_999);
        }
        assert_eq!(prim.parts[0].flags[0], 1);
        for _ in 0..70 {
            prim.tick(99_999_999);
        }
        assert_eq!(prim.parts[0].flags[0], 0);
    }
}
