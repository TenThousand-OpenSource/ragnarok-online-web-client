//! More kinds made of parts. Two of walls of light round the place of the effect, each wall a band of twenty
//! pieces as the kind 60 is (`slots.rs`), drawn the same way (`Render3DCasting`, `Slot::band`), but living
//! otherwise:
//!
//! - `PP_3DCASTING` (23, `Cast::Stone`): walls that rise once, as a sine does over ninety ticks, each edge to
//!   its own height, nothing at the wall's first and last edge and all of it in the middle
//!   (`UpdateStoneCurse`);
//! - `PP_3DCASTING_2` (24, `Cast::Turning`): walls that turn, come up slowly and fade over the last twenty
//!   ticks of their primitive, their edges swaying (`CEffectPrim::OnProcess`, `case PP_3DCASTING_2`).
//!
//! - `PP_3DCASTING_3` (25, `Cast::Fire`): the walls of the kind 23, but that the fourth, the pillar, comes up
//!   to an alpha of 100 and, once risen, breathes between a tenth and all of its height (`UpdateFireWall`).
//!
//! And `PP_CHEMICALPROTECTION` (54, `Cast::Rays`): rays of light in the flat of the screen (`casts/rays.rs`).
//!
//! `RenderTeiRect` draws a wall by its primitive's `size`. A primitive of the kind 23 whose size is 3 has only
//! its fourth wall live (`UpdateStoneCurse` passes the first three over, and they stay flat), and that one is
//! drawn at an alpha of 80 whatever its own is: `Cast::Stone { last_only: true }`.

use super::*;

mod rays;

/// Which of the kinds here a primitive is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cast {
    Stone { last_only: bool },
    Turning,
    Fire,
    /// One of the two drawings of the rays: the wide one, or the narrow one at half the alpha.
    Rays { wide: bool },
}

impl Prim {
    /// One tick of the parts of a primitive of one of the kinds here.
    pub(super) fn tick_casts(&mut self, cast: Cast) {
        let duration = self.duration;
        for (index, slot) in self.slots.iter_mut().enumerate().filter(|(_, slot)| slot.live) {
            match cast {
                Cast::Stone { last_only } if last_only && index < 3 => {}
                Cast::Stone { .. } => slot.rise_as_wall(index as i32, duration),
                Cast::Turning => slot.turn_as_wall(index as i32, duration),
                Cast::Fire if index < 3 => slot.rise_as_wall(index as i32, duration),
                Cast::Fire => slot.breathe_as_pillar(duration),
                Cast::Rays { .. } => slot.shine_as_ray(),
            }
        }
    }

    /// The quads of the parts now, round the primitive's middle. `right` and `up` are the screen's two
    /// directions in the world, for the rays.
    pub(super) fn lit_casts(&self, cast: Cast, right: [f32; 3], up: [f32; 3]) -> Vec<Lit> {
        let middle = self.middle([0.0; 3]);
        let live = self.slots.iter().enumerate().filter(|(_, slot)| slot.live);
        match cast {
            Cast::Stone { last_only: true } => live.filter(|(index, _)| *index == 3)
                .flat_map(|(_, slot)| slot.band(self.picture, middle))
                .map(|lit| Lit { alpha: 80.0, ..lit })
                .collect(),
            Cast::Rays { wide } => live.filter(|(_, slot)| slot.alpha > 0).map(|(_, slot)| slot.beam(self.picture, wide, middle, right, up)).collect(),
            _ => live.flat_map(|(_, slot)| slot.band(self.picture, middle)).collect(),
        }
    }
}

