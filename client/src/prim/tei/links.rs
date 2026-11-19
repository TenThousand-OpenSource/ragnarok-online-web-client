//! `PP_LINELINK` (38) of the client's primitives: a tube of light from the one its effect is on to another
//! actor, which it follows every tick (`CEffectPrim::PrimLineLink`), drawn by `RenderTube`. Only its first
//! part lives.
//!
//! The client keeps the other actor's ID in the part (`height[0]`) and looks that actor up at every tick; here
//! the world tells the primitive where the other is (`Prim::link_to`), and with no one there the part is dead,
//! as in the client.

use super::*;

impl Prim {
    /// Tells a primitive of this kind where the other end of its tube is now, from the place of the one its
    /// effect is on (east, up, north): 8 over the feet of the other actor (`vecTPre`). With no other actor the
    /// part lives no more. Nothing for a primitive of any other kind.
    pub fn link_to(&mut self, other: Option<[f32; 3]>) {
        if self.kind != Shape::Tei(Tei::LineLink) {
            return;
        }
        let Some(part) = self.parts.first_mut() else { return };
        match other {
            Some([east, up, north]) => part.at[3] = [east, -up - 8.0, north],
            None => (part.live, part.alpha) = (false, 0),
        }
    }
}

impl Part {
    /// `PrimLineLink`: the tube's near end is 8 over the feet of the one the effect is on (`vecTNow`).
    /// `heights[4]` says how it comes and goes: 2 comes up by 6 a tick for 19 ticks and after the 40th goes
    /// down by 3; 3 comes up by 1 a tick for 80 ticks while its thickness (`tall`) swells and shrinks; any
    /// other goes up by 2 for five ticks and down by 2 for five, over and over.
    pub(super) fn follow_as_link(&mut self) {
        self.process += 1;
        self.at[1] = [0.0, -8.0, 0.0];
        let kind = self.heights[4];
        if kind == 2.0 {
            if self.process > 40 {
                self.alpha = (self.alpha - 3).max(0);
            } else if self.process < 20 {
                self.alpha += 6;
            }
        } else if kind == 3.0 {
            if self.process <= 80 {
                self.alpha += 1;
            }
            self.tall += sine((self.process % 360) as f32) * 0.005;
        } else if self.process % 10 < 5 {
            self.alpha += 2;
        } else {
            self.alpha -= 2;
        }
    }

