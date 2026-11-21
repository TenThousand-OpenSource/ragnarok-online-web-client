//! `PP_HEAL_LOW` (110): walls of light that live as those of the kind 60 do (`PrimPulsingColumns`,
//! `Slot::pulse_as_wall`), drawn in ten pieces and not twenty (`Render3DCasting_LowPolygon`), each wall
//! standing aside from the effect's place by a way of its own (`vecBNow`, with the 1234.5 the effect writes
//! into its y to say so; here the part's `at`, east and north) that is turned with the camera: the walls of
//! `CRagEffect::RingRedPulse`, twenty of which stand in the shape of a heart that faces whoever looks.
//!
//! The client keeps the turn in the third part's `alphaT`: 360 less the whole degrees of the camera's yaw, set
//! every tick by `PrimPulsingColumns` for walls whose columns are of the kind 33. So it is here (the third
//! part's `holds`), the primitive being told the yaw before each tick (`Prim::watched`).

use super::*;

impl Prim {
    /// `PrimPulsingColumns`, its first lines: the turn of the heart for the camera's `yaw`, the client's
    /// degrees.
    pub(super) fn low_walls_watched(&mut self, yaw: f32) {
        if let Some(slot) = self.slots.get_mut(2) {
            slot.holds = 360 - yaw as i32;
        }
    }

    /// `Render3DCasting_LowPolygon`: every living wall as a band of ten pieces round a middle that stands the
    /// wall's own way aside from the primitive's, that way turned about the upright by the heart's turn.
    pub(super) fn lit_low_walls(&self) -> Vec<Lit> {
        let turn = self.slots.get(2).map_or(0, |slot| slot.holds) as f32;
        let (cos, sin) = (cosine(turn), sine(turn));
        let middle = self.middle([0.0; 3]);
        self.slots
            .iter()
            .filter(|slot| slot.live)
            .flat_map(|slot| {
                let (east, north) = (cos * slot.at[0] - sin * slot.at[2], sin * slot.at[0] + cos * slot.at[2]);
                slot.band_of(self.picture, [middle[0] + east, middle[1], middle[2] + north], 10)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of the twenty primitives of `RingRedPulse`, its walls so far east and north of the heart's middle
    /// and begun at no turn.
    fn pulse(east: f32, north: f32) -> Prim {
        let mut prim = Prim::new(Shape::CastingLow, "ring_red.tga");
        (prim.duration, prim.adds, prim.colour) = (1000, true, [255.0, 89.0, 182.0]);
        prim.slots = vec![Slot::wall(0, 2.3, 86, 12.0, 975), Slot::wall(0, 2.0, 90, 25.0, 975)];
        prim.slots.iter_mut().for_each(|slot| slot.at = [east, 0.0, north]);
        prim.slots.extend([Slot { live: false, ..Slot::wall(0, 0.0, 0, 0.0, 0) }, Slot { live: false, ..Slot::wall(0, 0.0, 0, 0.0, 0) }]);
        prim
    }

    #[test]
    fn its_walls_pulse_as_those_of_the_kind_60_and_are_drawn_in_ten_pieces() {
        let mut prim = pulse(0.0, 0.0);
        prim.watched(0.0);
        for _ in 0..20 {
            prim.tick(1000);
        }
        // 5 a tick over the first 15 ticks.
        assert_eq!((prim.slots[0].alpha, prim.slots[1].alpha), (75, 75));
        let lit = prim.lit(testing::RIGHT, testing::UP);
        assert_eq!(lit.len(), 20);
        // A piece shows its 21st of the picture; the tenth ends on the edge the first began at.
        assert!(testing::close(lit[9].corners[1].1[0], 10.0 / 21.0));
        assert_eq!(lit[9].corners[1].0, lit[0].corners[0].0);
        // The first wall's first foot is 2.3 east of the middle, on the ground.
        assert!(testing::close(lit[0].corners[0].0[0], 2.3) && lit[0].corners[0].0[1] == 0.0);
    }

    #[test]
    fn a_wall_stands_aside_by_its_way_turned_with_the_camera() {
        let foot = |yaw: f32| {
            let mut prim = pulse(0.0, 10.0);
            prim.watched(yaw);
            prim.tick(1000);
            let at = prim.lit(testing::RIGHT, testing::UP)[0].corners[0].0;
            [at[0] - 2.3, at[2]]
        };
        // Seen from the south (a yaw of nothing) the top of the heart is to the north, away from the camera;
        // with the camera looking east it is to the east: away again.
        let (north, east) = (foot(0.0), foot(90.0));
        assert!(testing::close(north[0], 0.0) && testing::close(north[1], 10.0));
        assert!(testing::close(east[0], 10.0) && testing::close(east[1], 0.0));
    }
}