impl Slot {
    /// `UpdateStoneCurse`, for the wall of that number of a primitive that lives `duration` ticks. The first
    /// three turn, by 3, 4 and 5 degrees a tick. Until 40 ticks before the primitive's end a wall comes up over
    /// its first 20 ticks (unless its number, `holds`, is 1): the fourth by 8 a tick to 60 at most, the others
    /// by 10 to 180; from then on the fourth goes down by 2 a tick and the others by 5. Over its first 90 ticks
    /// each edge rises as a sine does to its own height, which is the wall's as a sine goes along it.
    fn rise_as_wall(&mut self, index: i32, duration: i32) {
        self.process += 1;
        if index < 3 {
            self.turned = (self.turned + index + 3) % 360;
        }
        if self.process < duration - 40 {
            if self.process < 20 && self.holds != 1 {
                self.alpha = if index == 3 { (self.alpha + 8).min(60) } else { (self.alpha + 10).min(180) };
            }
        } else {
            self.alpha = (self.alpha - if index >= 3 { 2 } else { 5 }).max(0);
        }
        if self.process <= 90 {
            for (edge, height) in self.heights.iter_mut().enumerate() {
                *height = (self.tall * sine((edge * 9) as f32) * sine(self.process as f32)).max(0.0);
            }
        }
    }

    /// `UpdateFireWall`, for the fourth wall of a primitive that lives `duration` ticks (its first three live as
    /// those of `UpdateStoneCurse` do, but that nothing holds their alpha back). Until 40 ticks before the
    /// primitive's end it comes up by 8 a tick over its first 20 ticks, to 100 at most; from then on it goes
    /// down by 3 a tick. Over its first 90 ticks each edge rises as a sine does to its own height; an edge
    /// that has stood there a tick (the client's `flag1`, set at the 91st) then stands 55 hundredths of it and
    /// 45 more or less as a sine goes with the ticks.
    fn breathe_as_pillar(&mut self, duration: i32) {
        self.process += 1;
        if self.process < duration - 40 {
            if self.process < 20 {
                self.alpha = (self.alpha + 8).min(100);
            }
        } else {
            self.alpha = (self.alpha - 3).max(0);
        }
        for (edge, height) in self.heights.iter_mut().enumerate() {
            let own = self.tall * sine((edge * 9) as f32);
            if self.process <= 90 {
                *height = (own * sine(self.process as f32)).max(0.0);
            } else if self.process > 91 {
                *height = own * 0.45 * sine((self.process % 360) as f32) + own * 0.55;
            }
        }
    }

