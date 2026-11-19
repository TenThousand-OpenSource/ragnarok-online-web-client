//! The effects and the sounds a map places (`model::MapEffect`, `MapSound`), each kept by a maker that launches
//! it again and again while the player is near: the client's `CEffectMaker` (Source/Contents/Effects/Core/
//! EffectMaker.cpp) and `CSoundMaker` (Source/Contents/Sound/SoundMaker.cpp).

/// Within how many world units of the player's character a placed effect is launched.
pub const EFFECT_NEAR: f32 = 130.0;
/// The client's effect of fireflies (`EF_FIREFLY`), whose makers do not all begin together.
const EF_FIREFLY: u32 = 45;

/// When a placed effect is launched: how many of the client's frames have gone since the last launch, and how
/// many lie between two.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectMaker {
    pub elapsed: u32,
    pub interval: u32,
}

impl EffectMaker {
    /// The maker of an effect placed with that `delay` (`CEffectMaker::SendMsg`): its first launch comes up to
    /// 24 frames after the map shows; fireflies wait anything up to their delay, and their interval is the
    /// delay less a twentieth and more by up to a tenth. `lot(under)` is the client's `rand() % under`.
    pub fn new(effect: u32, delay: f32, lot: &mut dyn FnMut(u32) -> u32) -> Self {
        let mut below = |range: i32| if range <= 0 { 0 } else { lot(range as u32) };
        if effect != EF_FIREFLY {
            return Self { interval: delay as u32, elapsed: (delay - below(24) as f32) as u32 };
        }
        let more = below((delay / 10.0) as i32);
        Self { interval: (more as f32 + delay - delay / 20.0) as u32, elapsed: below((delay + 1.0) as i32) }
    }

    /// One frame of the client's (`CEffectMaker::OnProcess`) with the player so far away: whether the effect
    /// is launched now.
    pub fn frame(&mut self, away: f32) -> bool {
        let launches = away < EFFECT_NEAR && self.elapsed >= self.interval;
        if launches {
            self.elapsed = 0;
        }
        self.elapsed = self.elapsed.saturating_add(1);
        launches
    }
}

/// How loud (of 1) a placed sound is heard now, the player standing `east` and `north` of where it is placed
/// (`CSoundMaker::OnProcess`); none when it is not played at all. The sound comes from the edge of what sounds
/// (so wide and so deep) on the player's side. It is heard within its `range`, in full within a sixth of it
/// and by that sixth over the distance farther off (`CSoundMgr::PlayWave` with the range and its sixth as the
/// farthest and the nearest; that rule itself is `crate::earshot`'s, written here for other distances).
pub fn heard(east: f32, north: f32, (wide, deep): (i32, i32), range: f32, volume: f32) -> Option<f64> {
    if wide == 0 {
        return None;
    }
    let radius = ((wide * wide + deep * deep) as f32).sqrt();
    if f32::hypot(east, north) - (radius + range) > 130.0 {
        return None;
    }
    let (sign_east, sign_north) = (if east < 0.0 { -1.0 } else { 1.0 }, if north < 0.0 { -1.0 } else { 1.0 });
    let slope = north / east.max(0.001);
    // (The client divides the two whole numbers before it compares.)
    let (out_east, out_north) = if ((deep / wide) as f32) < slope.abs() {
        let out_north = deep as f32 * sign_north * 0.5;
        (out_north / slope.max(0.001), out_north)
    } else {
        let out_east = wide as f32 * sign_east * 0.5;
        (out_east, slope * out_east)
    };
    let away = f32::hypot(east - out_east, north - out_north);
    if away - range > 75.0 {
        return None;
    }
    let (farthest, nearest) = (range.trunc(), (range / 6.0).trunc());
    if away >= farthest {
        None
    } else if away <= nearest {
        Some(f64::from(volume))
    } else {
        Some(f64::from(volume * nearest / away))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_effect_is_launched_every_so_many_frames_near_the_player() {
        // A torch of delay 125 whose lot was 5: 120 frames have gone as the map shows.
        let mut maker = EffectMaker::new(47, 125.0, &mut |_| 5);
        assert_eq!(maker, EffectMaker { elapsed: 120, interval: 125 });
        let launched: Vec<u32> = (0..300).filter(|_| maker.frame(10.0)).collect();
        assert_eq!(launched, [5, 130, 255]);
        // Far from the player it waits, and is launched as soon as the player is near.
        let mut far = EffectMaker::new(47, 125.0, &mut |_| 0);
        assert!((0..500).all(|_| !far.frame(130.0)));
        assert!(far.frame(129.0));
    }

    #[test]
    fn fireflies_begin_apart() {
        let maker = EffectMaker::new(45, 500.0, &mut |under| under - 1);
        assert_eq!(maker, EffectMaker { elapsed: 500, interval: 49 + 500 - 25 });
        assert_eq!(EffectMaker::new(45, 0.0, &mut |under| 7 % under), EffectMaker { elapsed: 0, interval: 0 });
    }

    #[test]
    fn a_sound_is_whole_within_a_sixth_of_its_range() {
        // A sound one wide and deep, heard for 100: its edge is half a unit towards the player.
        assert_eq!(heard(10.5, 0.0, (1, 1), 100.0, 0.5), Some(0.5));
        let far = heard(32.5, 0.0, (1, 1), 100.0, 1.0).unwrap();
        assert!((far - 0.5).abs() < 1e-6, "{far}");
        assert_eq!(heard(100.5, 0.0, (1, 1), 100.0, 1.0), None);
        assert_eq!(heard(1.0, 0.0, (0, 0), 100.0, 1.0), None);
    }
}
