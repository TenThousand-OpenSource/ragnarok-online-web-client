//! The lights of the aura of the highest level. They are not kinds of `CEffectPrim`: the client has a class
//! of its own for each effect (`CLevel99Effect`, `CLevel99Orb1Effect`, `CLevel99Orb2Effect`,
//! `Contents/Effects/Skills`), which keeps particles (`CParticle`), moves them in its `OnProcess` once a tick
//! and draws them in its `Render`. Here each is a kind of this file's list, its particles kept in parts, so
//! that they live and are drawn as every other primitive is; which figure of a part stands for which of a
//! particle's is said at each kind:
//!
//! - `Level99Glow`: the two squares of light lying over the ground under one, which breathe (the `sparkles`
//!   of `CLevel99Effect`);
//! - `Level99Swirl`: the three bands of light that go round one, each rising and falling once on its way
//!   round (the `sprites` of `CLevel99Effect`, built by `AddRings`);
//! - `Level99Orb`: the sixteen lights that rise about one, wandering, and go out (`CLevel99Orb1Effect`; and
//!   `CLevel99Orb2Effect`, whose lights are smaller and turned over).
//!
//! How a particle is drawn (`CParticle::Render` with its `billboard` off): its four corners, (-1, 1, 1),
//! (1, 1, 1), (-1, -1, 1) and (1, -1, 1) unless the effect wrote others, go through the particle's own
//! `transform`, are turned about the upright by the camera's yaw (`AppendYRotation`) and put at the place of
//! the one the effect is on. Its light is added to what is behind it (`sourceBlend` 5, `destinationBlend` 2),
//! in its colour at its alpha. So what a particle's `transform` leaves along its x lies along the screen's
//! right on the ground, and along its z away from the eye on the ground (`yawed`).
//!
//! What is not as in the client: the lots of a light that begins anew are drawn from the part's own number
//! (`Part::lot`), where the client draws from its one `rand`; and the sines of a wandering light's two turns
//! are those of the turn itself where it has gone under nothing or past 359 (the client reads its table of
//! 360 sines outside it there, whatever lies beside it).

use super::*;

/// A place in the frame a particle is drawn in (its x across the screen on the ground, its y down, its z
/// away from the eye on the ground: the client's directions turned by the camera's yaw), in the client's
/// own directions from the effect's place. `right` is the screen's right in the world (east, up, north):
/// `matrix::MakeYRotation` takes x to `(cos, 0, -sin)` of the yaw and z to `(sin, 0, cos)`, which are this
/// camera's right and the way it looks along the ground (`prim/facing.rs`).
fn yawed(right: [f32; 3], [x, y, z]: [f32; 3]) -> [f32; 3] {
    [x * right[0] - z * right[2], y, x * right[2] + z * right[0]]
}

/// The four corners of a particle as `Rect::new` takes them (the picture's bottom left, bottom right, top
/// right, top left), each made of a corner of the particle's own square: `SetQuad` gives the corners with
/// y of 1 the picture's top.
fn square(corner: impl Fn(f32, f32) -> [f32; 3]) -> [[f32; 3]; 4] {
    [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)]
}

impl Prim {
    /// Its colour as a rectangle's.
    fn tint(&self) -> [u8; 3] {
        self.colour.map(|part| part as u8)
    }

    /// The rectangles of the three kinds of this file.
    pub(super) fn level99_glows(&self, right: [f32; 3]) -> Vec<Rect> {
        self.parts.iter().filter(|part| part.live).map(|part| part.level_glow(self.picture, self.tint(), right)).collect()
    }

    pub(super) fn level99_swirls(&self, right: [f32; 3]) -> Vec<Rect> {
        self.parts.iter().filter(|part| part.live).flat_map(|part| part.swirl(self.picture, self.tint(), right)).collect()
    }

    pub(super) fn level99_orbs(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        self.parts.iter().filter(|part| part.live).flat_map(|part| part.orb(self.picture, self.tint(), right, up)).collect()
    }
}

impl Part {
    /// `CLevel99Effect::OnProcess`, for one of its `sparkles`: its breath (`rise`, the particle's
    /// `phaseParameter`) goes on 3 degrees a tick.
    pub(super) fn breathe_as_glow(&mut self) {
        self.rise = under_a_round(self.rise + 3);
    }