    /// The kind 24, for the wall of that number: the first and third turn a degree a tick, the others two.
    /// Until 20 ticks before the primitive's end a wall comes up by 1 a tick over its first 100 ticks, to 80 at
    /// most; from then on it goes down by 5. Each edge stands the wall's height as a sine goes along it, and up
    /// to three tenths more or less as another goes with the ticks, each wall at its own pace.
    fn turn_as_wall(&mut self, index: i32, duration: i32) {
        self.process += 1;
        self.turned = (self.turned + if index == 0 || index == 2 { 1 } else { 2 }) % 360;
        if self.process < duration - 20 {
            if self.process < 100 {
                self.alpha = (self.alpha + 1).min(80);
            }
        } else {
            self.alpha = (self.alpha - 5).max(0);
        }
        let angle = if index * 45 < 90 { self.process + index * 90 } else { (self.process + index * 45) * 2 };
        for (edge, height) in self.heights.iter_mut().enumerate() {
            let own = self.tall * sine((edge * 9) as f32);
            *height = own * 0.3 * sine((angle % 360) as f32) + own;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    /// The four walls of `QuadRingStackSized`, the fourth the pillar.
    fn stone(last_only: bool, duration: i32) -> Prim {
        let mut walls = Prim::new(Shape::Cast(Cast::Stone { last_only }), "a");
        walls.duration = duration;
        walls.slots = vec![
            Slot::wall(0, 4.5, 70, 25.0, 0),
            Slot::wall(90, 5.0, 57, 22.0, 0),
            Slot::wall(180, 5.5, 45, 19.0, 0),
            Slot::wall(0, 4.0, 89, 250.0, 0)
        ];
        walls
    }

    #[test]
    fn a_wall_of_stone_rises_over_ninety_ticks_to_a_bow_and_stays() {
        let mut walls = stone(false, 200);
        for _ in 0..30 {
            walls.tick(200);
        }
        // Half its height in the middle at the thirtieth tick; nothing at its ends.
        assert!(close(walls.slots[3].heights[10], 125.0) && close(walls.slots[3].heights[0], 0.0) && close(walls.slots[3].heights[20], 0.0));
        // The first three have turned by 3, 4 and 5 a tick; the fourth has not.
        assert_eq!([0, 1, 2, 3].map(|index| walls.slots[index].turned), [90, 210, 330, 0]);
        // They have come up over 20 ticks, the fourth to 60 and the others to 180.
        assert_eq!((walls.slots[0].alpha, walls.slots[3].alpha), (180, 60));
        for _ in 0..100 {
            walls.tick(200);
        }
        assert!(close(walls.slots[3].heights[10], 250.0) && close(walls.slots[3].heights[5], 250.0 * 0.7071));
        // From 40 ticks before its end the walls go down.
        for _ in 0..40 {
            walls.tick(200);
        }
        assert_eq!((walls.slots[0].alpha, walls.slots[3].alpha), (180 - 11 * 5, 60 - 11 * 2));
    }

    #[test]
    fn of_a_primitive_of_size_three_only_the_fourth_wall_lives_and_shows_at_eighty() {
        let mut walls = stone(true, 100);
        walls.tick(100);
        assert_eq!((walls.slots[0].process, walls.slots[3].process), (0, 1));
        let lit = walls.lit([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(lit.len(), 20);
        assert!(lit.iter().all(|lit| lit.alpha == 80.0 && lit.picture == "a"));
    }

    #[test]
    fn a_pillar_of_fire_comes_up_to_a_hundred_and_breathes_once_risen() {
        let mut walls = stone(false, 600);
        walls.kind = Shape::Cast(Cast::Fire);
        walls.slots[3].alpha = 70;
        for _ in 0..30 {
            walls.tick(600);
        }
        // The low walls as those of stone; the pillar at 100, half risen in its middle.
        assert_eq!([0, 1, 2, 3].map(|index| walls.slots[index].turned), [90, 210, 330, 0]);
        assert_eq!((walls.slots[0].alpha, walls.slots[3].alpha), (180, 100));
        assert!(close(walls.slots[3].heights[10], 125.0) && close(walls.slots[0].heights[10], 12.5));
        // At its 270th tick the sine is at its lowest: a tenth of its height; the low walls stay risen.
        for _ in 0..240 {
            walls.tick(600);
        }
        assert!(close(walls.slots[3].heights[10], 25.0) && close(walls.slots[0].heights[10], 25.0));
        // From 40 ticks before the end it goes down by 3 a tick.
        for _ in 0..300 {
            walls.tick(600);
        }
        assert_eq!(walls.slots[3].alpha, 100 - 11 * 3);
    }

    #[test]
    fn a_turning_wall_comes_up_by_one_a_tick_and_fades_over_its_last_twenty() {
        let mut walls = Prim::new(Shape::Cast(Cast::Turning), "");
        walls.duration = 200;
        walls.slots = vec![Slot::wall(180, 8.0, 90, 40.0, 0), Slot::wall(270, 7.9, 90, 39.0, 0)];
        for _ in 0..90 {
            walls.tick(200);
        }
        assert_eq!((walls.slots[0].alpha, walls.slots[0].turned, walls.slots[1].turned), (80, 270, 90));
        // The first wall at its ninetieth tick: a sine of 90 with the ticks, so three tenths over its height.
        assert!(close(walls.slots[0].heights[10], 52.0) && close(walls.slots[0].heights[0], 0.0));
        for _ in 0..92 {
            walls.tick(200);
        }
        assert_eq!(walls.slots[0].alpha, 80 - 3 * 5);
        assert_eq!(walls.lit([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).len(), 40);
    }
}
