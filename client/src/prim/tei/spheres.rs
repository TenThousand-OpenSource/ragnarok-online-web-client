//! The kind 32 of the client's primitives: a sphere of light that goes round the one its effect is on,
//! wandering a little (`PrimDriftingOrbs`, drawn by `RenderLayeredGlow`): a spirit sphere of a monk, a coin
//! of a gunslinger (`CRagEffect::ElementalOrbit` launches one primitive a sphere).
//!
//! The sphere is the first part's, and only that part lives. Its flag (`flags[0]`) says how it turns and in
//! which colours it is drawn: none for a plain sphere, 1 for a coin, 20 and 21 for a sphere that stands for
//! six or for eleven.
//!
//! Left out, in both functions: the parts flagged 10 to 19, the four elemental spirits (`EF_CHOOKGI_FIRE`
//! and the three beside it, later content that nothing here launches). They go round by the primitive's
//! `isChangeProcessed`, which is not kept here, and are drawn in colours and a blend of their own.
//!
//! The kind 145 (`tei/lights.rs`) lives the same way with squares about it; the two functions of the client
//! are written apart there, and so they are here.

use super::*;

impl Prim {
    /// `PrimDriftingOrbs`. The sphere's count goes up, and over its first 200 ticks it comes up by 1 a tick.
    /// Its three turns wander (a degree a tick either way). It goes a degree on round the one the effect is
    /// on (`heights[6]`) every tick, a coin (flagged 1) every other tick. It is 7 from that one, or 5.5
    /// when it goes round nearer (`heights[10]` of 1), as high as it was put (`heights[7]`), and off that
    /// place by the sines of its three turns: whole for a plain sphere and those flagged 20 and over, halved
    /// for a coin, seven tenths for one that goes round nearer. Its swell (`heights[8]`) goes on 5 to 9
    /// degrees a tick.
    pub(super) fn drift_as_sphere(&mut self) {
        let Some(sphere) = self.parts.first_mut().filter(|sphere| sphere.live) else { return };
        sphere.process += 1;
        if sphere.process <= 200 {
            sphere.alpha += 1;
        }
        for pair in 0..3 {
            sphere.wander_by(pair * 2, 1.0, 1.0);
        }
        let plain = sphere.flags[0] == 0 || sphere.flags[0] >= 20;
        if plain || sphere.process & 1 == 0 {
            sphere.heights[6] += 1.0;
            if sphere.heights[6] >= 360.0 {
                sphere.heights[6] -= 360.0;
            }
        }
        let (off, reach) = match (sphere.heights[10] == 1.0, plain) {
            (true, _) => (0.7, 5.5),
            (false, true) => (1.0, 7.0),
            (false, false) => (0.5, 7.0),
        };
        let turn = |which: usize| sine(sphere.heights[which].trunc()) * off;
        sphere.at[0] = [sine(sphere.heights[6].trunc()) * reach + turn(0), sphere.heights[7] + turn(4), cosine(sphere.heights[6].trunc()) * reach + turn(2)];
        sphere.heights[8] += (sphere.lot(5) + 5) as f32;
        if sphere.heights[8] >= 360.0 {
            sphere.heights[8] -= 360.0;
        }
    }