    /// One of the `sparkles` as `CLevel99Effect::OnProcess` leaves its `transform` and `Render` draws it: the
    /// particle's square made `size` times as large, turned in its own plane by the particle's `angle`
    /// (`turned`; `MakeZRotation`) and laid level (`AppendXRotation` by a quarter, which takes its z of 1 to 1
    /// over the ground). `size` is the particle's `scale` (`out`) and up to two tenths more as the sine of
    /// its breath says.
    fn level_glow(&self, picture: &'static str, colour: [u8; 3], right: [f32; 3]) -> Rect {
        let size = (sine(self.rise as f32) + 1.0) * (self.out * 0.1) + self.out;
        let (sin, cos) = (self.turned as f32).to_radians().sin_cos();
        let corner = |x: f32, y: f32| yawed(right, [(x * cos - y * sin) * size, -1.0, (x * sin + y * cos) * size]);
        Rect::new(picture, self.alpha, square(corner), (0.0, 1.0), colour, true)
    }

    /// `CLevel99Effect::OnProcess`, for the particles of one band of its `sprites` (all those of a band turn
    /// alike, so one part stands for the band): the turn (`turned`, the particle's `angle`) goes on 3 degrees
    /// a tick and one more for each band before it (`flags[0]`, the particle's `rowIndex`).
    pub(super) fn turn_as_swirl(&mut self) {
        self.turned += i32::from(self.flags[0]) + 3;
    }

    /// One band as `CLevel99Effect::AddRings` builds it and `Render` draws it: 22 edges, the one of each
    /// number a 20th of the band's reach round (`round`, the particle's `angleParameter` of 315: 15 degrees)
    /// further on from where the band begins (`heights[0]`). An edge's foot is `out` (the particle's
    /// `scale`) from the middle at the level of the feet; its top lies further out and up along a slope of
    /// `rise` degrees (`phaseParameter`), as far as the sine of 9 degrees for each edge before it times
    /// `tall` (`speedParameter`): the band comes up from nothing, is tallest at its tenth edge and back at
    /// nothing at its 20th. Between each two edges a piece shows its 21st of the picture's width (the first
    /// particle draws nothing: `columnIndex > 0`). The whole is turned about the upright by `turned`
    /// (`MakeYRotation`).
    fn swirl(&self, picture: &'static str, colour: [u8; 3], right: [f32; 3]) -> Vec<Rect> {
        let (sin, cos) = (self.turned.rem_euclid(360) as f32).to_radians().sin_cos();
        let turned = |[x, y, z]: [f32; 3]| yawed(right, [x * cos + z * sin, y, -x * sin + z * cos]);
        let edge = |index: i32| {
            let reach = sine((index * 9) as f32) * self.tall;
            let angle = (((self.round / 20) * index) as f32 + self.heights[0]) as i32 % 360;
            let (across, along) = (cosine(angle as f32), sine(angle as f32));
            let (spread, height) = (reach * cosine(self.rise as f32), reach * sine(self.rise as f32));
            let foot = [self.out * across, 0.0, self.out * along];
            (turned(foot), turned([foot[0] + spread * across, -height, foot[2] + spread * along]))
        };
        (1..=21)
            .map(|index| {
                let ((foot_before, top_before), (foot, top)) = (edge(index - 1), edge(index));
                Rect::new(picture, self.alpha, [foot_before, foot, top, top_before], ((index - 1) as f32 / 21.0, index as f32 / 21.0), colour, true)
            })
            .collect()
    }

    /// The client's `Random` of these effects: `rand() % range`, nothing for a range of nothing.
    fn random(&mut self, range: i32) -> i32 {
        if range == 0 { 0 } else { self.lot(range.unsigned_abs()) }
    }

    /// A light begins anew (`CLevel99Orb1Effect::Open`, and `OnProcess` for one that has gone out): at the
    /// middle, up to 99 under the feet, its two turns and their two marks by lot, showing nothing.
    pub fn begin_as_orb(&mut self) {
        self.at[0] = [0.0, self.random(100) as f32, 0.0];
        for turn in 0..4 {
            self.heights[turn] = self.random(360) as f32;
        }
        self.alpha = 0;
    }

