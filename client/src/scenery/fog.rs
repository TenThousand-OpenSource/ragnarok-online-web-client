//! The fog of a map: `data\fogparametertable.txt` as `CGameMode::ApplyMapFog` reads a map's line of it, and how
//! thick the fog is at a depth of the view (the client's `DepthAlpha`: even from its start to its end).
//!
//! The units, as the client has them. A line's first two figures are shares of the client's depth, which
//! runs from 10 to 1500 world units from the eye along the way it looks: `DepthAlpha` takes a corner's depth
//! (one over the `rhw` that `RendererViewport::Project` gives it: the last of its place as the view's matrix
//! makes it, the view being a plain look-at, `CView::Refresh`), takes 10 off, divides by 1490 and measures
//! that between the two figures. What it gets is the corner's fog, put in the alpha of its second colour,
//! which Direct3D takes as the fog of a corner already placed on the screen; the start and the end the client
//! also sets on the device (`CRendererDX9::ApplyFog`: the same two figures times 1490 and 10 more) say the
//! same in world units. The fourth figure, a density, is set on the device and used by nothing: the fog is
//! linear. A map with no line in the table has no fog at all (`CGameMode::ApplyMapFog`: `SetFog(0)`).
//!
//! So the fog depends on how far the camera is: with the camera at the 300 a map starts with, the ground at
//! the character's feet is 300 deep, short of the 382.5 where the fog of a Prontera field begins. With the
//! client's Zoom Out option the camera goes to 730, and the same field is then two thirds fog at the
//! character's feet, in the client as here; `gef_fild13` (0.2 and 0.45: from 308 to 680.5) is then nothing
//! but its fog's colour, the ground, the models and the water alike. The table's figures suit a camera no
//! farther than 400, and the client does nothing to them when its Zoom Out lets the camera farther.
//!
//! Checked against the client's program itself (`Ragexe` of 2026-02-19), not its decompiled source alone:
//! `DepthAlpha` at 0x55D190 divides 1.0 by the `rhw`, takes off the 10.0 at 0x10DD24C and divides by the
//! 1490.0 at 0x10DD260; the start and the end (0x138965C, 0x1389660) are written in two places only, both from
//! the table as it is (`CGameMode::ApplyMapFog` and the entering of a map), and read by `DepthAlpha` and the
//! two `ApplyFog` alone: nothing scales them by the camera. The depth is in the world's own units (5 a cell
//! of the `.gat`): the view is `matrix::LookAt` of the eye `distance` away from the character (`CView::Refresh`),
//! with no scale, and `RendererViewport::Project` takes its last row as it is. The camera goes as far as 400,
//! or 730 with Zoom Out (0x1112BEC, 0x10DD250, `ApplyServiceOptionJitterScale`), which starts on, as the fog
//! does (`OptionInfo\GameSettings`). The colour's first byte (0x22 of 0x2275A9FF) is not the fog's strength:
//! Direct3D's fog colour has no alpha; the client uses it only to mix the fog's colour into the colour the
//! screen is cleared with on a map whose sky says so (`CRendererDX9::ClearWorld`, `BlendColor`).

use super::{records, table_name};

