//! `PP_SMA` (120) and the kind 151 of the client's primitives, which show nothing themselves: each is a place
//! that moves, and what the client's drawing function of it does is launch another effect there, a new one at
//! every frame it is drawn (`RenderSMA3`, `RenderLaunchEffects`). Here a primitive is asked once a tick what
//! effect it wants at its place (`Prim::asks`), and the world launches it (`web/world/prims.rs`).
//!
//! - 120: a place that flies level, at a speed that does not change, the way from the effect's place to the
//!   place the effect aims at (`PrimSMA`): the ball of Estun, whose sparks are `EF_SMA3` (556);
//! - 151: a place that goes round the one the effect is on, 25 over the ground, in a ring that slowly widens
//!   (`PrimSpiralFade`): the lights of Harmonize and of Arrullo, whose sparks are `EF_SOULLIGHT2` (850) and
//!   `EF_SOULLIGHT3` (932).

use super::*;

/// The client's numbers of the effects asked for here.
const EF_SMA3: u32 = 556;
const EF_SOULLIGHT2: u32 = 850;
const EF_SOULLIGHT3: u32 = 932;

impl Part {
    /// `PrimSMA`, for one part: it waits while its count is under one. At its first tick it takes the way to
    /// the place its effect aims at, which the effect gave it as so far east and north (`heights[2]`, `[3]`;
    /// the client reads its primitive's `deltaPos` and place there); from then on it goes `heights[1]` a
    /// tick that way, and never stops.
    pub(super) fn fly_as_sma(&mut self) {
        if self.waits() {
            return;
        }
        if self.process == 1 {
            self.turned = angle_of(self.heights[2], self.heights[3]);
        }
        self.at[0][0] += self.heights[1] * cosine(self.turned as f32);
        self.at[0][2] += self.heights[1] * sine(self.turned as f32);
    }

    /// `PrimSpiralFade`, for one part: it waits while its count is under one. Its light (`heights[13]`) comes
    /// up by 20 a tick over ten ticks, and goes down by 5 a tick after its 270th, or for a part whose
    /// `flags[0]` is 1 or more after its 90th. Its ring (`at[1][0]`, the client's `vecTNow.x`) widens by 0.05
    /// a tick, and it stands a whole number of that out, 25 over the ground, six degrees further round every
    /// tick: from the north by the east, or for a part whose `flags[0]` is 1 the other way.
    pub(super) fn circle_as_light(&mut self) {
        if self.waits() {
            return;
        }
        let mode = self.flags[0];
        let mut round = self.process % 60;
        if self.process <= 10 {
            self.heights[13] += 20.0;
        } else if self.process > if mode >= 1 { 90 } else { 270 } {
            self.heights[13] = (self.heights[13] - 5.0).max(0.0);
        }
        if mode == 1 {
            round = 60 - round;
        }
        self.at[1][0] += 0.05;
        let radius = self.at[1][0].trunc();
        self.at[2] = [radius * sine((6 * round) as f32), -25.0, radius * cosine((6 * round) as f32)];
    }
}