    /// `RenderTube`: a tube of ten sides from the near end to the far one, twice: once `tall` thick and once
    /// twice that. For its first 15 ticks the far end is on its way out, as the sine of six degrees a tick.
    /// The kind (`heights[4]`) gives the colours: 2 a dark blue inside red, 3 pink in pink, 4 dark grey mixed
    /// in at half its alpha and at its whole; any other yellow inside blue. But for the kind 4 the light is
    /// added, and the outer tube has twice the alpha (of which the low byte shows).
    pub(super) fn tube(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (near, far) = (self.at[1], self.at[3]);
        let side = ((angle_of(near[0] - far[0], near[2] - far[2]) + 90) % 360) as f32;
        let reach = if self.process >= 15 { 1.0 } else { sine((6 * self.process) as f32) };
        let end: [f32; 3] = std::array::from_fn(|axis| (far[axis] - near[axis]) * reach + near[axis]);
        let kind = self.heights[4];
        let mut rects = Vec::new();
        for outer in [false, true] {
            let radius = if outer { self.tall + self.tall } else { self.tall };
            let (colour, alpha, adds) = match (kind as i32, outer) {
                (2, false) => ([0, 100, 150], self.alpha, true),
                (2, true) => ([255, 50, 0], 2 * self.alpha, true),
                (3, false) => ([255, 89, 182], self.alpha, true),
                (3, true) => ([255, 89, 182], 2 * self.alpha, true),
                (4, false) => ([50; 3], self.alpha / 2, false),
                (4, true) => ([50; 3], self.alpha, false),
                (_, false) => ([255, 255, 0], self.alpha, true),
                (_, true) => ([0, 0, 255], 2 * self.alpha, true),
            };
            let ends = |step: i32| {
                let angle = (step * 36) as f32;
                let (across, down) = (radius * cosine(angle), radius * sine(angle));
                let off = [across * cosine(side), down, across * sine(side)];
                (std::array::from_fn(|axis| off[axis] + near[axis]), std::array::from_fn(|axis| off[axis] + end[axis]))
            };
            for step in 1..=10 {
                let ((start_before, end_before), (start, finish)): (([f32; 3], [f32; 3]), ([f32; 3], [f32; 3])) = (ends(step - 1), ends(step));
                rects.push(Rect::new(picture, alpha, [start_before, start, finish, end_before], (0.0, 1.0), colour, adds));
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(kind: f32, tall: f32, alpha: i32) -> Prim {
        let mut heights = [0.0; 21];
        heights[4] = kind;
        let mut prim = Prim::new(Shape::Tei(Tei::LineLink), "picture");
        prim.duration = 1000;
        prim.parts.push(Part { tall, alpha, heights, ..Part::default() });
        prim
    }

    #[test]
    fn the_tube_follows_the_other_and_dies_without_one() {
        let mut tube = link(2.0, 0.2, 0);
        tube.link_to(Some([30.0, 2.0, 40.0]));
        tube.tick(1000);
        assert_eq!((tube.parts[0].at[1], tube.parts[0].at[3], tube.parts[0].alpha), ([0.0, -8.0, 0.0], [30.0, -10.0, 40.0], 6));
        tube.link_to(None);
        assert!(!tube.parts[0].live && tube.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
        // A primitive of another kind is not touched.
        let mut other = Prim::new(Shape::Tei(Tei::PentaRing), "picture");
        other.parts.push(Part::default());
        other.link_to(None);
        assert!(other.parts[0].live);
    }

    #[test]
    fn it_reaches_out_over_its_first_ticks_in_two_tubes_of_ten_sides() {
        let mut tube = link(2.0, 0.2, 0);
        tube.link_to(Some([0.0, 0.0, 50.0]));
        for _ in 0..5 {
            tube.tick(1000);
        }
        let rects = tube.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 20);
        assert_eq!((rects[0].colour, rects[0].alpha, rects[0].adds), ([0, 100, 150], 30.0, true));
        assert_eq!((rects[10].colour, rects[10].alpha), ([255, 50, 0], 60.0));
        // Half way out at its fifth tick (the sine of 30 degrees), 8 over the ground; its side is across its way.
        let (start, finish) = (rects[0].corners[0].0, rects[0].corners[3].0);
        assert!((start[0].abs() - 0.2).abs() < 1e-3 && (start[1] - 8.0).abs() < 1e-3 && start[2].abs() < 1e-3);
        assert!((finish[2] - 25.0).abs() < 0.05);
        for _ in 0..10 {
            tube.tick(1000);
        }
        assert!((tube.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].corners[3].0[2] - 50.0).abs() < 1e-3);
    }

    #[test]
    fn each_kind_comes_and_goes_its_own_way() {
        let mut soul = link(2.0, 0.2, 0);
        let mut strings = link(3.0, 0.7, 0);
        let mut cloud = link(0.0, 0.4, 45);
        for prim in [&mut soul, &mut strings, &mut cloud] {
            prim.link_to(Some([10.0, 0.0, 0.0]));
            for _ in 0..50 {
                prim.tick(1000);
            }
        }
        // 19 ticks up by 6, ten down by 3.
        assert_eq!(soul.parts[0].alpha, 19 * 6 - 30);
        assert_eq!(strings.parts[0].alpha, 50);
        assert!(strings.parts[0].tall > 0.7);
        assert_eq!(cloud.parts[0].alpha, 45);
        assert_eq!(cloud.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[10].colour, [0, 0, 255]);
    }
}
