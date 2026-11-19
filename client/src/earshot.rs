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
    let away = f32::hypot(east, north);
    if away >= FARTHEST {
        None
    } else if away <= NEAREST {
        Some(1.0)
    } else {
        Some(f64::from(NEAREST / away))
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
    fn the_music_fades_whole_in_1270_milliseconds() {
        assert_eq!(fade(1270.0), 1.0);
        assert_eq!(fade(10.0), 1.0 / 127.0);
    }
}