    /// `RenderLayeredGlow`: nothing while the sphere does not live or shows nothing. Squares of the picture
    /// that face the screen about its place, their light added, each so many times `out` from its middle to
    /// a corner and a twentieth of `out` more or less as its swell goes (`BuildLayeredGlowSquare`):
    ///
    /// - one flagged 21 first has two blue squares, 3 times at half its alpha and 1.7 times; one flagged
    ///   20 one blue square 1.3 times;
    /// - then every one a square of its whole reach and one of half of it over that: blue under yellow for a
    ///   plain sphere (which together show white in a blue rim), a dark brown under white for a coin, a
    ///   paler blue under a pale blue or nearly white for those flagged 20 and 21.
    pub(super) fn layered_glow(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let Some(sphere) = self.parts.first().filter(|sphere| sphere.live && sphere.alpha > 0) else { return Vec::new() };
        let swell = sphere.out * sine(sphere.heights[8].trunc()) * 0.05;
        let square = |times: f32, alpha: i32, colour: [u8; 3]| Rect::new(
            self.picture,
            alpha,
            Part::facing(sphere.at[0], swell + sphere.out * times, 0, right, up),
            (0.0, 1.0),
            colour,
            true
        );
        const BLUE: [u8; 3] = [0, 0, 255];
        let mut rects = match sphere.flags[0] {
            21 => vec![square(3.0, sphere.alpha / 2, BLUE), square(1.7, sphere.alpha, BLUE)],
            20 => vec![square(1.3, sphere.alpha, BLUE)],
            _ => Vec::new(),
        };
        let (whole, half) = match sphere.flags[0] {
            1 => ([68, 42, 30], [255, 255, 255]),
            20 => ([40, 40, 255], [150, 150, 255]),
            21 => ([40, 40, 255], [250, 250, 255]),
            _ => (BLUE, [255, 255, 0]),
        };
        rects.extend([square(1.0, sphere.alpha, whole), square(0.5, sphere.alpha, half)]);
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    /// A sphere as `CRagEffect::ElementalOrbit` launches it: so far round, so high, reaching so far, flagged.
    fn sphere(round: f32, high: f32, out: f32, near: bool, flag: i8) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::DriftingOrbs), "thunder_center.bmp");
        prim.duration = 999_999;
        let mut part = Part { out, seed: 7, ..Part::default() };
        (part.heights[6], part.heights[7], part.heights[10], part.flags[0]) = (round, high, if near { 1.0 } else { 0.0 }, flag);
        prim.parts.push(part);
        prim
    }

    /// How far the sphere is from the one it is about along the ground, and how high over the feet.
    fn stands(prim: &Prim) -> (f32, f32) {
        let at = prim.parts[0].at[0];
        (f32::hypot(at[0], at[2]), -at[1])
    }

    #[test]
    fn a_sphere_comes_up_and_goes_round_seven_from_the_one_it_is_about() {
        let mut prim = sphere(90.0, -15.0, 2.0, false, 0);
        assert!(prim.rects(RIGHT, UP).is_empty());
        for _ in 0..10 {
            prim.tick(999_999);
        }
        // A degree round every tick; its turns wander within a degree of nothing, so it is 7 out and 15 up
        // to within the sine of that.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].heights[6]), (10, 100.0));
        let (out, high) = stands(&prim);
        assert!((out - 7.0).abs() < 1.5 && (high - 15.0).abs() < 1.01);
        for _ in 10..400 {
            prim.tick(999_999);
        }
        assert_eq!((prim.parts[0].alpha, prim.parts[0].heights[6]), (200, 130.0));
    }

    #[test]
    fn a_coin_goes_round_every_other_tick_and_a_baby_s_sphere_nearer() {
        let (mut coin, mut small) = (sphere(0.0, -15.0, 1.0, false, 1), sphere(0.0, -14.0, 1.3, true, 0));
        for _ in 0..10 {
            coin.tick(999_999);
            small.tick(999_999);
        }
        assert_eq!((coin.parts[0].heights[6], small.parts[0].heights[6]), (5.0, 10.0));
        let (out, high) = stands(&small);
        assert!((out - 5.5).abs() < 1.0 && (high - 14.0).abs() < 0.71);
        assert!((stands(&coin).0 - 7.0).abs() < 0.71);
    }

    #[test]
    fn it_is_two_squares_facing_the_screen_one_half_the_other() {
        let mut prim = sphere(0.0, -15.0, 2.0, false, 0);
        prim.tick(999_999);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 2);
        assert_eq!(
            (rects[0].colour, rects[1].colour, rects[0].alpha, rects[0].adds, rects[0].picture),
            ([0, 0, 255], [255, 255, 0], 1.0, true, "thunder_center.bmp")
        );
        // From its middle to a corner: its reach and a twentieth of it more or less; the second half of that.
        let reach = |rect: &Rect| (rect.corners[0].0[0] - rect.corners[2].0[0]).abs() / 2.0;
        let swell = 2.0 * sine(prim.parts[0].heights[8].trunc()) * 0.05;
        assert!((reach(&rects[0]) - (2.0 + swell)).abs() < 1e-4 && (reach(&rects[1]) - (1.0 + swell)).abs() < 1e-4);
        // Both about the sphere's own place (here y is up).
        let at = prim.parts[0].at[0];
        let middle = |rect: &Rect| std::array::from_fn::<f32, 3, _>(|axis| (rect.corners[0].0[axis] + rect.corners[2].0[axis]) / 2.0);
        assert!(rects.iter().all(|rect| middle(rect).iter().zip([at[0], -at[1], at[2]]).all(|(one, other)| (one - other).abs() < 1e-4)));
    }

    #[test]
    fn a_coin_and_the_spheres_that_stand_for_more_have_colours_of_their_own() {
        let colours = |flag: i8| {
            let mut prim = sphere(0.0, -15.0, 2.0, false, flag);
            prim.tick(999_999);
            prim.tick(999_999);
            prim.rects(RIGHT, UP).iter().map(|rect| (rect.colour, rect.alpha)).collect::<Vec<_>>()
        };
        assert_eq!(colours(1), [([68, 42, 30], 2.0), ([255, 255, 255], 2.0)]);
        assert_eq!(colours(20), [([0, 0, 255], 2.0), ([40, 40, 255], 2.0), ([150, 150, 255], 2.0)]);
        assert_eq!(colours(21), [([0, 0, 255], 1.0), ([0, 0, 255], 2.0), ([40, 40, 255], 2.0), ([250, 250, 255], 2.0)]);
    }
}
