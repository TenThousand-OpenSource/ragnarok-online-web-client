//! The kind 126 of the client's primitives: a square of wind standing upright along the way it flies, that
//! spins like a wheel and rolls off along the ground, three fainter ones trailing it (`CEffectPrim::OnProcess`,
//! `case 126`, which is the kind 119 fainter: `Prim::blow_as_gust` in `tei/gusts.rs`; drawn by
//! `RenderRotatedRing`).

use super::*;

impl Part {
    /// `RenderRotatedRing`, for one part: a square about its place (`at[0]`) standing in the upright plane
    /// along the way it is turned, its corners `out` from the middle, the first a quarter of a turn on from
    /// its spin (`rise`) and each next a quarter on; green, its light added.
    pub(super) fn upright_square(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (cos, sin) = (cosine(self.turned as f32), sine(self.turned as f32));
        let corner = |quarter: i32| {
            let angle = (self.rise + quarter * 90).rem_euclid(360) as f32;
            let along = self.out * cosine(angle);
            [cos * along + self.at[0][0], self.out * sine(angle) + self.at[0][1], sin * along + self.at[0][2]]
        };
        vec![Rect::new(picture, self.alpha, [corner(1), corner(2), corner(3), corner(4)], (0.0, 1.0), [80, 255, 80], true)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square as `TexturedSlashCross` launches one, sent north.
    fn ring() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::GustRing), "wind");
        prim.duration = 200;
        prim.parts = (0..4).map(|index| {
            let mut part = Part { process: index - 3, out: 20.0, rise: index, turned: 90, ..Part::default() };
            (part.at[0][1], part.heights[1]) = (-10.0, 1.5);
            part
        }).collect();
        prim
    }

    #[test]
    fn it_comes_up_to_sixty_and_goes_down_by_three() {
        let mut prim = ring();
        for _ in 0..13 {
            prim.tick(200);
        }
        assert_eq!(prim.parts[0].alpha, 54);
        for _ in 0..5 {
            prim.tick(200);
        }
        assert_eq!(prim.parts[0].alpha, 39);
        for _ in 0..14 {
            prim.tick(200);
        }
        assert!(!prim.parts[0].live);
    }

    #[test]
    fn it_stands_upright_along_its_way() {
        let mut prim = ring();
        for _ in 0..6 {
            prim.tick(200);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([80, 255, 80], true, 12.0));
        // Sent north, none of its corners is east or west of its middle; spun to 90 degrees its first corner
        // is 20 behind the middle, level with it, and its second 20 over it (the client's down turned up).
        assert!(rects[0].corners.iter().all(|corner| corner.0[0].abs() < 1e-3));
        let north = prim.parts[0].at[0][2];
        assert!((rects[0].corners[0].0[2] - (north - 20.0)).abs() < 1e-3 && (rects[0].corners[0].0[1] - 10.0).abs() < 1e-3);
        assert!((rects[0].corners[1].0[1] - 30.0).abs() < 1e-3);
    }
}