    /// `CLevel99Orb1Effect::OnProcess` (and `CLevel99Orb2Effect`'s, which is the same), for one light: where
    /// it is from the effect's place is `at[0]` (the effect's `offsets`), its two turns and their marks
    /// `heights[0..4]` (the particle's `animationParameter` and the three after it), its speed `tall`
    /// (`speedParameter`).
    ///
    /// It rises by its speed every tick. While it is under the feet nothing else happens: it waits there,
    /// showing nothing. Over them, each of its two turns goes 3 degrees a tick up towards its mark or 2 down
    /// from it, and within 3 of the mark gets a new one by lot (under itself if it was going up, else as far
    /// under 359); it drifts by its speed times the sine of the first turn across and of the second away.
    /// It shows whole up to 20 over the feet, fades by 4 a tick above that, and past 30 begins anew.
    pub(super) fn rise_as_orb(&mut self) {
        let speed = self.tall;
        if self.at[0][1] < 0.0 {
            if self.at[0][1] < -30.0 {
                self.begin_as_orb();
            } else {
                for turn in 0..2 {
                    let current = self.heights[turn] as i32;
                    let delta = self.heights[turn + 2] as i32 - current;
                    self.heights[turn] = (current + if delta > 0 { 3 } else { -2 }) as f32;
                    if delta.abs() < 3 {
                        self.heights[turn + 2] = if delta > 0 { self.random(current) } else { 359 - self.random(current) } as f32;
                    }
                }
                self.at[0][0] += speed * sine(self.heights[0]);
                self.at[0][2] += speed * sine(self.heights[1]);
                if self.at[0][1] < -20.0 {
                    self.alpha = (self.alpha - 4).max(0);
                } else if self.alpha != 255 {
                    self.alpha = 255;
                }
            }
        }
        self.at[0][1] -= speed;
    }

