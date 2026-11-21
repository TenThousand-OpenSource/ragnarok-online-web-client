//! `PP_GHOST` (84): a sprite that goes round a place and round, bobbing up and down, comes into sight slowly
//! and after a long while goes again (`PrimGhost`, drawn by `RenderSegmentSprites2` as the other sprites are):
//! the ghosts and the bats of `CRagEffect::Ghost`, which the maps of Niflheim place.
//!
//! The client keeps its figures in its first part, and so it is here (`Part`): `flags[2]` is what it is (0 a
//! ghost, 1 a bat, 2 a bat that flies a figure of eight), `flags[0]` how many degrees it goes round a tick,
//! `turned` where round it is, `out` how far from the place, `rise` and `tall` the turn and the reach of its
//! bobbing, `heights[1]` the most its alpha comes up to.
//!
//! It looks at the camera every tick: a ghost shows the action of its sprite that faces the way it goes as
//! the camera sees it, and a bat changes its action by lot. The primitive is told the camera's yaw before
//! each tick (`Prim::watched`), where the action of the tick to come is picked: so the frames of that action
//! are counted, as the client's `AnimMotion` counts those of the action just picked.

use super::*;

/// The count from which it widens its round and fades (`PrimGhost`).
const LEAVES_AT: i32 = 2600;

impl Part {
    /// How many degrees a ghost or a bat goes round at the tick it is about to live: its `flags[0]`, which
    /// from its 2600th tick goes up by one whenever its count is a whole multiple of a round divided by it
    /// (but for the bat of the figure of eight, which keeps its two).
    fn ghost_step(&self) -> i8 {
        let process = self.process + 1;
        let grows = self.flags[2] <= 1 && process >= LEAVES_AT && process % (360 / i32::from(self.flags[0]).max(1)) == 0;
        self.flags[0] + i8::from(grows)
    }
}

impl Prim {
    /// `PrimGhost`, what it does with the camera's `yaw` (the client's degrees), for the tick about to be
    /// lived: a ghost that goes, as the camera sees it, through the first half of a round (where round it will
    /// be and the yaw's whole degrees, brought once into a round) shows its first action, through the second
    /// half its seventh; a bat takes its second action at one lot of 999 and its first again at one of 1001.
    pub(super) fn ghost_watched(&mut self, yaw: f32) {
        let Some(part) = self.parts.first_mut().filter(|part| part.live) else { return };
        if part.flags[2] == 0 {
            let mut angle = (part.turned + i32::from(part.ghost_step())) % 360 + yaw as i32;
            if angle < 0 {
                angle += 360;
            }
            if angle >= 360 {
                angle -= 360;
            }
            self.action = if (0..180).contains(&angle) { 0 } else { 6 };
            return;
        }
        if part.lot(999) == 1 {
            self.action = 1;
        }
        if part.lot(1001) == 0 {
            self.action = 0;
        }
    }

