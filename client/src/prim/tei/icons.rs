//! The kind 35 and `PP_DOUBLEGUMGANG` (99) of the client's primitives: signs that flash up round the head of
//! one in a fury, each a square facing the screen that comes up, goes, and begins anew at another place
//! (`PrimFloatingBubbles`; drawn by `RenderOrbitIcons` for the kind 35 and by `RenderDoubleGumGang` for the
//! kind 99): Fury, Steel Body, a monster's Power Up, the Warmths.
//!
//! A part is four signs. Each has four figures of the part's `heights`, from `heights[0]`, `[4]`, `[8]` and
//! `[12]` on: its count, its alpha, which of the primitive's five pictures it shows, and where round the one
//! the effect is on it stands, in degrees; and it has one of the part's four places for its height (`vecBPre`,
//! `vecBNow`, `vecTPre`, `vecTNow`). `heights[20]` is how high over the feet the name of the one the effect
//! is on stands (`CGameActor::GetNameOffset`), which the client asks at every tick and the world tells here
//! (`Prim::lifted`).
//!
//! The pictures are drawings of one side of a flame. The client turns a sign over by where it stands as the
//! camera sees it: `FaceCamera` writes 360 less the camera's yaw into the first part, and a sign whose own
//! angle is within 70 over or 80 under that (the round closed) shows its picture one way, any other the
//! other way; the pictures 0 and 2 are drawn the other way round from the pictures 1, 3 and 4.

use super::*;

/// The four signs of a part: where the figures of each begin in `heights`, and which of the part's places is
/// its own.
const SIGNS: [(usize, usize); 4] = [(0, 2), (4, 0), (8, 3), (12, 1)];

/// What `FaceCamera` writes into the first part for the camera's turn: 360 less its yaw in whole degrees, the
/// yaw being from 0 up to 360. `right` is the screen's right in the world (east, up, north).
pub(super) fn screen_turn(right: [f32; 3]) -> i32 {
    360 - camera_yaw(right).rem_euclid(360.0) as i32
}

impl Prim {
    /// Tells a primitive of these kinds how high over the feet the name of the one its effect is on stands
    /// (`PrimFloatingBubbles` asks `GetNameOffset` of its effect's parent for every part that lives, every
    /// tick). Nothing for a primitive of any other kind.
    pub fn lifted(&mut self, offset: f32) {
        if matches!(self.kind, Shape::Tei(Tei::OrbitIcons | Tei::DoubleGumGang)) {
            self.parts.iter_mut().filter(|part| part.live).for_each(|part| part.heights[20] = offset);
        }
    }
}

impl Part {
    /// `PrimFloatingBubbles`, for one part: each of its four signs counts on; under a count of 10 it comes up
    /// by 25 a tick, and from then on goes down by 12. Once under 12 it begins anew at nothing: at another
    /// angle and with another picture by lot, 3 to 10 over the level it is measured from.
    pub(super) fn float_as_signs(&mut self) {
        self.process += 1;
        for (first, place) in SIGNS {
            self.heights[first] += 1.0;
            if self.heights[first] < 10.0 {
                self.heights[first + 1] += 25.0;
                continue;
            }
            self.heights[first + 1] -= 12.0;
            if self.heights[first + 1] < 12.0 {
                (self.heights[first], self.heights[first + 1]) = (0.0, 0.0);
                self.heights[first + 3] = self.lot(360) as f32;
                self.heights[first + 2] = self.lot(5) as f32;
                self.at[place][1] = (-3 - self.lot(8)) as f32;
            }
        }
    }