    /// One light as `OnProcess` leaves its `transform` and `Render` draws it, twice over: the particle's
    /// square `out` times as large (its `scale`), tipped by the camera's pitch (`MakeXRotation`) and so,
    /// turned by the yaw as every particle is, facing the screen; put where the light is, which is a place of
    /// the frame turned by the yaw alone. The square's z of 1 puts it 1 further from the eye than that place.
    /// A light of `CLevel99Orb2Effect` (`flags[0]` set) is turned over first (`MakeXRotation` by half a turn):
    /// the other way up, and 1 nearer the eye instead.
    ///
    /// Tipped by the pitch and turned by the yaw, the particle's x is the screen's right, its y the screen's
    /// down and its z the way the eye looks (`CView::Refresh` makes the eye's place of the same two turns).
    fn orb(&self, picture: &'static str, colour: [u8; 3], right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let over = if self.flags[0] != 0 { -1.0 } else { 1.0 };
        // The way the eye looks, in the world's directions (east, up, north): the screen's right crossed with
        // its up.
        let ahead = [right[1] * up[2] - right[2] * up[1], right[2] * up[0] - right[0] * up[2], right[0] * up[1] - right[1] * up[0]];
        let light = yawed(right, self.at[0]);
        let middle = [light[0] + ahead[0] * over, light[1] - ahead[1] * over, light[2] + ahead[2] * over];
        let corner = |x: f32, y: f32| to_screen(middle, x * self.out, y * over * self.out, right, up);
        vec![Rect::new(picture, self.alpha, square(corner), (0.0, 1.0), colour, true); 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EAST: [f32; 3] = [1.0, 0.0, 0.0];
    use crate::prim::testing::UP;

    fn near(one: [f32; 3], other: [f32; 3]) -> bool {
        one.iter().zip(other).all(|(one, other)| (one - other).abs() < 0.001)
    }

    /// The two squares as `CLevel99Effect::Open` makes them: 9 large at an alpha of 200, the second turned 22
    /// degrees and half a breath on.
    fn glows() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Level99Glow), "pikapika2.bmp");
        prim.duration = 99_999_999;
        prim.parts.extend((0..2).map(|index| Part { turned: index * 22, rise: index * 180, out: 9.0, alpha: 200, ..Part::default() }));
        prim
    }

    /// The three bands as `CLevel99Effect::AddRings` makes them.
    fn swirls() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Level99Swirl), "ring_blue.tga");
        (prim.duration, prim.colour) = (99_999_999, [100.0, 100.0, 255.0]);
        prim.parts.extend((0..3).map(|band| {
            let mut part = Part { turned: band * 90, round: 315, rise: 55 - band * 5, tall: 15.0 - band as f32 * 2.0, out: band as f32 * 0.2 + 3.9, alpha: 120, ..Part::default() };
            (part.flags[0], part.heights[0]) = (band as i8, (band * 90) as f32);
            part
        }));
        prim
    }

    /// One light of `CLevel99Orb1Effect` so far under the feet (over them for a height under nothing).
    fn orb(under: f32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Level99Orb), "whitelight.tga");
        (prim.duration, prim.colour) = (99_999_999, [80.0, 80.0, 255.0]);
        let mut part = Part { tall: 0.15, out: 1.8, seed: 7, ..Part::default() };
        part.at[0][1] = under;
        prim.parts.push(part);
        prim
    }

    #[test]
    fn the_squares_lie_one_over_the_ground_and_breathe() {
        let mut prim = glows();
        let rects = prim.rects(EAST, UP);
        assert_eq!(rects.len(), 2);
        assert_eq!((rects[0].picture, rects[0].colour, rects[0].alpha, rects[0].adds), ("pikapika2.bmp", [255; 3], 200.0, true));
        // The first begins a tenth over its 9; its bottom left corner is to the west and the south, 1 up.
        assert!(near(rects[0].corners[0].0, [-9.9, 1.0, -9.9]) && near(rects[0].corners[2].0, [9.9, 1.0, 9.9]));
        assert_eq!((rects[0].corners[0].1, rects[0].corners[2].1), ([0.0, 1.0], [1.0, 0.0]));
        for _ in 0..30 {
            prim.tick(99_999_999);
        }
        // A quarter of a breath on the first is two tenths over its 9, the second, half a breath apart, at 9.
        assert_eq!((prim.parts[0].rise, prim.parts[1].rise), (90, 270));
        let rects = prim.rects(EAST, UP);
        assert!(near(rects[0].corners[1].0, [10.8, 1.0, -10.8]));
        // The second is turned 22 degrees in its plane: its corners are 9 times the root of two out.
        let corner = rects[1].corners[1].0;
        assert!((f32::hypot(corner[0], corner[2]) - 9.0 * 2f32.sqrt()).abs() < 0.001 && (corner[2].atan2(corner[0]).to_degrees() + 23.0).abs() < 0.001);
        // Seen from the east side (the screen's right to the south), the same square lies turned with the eye.
        assert!(near(prim.rects([0.0, 0.0, -1.0], UP)[0].corners[1].0, [-10.8, 1.0, -10.8]));
    }

    #[test]
    fn a_band_rises_and_falls_on_its_way_round_and_turns() {
        let mut prim = swirls();
        let rects = prim.rects(EAST, UP);
        assert_eq!(rects.len(), 63);
        assert_eq!((rects[0].colour, rects[0].alpha, rects[0].adds), ([100, 100, 255], 120.0, true));
        // The first piece shows the first 21st of the picture; its first edge has no height.
        assert_eq!((rects[0].corners[0].1, rects[0].corners[1].1, rects[0].corners[2].1), ([0.0, 1.0], [1.0 / 21.0, 1.0], [1.0 / 21.0, 0.0]));
        assert!(near(rects[0].corners[0].0, [3.9, 0.0, 0.0]) && near(rects[0].corners[3].0, [3.9, 0.0, 0.0]));
        // The tenth edge, 150 degrees round, is the tallest: 15 along a slope of 55 degrees.
        let (out, tall) = (3.9 + 15.0 * 55f32.to_radians().cos(), 15.0 * 55f32.to_radians().sin());
        assert!(near(rects[9].corners[2].0, [out * 150f32.to_radians().cos(), tall, out * 150f32.to_radians().sin()]));
        // The 20th is back at the ground, and the last piece ends 315 degrees round.
        assert!(rects[19].corners[2].0[1].abs() < 0.001);
        assert!(near(rects[20].corners[1].0, [3.9 * 315f32.to_radians().cos(), 0.0, 3.9 * 315f32.to_radians().sin()]));
        // The second band is built a quarter round and begins turned by as much again (to the east: the
        // turn goes from the north by the east), a little wider.
        assert!(near(rects[21].corners[0].0, [4.1, 0.0, 0.0]));
        prim.tick(99_999_999);
        // A tick on the bands have turned 3, 4 and 5 degrees (`MakeYRotation`: from the east by the south).
        assert_eq!((prim.parts[0].turned, prim.parts[1].turned, prim.parts[2].turned), (3, 94, 185));
        assert!(near(prim.rects(EAST, UP)[0].corners[0].0, [3.9 * 3f32.to_radians().cos(), 0.0, -3.9 * 3f32.to_radians().sin()]));
    }

    #[test]
    fn a_light_waits_under_the_feet_then_rises_fades_and_begins_anew() {
        let mut prim = orb(0.3);
        // Under the feet it only rises, and shows nothing.
        for _ in 0..2 {
            prim.tick(99_999_999);
        }
        assert!((prim.parts[0].at[0][1] - 0.0).abs() < 0.001 && prim.parts[0].alpha == 0);
        assert!(prim.rects(EAST, UP).iter().all(|rect| rect.alpha == 0.0));
        // Over them it shows whole and wanders.
        for _ in 0..3 {
            prim.tick(99_999_999);
        }
        assert_eq!(prim.parts[0].alpha, 255);
        let rects = prim.rects(EAST, [0.0, 0.6, 0.8]);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds), (2, [80, 80, 255], true));
        // Past 20 over the feet it fades by 4 a tick.
        prim.parts[0].at[0][1] = -20.1;
        prim.tick(99_999_999);
        assert_eq!(prim.parts[0].alpha, 251);
        // Past 30 it begins anew: at the middle, under the feet, showing nothing.
        prim.parts[0].at[0][1] = -30.1;
        prim.tick(99_999_999);
        let light = &prim.parts[0];
        assert!(light.alpha == 0 && light.at[0][0] == 0.0 && light.at[0][2] == 0.0 && (-0.15..99.0).contains(&light.at[0][1]));
        assert!(light.heights[..4].iter().all(|turn| (0.0..360.0).contains(turn)));
    }

    #[test]
    fn a_turn_of_a_light_goes_to_its_mark_and_gets_a_new_one() {
        let mut prim = orb(-1.0);
        (prim.parts[0].heights[0], prim.parts[0].heights[2], prim.parts[0].heights[1], prim.parts[0].heights[3]) = (90.0, 100.0, 90.0, 50.0);
        prim.tick(99_999_999);
        let light = &prim.parts[0];
        // The first goes 3 up towards its mark, the second 2 down from its own; neither is within 3 yet.
        assert_eq!((light.heights[0], light.heights[1], light.heights[2], light.heights[3]), (93.0, 88.0, 100.0, 50.0));
        // It drifts by its speed times the sines of the two: across, and away from the eye.
        assert!((light.at[0][0] - 0.15 * 93f32.to_radians().sin()).abs() < 0.0001 && (light.at[0][2] - 0.15 * 88f32.to_radians().sin()).abs() < 0.0001);
        let mut prim = orb(-1.0);
        (prim.parts[0].heights[0], prim.parts[0].heights[2], prim.parts[0].heights[1], prim.parts[0].heights[3]) = (98.0, 100.0, 51.0, 50.0);
        prim.tick(99_999_999);
        let light = &prim.parts[0];
        // Within 3 of its mark going up, the new mark is under where it was; going down, over 359 less that.
        assert!(light.heights[0] == 101.0 && (0.0..98.0).contains(&light.heights[2]));
        assert!(light.heights[1] == 49.0 && (309.0..=359.0).contains(&light.heights[3]));
    }

    #[test]
    fn a_light_faces_the_screen_one_behind_its_place_and_the_other_kind_is_turned_over() {
        // The eye looks north and down: the screen's up leans north.
        let up = [0.0, 0.6, 0.8];
        let mut prim = orb(-10.0);
        prim.parts[0].alpha = 255;
        (prim.parts[0].at[0][0], prim.parts[0].at[0][2]) = (2.0, 3.0);
        let rect = &prim.rects(EAST, up)[0];
        // Its middle: 2 east, 10 up, 3 north, and 1 along the way the eye looks (north and down).
        let middle = [2.0, 10.0 - 0.8, 3.0 + 0.6];
        // The particle's y points down the screen: the picture's bottom is at the top of the screen.
        assert!(near(rect.corners[0].0, [middle[0] - 1.8, middle[1] + 1.8 * 0.6, middle[2] + 1.8 * 0.8]));
        assert!(near(rect.corners[2].0, [middle[0] + 1.8, middle[1] - 1.8 * 0.6, middle[2] - 1.8 * 0.8]));
        // The other kind: 1 nearer the eye, and turned over, its picture's bottom at the bottom.
        (prim.parts[0].flags[0], prim.parts[0].out) = (1, 0.55);
        let rect = &prim.rects(EAST, up)[0];
        let middle = [2.0, 10.0 + 0.8, 3.0 - 0.6];
        assert!(near(rect.corners[0].0, [middle[0] - 0.55, middle[1] - 0.55 * 0.6, middle[2] - 0.55 * 0.8]));
    }
}