    /// `PrimGhost`: over its first 250 ticks its alpha comes up by 1 a tick to its most. From its 2600th a
    /// ghost and a plain bat go round faster and faster (`Part::ghost_step`) and 0.002 further out every tick,
    /// and every 15th tick the alpha goes down by 1 (by 5 for the bat of the figure of eight). It bobs 3
    /// degrees on a tick and goes round by its step, and stands `out` from where its effect was as it was
    /// launched: along the cosine and the sine of where round it is (the bat of the figure of eight along the
    /// sine of twice that), and up or down by `tall` times the sine of its bobbing.
    ///
    /// Then `ProcessAlpha`: a primitive that would outlive its effect is, at its first tick, set to fade to
    /// nothing by the effect's end from the alpha it has then (1: nearly nothing a tick), once and for all
    /// (`isDisappear`); and `ShiftSegment` and `AnimMotion` as for every sprite. Gives back whether it lives
    /// on.
    pub(super) fn haunt(&mut self, master: &Master, frames: usize) -> bool {
        if let Some(part) = self.parts.first_mut().filter(|part| part.live) {
            let step = part.ghost_step();
            part.process += 1;
            if part.process <= 250 && part.heights[1] > self.alpha {
                self.alpha += 1.0;
            }
            if part.process >= LEAVES_AT {
                if part.flags[2] <= 1 {
                    part.out *= 1.002;
                }
                if part.process % 15 == 0 {
                    self.alpha = (self.alpha - if part.flags[2] == 2 { 5.0 } else { 1.0 }).max(0.0);
                }
            }
            part.flags[0] = step;
            part.rise = (part.rise + 3) % 360;
            part.turned = (part.turned + i32::from(step)) % 360;
            let across = if part.flags[2] == 2 { (part.turned * 2) % 360 } else { part.turned };
            // (The client's y points down.)
            self.moved = [
                part.at[0][0] + part.out * cosine(part.turned as f32),
                -(part.at[0][1] + part.tall * sine(part.rise as f32)),
                part.at[0][2] + part.out * sine(across as f32)
            ];
        }
        if self.count == 0 && self.duration > master.left + 1 {
            self.alpha_speed = -(self.alpha / master.left.max(1) as f32);
        }
        self.alpha = (self.alpha + self.alpha_speed).clamp(0.0, self.max_alpha);
        let shown = self.trail_and_frame(frames);
        self.count += 1;
        shown && self.count <= self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The primitive `CRagEffect::Ghost` launches at a tick of its first ten: `kind` 0 a ghost, 1 a bat, 2 a
    /// bat of the figure of eight.
    fn ghost(kind: i8, most: f32) -> Prim {
        let mut prim = Prim::new(Shape::Ghost, if kind == 0 { "이팩트/유령" } else { "이팩트/박쥐" });
        (prim.duration, prim.size, prim.alpha, prim.anim_speed) = (4000, 1.5, 0.0, 2);
        let mut part = Part { out: 16.0, tall: 4.0, seed: 7, ..Part::default() };
        (part.heights[1], part.flags[0], part.flags[2]) = (most, if kind == 2 { 2 } else { 1 }, kind);
        prim.parts.push(part);
        prim
    }

    #[test]
    fn it_goes_round_bobbing_and_comes_into_sight() {
        let mut prim = ghost(0, 30.0);
        prim.tick(4000);
        // A degree round at 16 out, and 4 times the sine of 3 degrees down (the client's y), so up is less.
        assert!(testing::close(prim.moved[0], 16.0 * cosine(1.0)) && testing::close(prim.moved[2], 16.0 * sine(1.0)));
        assert!(testing::close(prim.moved[1], -4.0 * sine(3.0)));
        assert_eq!(prim.alpha, 1.0);
        for _ in 0..100 {
            prim.tick(3999);
        }
        // Its alpha stops at its most; it shows its sprite once, half as large again.
        assert_eq!(prim.alpha, 30.0);
        let seen = prim.seen();
        assert!(prim.shows_sprite() && seen.len() == 1 && seen[0].scale == 1.5 && seen[0].alpha == 30.0);
        // A quarter round after 90 ticks, it was east; now it is on round.
        assert_eq!(prim.parts[0].turned, 101);
    }

    #[test]
    fn a_bat_of_the_figure_of_eight_goes_two_degrees_and_crosses_over() {
        let mut prim = ghost(2, 250.0);
        for _ in 0..45 {
            prim.tick(4000);
        }
        // A quarter round: east nothing, and north along the sine of half a round, nothing again.
        assert_eq!(prim.parts[0].turned, 90);
        assert!(prim.moved[0].abs() < 0.001 && prim.moved[2].abs() < 0.001);
    }

    #[test]
    fn one_launched_after_its_effect_fades_a_touch_from_its_first_tick() {
        // Launched at the effect's tenth tick, it would outlive it: the alpha it has at its first tick, 1,
        // is set to go by the effect's end.
        let mut prim = ghost(0, 3.0);
        prim.tick(3991);
        assert!(testing::close(prim.alpha_speed, -1.0 / 3991.0) && prim.alpha < 1.0);
        // One launched at the effect's first or second tick never is.
        let mut first = ghost(0, 30.0);
        first.tick(3999);
        assert_eq!((first.alpha_speed, first.alpha), (0.0, 1.0));
    }

    #[test]
    fn after_a_long_while_it_goes_faster_wider_and_fainter() {
        let mut prim = ghost(1, 30.0);
        for _ in 0..2599 {
            prim.tick(4000);
        }
        assert_eq!((prim.parts[0].flags[0], prim.parts[0].out, prim.alpha), (1, 16.0, 30.0));
        for _ in 0..281 {
            prim.tick(4000);
        }
        // At its 2880th tick, eight whole rounds, its step went up to two; every 15th tick took 1 of its alpha.
        assert_eq!((prim.parts[0].process, prim.parts[0].flags[0]), (2880, 2));
        assert!(prim.parts[0].out > 16.0 * 1.7 && prim.alpha == 30.0 - 19.0);
        // The bat of the figure of eight keeps its round and its step, and loses 5 at a time.
        let mut eight = ghost(2, 250.0);
        for _ in 0..2610 {
            eight.tick(4000);
        }
        assert_eq!((eight.parts[0].flags[0], eight.parts[0].out, eight.alpha), (2, 16.0, 245.0));
    }

    #[test]
    fn a_ghost_shows_the_action_that_faces_its_way_as_the_camera_sees_it() {
        let mut prim = ghost(0, 30.0);
        // About to be a degree round: seen from a yaw of nothing it is in the first half, from 180 in the second.
        prim.watched(0.0);
        assert_eq!(prim.action, 0);
        prim.watched(180.0);
        assert_eq!(prim.action, 6);
        // A yaw under nothing is brought into the round once: -90 and a degree are 271.
        prim.watched(-90.0);
        assert_eq!(prim.action, 6);
        prim.parts[0].turned = 200;
        prim.watched(-90.0);
        assert_eq!(prim.action, 0);
        // A bat's action goes by lot alone: it is one of its two, whatever the yaw.
        let mut bat = ghost(1, 30.0);
        for _ in 0..5000 {
            bat.watched(123.0);
            assert!(bat.action <= 1);
        }
    }
}
