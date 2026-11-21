//! The kind `PP_THROWITEM` (53) of the client's primitives: the picture of an item thrown in a bow from the
//! one who throws it to a place, turning, three fainter ones behind it (`PrimThrowItem`, drawn by
//! `RenderFieldPanel`, `tei/field.rs`). The bottle of Acid Terror, the potion of Potion Pitcher and the stone
//! of Throw Stone are of it.

use super::*;

impl Prim {
    /// `PrimThrowItem`, a tick: every other tick the three behind take each the look of the one before
    /// (the second 150 fainter than the first, the others 25); the first turns 10 degrees a tick (45, for the
    /// look 15 of a knife); it flies `heights[1]` a tick the way it is `turned` and comes `heights[2]` a tick
    /// nearer the height of the place aimed at, over `flags[1]` ticks, in a bow whose top is 30 over the
    /// straight way (10 for the look 0 of a stone, 1 for a knife's); a twentieth past the place it flies no
    /// more and fades by 20 a tick.
    ///
    /// The client measures the way to the place as the part's count reaches 1, from where the part is; the
    /// part has not moved by then, and here the effect gives it that way as it launches it.
    pub(super) fn fly_as_thrown_item(&mut self) {
        let [head, tail @ ..] = &mut self.parts[..] else { return };
        if !head.live {
            return;
        }
        head.process += 1;
        if head.process % 2 == 1 {
            Part::trail(head, tail, 150);
        }
        head.round = under_a_round(head.round + if head.flags[2] == 15 { 45 } else { 10 });
        let mut progress = head.process as f32 / f32::from(head.flags[1]);
        if progress > 1.05 {
            head.alpha = (head.alpha - 20).max(0);
            return;
        }
        if progress > 1.0 {
            progress = 1.0;
        }
        let bow = sine((180.0 * progress) as i32 as f32);
        head.at[2][1] += head.heights[2];
        head.at[0][1] = head.at[2][1] - bow * match head.flags[2] {
            0 => 10.0,
            15 => 1.0,
            _ => 30.0,
        };
        head.fly_level();
    }
}

impl Part {
    /// The first part of a thing thrown as `CRagEffect::ThrowItem` makes it, to a place so `far` off the way
    /// `turned` and `lower` than the thrower's feet (the client's y, which grows downwards): it begins 8 over
    /// the feet. `kind` is the function's: 0 a bottle (tinted green), 1 a potion in the colour of `count`
    /// (1 red, 2 orange, 3 yellow, 4 white, 5 blue), 2 a stone, which flies 2 a tick, 3 a knife, which flies
    /// 3 a tick, turns faster and begins turned, 10 and 11 a bottle in white; the others reach the place in
    /// 25 ticks.
    pub fn thrown_item(mut self, kind: i8, count: i8, far: f32, turned: i32, lower: f32) -> Part {
        (self.live, self.process, self.alpha) = (true, 0, 255);
        (self.tall, self.turned) = (far, turned);
        self.out = match kind {
            2 => 2.0,
            3 => 4.0,
            _ => 3.0,
        };
        self.at[2][1] = -8.0;
        self.heights[1] = match kind {
            3 => 3.0,
            2 => 2.0,
            _ => far / 25.0,
        };
        // (For a place no way off the client divides by nothing; here it comes no nearer its height.)
        self.heights[2] = if far > 0.0 { self.heights[1] / far * lower } else { 0.0 };
        (self.round, self.flags[2], self.flags[3]) = match kind {
            3 => (330, 15, 0),
            0 => (0, 8, 0),
            1 => (0, 9, count),
            10.. => (0, 9, 4),
            // (A stone, and any other kind.)
            _ => (0, 0, 0),
        };
        // The ticks of the flight: the first count of them that takes it past the place.
        self.flags[1] = (0..100).find(|ticks| far < *ticks as f32 * self.heights[1]).unwrap_or(0) as i8;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::close;

    fn thrown(kind: i8, count: i8, far: f32) -> Prim {
        let mut item = Prim::new(Shape::Tei(Tei::ThrowItem), "유저인터페이스\\item\\빈병.bmp");
        item.duration = 300;
        item.parts.push(Part { seed: 3, ..Part::default() }.thrown_item(kind, count, far, 0, 0.0));
        item.parts.extend((0..3).map(|_| Part { live: true, alpha: 0, ..Part::default() }));
        item
    }

    #[test]
    fn a_bottle_flies_a_bow_to_the_place_in_25_ticks_and_fades() {
        let mut bottle = thrown(0, 0, 50.0);
        // 2 a tick: past 50 at the 26th count.
        assert!(close(bottle.parts[0].heights[1], 2.0));
        assert_eq!(bottle.parts[0].flags[1], 26);
        for _ in 0..13 {
            bottle.tick(300);
        }
        // Half way: at the top of its bow, 30 over the straight way 8 over the feet, and turned 130 degrees.
        let head = &bottle.parts[0];
        assert!(close(head.at[0][0], 26.0) && close(head.at[0][1], -38.0), "{:?}", head.at[0]);
        assert_eq!((head.round, head.alpha), (130, 255));
        for _ in 0..13 {
            bottle.tick(300);
        }
        let head = &bottle.parts[0];
        assert!(close(head.at[0][0], 52.0) && head.at[0][1].abs() - 8.0 < 0.01);
        // A twentieth past the place (from the 28th tick of 26) it stays and fades by 20 a tick.
        for _ in 0..3 {
            bottle.tick(300);
        }
        let head = &bottle.parts[0];
        assert!(close(head.at[0][0], 54.0));
        assert_eq!(head.alpha, 215);
        // It is drawn tinted green, not added; the ones behind it follow, fainter.
        let rects = bottle.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(rects.len() > 1 && rects.iter().all(|rect| !rect.adds && rect.colour == [135, 255, 135]));
    }

    #[test]
    fn a_potion_has_the_colour_of_its_level_and_a_stone_flies_low() {
        let colour = |item: &Prim| item.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].colour;
        let mut red = thrown(1, 1, 20.0);
        red.tick(300);
        assert_eq!(colour(&red), [255, 135, 135]);
        let mut blue = thrown(1, 5, 20.0);
        blue.tick(300);
        assert_eq!(colour(&blue), [135, 135, 255]);
        // A stone: 2 a tick whatever the way, a bow 10 high, its own colours.
        let mut stone = thrown(2, 0, 20.0);
        assert_eq!(stone.parts[0].flags[1], 11);
        for _ in 0..5 {
            stone.tick(300);
        }
        assert!(close(stone.parts[0].at[0][0], 10.0));
        assert!(stone.parts[0].at[0][1] > -18.01 && stone.parts[0].at[0][1] < -17.0, "{}", stone.parts[0].at[0][1]);
        assert_eq!(colour(&stone), [255; 3]);
        // A knife turns 45 degrees a tick from 330.
        let mut knife = thrown(3, 0, 30.0);
        knife.tick(300);
        assert_eq!(knife.parts[0].round, 15);
    }
}
