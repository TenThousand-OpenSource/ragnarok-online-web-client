//! The kind 29: lights that rise wandering about the place of the effect and begin anew, four to a part
//! (`PrimFloatingMotes`, drawn by `RenderGroundFog`), as `CRagEffect::FreezingCircle` launches them.
//!
//! A part's four lights are at its four places (`vecBPre`, `vecBNow`, `vecTPre`, `vecTNow`: `at[2]`, `at[0]`,
//! `at[3]`, `at[1]`), each with two turns of its own among the part's `heights` that wander
//! (`DriftSlotAngles`), by whose sines it drifts east and north; `tall` is how far it rises a tick, `out` how
//! far a light reaches from its middle to a corner, `flags[0]` which lights they are.
//!
//! Ported for the lights numbered 0 (white) and 1 (blue). Not ported, as no effect ported launches them:
//! those numbered 11, 14, 15 and 16, which move and fade otherwise, and the colours of the other numbers.

use super::*;

/// Of a part's four lights, in the client's order: which of its places it is at, where among `heights` its
/// two turns begin (east, north), and whether it drifts against each (`MoveFloatingMote`'s `subtractX` and
/// `subtractZ`).
const LIGHTS: [(usize, usize, usize, bool, bool); 4] = [(2, 0, 2, false, false), (0, 4, 6, true, true), (3, 8, 10, false, true), (1, 12, 14, true, false)];

impl Part {
    /// `PrimFloatingMotes`, for one part: its eight turns wander, 3 degrees a tick up and 2 down; each light
    /// that is over the level of the effect's place drifts 0.15 of the sines of its two turns, and past 30
    /// over it begins anew at the middle, up to 99 under the level, by lot; every light rises by `tall`.
    pub(super) fn float_as_motes(&mut self) {
        for pair in 0..8 {
            self.wander_by(pair * 2, 3.0, 2.0);
        }
        for (place, east, north, against_east, against_north) in LIGHTS {
            let mut at = self.at[place];
            if at[1] < 0.0 {
                let (by_east, by_north) = (sine(self.heights[east]) * 0.15, sine(self.heights[north]) * 0.15);
                at[0] += if against_east { -by_east } else { by_east };
                at[2] += if against_north { -by_north } else { by_north };
                if at[1] < -30.0 {
                    at = [0.0, self.lot(100) as f32, 0.0];
                }
            }
            at[1] -= self.tall;
            self.at[place] = at;
        }
    }

    /// `RenderGroundFog`, for one part whose lights are numbered 0 or 1: each light a square that faces the
    /// screen about its place, `out` from its middle to a corner, drawn twice, its light added: white, or blue
    /// for the number 1. It shows at 250 up to 20 over the level of the effect's place (by its first corner)
    /// and 25 less for each one above that; one whose first corner is under the level is left out.
    pub(super) fn floating_motes(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let colour = if self.flags[0] == 1 { [80, 80, 255] } else { [255; 3] };
        LIGHTS
            .iter()
            .filter_map(|(place, ..)| {
                let corners = Self::facing(self.at[*place], self.out, 45, right, up);
                let level = corners[0][1];
                let alpha = if level < -20.0 { ((level + 20.0) * 25.0 + 250.0) as i32 } else { 250 }.max(0);
                (level <= 0.0).then(|| Rect::new(picture, alpha, corners, (0.0, 1.0), colour, true))
            })
            .flat_map(|rect| [rect.clone(), rect])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    fn motes() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::FloatingMotes), "whitelight.tga");
        prim.duration = 99_999_999;
        let mut part = Part { tall: 0.15, out: 2.4, seed: 7, ..Part::default() };
        part.flags[0] = 1;
        prim.parts.push(part);
        prim
    }

    #[test]
    fn a_light_under_the_level_only_rises_and_shows_nothing() {
        let mut prim = motes();
        (0..4).for_each(|place| prim.parts[0].at[place] = [0.0, 50.0, 0.0]);
        prim.tick(99_999_999);
        assert!(prim.parts[0].at.iter().all(|at| at[0] == 0.0 && (at[1] - 49.85).abs() < 0.001 && at[2] == 0.0));
        assert!(prim.rects(RIGHT, UP).is_empty());
    }

    #[test]
    fn a_light_over_the_level_drifts_by_its_turns_fades_high_up_and_begins_anew() {
        let mut prim = motes();
        // The first light's turns stand at 90 with their marks above: they go to 93, and it drifts with both;
        // the second light's drift against both.
        prim.parts[0].heights[..8].copy_from_slice(&[90.0, 200.0, 90.0, 200.0, 90.0, 200.0, 90.0, 200.0]);
        (prim.parts[0].at[2], prim.parts[0].at[0]) = ([0.0, -10.0, 0.0], [0.0, -10.0, 0.0]);
        (prim.parts[0].at[3], prim.parts[0].at[1]) = ([0.0, -25.0, 0.0], [0.0, -30.1, 0.0]);
        prim.tick(99_999_999);
        let step = 0.15 * 93f32.to_radians().sin();
        let part = &prim.parts[0];
        assert!((part.at[2][0] - step).abs() < 0.0001 && (part.at[2][2] - step).abs() < 0.0001 && (part.at[2][1] + 10.15).abs() < 0.0001);
        assert!((part.at[0][0] + step).abs() < 0.0001 && (part.at[0][2] + step).abs() < 0.0001);
        // The fourth was past 30 over the level: it is back at the middle, under the level.
        assert!(part.at[1][0] == 0.0 && part.at[1][2] == 0.0 && (-0.15..99.0).contains(&part.at[1][1]));
        // Three show, each twice, blue and added; the third, 25.15 over the level and its first corner higher
        // still, is fainter than the first two.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 6);
        assert!(rects.iter().all(|rect| rect.colour == [80, 80, 255] && rect.adds));
        assert_eq!((rects[0].alpha, rects[0] == rects[1]), (250.0, true));
        assert!(rects[4].alpha < 250.0);
    }
}