/// A map's fog: from how deep in the view it begins and at what depth nothing shows through it (world units
/// from the eye along the way it looks), and
/// its colour (red, green and blue, each of 1).
/// How far the camera is from the character at most without the client's Zoom Out (`/zoom`).
pub const NEAR_CAMERA: f32 = 400.0;
/// How much of the camera's way out beyond that the fog goes with it (this project's own, set by the owner's
/// eye: with all of it the picture stayed as clear as at the limit, which he found too thin).
pub const FOLLOWS: f32 = 0.8;

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

    /// The fog as it is drawn with the camera `distance` away from the character. This project's own, by the
    /// owner's decision: the client's table is made for a camera no farther than `NEAR_CAMERA` (its limit
    /// without Zoom Out), and with the camera farther out the client leaves the fog where it is, so that at
    /// 730 a field such as gef_fild13 is nothing but the fog's colour. Here the fog goes out with the camera
    /// beyond that distance, by `FOLLOWS` of the way: the picture grows a little foggier as the camera goes out and never
    /// turns into the fog's colour alone.
    pub fn seen_from(&self, distance: f32) -> Self {
        let out = (distance - NEAR_CAMERA).max(0.0) * FOLLOWS;
        Self { start: self.start + out, end: self.end + out, colour: self.colour }
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
    fn the_fog_goes_out_with_a_camera_beyond_the_near_limit() {
        // gef_fild13: 0.2 and 0.45 of the depth.
        let fog = Fog { start: 308.0, end: 680.5, colour: [0.0; 3] };
        // Up to the limit it is the client's own.
        assert_eq!(fog.seen_from(300.0), fog);
        assert_eq!(fog.seen_from(400.0), fog);
        // Farther out the ground under the character is a little foggier than at the limit, and not lost as
        // it is in the client.
        let (at_limit, far_out) = (fog.clear(400.0), fog.seen_from(730.0).clear(730.0));
        assert!(far_out < at_limit && far_out > 0.5, "{far_out} against {at_limit}");
        assert_eq!(fog.clear(730.0), 0.0);
    }

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

    /// The line of `prt_fild08` as the client's table has it (the lines after it stand right under it, with
    /// no empty line between).
    const FIELDS: &str =
        "prt_fild07.rsw#\n0.25#\n0.6#\n0xffBAFF77#\n0.3#\nprt_fild08.rsw#\n0.25#\n0.6#\n0xffBAFF77#\n0.3#\nprt_fild09.rsw#\n0.25#\n0.6#\n0xffBAFF77#\n0.3#\n";

    /// How deep in the view the ground is that the camera looks at from `distance`, tipped 45 degrees over
    /// level ground (`web/world/camera.rs`): where it looks (the character's feet), and at the top edge of the
    /// screen, half the view's 15 degrees above that.
    fn depths(distance: f32) -> (f32, f32) {
        let (tip, half) = (45f32.to_radians(), 7.5f32.to_radians());
        let height = distance * tip.sin();
        (distance, height / (tip - half).sin() * half.cos())
    }

    #[test]
    fn the_ground_under_the_character_is_clear_of_a_fields_fog() {
        let fog = Fog::of(FIELDS, "prt_fild08").unwrap();
        // A quarter and six tenths of the client's depth of 10 to 1500, in world units.
        assert!((fog.start - 382.5).abs() < 1e-3 && (fog.end - 904.0).abs() < 1e-3);
        assert_eq!(fog.colour, [186.0 / 255.0, 1.0, 119.0 / 255.0]);
        // The camera a map starts with, 300 away: no fog at the character's feet, nor anywhere on level
        // ground in the view.
        let (feet, top) = depths(300.0);
        assert_eq!((fog.clear(feet), fog.clear(top)), (1.0, 1.0));
        // As far as the camera goes without the Zoom Out option, 400: next to none at the feet, and less
        // than a sixth at the very top of the screen.
        let (feet, top) = depths(400.0);
        assert!(fog.clear(feet) > 0.96 && fog.clear(top) > 0.84);
        // With Zoom Out and the camera at its 730, the same sum leaves a third of the ground at the feet:
        // this is the washed-out picture, and it is the client's own.
        let (feet, top) = depths(730.0);
        assert!((fog.clear(feet) - 0.3336).abs() < 1e-3 && fog.clear(top) < 0.15);
    }

    /// `gef_fild13` as the client's table has it: a fog that begins nearer and ends much nearer than a
    /// Prontera field's. The sums are the client's `DepthAlpha` (see the top of the file).
    #[test]
    fn a_geffen_fields_fog_ends_short_of_the_zoomed_out_camera() {
        let fog = Fog::of("gef_fild13.rsw#\n0.2#\n0.45#\n0x2275A9FF#\n0.5#\n", "gef_fild13").unwrap();
        assert!((fog.start - 308.0).abs() < 1e-3 && (fog.end - 680.5).abs() < 1e-3);
        // The colour is the last three bytes; the first (0x22) is no part of the fog over the ground.
        assert_eq!(fog.colour, [117.0 / 255.0, 169.0 / 255.0, 1.0]);
        // The camera a map starts with, 300: clear at the feet, a tenth fog at the top of the screen.
        let (feet, top) = depths(300.0);
        assert!(fog.clear(feet) == 1.0 && (fog.clear(top) - 0.899).abs() < 2e-3);
        // At 400, as far as it goes without Zoom Out: a quarter fog at the feet, two fifths at the top.
        let (feet, top) = depths(400.0);
        assert!((fog.clear(feet) - 0.753).abs() < 1e-3 && (fog.clear(top) - 0.590).abs() < 2e-3);
        // At the 730 of Zoom Out the feet are past the fog's end: nothing of the map shows through.
        let (feet, top) = depths(730.0);
        assert_eq!((fog.clear(feet), fog.clear(top)), (0.0, 0.0));
        // The last distance at which the feet still show at all is the fog's end itself.
        assert!(fog.clear(680.0) > 0.0 && fog.clear(681.0) == 0.0);
    }

    #[test]
    fn a_map_the_table_does_not_name_has_no_fog() {
        assert_eq!(Fog::of(FIELDS, "prontera"), None);
        assert_eq!(Fog::of(FIELDS, "prt_fild0"), None);
    }
}
