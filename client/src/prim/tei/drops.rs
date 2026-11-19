//! The kind 87 of the client's primitives: a square facing the screen that comes down turning from high over
//! the one the effect is on, three fainter ones behind it, stops and fades (`PrimBubbleSwirl`, drawn by
//! `RenderFieldPanel`, `tei/field.rs`): the cross of Pressure, the shield of Shield Press.
//!
//! As it stops the client's primitive asks for a shake of the screen (`MM_QUAKE`); a primitive here asks
//! nothing of the world, and the effect that launches it asks at that tick.

use super::*;

impl Prim {
    /// `PrimBubbleSwirl`: only the first part moves. Once its count is over nothing it comes up by 20 a tick
    /// over ten ticks; for its first 32 ticks it sinks `heights[1]` a tick from where it was put over the
    /// effect's place (`at[2]`, the client's `vecBPre`) and turns 15 degrees a tick against the clock, and
    /// from the 33rd it goes down by 5 a tick. (One whose `flags[4]` is not nothing sinks and turns for 24
    /// ticks only; one whose `flags[4]` is 4 sinks until it is 60 over the place and its turn is between 250
    /// and 280 degrees, and from then on is as one of nothing.) The other three parts take each the look of
    /// the one before, the fourth first, every tick: 25 fainter, the second 100 fainter than the first.
    pub(super) fn drop_turning(&mut self) {
        let Some((head, tail)) = self.head_and_tail() else { return };
        Part::trail(head, tail, 100);
        if head.process <= 10 {
            head.alpha += 20;
        }
        let sinks = if head.flags[4] == 4 {
            true
        } else if head.process <= 32 {
            head.flags[4] == 0 || head.process <= 24
        } else {
            head.alpha = (head.alpha - 5).max(0);
            return;
        };
        if sinks {
            head.round -= 15;
            if head.round < 0 {
                head.round += 360;
            }
            head.at[2][1] += head.heights[1];
        }
        head.at[0] = head.at[2];
        if head.flags[4] == 4 && head.at[2][1] >= -60.0 && (250..=280).contains(&head.round) {
            head.flags[4] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cross as `TexturedRain` launches it for Pressure: 115 over the feet, to sink 3 a tick.
    fn cross() -> Prim {
        let mut cross = Prim::new(Shape::Tei(Tei::Drop), "cross_old.bmp");
        cross.duration = 300;
        let mut head = Part { out: 12.0, ..Part::default() };
        head.at[2] = [0.0, -115.0, 0.0];
        (head.heights[1], head.flags[2]) = (3.0, 7);
        cross.parts.push(head);
        cross.parts.extend((0..3).map(|_| Part::default()));
        cross
    }

    #[test]
    fn the_cross_comes_down_turning_for_32_ticks_and_fades_where_it_stopped() {
        let mut cross = cross();
        for _ in 0..10 {
            cross.tick(300);
        }
        let head = &cross.parts[0];
        assert_eq!((head.alpha, head.at[0], head.round), (200, [0.0, -85.0, 0.0], 210));
        for _ in 0..22 {
            cross.tick(300);
        }
        assert_eq!((cross.parts[0].at[0], cross.parts[0].alpha), ([0.0, -19.0, 0.0], 200));
        // From the 33rd tick it stays there and goes down by 5 a tick.
        cross.tick(300);
        assert_eq!((cross.parts[0].at[0], cross.parts[0].alpha), ([0.0, -19.0, 0.0], 195));
        for _ in 0..39 {
            cross.tick(300);
        }
        assert_eq!(cross.parts[0].alpha, 0);
    }

    #[test]
    fn three_fainter_crosses_trail_it_and_all_are_white_and_added() {
        let mut cross = cross();
        for _ in 0..12 {
            cross.tick(300);
        }
        // The second is where the first was a tick ago, 100 fainter; the third and fourth 25 fainter each.
        let parts = &cross.parts;
        assert_eq!((parts[1].at[0], parts[1].alpha), ([0.0, -82.0, 0.0], 100));
        assert_eq!((parts[2].at[0], parts[2].alpha, parts[3].alpha), ([0.0, -85.0, 0.0], 75, 30));
        let rects = cross.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 4);
        assert!(rects.iter().all(|rect| rect.colour == [255; 3] && rect.adds));
        // The first stands 79 over the effect's place.
        let height = rects[0].corners.iter().map(|corner| corner.0[1]).sum::<f32>() / 6.0;
        assert!((height - 79.0).abs() < 0.001);
    }
}
