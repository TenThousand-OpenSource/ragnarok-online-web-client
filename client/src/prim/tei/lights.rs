//! The kind 145 of the client's primitives: a light that goes round the one its effect is on, wandering a
//! little, with four upright squares of light about it that grow and begin anew (`PrimDriftingSparks`, drawn
//! by `RenderPillarSquares`): the lights of Duple Light.
//!
//! The light is the first part's (`at[0]`); all four parts are squares about it, whether they are said to
//! live or not, as the client has it.

use super::*;

impl Prim {
    /// `PrimDriftingSparks`. The first part's count goes up, and over its first 200 ticks it comes up by 1 a
    /// tick. Its three turns wander (a degree a tick either way), and every other tick it goes a degree on
    /// round the one the effect is on (`heights[6]`): it is 7 from that one, as high as it was put
    /// (`heights[7]`), and off that place by the sines of its three turns, halved for a part whose `flags[0]`
    /// is set. Its swell (`heights[8]`) goes on 5 to 9 degrees a tick.
    ///
    /// Each part's square, once its count is over nothing: its breath (`round`) goes on a degree a tick and
    /// it grows by a fiftieth (`tall`); grown past 6 it fades by 3 a tick (`rise`), and faded out begins
    /// anew, 3 tall, at 200, turned by lot. How tall it shows (`heights[12]`) is nine tenths of that and up
    /// to a tenth more or less as its breath goes.
    pub(super) fn drift_as_lights(&mut self) {
        let Some(light) = self.parts.first_mut().filter(|light| light.live) else { return };
        light.process += 1;
        if light.process <= 200 {
            light.alpha += 1;
        }
        for pair in 0..3 {
            light.wander_by(pair * 2, 1.0, 1.0);
        }
        if light.process & 1 == 0 {
            light.heights[6] += 1.0;
            if light.heights[6] >= 360.0 {
                light.heights[6] -= 360.0;
            }
        }
        let off = if light.flags[0] == 0 { 1.0 } else { 0.5 };
        let turn = |which: usize| sine(light.heights[which].trunc()) * off;
        light.at[0] = [sine(light.heights[6].trunc()) * 7.0 + turn(0), light.heights[7] + turn(4), cosine(light.heights[6].trunc()) * 7.0 + turn(2)];
        light.heights[8] += (light.lot(5) + 5) as f32;
        if light.heights[8] >= 360.0 {
            light.heights[8] -= 360.0;
        }
        for (index, square) in self.parts.iter_mut().enumerate().take(4) {
            if index != 0 {
                square.process += 1;
            }
            if square.process <= 0 {
                continue;
            }
            square.round = (square.round + 1) % 360;
            square.tall *= 1.02;
            if square.tall > 6.0 {
                square.rise -= 3;
                if square.rise <= 0 {
                    (square.tall, square.turned, square.rise) = (3.0, square.lot(360), 200);
                }
            }
            square.heights[12] = (square.tall * sine(square.round as f32) * 0.1 + square.tall * 0.9).max(0.0);
        }
    }

