//! The shake of the screen that some effects ask for (the client's `MM_QUAKE`, which `CGameMode::SendMsg` hands
//! to `CView::Start`, and `CView::Apply`, `Contents/Camera/CameraShake.cpp`): for a while the place the camera
//! looks at, how far it is from it and how it is tipped are thrown about, a step at every frame drawn.
//!
//! `MM_QUAKE` comes with a number: 0 shakes for 650 milliseconds, 1 for 200, 650 or more for as long as the
//! number says (anything between is taken as 0). The client asks its option `TT_QUAKE_ON_OFF` first; here the
//! screen always shakes, there being no such option yet.
//!
//! At every frame the place looked at goes `across` along the camera's right, the distance changes by
//! `distance` and the tip by `pitch`, each with a lot of its own added (a whole number from -2 to 2 over ten, for
//! the tip over fifteen), and each of the three turns over for the next frame. The client adds
//! these to its camera frame after frame and puts the camera back when the time is over; so it is here: what
//! was added so far is kept (`Quake::off`).
//!
//! The three sizes are 1, 0.24 and 0.2 (`CView::Start` as `MM_QUAKE` calls it; `MM_SETQUAKEINFO`, by which
//! `EarthSpike` sets them again, gives the same three). In the client the three that turn over are kept
//! across shakes (they are statics), so a shake may begin with any of them turned over; here each begins as
//! it is given.

/// How long a shake asked for with this number lasts, in milliseconds (`CView::Start`).
pub fn lasts(kind: u32) -> f64 {
    match kind {
        1 => 200.0,
        650.. => f64::from(kind),
        _ => 650.0,
    }
}

/// A shake under way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quake {
    /// When it is over, in milliseconds.
    pub until: f64,
    /// What the next frame adds: across, to the distance, to the tip (degrees). Each turns over at every frame.
    step: [f32; 3],
    /// What has been added so far: across (along the camera's right), to the distance, to the tip.
    pub off: [f32; 3],
}

impl Quake {
    /// A shake that begins at `time` (`MM_QUAKE` with `kind`).
    pub fn begun(kind: u32, time: f64) -> Self {
        Self { until: time + lasts(kind), step: [1.0, 0.24, 0.2], off: [0.0; 3] }
    }

    /// One frame is drawn (`CView::Apply`): each of the three is thrown by its step and a lot, and the step
    /// turns over. `lot` is the client's `rand() % 5`. The lot of the place looked at is drawn twice in the
    /// client, once for each direction on the ground; the first is taken for both here, the camera's right
    /// being one direction.
    pub fn frame(&mut self, lot: &mut dyn FnMut() -> u32) {
        for (index, divisor) in [10.0, 10.0, 15.0].into_iter().enumerate() {
            let jitter = (lot().min(4) as f32 - 2.0) / divisor;
            if index == 0 {
                lot();
            }
            self.off[index] += jitter + self.step[index];
            self.step[index] = -self.step[index];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shake_lasts_by_its_number() {
        assert_eq!((lasts(0), lasts(1), lasts(2), lasts(649), lasts(650), lasts(900)), (650.0, 200.0, 650.0, 650.0, 650.0, 900.0));
        assert_eq!(Quake::begun(1, 1000.0).until, 1200.0);
    }

    #[test]
    fn every_frame_throws_the_camera_the_other_way() {
        let mut quake = Quake::begun(0, 0.0);
        // A lot of 2 adds nothing.
        quake.frame(&mut || 2);
        assert_eq!(quake.off, [1.0, 0.24, 0.2]);
        quake.frame(&mut || 2);
        assert_eq!(quake.off, [0.0; 3]);
        quake.frame(&mut || 4);
        assert!((quake.off[0] - 1.2).abs() < 1e-6 && (quake.off[1] - 0.44).abs() < 1e-6 && (quake.off[2] - (0.2 + 2.0 / 15.0)).abs() < 1e-6);
    }

    #[test]
    fn the_lot_of_the_place_is_drawn_twice() {
        let mut drawn = 0;
        Quake::begun(0, 0.0).frame(&mut || {
            drawn += 1;
            2
        });
        assert_eq!(drawn, 4);
    }
}
