//! The fog of a map: `data\fogparametertable.txt` as `CGameMode::ApplyMapFog` reads a map's line of it, and how
//! thick the fog is at a depth of the view (the client's `DepthAlpha`: even from its start to its end).

use super::{records, table_name};

/// A map's fog: from how deep in the view it begins and at what depth nothing shows through it (world units
/// from the eye along the way it looks), and
/// its colour (red, green and blue, each of 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fog {
    pub start: f32,
    pub end: f32,
    pub colour: [f32; 3],
}

impl Fog {
    /// The fog the table has for a map, if it has one: a line of the map's file and four figures: where the
    /// fog begins and where it ends, each a share of the client's depth (10 to 1500), its colour as
    /// `0xAARRGGBB`, and a density the client reads and does not use.
    pub fn of(table: &str, map: &str) -> Option<Self> {
        let name = table_name(map);
        let (_, figures) = records(table, 4).into_iter().find(|(known, _)| *known == name)?;
        let colour = figures[2] as u32;
        Some(Self {
            start: figures[0] as f32 * 1490.0 + 10.0,
            end: figures[1] as f32 * 1490.0 + 10.0,
            colour: [(colour >> 16) & 255, (colour >> 8) & 255, colour & 255].map(|part| part as f32 / 255.0),
        })
    }

    /// How much of what is so deep in the view shows through the fog: all of it (1) up to the fog's start,
    /// nothing (0) from its end. (The shaders of the ground, the models and the water do the same sum.)
    pub fn clear(&self, away: f32) -> f32 {
        ((self.end - away) / (self.end - self.start).max(0.001)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_has_the_fog_of_its_line() {
        let table = "//a note\nnif_fild01.rsw#\n0.1#\n0.6#\n0xffAA7755#\n0.15#\n\nyuno.rsw#\n0.2#\n0.9#\n0xff99ccff#\n0.3#\n";
        let fog = Fog::of(table, "nif_fild01").unwrap();
        assert!((fog.start - 159.0).abs() < 1e-3 && (fog.end - 904.0).abs() < 1e-3);
        assert_eq!(fog.colour, [170.0 / 255.0, 119.0 / 255.0, 85.0 / 255.0]);
        assert_eq!(Fog::of(table, "prontera"), None);
        assert_eq!((fog.clear(100.0), fog.clear(1000.0)), (1.0, 0.0));
        assert!((fog.clear(531.5) - 0.5).abs() < 1e-4);
    }
}
