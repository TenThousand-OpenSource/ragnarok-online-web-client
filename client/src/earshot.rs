//! How loud a sound of the world is by how far from the player's character it starts, and how the music
//! fades: the client's two rules of loudness.
//!
//! From `CSoundMgr::PlayData2D`, which every sound of the world goes through as `PlayWave(name, dx, dy, dz,
//! 250, 40, …)`: nothing from 250 units away, all of it within 40, between them 40 over the distance; a
//! cell is 5 units. And from `CSoundMgr::FadeThread`: the music's volume, of 127 steps, goes one step every
//! 10 milliseconds.

/// From how far a sound is not heard, and within how far it is as loud as it gets (world units).
pub const FARTHEST: f32 = 250.0;
pub const NEAREST: f32 = 40.0;

/// How loud (0 to 1) a sound is that starts so far east and north of the player's character, in world units;
/// none for one out of earshot.
pub fn loudness(east: f32, north: f32) -> Option<f64> {
    loudness_within(f32::hypot(east, north), FARTHEST, NEAREST, 1.0)
}

/// The rule itself, for any two distances (`CSoundMgr::PlayWave`, which a sound a map places goes through
/// with distances of its own: `crate::scenery::makers`): a sound of that `volume` (of 1) starting so far
/// `away` is not heard from `farthest` on, is as loud as it is within `nearest`, and between them `nearest`
/// over the distance of it.
pub fn loudness_within(away: f32, farthest: f32, nearest: f32, volume: f32) -> Option<f64> {
    if away >= farthest {
        None
    } else if away <= nearest {
        Some(f64::from(volume))
    } else {
        Some(f64::from(volume * nearest / away))
    }
}

/// How much of its loudness (0 to 1) the music gains or loses in so many milliseconds of a fade.
pub fn fade(milliseconds: f64) -> f64 {
    milliseconds / 10.0 / 127.0
}

/// How long the music falls at a change of map or screen before it rises again: the client's screen goes
/// black over 255 milliseconds (`FadeOutMode`), and the music is told to rise as the next shows (`DrawFadeIn`).
pub const DIP_MS: f64 = 255.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sound_is_whole_within_eight_cells_and_gone_at_fifty() {
        assert_eq!(loudness(0.0, 0.0), Some(1.0));
        assert_eq!(loudness(40.0, 0.0), Some(1.0));
        assert_eq!(loudness(0.0, 80.0), Some(0.5));
        assert_eq!(loudness(96.0, 128.0), Some(0.25));
        assert_eq!(loudness(250.0, 0.0), None);
        assert!(loudness(249.0, 0.0).is_some());
    }

    #[test]
    fn the_rule_holds_for_other_distances_and_volumes() {
        assert_eq!(loudness_within(10.0, 60.0, 10.0, 0.5), Some(0.5));
        assert_eq!(loudness_within(20.0, 60.0, 10.0, 0.5), Some(0.25));
        assert_eq!(loudness_within(60.0, 60.0, 10.0, 0.5), None);
        // A sound whose range leaves no room between the two is all or nothing.
        assert_eq!((loudness_within(0.0, 0.0, 0.0, 1.0), loudness_within(3.0, 5.0, 5.0, 1.0)), (None, Some(1.0)));
        // The world's own rule is this one with its two distances, whole.
        for away in [0.0, 39.0, 40.0, 41.5, 77.7, 249.9, 250.0, 400.0] {
            let old = if away >= FARTHEST { None } else if away <= NEAREST { Some(1.0) } else { Some(f64::from(NEAREST / away)) };
            assert_eq!(loudness(away, 0.0), old, "{away}");
        }
    }

    #[test]
    fn the_music_fades_whole_in_1270_milliseconds() {
        assert_eq!(fade(1270.0), 1.0);
        assert_eq!(fade(10.0), 1.0 / 127.0);
    }
}