    /// `RenderPillarSquares`: nothing while the first part does not live or shows nothing. The light is two
    /// squares of the first picture that face the screen about its place, their light added: one `out` from
    /// its middle to a corner and a twentieth more or less as its swell goes, in a deep blue, and one of
    /// half that reach in a pale blue. Then each part's square of the second picture: upright in the world
    /// about the light, as wide as it is tall (`heights[12]`), turned about the upright as the part is
    /// (`turned`), as see-through as the part has faded (`rise`), each part in a blue of its own.
    pub(super) fn pillar_squares(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let Some(light) = self.parts.first().filter(|light| light.live && light.alpha > 0) else { return Vec::new() };
        let picture = |which: usize| self.pictures.get(which).copied().unwrap_or(self.picture);
        let swell = light.out * sine(light.heights[8].trunc()) * 0.05;
        let mut rects: Vec<Rect> = [(swell + light.out, [68, 42, 255]), (swell + light.out * 0.5, [155, 155, 255])].into_iter().map(|(reach, colour)| Rect::new(picture(0), light.alpha, Part::facing(light.at[0], reach, 0, right, up), (0.0, 1.0), colour, true)).collect();
        const SHADES: [u8; 4] = [105, 35, 75, 5];
        for (square, shade) in self.parts.iter().zip(SHADES).filter(|(square, _)| square.heights[12] > 0.0) {
            let (tall, half) = (square.heights[12], square.heights[12] * 0.5);
            let side = |turned: i32, down: f32| [light.at[0][0] + cosine(turned as f32) * half, light.at[0][1] + down, light.at[0][2] + sine(turned as f32) * half];
            let (near, far) = (square.turned, square.turned + if square.turned < 180 { 180 } else { -180 });
            let corners = [side(near, half), side(far, half), side(far, half - tall), side(near, half - tall)];
            rects.push(Rect::new(picture(1), square.rise, corners, (0.0, 1.0), [shade, shade, 255], true));
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
    const UP: [f32; 3] = [0.0, 1.0, 0.0];

    /// A light as `CRagEffect::ThunderFreeze` launches it: 15 over the feet, its squares begun 20 ticks apart.
    fn light(round: f32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::DriftingSparks), "thunder_center.bmp");
        prim.pictures = &["thunder_center.bmp", "freeze_a.bmp"];
        prim.duration = 999_999;
        prim.parts.extend((0..4).map(|index| Part { live: index == 0, process: -20 * index, tall: 3.0, rise: 200, turned: 90, seed: 7, ..Part::default() }));
        let first = &mut prim.parts[0];
        (first.heights[6], first.heights[7], first.out, first.flags[0]) = (round, -15.0, 1.0, 1);
        prim
    }

    #[test]
    fn a_light_comes_up_and_goes_round_seven_from_the_one_it_is_on() {
        let mut prim = light(90.0);
        assert!(prim.rects(RIGHT, UP).is_empty());
        for _ in 0..10 {
            prim.tick(999_999);
        }
        let first = &prim.parts[0];
        // A degree round every other tick; its turns at nothing, it is 7 out and 15 up.
        assert_eq!((first.alpha, first.heights[6]), (10, 95.0));
        assert!((first.at[0][0] - 7.0 * sine(95.0)).abs() < 0.51 && (first.at[0][1] + 15.0).abs() < 0.51 && (first.at[0][2] - 7.0 * cosine(95.0)).abs() < 0.51);
        for _ in 10..400 {
            prim.tick(999_999);
        }
        assert_eq!(prim.parts[0].alpha, 200);
    }

    #[test]
    fn its_squares_begin_one_after_the_other_grow_fade_and_begin_anew() {
        let mut prim = light(0.0);
        prim.tick(999_999);
        // The light's two squares facing the screen, and the first part's upright one.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 3);
        assert_eq!((rects[0].picture, rects[0].colour, rects[1].colour, rects[0].adds), ("thunder_center.bmp", [68, 42, 255], [155, 155, 255], true));
        assert_eq!((rects[2].picture, rects[2].colour, rects[2].alpha), ("freeze_a.bmp", [105, 105, 255], 200.0));
        // Turned a quarter, the square runs north and south of the light, as wide as it is tall.
        let tall = prim.parts[0].heights[12];
        let (one, other, low) = (rects[2].corners[0].0, rects[2].corners[1].0, rects[2].corners[2].0);
        assert!((one[2] - other[2] - tall).abs() < 1e-4 && (low[1] - other[1] - tall).abs() < 1e-4);
        for _ in 1..21 {
            prim.tick(999_999);
        }
        assert_eq!(prim.rects(RIGHT, UP).len(), 4);
        // 3 grows past 6 in 36 ticks of a fiftieth each, then fades by 3 a tick from 200 and begins anew.
        for _ in 21..36 {
            prim.tick(999_999);
        }
        assert!(prim.parts[0].tall > 6.0 && prim.parts[0].rise == 197);
        for _ in 0..66 {
            prim.tick(999_999);
        }
        assert_eq!((prim.parts[0].tall, prim.parts[0].rise), (3.0, 200));
    }
}
