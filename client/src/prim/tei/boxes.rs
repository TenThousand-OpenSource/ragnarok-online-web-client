//! The kind `PP_GRANDCROSS2` (37) of the client's primitives: long boxes of light lying over the ground that
//! rise as they come up, the two bars of Grand Cross (`CEffectPrim::OnProcess`, drawn by `RenderGrandCross2`).

use super::*;

impl Part {
    /// The kind 37 in `CEffectPrim::OnProcess`, for one part: over its first nine ticks it comes up by 9 a
    /// tick; from the tenth it goes down by 1 a tick, and once it shows nothing is gone after its 30th tick.
    /// Every tick it is 0.005 longer and wider to each side (`heights[0]`, `heights[1]`), and as tall
    /// (`heights[2]`) as its height (`tall`) times the sine of its count, in degrees.
    pub(super) fn rise_as_box(&mut self) {
        self.process += 1;
        if self.process < 10 {
            self.alpha += 9;
        } else {
            self.alpha -= 1;
            if self.alpha < 1 {
                self.alpha = 0;
                if self.process > 30 {
                    self.live = false;
                }
            }
        }
        self.heights[1] += 0.005;
        self.heights[0] += 0.005;
        self.heights[2] = sine(self.process as f32) * self.tall;
    }

    /// `RenderGrandCross2`, for one part: a box about the effect's place, turned by `turned`, `heights[0]` to
    /// each side along its length and `heights[1]` across, its floor 1 over the ground and its roof
    /// `heights[2]` over that. Its four walls are white and mixed in at its alpha; its floor is drawn twice,
    /// seen from above and from below, at half that alpha, its light added. (It has no roof.)
    pub(super) fn open_box(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (cos, sin) = (cosine(self.turned as f32), sine(self.turned as f32));
        let (long, wide) = (self.heights[0], self.heights[1]);
        let floor = |x: f32, z: f32| [cos * x - sin * z, -1.0, sin * x + cos * z];
        let roof = |[x, y, z]: [f32; 3]| [x, y - self.heights[2], z];
        // The client's `vecBPre`, `vecBNow`, `vecTNow` and `vecTPre`.
        let [first, second, third, fourth] = [floor(long, wide), floor(long, -wide), floor(-long, -wide), floor(-long, wide)];
        let wall = |near: [f32; 3], far: [f32; 3]| Rect::new(picture, self.alpha, [near, far, roof(far), roof(near)], (0.0, 1.0), [255; 3], false);
        let ground = |corners: [[f32; 3]; 4]| Rect::new(picture, self.alpha / 2, corners, (0.0, 1.0), [255; 3], true);
        vec![wall(first, second), wall(second, third), wall(third, fourth), wall(fourth, first), ground([first, second, third, fourth]), ground([fourth, third, second, first])]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    /// A bar as `GRANDCROSS2` launches one, turned by `turned`.
    fn bar(turned: i32) -> Prim {
        let mut heights = [0.0; 21];
        (heights[0], heights[1]) = (24.0, 3.2);
        let mut bars = Prim::new(Shape::Tei(Tei::GrandCross2), "ring_yellow.tga");
        bars.duration = 999;
        bars.parts.push(Part { tall: 60.0, turned, heights, ..Part::default() });
        bars
    }

    #[test]
    fn a_bar_comes_up_rises_and_is_gone() {
        let mut bars = bar(0);
        assert!(bars.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
        for _ in 0..9 {
            bars.tick(999);
        }
        let part = &bars.parts[0];
        assert_eq!(part.alpha, 81);
        assert!(close(part.heights[0], 24.045) && close(part.heights[1], 3.245));
        assert!(close(part.heights[2], 60.0 * 9.0_f32.to_radians().sin()));
        for _ in 0..80 {
            bars.tick(999);
        }
        assert!(bars.parts[0].live && bars.parts[0].alpha == 1);
        bars.tick(999);
        assert!(!bars.parts[0].live);
    }

    #[test]
    fn a_bar_is_four_walls_and_a_floor_seen_from_both_sides() {
        let mut bars = bar(90);
        (bars.parts[0].alpha, bars.parts[0].heights[2]) = (81, 10.0);
        let rects = bars.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 6);
        // Turned a quarter, its length lies north and south: the first wall is the northern end.
        let wall = &rects[0];
        assert!(close(wall.corners[0].0[0], -3.2) && close(wall.corners[0].0[1], 1.0) && close(wall.corners[0].0[2], 24.0));
        assert!(close(wall.corners[1].0[0], 3.2) && close(wall.corners[1].0[2], 24.0));
        assert!(close(wall.corners[2].0[1], 11.0) && close(wall.corners[3].0[1], 11.0));
        assert_eq!((wall.alpha, wall.adds, wall.colour), (81.0, false, [255; 3]));
        // The floor: half as strong, added, once each way round.
        assert_eq!((rects[4].alpha, rects[4].adds, rects[5].alpha), (40.0, true, 40.0));
        assert_eq!(rects[4].corners[0].0, rects[5].corners[3].0);
        assert!(rects[4].corners.iter().all(|corner| close(corner.0[1], 1.0)));
    }
}
