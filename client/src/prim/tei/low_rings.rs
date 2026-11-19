//! The kind 36 of the client's primitives: walls of light of ten pieces that lean outwards more and more as
//! they widen, and begin anew for as long as their primitive lives (`CEffectPrim::OnProcess`, `case 36`; drawn
//! by `Render3DCasting_LowPolygon`). It is the kind 30 (`prim/walls.rs`) made smaller and of fewer pieces.

use super::*;

impl Part {
    /// The kind 36, for the part of that number: every tick it stands 0.04 further out and leans a degree
    /// more outwards (once it rises at less than 10 degrees it shows nothing). Nearer than 2.5 it comes up by
    /// 20 a tick; further out it goes down by 2, and having gone it begins anew 2.14 out, rising at 74
    /// degrees. Its edges stand its height and up to three tenths more or less as a sine goes along the wall,
    /// half a round from its first edge to its last, and as another goes with the ticks, each part at its own
    /// pace.
    pub(super) fn widen_as_low_ring(&mut self, index: i32) {
        self.process += 1;
        self.out += 0.04;
        self.rise -= 1;
        if self.rise < 10 {
            (self.rise, self.alpha) = (10, 0);
        }
        if self.out < 2.5 {
            self.alpha += 20;
        } else {
            self.alpha -= 2;
            if self.alpha < 1 {
                (self.alpha, self.out, self.rise) = (0, 2.14, 74);
            }
        }
        let angle = if index * 45 < 90 { self.process + index * 90 } else { (self.process + index * 45) * 2 };
        for (edge, height) in self.heights.iter_mut().enumerate() {
            *height = self.tall * sine((edge * 9) as f32) * 0.3 * sine(angle.rem_euclid(360) as f32) + self.tall;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wall as `TexturedFlare` launches it, begun at no turn and showing nothing.
    fn flare() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::LowRing), "ring_red.tga");
        (prim.duration, prim.size) = (29999, 0.0);
        prim.parts.push(Part { round: 360, tall: 15.0, out: 2.6, rise: 80, ..Part::default() });
        prim
    }

    #[test]
    fn a_wall_begins_anew_near_comes_up_and_fades_as_it_widens() {
        let mut prim = flare();
        prim.tick(29999);
        // Begun past 2.5 with nothing showing, it begins anew at once.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].out, prim.parts[0].rise), (0, 2.14, 74));
        for _ in 0..8 {
            prim.tick(29999);
        }
        assert_eq!((prim.parts[0].alpha, prim.parts[0].rise), (160, 66));
        // Its edges sway about its height: at its ninth tick the middle edge is over it by three tenths of
        // the sine of 9 degrees.
        assert!((prim.parts[0].heights[10] - (15.0 + 15.0 * 0.3 * 9.0_f32.to_radians().sin())).abs() < 0.001);
        for _ in 0..40 {
            prim.tick(29999);
        }
        // Past 2.5 out it goes down by 2 a tick.
        assert!(prim.parts[0].alpha < 120 && prim.parts[0].out > 3.5);
        assert_eq!(prim.parts[0].heights[0], 15.0);
    }

    #[test]
    fn it_is_drawn_in_ten_pieces_mixed_in_for_a_primitive_of_no_size() {
        let mut prim = flare();
        for _ in 0..5 {
            prim.tick(29999);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 10);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([255; 3], false, 80.0));
        // A piece shows a 21st of the picture, so the ten show under half of it.
        assert!((rects[9].corners[1].1[0] - 10.0 / 21.0).abs() < 0.001);
        assert_eq!(rects[9].corners[1].0, rects[0].corners[0].0);
    }
}