impl Prim {
    /// The effect a primitive of these kinds wants launched now, by the client's number, and where: so far
    /// east, up and north of the place the primitive is drawn about. Nothing for any other kind.
    ///
    /// - `RenderSMA3`: while its first part lives, `EF_SMA3` where that part is (even while the part still
    ///   waits), for a part whose `flags[0]` is 1. (Its `flags[0]` of 0 and of 99 ask for other effects every
    ///   eighth tick and tell them more than a place; no effect here gives a part those, and they are not
    ///   ported.)
    /// - `RenderLaunchEffects`: for every part that lives, under its 300th tick and with any light left,
    ///   `EF_SOULLIGHT2` (a part whose `flags[0]` is 0) or `EF_SOULLIGHT3` where the part is. (The client
    ///   also leaves it out with the place's square under the ground, which 25 over the ground it never is.)
    pub fn asks(&self) -> Vec<(u32, [f32; 3])> {
        let up = |at: [f32; 3]| [at[0], -at[1], at[2]];
        match self.kind {
            Shape::Tei(Tei::Sma) => self.parts.first().filter(|part| part.live && part.flags[0] == 1).map(|part| (EF_SMA3, up(part.at[0]))).into_iter().collect(),
            Shape::Tei(Tei::SpiralFade) => {
                let lit = self.parts.iter().filter(|part| part.live && part.process < 300 && part.heights[13] as i32 > 0);
                lit.map(|part| (if part.flags[0] == 0 { EF_SOULLIGHT2 } else { EF_SOULLIGHT3 }, up(part.at[2]))).collect()
            }
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ball_waits_then_flies_level_to_the_place_aimed_at_and_asks_for_sparks_all_the_while() {
        let mut prim = Prim::new(Shape::Tei(Tei::Sma), "");
        prim.duration = 200;
        // As `SMA` launches its first part: 3 ticks to wait, 8 over the ground, the one struck 10 to the north.
        let mut part = Part { process: -3, ..Part::default() };
        (part.at[0], part.heights[1], part.heights[3], part.flags[0]) = ([0.0, -8.0, 0.0], 0.8, 10.0, 1);
        prim.parts.push(part);
        for _ in 0..3 {
            prim.tick(200);
        }
        assert_eq!(prim.asks(), vec![(556, [0.0, 8.0, 0.0])]);
        for _ in 0..5 {
            prim.tick(200);
        }
        let asked = prim.asks();
        assert!(asked.len() == 1 && asked[0].0 == 556 && asked[0].1[0].abs() < 1e-4 && (asked[0].1[2] - 4.0).abs() < 1e-4 && asked[0].1[1] == 8.0);
        // It shows nothing itself, and a part of another flag asks for nothing.
        assert!(prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
        prim.parts[0].flags[0] = 0;
        assert!(prim.asks().is_empty());
    }

    #[test]
    fn the_light_goes_round_in_a_widening_ring_and_asks_for_sparks_while_it_is_lit() {
        let mut prim = Prim::new(Shape::Tei(Tei::SpiralFade), "whitelight.tga");
        prim.duration = 500;
        let mut part = Part { out: 2.5, ..Part::default() };
        part.at[1][0] = 5.0;
        prim.parts.push(part);
        for _ in 0..15 {
            prim.tick(500);
        }
        // 15 ticks: a quarter round, to the east, 5 out and 25 up.
        let asked = prim.asks();
        assert_eq!(asked.len(), 1);
        assert!(asked[0].0 == 850 && (asked[0].1[0] - 5.0).abs() < 1e-4 && asked[0].1[1] == 25.0 && asked[0].1[2].abs() < 1e-4);
        assert_eq!(prim.parts[0].heights[13], 200.0);
        // After 25 ticks its ring is a whole step wider.
        for _ in 15..25 {
            prim.tick(500);
        }
        assert!((prim.parts[0].at[2][0] - 6.0 * sine(150.0)).abs() < 1e-3);
        for _ in 25..300 {
            prim.tick(500);
        }
        // From its 300th tick it asks no more, though some of its light is left.
        assert!(prim.parts[0].heights[13] > 0.0 && prim.asks().is_empty());
    }

    #[test]
    fn the_lights_of_arrullo_go_out_early_and_one_goes_round_the_other_way() {
        let light = |mode: i8| {
            let mut prim = Prim::new(Shape::Tei(Tei::SpiralFade), "whitelight.tga");
            prim.duration = 500;
            let mut part = Part::default();
            (part.at[1][0], part.flags[0]) = (5.0, mode);
            prim.parts.push(part);
            prim
        };
        let (mut back, mut on) = (light(1), light(2));
        for _ in 0..15 {
            back.tick(500);
            on.tick(500);
        }
        // A quarter round: one to the east and one to the west, both asking for the other sparks.
        assert!((on.asks()[0].1[0] - 5.0).abs() < 1e-4 && (back.asks()[0].1[0] + 5.0).abs() < 1e-4);
        assert_eq!((on.asks()[0].0, back.asks()[0].0), (932, 932));
        for _ in 15..130 {
            on.tick(500);
        }
        assert!(on.asks().is_empty());
    }
}
