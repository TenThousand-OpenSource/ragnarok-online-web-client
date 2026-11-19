//! The kind `PP_GUARD` (61) of the client's primitives: half a ball of panels before someone, three rows of
//! five, each drawn three times a little apart (`PrimGuard`, drawn by `RenderSphereCage`): the shield that
//! shows as a blow is guarded.

use super::*;

impl Part {
    /// `PrimGuard`, for one part: over its first nine ticks it comes up by 20 a tick, to 100; after its 26th
    /// it goes down by 6 a tick and its outer panels stand a tenth further out every tick (`heights[0]`). One
    /// whose `flags[0]` is 1 turns 10 degrees a tick from its second tick on.
    ///
    /// (Any other the client turns every tick to where the one guarded faces, `turned` being that one's angle
    /// and 270; here it stays as the effect turned it when it launched it.)
    pub(super) fn guard(&mut self) {
        self.process += 1;
        if self.process > 1 && self.flags[0] == 1 {
            self.turned += 10;
        }
        self.turned = under_a_round(self.turned);
        if self.process < 10 {
            if self.alpha < 100 {
                self.alpha += 20;
            }
        } else if self.process > 26 {
            self.heights[0] += 0.1;
            self.alpha = (self.alpha - 6).max(0);
        }
    }

    /// `BuildSphereCageQuad`: the panel of a row (`latitude`) and a column (`turn`) of a ball `radius` wide
    /// whose middle is `drop` over the effect's place: `half` from its middle to each side, lying against the
    /// ball.
    fn cage_panel(&self, latitude: i32, turn: i32, half: f32, radius: f32, drop: f32) -> [[f32; 3]; 4] {
        let wrapped = |angle: i32| if angle >= 360 { angle - 360 } else if angle < 0 { angle + 360 } else { angle } as f32;
        let (pitch, tilt) = (wrapped(latitude + 360), wrapped(latitude + 450));
        let (yaw, side) = (wrapped(self.turned + turn - 90), wrapped(self.turned + turn - 180));
        let reach = radius * cosine(pitch);
        let middle = [reach * cosine(yaw), radius * sine(pitch) - drop, reach * sine(yaw)];
        [(half, half), (-half, half), (-half, -half), (half, -half)].map(|(lateral, depth)| {
            let along = depth * cosine(tilt);
            [lateral * cosine(side) - along * sine(side) + middle[0], depth * sine(tilt) + middle[1], along * cosine(side) + lateral * sine(side) + middle[2]]
        })
    }

    /// `RenderSphereCage`, for one part: three rows of five panels over half a turn from `turned` on, the
    /// middle row's panels 3 to each side and the others' 2. Each panel is drawn three times: on a ball as
    /// wide as `tall` whose middle is 2 higher, on one 0.2 wider and on one 0.4 and `heights[0]` wider. The
    /// first two show the first picture, their light added; the third the second picture, mixed in. `flags[0]`
    /// gives the colour: orange (3, 2), pink (4), blue (5: its third panel too shows the first picture,
    /// added), any other a pale green that is paler on the outer panels.
    pub(super) fn cage(&self, pictures: &[&'static str]) -> Vec<Rect> {
        let (Some(&first), Some(&second)) = (pictures.first(), pictures.get(1)) else { return Vec::new() };
        let tinted = match self.flags[0] {
            3 => Some([255, 100, 0]),
            2 => Some([255, 155, 0]),
            4 => Some([255, 150, 255]),
            5 => Some([100, 100, 255]),
            _ => None,
        };
        let mut rects = Vec::new();
        for (row, latitude) in [-405, -360, -315].into_iter().enumerate() {
            let half = if row == 1 { 3.0 } else { 2.0 };
            for turn in [0, 45, 90, 135, 180] {
                let layers = [(self.tall, self.tall + 2.0, [205, 255, 205]), (self.tall + 0.2, self.tall, [155, 255, 155]), (self.tall + 0.4 + self.heights[0], self.tall, [225, 255, 225])];
                for (layer, (radius, drop, green)) in layers.into_iter().enumerate() {
                    let outer = layer == 2 && self.flags[0] != 5;
                    let corners = self.cage_panel(latitude, turn, half, radius, drop);
                    rects.push(Rect::new(if outer { second } else { first }, self.alpha, corners, (0.0, 1.0), tinted.unwrap_or(green), !outer));
                }
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    /// The shield as `KyrieGuard` launches it before one who faces south (an angle of 0).
    fn shield(number: i8) -> Prim {
        let mut cage = Prim::new(Shape::Tei(Tei::Guard), "guardK.tga");
        (cage.duration, cage.pictures) = (200, &["guardK.tga", "guardK2.tga"]);
        let mut part = Part { tall: 7.8, out: 1.0, turned: 270, ..Part::default() };
        part.flags[0] = number;
        cage.parts.push(part);
        cage
    }

    #[test]
    fn the_shield_comes_up_holds_and_fades_as_its_outer_panels_draw_away() {
        let mut cage = shield(0);
        for _ in 0..9 {
            cage.tick(200);
        }
        assert_eq!((cage.parts[0].alpha, cage.parts[0].turned), (100, 270));
        for _ in 0..17 {
            cage.tick(200);
        }
        assert_eq!(cage.parts[0].alpha, 100);
        for _ in 0..4 {
            cage.tick(200);
        }
        assert_eq!(cage.parts[0].alpha, 76);
        assert!(close(cage.parts[0].heights[0], 0.4));
        // The one of Preserve turns 10 degrees a tick from its second tick on.
        let mut turning = shield(1);
        for _ in 0..10 {
            turning.tick(200);
        }
        assert_eq!(turning.parts[0].turned, 0);
    }

    #[test]
    fn the_shield_is_three_rows_of_five_panels_each_drawn_three_times() {
        let mut cage = shield(0);
        cage.parts[0].alpha = 100;
        let rects = cage.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 45);
        assert_eq!((rects[0].picture, rects[0].adds, rects[0].colour), ("guardK.tga", true, [205, 255, 205]));
        assert_eq!((rects[1].picture, rects[1].adds, rects[1].colour), ("guardK.tga", true, [155, 255, 155]));
        assert_eq!((rects[2].picture, rects[2].adds, rects[2].colour), ("guardK2.tga", false, [225, 255, 225]));
        // The middle panel of the middle row, on the second ball: its middle is 8 from the upright through
        // the place, 7.8 over the ground, to the south, where the one guarded faces (a turn of 270 and 90
        // less 90 is the way of 270 degrees: north to south down the client's z).
        let panel = &rects[(5 + 2) * 3 + 1];
        let middle: [f32; 3] = std::array::from_fn(|axis| panel.corners[..4].iter().map(|corner| corner.0[axis]).sum::<f32>() / 4.0);
        assert!(close(middle[0], 0.0) && close(middle[1], 7.8) && close(middle[2], -8.0), "{middle:?}");
        // It stands upright, 6 wide and 6 tall.
        let (xs, ys): (Vec<f32>, Vec<f32>) = panel.corners[..4].iter().map(|corner| (corner.0[0], corner.0[1])).unzip();
        assert!(close(xs.iter().cloned().fold(f32::MIN, f32::max), 3.0) && close(ys.iter().cloned().fold(f32::MIN, f32::max), 10.8));
        // The blue one draws its outer panels like the others.
        let blue = shield(5).parts[0].cage(&["a", "b"]);
        assert_eq!((blue[2].picture, blue[2].adds, blue[2].colour), ("a", true, [100, 100, 255]));
        assert!(shield(0).parts[0].cage(&["a"]).is_empty());
    }
}
