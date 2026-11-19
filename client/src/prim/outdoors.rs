//! Two kinds of sprite the effects of a map itself are made of:
//!
//! - a sprite whose way wanders (`Prim3DParticle` with its `pattern & 4`, and the marks at which its alpha
//!   starts over, `ProcessAlpha` with its `alphaCycle`): the fireflies of `CRagEffect::FireSparks`;
//! - a sprite that falls wandering from side to side (`PrimSakura`, the kind 40): the petals of
//!   `CRagEffect::Sakura` and the leaves of `CRagEffect::Maple`.
//!
//! Each keeps its lots and its wandering in its first part, as the client keeps a petal's.

use super::*;

/// The ticks at which a firefly's alpha starts over, with the alpha and the most it may be from then
/// (`chPoint`, `chVal1`, `chVal2` of `FireSparks`).
const FIREFLY_MARKS: [(i32, f32, f32); 2] = [(40, 120.0, 200.0), (100, 0.0, 80.0)];

impl Prim {
    /// `Prim3DParticle` of a firefly: it flies as a sprite does; every so many ticks (its part's `process`, the
    /// client's `chCnt`) it takes a new speed of 0.15 to 0.49, turns up to 40 degrees either way about the
    /// upright and about the line across, and draws how many ticks it keeps that way (5 to 14). At the marks
    /// of its alpha it starts over from there.
    pub(super) fn roam(&mut self) {
        self.fly();
        let count = self.count;
        if let Some(part) = self.parts.first_mut() {
            if count % part.process.max(1) == 0 {
                self.speed = (part.lot(35) + 15) as f32 / 100.0;
                self.longitude += (part.lot(80) - 40) as f32;
                self.latitude += (part.lot(80) - 40) as f32;
                part.process = part.lot(10) + 5;
            }
        }
        if let Some((_, alpha, most)) = FIREFLY_MARKS.iter().find(|(tick, ..)| *tick == count) {
            (self.alpha, self.max_alpha) = (*alpha, *most);
        }
    }

    /// `PrimSakura`: each of its two turns wanders (`Part::wander`); it falls by its part's `tall` a tick and
    /// drifts east by 0.24 and north by 0.30 of the sines of the two turns, a `slow` one by half of that.
    /// (The client puts one that has come down under its effect's place back up at once; the effect does
    /// that here, `web/world/prims/weather.rs`, which knows where its place is now.)
    pub(super) fn fall_as_petal(&mut self, slow: bool) {
        let Some(part) = self.parts.first_mut().filter(|part| part.live) else { return };
        part.wander(0);
        part.wander(2);
        let (east, north) = if slow { (0.12, 0.15) } else { (0.24, 0.30) };
        self.moved = [self.moved[0] + east * sine(part.heights[0].trunc()), self.moved[1] - part.tall, self.moved[2] + north * sine(part.heights[2].trunc())];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_firefly_changes_its_way_and_its_alpha_starts_over() {
        let mut fly = Prim::new(Shape::Roamer, "이팩트/particle1");
        (fly.duration, fly.yaw_first, fly.flickers) = (140, true, true);
        (fly.alpha, fly.max_alpha, fly.alpha_speed, fly.speed) = (0.0, 80.0, 80.0 / 1.5, 0.3);
        fly.parts.push(Part { process: 7, seed: 11, ..Part::default() });
        // Its first tick is one of change: a new speed, a new way, a new count.
        fly.tick(140);
        assert!((0.15..0.5).contains(&fly.speed) && (5..15).contains(&fly.parts[0].process));
        assert!(fly.longitude.abs() <= 40.0 && fly.latitude.abs() <= 40.0);
        assert!(fly.shows_sprite() && fly.seen().len() == 1);
        // It flickers between nothing and its most.
        assert!((fly.alpha - 80.0 / 1.5).abs() < 1e-3);
        for _ in 1..41 {
            fly.tick(140);
        }
        // At its 40th tick it started over from 120 with 200 at most, on its way up or down from there.
        assert_eq!(fly.max_alpha, 200.0);
        assert!(fly.alpha > 60.0 && fly.alpha <= 200.0, "{}", fly.alpha);
    }

    #[test]
    fn a_petal_falls_and_drifts_by_the_sines_of_its_turns() {
        let mut petal = Prim::new(Shape::Petal { slow: false }, "이팩트/sakura01");
        (petal.duration, petal.alpha) = (99_999_999, 200.0);
        petal.moved = [0.0, 200.0, 0.0];
        let mut part = Part { tall: 0.3, seed: 5, ..Part::default() };
        part.heights[..4].copy_from_slice(&[90.0, 200.0, 30.0, 100.0]);
        petal.parts.push(part);
        petal.tick(99_999_999);
        assert!((petal.moved[0] - 0.24 * sine(93.0)).abs() < 1e-5 && (petal.moved[1] - 199.7).abs() < 1e-4 && (petal.moved[2] - 0.30 * sine(33.0)).abs() < 1e-5);
        // It keeps its alpha: nothing fades it.
        assert_eq!(petal.alpha, 200.0);
        let mut leaf = Prim { kind: Shape::Petal { slow: true }, moved: [0.0, 200.0, 0.0], ..petal.clone() };
        leaf.parts[0].heights[..4].copy_from_slice(&[90.0, 200.0, 30.0, 100.0]);
        leaf.tick(99_999_999);
        assert!((leaf.moved[0] - 0.12 * sine(93.0)).abs() < 1e-5);
    }
}
