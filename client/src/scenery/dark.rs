//! The darkening of the world at night: the client's `CWeatherEffect::StartDarken`, `StopDarken` and `Process`
//! (Source/Contents/Map/MapSkyMgr.cpp). The server puts the player's own character in the state `EFST_SKE`
//! for a night (rAthena's `night` mode: `clif_status_load(sd, EFST_SKE, 1)`), and the client then takes red and
//! green out of the light on the ground and the models (`GroundGeometry::DrawLightmapped`, `C3dNodeV1::Draw`),
//! a little every frame, down to half.

/// The state the server puts a player's own character in while it is night (`EFST_SKE`).
pub const EFST_SKE: u32 = 160;

/// How much of the red, the green and the blue of the map's light is left (each of 1), and whether it is being
/// darkened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dark {
    pub light: [f32; 3],
    pub on: bool,
}

impl Default for Dark {
    fn default() -> Self {
        Self { light: [1.0; 3], on: false }
    }
}

impl Dark {
    /// So many frames of the client's go by (`CWeatherEffect::Process`): while it darkens, red and green lose
    /// 0.005 a frame until they are at half; else each comes back by as much until it is whole.
    ///
    /// (The client brings a colour back only up to a mark that nothing read of it ever sets, so that by what
    /// was read the world would stay dark after a night: here the light comes back whole.)
    pub fn frames(&mut self, frames: u32) {
        for _ in 0..frames.min(1000) {
            if self.on {
                for part in &mut self.light[..2] {
                    if *part > 0.5 {
                        *part -= 0.005;
                    }
                }
            } else {
                for part in &mut self.light {
                    if *part < 1.0 {
                        *part = (*part + 0.005).min(1.0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_night_takes_half_of_red_and_green() {
        let mut dark = Dark { on: true, ..Dark::default() };
        dark.frames(10);
        assert!((dark.light[0] - 0.95).abs() < 1e-5 && dark.light[2] == 1.0);
        dark.frames(500);
        assert!(dark.light[0] <= 0.5 && dark.light[0] > 0.49 && dark.light[1] == dark.light[0]);
        dark.on = false;
        dark.frames(500);
        assert_eq!(dark.light, [1.0; 3]);
    }
}
