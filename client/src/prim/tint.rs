//! The colour of a primitive that changes as it lives: what `MoveEffectPrimitive` does with the three speeds of
//! colour (`redSpeed`, `greenSpeed`, `blueSpeed`) of one that has its `pattern & 0x800`, as the mark of
//! `LockOnMarker` has.
//!
//! A picture in the world that was given a `Cycle` goes through this every tick (`plate.rs`).

/// The speeds of the colour of a primitive whose colour goes round (`redSpeed`, `greenSpeed`, `blueSpeed`),
/// every how many ticks it changes while a part of it stands at its most (`RGBCycleDelay`), and how many of
/// those it has waited (`RGBCycleCnt`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cycle {
    pub speeds: [f32; 3],
    pub delay: i32,
    pub waited: i32,
}

impl Cycle {
    /// One tick of it on a colour.
    pub(super) fn turn(&mut self, colour: &mut [f32; 3]) {
        cycle(colour, self.speeds, self.delay, &mut self.waited);
    }
}

/// One tick of a colour that goes round (`pattern & 0x800`): each part changes by its speed, and one that goes
/// past 254 starts again at nothing, one that goes under nothing at 254. While any part stands at 254 the
/// colour waits: only every `delay` ticks does it change (`RGBCycleDelay`, counted in `waited`, the client's
/// `RGBCycleCnt`); with no delay it does not wait.
fn cycle(colour: &mut [f32; 3], speeds: [f32; 3], delay: i32, waited: &mut i32) {
    if colour.contains(&254.0) {
        if delay != 0 {
            *waited += 1;
        }
        if delay != *waited {
            return;
        }
        *waited = 0;
    }
    for (part, speed) in colour.iter_mut().zip(speeds) {
        *part += speed;
        if *part > 254.0 {
            *part = 0.0;
        } else if *part < 0.0 {
            *part = 254.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_runs_down_waits_at_its_most_and_runs_down_again() {
        // As the mark of Sense: red stays, green and blue fall by 15 a tick from 150.
        let (mut colour, mut waited) = ([250.0, 150.0, 150.0], 0);
        let mut tick = |colour: &mut [f32; 3]| cycle(colour, [0.0, -15.0, -15.0], 20, &mut waited);
        for _ in 0..10 {
            tick(&mut colour);
        }
        assert_eq!(colour, [250.0, 0.0, 0.0]);
        // Under nothing it starts again at 254, and waits there 19 ticks.
        tick(&mut colour);
        assert_eq!(colour, [250.0, 254.0, 254.0]);
        for _ in 0..19 {
            tick(&mut colour);
        }
        assert_eq!(colour, [250.0, 254.0, 254.0]);
        // At the twentieth it goes on, and then a tick at a time.
        tick(&mut colour);
        assert_eq!(colour, [250.0, 239.0, 239.0]);
        tick(&mut colour);
        assert_eq!(colour, [250.0, 224.0, 224.0]);
    }

    #[test]
    fn a_colour_with_no_delay_does_not_wait_and_starts_again_past_its_most() {
        let (mut colour, mut waited) = ([254.0, 10.0, 250.0], 0);
        cycle(&mut colour, [0.0, -5.0, 5.0], 0, &mut waited);
        assert_eq!((colour, waited), ([254.0, 5.0, 0.0], 0));
    }
}