    /// `RenderOrbitIcons` and `RenderDoubleGumGang`, for one part: every sign a square facing the screen, 4
    /// out from the one the effect is on at the sign's angle and as high as its place says over where that
    /// one's name stands, its light added. A sign is drawn once for each of `layers`, a colour and how many
    /// times the part's reach (`out`) it is from its middle to a corner; `capped` holds its alpha to the part's
    /// count (`RenderDoubleGumGang`). `turn` is `screen_turn`. (The client draws them a touch nearer than
    /// they stand; here they are drawn where they stand.)
    pub(super) fn signs(&self, pictures: &[&'static str], turn: i32, layers: &[([u8; 3], f32)], capped: bool, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let mut rects = Vec::new();
        for (colour, times) in layers {
            for (first, place) in SIGNS {
                let angle = self.heights[first + 3] as i32;
                let number = self.heights[first + 2] as i32;
                let Some(picture) = pictures.get(number as usize) else { continue };
                let middle = [cosine(angle as f32) * 4.0, self.at[place][1] - self.heights[20], sine(angle as f32) * 4.0];
                let [first_corner, second, third, fourth] = Self::facing(middle, self.out * times, 45, right, up);
                let apart = angle - turn;
                let before = (apart < 70 && apart > -80) || apart < -270 || apart > 270;
                let corners = if matches!(number, 0 | 2) != before { [second, first_corner, fourth, third] } else { [first_corner, second, third, fourth] };
                let alpha = self.heights[first + 1] as i32;
                rects.push(Rect::new(picture, if capped { alpha.min(self.process) } else { alpha }, corners, (0.0, 1.0), *colour, true));
            }
        }
        rects
    }

    /// `RenderOrbitIcons` as the kind 35 calls it: the signs once, blue, or red for a part whose `flags[4]` is
    /// 1 (a monster's). (In the client a red part makes every part after it red too; no effect mixes them.)
    pub(super) fn orbit_icons(&self, pictures: &[&'static str], right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let colour = if self.flags[4] == 1 { [255, 20, 20] } else { [100, 100, 255] };
        self.signs(pictures, screen_turn(right), &[(colour, 1.0)], false, right, up)
    }

    /// `RenderDoubleGumGang`: the signs three times over, the second a fifth larger and the third twice as
    /// large, both darker: blue for a part whose `flags[4]` is 0, grey for 1, red for any other. None shows
    /// more than the part's count, so the whole comes up slowly.
    pub(super) fn double_signs(&self, pictures: &[&'static str], right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let colours = match self.flags[4] {
            0 => [[100, 100, 255], [0, 0, 120], [0, 0, 120]],
            1 => [[90, 90, 80], [60, 60, 50], [30, 30, 20]],
            _ => [[255, 100, 100], [120, 0, 0], [120, 0, 0]],
        };
        self.signs(pictures, screen_turn(right), &[(colours[0], 1.0), (colours[1], 1.2), (colours[2], 2.0)], true, right, up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: [f32; 3] = [0.0, 1.0, 0.0];
    const PICTURES: [&str; 5] = ["super1.bmp", "super2.bmp", "super3.bmp", "super4.bmp", "super5.bmp"];

    /// The screen's right for a camera of that yaw (it looks along (sin, cos) on the ground).
    fn right_at(yaw: f32) -> [f32; 3] {
        let (sin, cos) = yaw.to_radians().sin_cos();
        [cos, 0.0, -sin]
    }

    /// A part as `SuperSlashFan` launches one: its four signs at a count of 10 and alphas of 70 to 250.
    fn part(angle: f32, picture: f32) -> Part {
        let mut part = Part { out: 3.5, seed: 3, ..Part::default() };
        for (number, (first, place)) in SIGNS.into_iter().enumerate() {
            part.heights[first..first + 4].copy_from_slice(&[10.0, 70.0 + 60.0 * number as f32, picture, angle]);
            part.at[place][1] = -5.0;
        }
        part
    }

    #[test]
    fn a_sign_goes_down_begins_anew_and_comes_up() {
        let mut prim = Prim::new(Shape::Tei(Tei::OrbitIcons), PICTURES[0]);
        (prim.duration, prim.pictures) = (99999, &PICTURES);
        prim.parts.push(part(0.0, 1.0));
        prim.tick(99999);
        assert_eq!((prim.parts[0].heights[1], prim.parts[0].heights[13]), (58.0, 238.0));
        for _ in 0..4 {
            prim.tick(99999);
        }
        // Having gone under 12 the first sign is at nothing, with an angle and a picture of the lot and a
        // height of 3 to 10.
        let part = &prim.parts[0];
        assert_eq!((part.heights[0], part.heights[1]), (0.0, 0.0));
        assert!((0.0..360.0).contains(&part.heights[3]) && (0.0..5.0).contains(&part.heights[2]) && (-10.0..=-3.0).contains(&part.at[2][1]));
        for _ in 0..9 {
            prim.tick(99999);
        }
        assert_eq!((prim.parts[0].heights[0], prim.parts[0].heights[1]), (9.0, 225.0));
        prim.tick(99999);
        assert_eq!(prim.parts[0].heights[1], 213.0);
    }

    #[test]
    fn the_signs_stand_round_the_head_over_the_name_and_are_blue_or_red() {
        let mut prim = Prim::new(Shape::Tei(Tei::OrbitIcons), PICTURES[0]);
        (prim.duration, prim.pictures) = (99999, &PICTURES);
        prim.parts.push(part(90.0, 3.0));
        prim.lifted(6.0);
        let rects = prim.rects(right_at(0.0), UP);
        assert_eq!(rects.len(), 4);
        assert_eq!((rects[0].picture, rects[0].colour, rects[0].adds, rects[0].alpha, rects[3].alpha), ("super4.bmp", [100, 100, 255], true, 70.0, 250.0));
        // 4 to the north of the middle, 5 and the 6 of the name over the feet.
        let middle: [f32; 3] = std::array::from_fn(|axis| (rects[0].corners[0].0[axis] + rects[0].corners[2].0[axis]) / 2.0);
        assert!(middle[0].abs() < 1e-3 && (middle[1] - 11.0).abs() < 1e-3 && (middle[2] - 4.0).abs() < 1e-3);
        prim.parts[0].flags[4] = 1;
        assert_eq!(prim.rects(right_at(0.0), UP)[0].colour, [255, 20, 20]);
        // Another kind of primitive is not told.
        let mut other = Prim::new(Shape::Tei(Tei::Aura), "");
        other.parts.push(Part::default());
        other.lifted(6.0);
        assert_eq!(other.parts[0].heights[20], 0.0);
    }

    #[test]
    fn a_sign_is_turned_over_by_where_it_stands_as_the_camera_sees_it() {
        // With a yaw of 0 the camera's turn is 360: a sign at 300 degrees is 60 under it, one at 200 is not
        // near; at 20 the round closes (340 under).
        assert_eq!(screen_turn(right_at(0.0)), 360);
        assert_eq!(screen_turn(right_at(90.0)), 270);
        let first_corner = |angle: f32, picture: f32, yaw: f32| {
            let part = part(angle, picture);
            (part.orbit_icons(&PICTURES, right_at(yaw), UP)[0].corners[0].0, Part::facing([cosine(angle) * 4.0, -5.0, sine(angle) * 4.0], 3.5, 45, right_at(yaw), UP))
        };
        let same = |(corner, square): ([f32; 3], [[f32; 3]; 4]), which: usize| (0..3).all(|axis| (corner[axis] - [square[which][0], -square[which][1], square[which][2]][axis]).abs() < 1e-4);
        // The pictures 0 and 2 keep their corners near the camera's turn and are turned over away from it.
        assert!(same(first_corner(300.0, 0.0, 0.0), 0) && same(first_corner(200.0, 2.0, 0.0), 1) && same(first_corner(20.0, 0.0, 0.0), 0));
        // The pictures 1, 3 and 4 the other way round.
        assert!(same(first_corner(300.0, 1.0, 0.0), 1) && same(first_corner(200.0, 4.0, 0.0), 0));
        // And with the camera turned a quarter the sign at 200 is the near one.
        assert!(same(first_corner(200.0, 2.0, 90.0), 0));
    }

    #[test]
    fn the_signs_of_the_warmths_are_three_layers_that_come_up_with_the_count() {
        let mut prim = Prim::new(Shape::Tei(Tei::DoubleGumGang), PICTURES[0]);
        (prim.duration, prim.pictures) = (99999, &PICTURES);
        let mut warm = part(0.0, 0.0);
        (warm.out, warm.flags[4]) = (2.0, 2);
        prim.parts.push(warm);
        for _ in 0..3 {
            prim.tick(99999);
        }
        let rects = prim.rects(right_at(0.0), UP);
        assert_eq!(rects.len(), 12);
        // Red, dark red twice; none over the count of 3 though the fourth sign's alpha is 214.
        assert_eq!((rects[0].colour, rects[4].colour, rects[8].colour, rects[3].alpha), ([255, 100, 100], [120, 0, 0], [120, 0, 0], 3.0));
        // The third layer reaches twice as far from its middle to a corner.
        let reach = |rect: &Rect| (0..3).map(|axis| (rect.corners[0].0[axis] - rect.corners[2].0[axis]).powi(2)).sum::<f32>().sqrt() / 2.0;
        assert!((reach(&rects[0]) - 2.0).abs() < 1e-3 && (reach(&rects[4]) - 2.4).abs() < 1e-3 && (reach(&rects[8]) - 4.0).abs() < 1e-3);
    }
}
