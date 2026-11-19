//! The kinds the client builds corner by corner out of its four parts and draws with `RenderTeiRect`: each part
//! (`EffectPrimitiveSlot`) lives by a count of its own and has its own alpha, and the kind's drawing function
//! makes rectangles of the parts' figures, each rectangle with its own colour and its own way of being laid on
//! what is behind it. `prim/slots.rs` has two such kinds written out by hand; the ones here keep the client's
//! part whole (`Part`) and its own directions, so that each is its function of the client line for line:
//!
//! - the kind 89: beams that open like a fan from a place (`tei/beams.rs`);
//! - the kind 69: a streak in five sides between two places (`tei/beams.rs`);
//! - `PP_WIND` (62), `PP_SPHEREWIND` (77), `PP_SPHEREWIND2` (91): bands of wind that go round (`tei/winds.rs`);
//! - the kinds 92 and 103: squares that slide along the ground, and squares stacked that turn
//!   (`tei/squares.rs`);
//! - the kind 115 and `PP_SHIELDBOOMERANG3` (116): squares that face the screen (`tei/panels.rs`);
//! - `PP_HITLINE3` (100): sparks that leap out in a bow (`tei/arcs.rs`);
//! - the kinds 63, 60 (its walls that turn), 36 and 39: rings that widen, walls that turn, low walls that
//!   lean out, a square that breathes (`tei/ripples.rs`, `columns.rs`, `low_rings.rs`, `halo.rs`).
//! - `PP_GI_3` (34) and `PP_GRANDCROSS2` (37): quarters of a wall and long boxes, the walls and the bars of
//!   Grand Cross (`tei/quarters.rs`, `tei/boxes.rs`);
//! - `PP_SHIELDBOOMERANG` (42) and `PP_TEIHIT2` (45): squares facing the screen that are thrown and come
//!   back, or fly from the one struck (`tei/boomerang.rs`, `tei/flecks.rs`), drawn by `RenderFieldPanel`
//!   (`tei/field.rs`);
//! - `PP_INTIMIDATE` (43) and the kind 44: squares that fly out spinning like coins, and long plates that
//!   shoot out every way (`tei/orbit.rs`, `tei/plates.rs`);
//! - `PP_GUARD` (61): half a ball of panels before someone (`tei/cage.rs`);
//! - `PP_HITLINE` (70), `PP_MAGNUM2` (73) and the kind 87: sparks that leap and leap on, walls that lean out
//!   like a funnel, a square that comes down turning (`tei/bounces.rs`, `tei/cones.rs`, `tei/drops.rs`).
//! - `PP_LINELINK` (38): a tube of light from one actor to another, which it follows (`tei/links.rs`).
//! - `PP_SLASH1` (75): blades that shoot out along the ground (`tei/slashes.rs`); the kinds 119 and 126,
//!   squares and wheels of wind that fly off the way they are sent (`tei/gusts.rs`, `tei/gust_rings.rs`),
//!   and 122 and `PP_STIN4` (125), which fly round to someone in a bow (`tei/chasers.rs`);
//! - the kind 135: a beam that sweeps round with sheets of light behind it, and `PP_ELECTRIC` (101): arcs
//!   of lightning that reach out (`tei/scans.rs`);
//! - the kinds 82, 140 and 141: a picture over someone that faces the screen and glows, runs through its
//!   pictures, or is drawn out like a stroke (`tei/auras.rs`, `tei/pillars.rs`, `tei/stretch.rs`);
//! - `PP_FIRSTAID` (58) and the kind 67: a glitter that comes and goes, a footprint that fades (`tei/aid.rs`);
//!   the kind 145: a light that goes round someone with upright squares about it (`tei/lights.rs`);
//! - the kind 132: upright squares of light that grow and begin anew (`tei/twinkles.rs`); the kind 137:
//!   shafts that shoot up (`tei/pillars.rs`); the kind 146: walls that fly off one after the other
//!   (`tei/drifts.rs`); the kind 159: needles that fly in and whirl (`tei/blades.rs`);
//! - `PP_ASURACASTING` (68) and the kind 147: a picture drawn ten times over, larger and fainter
//!   (`tei/layers.rs`); `PP_SOULLINK` (109) and `PP_SOULPARTICLE` (111): the light of a soul link and its
//!   sparks (`tei/souls.rs`); the kinds 154 and 156: two wedges towards a place, a picture that hovers
//!   (`tei/fins.rs`); the kind 157: squares that fall and go out (`tei/dust.rs`).
//! - the kind 35 and `PP_DOUBLEGUMGANG` (99): signs that flash up round a head (`tei/icons.rs`); `PP_PEONG`
//!   (94) and `PP_PEONGUP` (96): the puff of a landing (`tei/peong.rs`); `PP_FLOWERCAST` (108): walls like
//!   petals (`tei/flowers.rs`); the kinds 155 and 170: lights whose way wanders (`tei/flames.rs`);
//!   `PP_FOREST` (66): shafts of light that slant down (`tei/forest.rs`); `PP_SMA` (120) and the kind 151,
//!   which show nothing and ask for other effects where they are (`tei/launchers.rs`).
//! - the kind 32: a sphere of light that goes round someone, a monk's spirit sphere (`tei/spheres.rs`).
//! - the lights of the aura of the highest level, which the client keeps as particles of three effects of
//!   their own and not as primitives: squares that breathe over the ground, bands that go round, lights that
//!   rise (`tei/max_level.rs`).
//!
//! The places of a part are in the client's directions, from the effect's place as the primitive was launched:
//! x east, y DOWN, z north. `Rect` gives them in this client's (east, up, north).
//!
//! What `RenderTeiRect` does besides and is not done here: it lets its effect tell it another colour and
//! another blend (`colorChannels`, which no effect ported sets), and it draws some rectangles over everything
//! or a touch nearer than they stand (its last argument); here all are drawn where they stand. Its alpha is a
//! byte of the colour, so only the low eight bits of a part's alpha show (`Rect::new`).

use super::*;

mod part;
mod rect;

pub use part::Part;
use part::under_a_round;
pub use rect::Rect;
use rect::{edges, to_screen};

mod aid;
mod arcs;
mod auras;
mod beams;
mod blades;
mod boomerang;
mod bounces;
mod boxes;
mod cage;
mod chasers;
mod columns;
mod drifts;
mod dust;
mod cones;
mod drops;
mod field;
mod fins;
mod flames;
mod flecks;
mod flowers;
mod forest;
mod gust_rings;
mod gusts;
mod halo;
mod icons;
mod launchers;
mod layers;
mod lights;
mod links;
mod low_rings;
mod max_level;
mod orbit;
mod panels;
mod peong;
mod pillars;
mod plates;
mod quarters;
mod ripples;
mod scans;
mod sizes;
mod sky;
pub use sky::PUT_ANEW;
mod slashes;
mod souls;
mod spheres;
mod squares;
mod stretch;
mod twinkles;
mod winds;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod more_tests;

/// The kinds here, by the client's numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tei {
    /// The kind 89.
    FanBeam,
    /// The kind 69.
    PentaRing,
    /// `PP_WIND` (62).
    Wind,
    /// `PP_SPHEREWIND` (77).
    SphereWind,
    /// `PP_SPHEREWIND2` (91).
    SphereWind2,
    /// The kind 92.
    FloorSquare,
    /// The kind 103.
    StackedRings,
    /// The kind 115.
    Spread,
    /// `PP_SHIELDBOOMERANG3` (116).
    ShieldBoomerang3,
    /// `PP_HITLINE3` (100).
    HitLine3,
    /// The kind 63 (`tei/ripples.rs`).
    Ripple,
    /// The kind 60 whose columns are of the kinds 1, 2 and 3 (`tei/columns.rs`).
    Columns,
    /// The kind 36 (`tei/low_rings.rs`).
    LowRing,
    /// The kind 39 (`tei/halo.rs`).
    Halo,
    /// `PP_GI_3` (34) (`tei/quarters.rs`).
    Gi3,
    /// `PP_GRANDCROSS2` (37) (`tei/boxes.rs`).
    GrandCross2,
    /// `PP_SHIELDBOOMERANG` (42) (`tei/boomerang.rs`).
    ShieldBoomerang,
    /// `PP_INTIMIDATE` (43) (`tei/orbit.rs`).
    Intimidate,
    /// The kind 44 (`tei/plates.rs`).
    ExpandPulse,
    /// `PP_TEIHIT2` (45) (`tei/flecks.rs`).
    TeiHit2,
    /// `PP_GUARD` (61) (`tei/cage.rs`).
    Guard,
    /// `PP_HITLINE` (70) (`tei/bounces.rs`).
    HitLine,
    /// `PP_MAGNUM2` (73) (`tei/cones.rs`).
    Magnum2,
    /// The kind 87 (`tei/drops.rs`).
    Drop,
    /// `PP_SLASH1` (75) (`tei/slashes.rs`).
    Slash1,
    /// The kinds 119 and 126 (`tei/gusts.rs`, `tei/gust_rings.rs`).
    Gust,
    GustRing,
    /// The kinds 122 and `PP_STIN4` (125) (`tei/chasers.rs`).
    GustChase,
    GustSwerve,
    /// The kind 135 (`tei/scans.rs`), and `PP_ELECTRIC` (101) there.
    JitterSweep,
    Electric,
    /// The kind 82 (`tei/auras.rs`).
    Aura,
    /// The kind 132 (`tei/twinkles.rs`).
    Twinkle,
    /// The kind 146 (`tei/drifts.rs`).
    DriftRing,
    /// `PP_LINELINK` (38) (`tei/links.rs`).
    LineLink,
    /// The kinds 137 and 140 (`tei/pillars.rs`).
    ArcWall,
    FrameFade,
    /// The kind 159 (`tei/blades.rs`).
    SpiralBlade,
    /// `PP_ASURACASTING` (68) and the kind 147 (`tei/layers.rs`).
    AsuraCasting,
    RisingFan,
    /// `PP_SOULLINK` (109) and `PP_SOULPARTICLE` (111) (`tei/souls.rs`).
    SoulLink,
    SoulParticle,
    /// The kinds 154 and 156 (`tei/fins.rs`).
    BladeFins,
    Hover,
    /// The kind 141 (`tei/stretch.rs`).
    TaperedQuad,
    /// The kind 157 (`tei/dust.rs`).
    GlowSquare,
    /// `PP_FIRSTAID` (58) and the kind 67 (`tei/aid.rs`).
    FirstAid,
    Footprint,
    /// The kind 145 (`tei/lights.rs`).
    DriftingSparks,
    /// The kind 35 and `PP_DOUBLEGUMGANG` (99) (`tei/icons.rs`).
    OrbitIcons,
    DoubleGumGang,
    /// `PP_PEONG` (94) and `PP_PEONGUP` (96) (`tei/peong.rs`).
    Peong,
    PeongUp,
    /// `PP_FLOWERCAST` (108) (`tei/flowers.rs`).
    FlowerCast,
    /// The kinds 155 and 170 (`tei/flames.rs`).
    WanderingFlames,
    WanderingFlames2,
    /// `PP_SMA` (120) and the kind 151, which show nothing and ask for effects (`tei/launchers.rs`).
    Sma,
    SpiralFade,
    /// `PP_FOREST` (66) (`tei/forest.rs`).
    Forest,
    /// The kind 33: squares facing the screen that come and go over the whole view: the clouds and the
    /// stars of a sky (`tei/sky.rs`).
    Sky,
    /// The kind 32 (`tei/spheres.rs`).
    DriftingOrbs,
    /// The lights of the aura of the highest level, which are no kinds of `CEffectPrim` (`tei/max_level.rs`).
    Level99Glow,
    Level99Swirl,
    Level99Orb,
}

impl Prim {
    /// One tick of the parts of a kind of this file.
    pub(super) fn tick_parts(&mut self, kind: Tei) {
        match kind {
            Tei::ShieldBoomerang3 => return self.fly_with_a_tail(),
            Tei::Ripple => return self.ripple_rings(),
            Tei::ShieldBoomerang => return self.fly_out_and_back(),
            Tei::Drop => return self.drop_turning(),
            Tei::Gust => return self.blow_as_gust(false),
            Tei::GustRing => return self.blow_as_gust(true),
            Tei::GustChase => return self.chase_as_gust(),
            Tei::GustSwerve => return self.swerve_as_gust(),
            Tei::DriftingSparks => return self.drift_as_lights(),
            Tei::DriftingOrbs => return self.drift_as_sphere(),
            _ => {}
        }
        for (index, part) in self.parts.iter_mut().enumerate().filter(|(_, part)| part.live) {
            let index = index as i32;
            match kind {
                Tei::FanBeam => part.open_as_beam(),
                Tei::PentaRing => part.reach_as_streak(),
                Tei::Wind => part.blow_as_wind(),
                Tei::SphereWind => part.go_round(),
                Tei::SphereWind2 => part.go_round_widening(),
                Tei::FloorSquare => part.slide_as_square(),
                Tei::StackedRings => part.spin_and_fade(),
                Tei::Spread => part.spread_sparks(),
                Tei::ShieldBoomerang3 | Tei::Ripple | Tei::ShieldBoomerang | Tei::Gust | Tei::GustRing | Tei::GustChase | Tei::GustSwerve => {}
                Tei::Slash1 => part.shoot_as_slash(),
                Tei::JitterSweep => part.shift_as_trail(index),
                Tei::Electric => part.arc_as_lightning(),
                Tei::Aura if index == 0 => part.glow_as_aura(),
                Tei::Aura => {}
                Tei::Twinkle => part.twinkle(),
                Tei::DriftRing => part.drift_as_ring(),
                Tei::ArcWall => part.ramp_up(),
                Tei::GlowSquare => part.fall_as_dust(),
                Tei::OrbitIcons | Tei::DoubleGumGang => part.float_as_signs(),
                Tei::Peong => part.puff_out(),
                Tei::PeongUp => part.puff_up(),
                Tei::FlowerCast => part.breathe_as_petal(),
                Tei::WanderingFlames => part.wander_as_flames(false),
                Tei::WanderingFlames2 => part.wander_as_flames(true),
                Tei::Sma => part.fly_as_sma(),
                Tei::SpiralFade => part.circle_as_light(),
                Tei::Forest => part.shine_as_shaft(index),
                Tei::Sky => part.drift_as_cloud(),
                Tei::Level99Glow => part.breathe_as_glow(),
                Tei::Level99Swirl => part.turn_as_swirl(),
                Tei::Level99Orb => part.rise_as_orb(),
                Tei::TaperedQuad => part.draw_out(),
                Tei::BladeFins => part.flash_and_fade(),
                Tei::Hover if index == 0 => part.hover(),
                Tei::Hover => {}
                Tei::SoulLink => part.fly_as_soul(),
                Tei::SoulParticle => part.drift_as_soul_sparks(),
                Tei::AsuraCasting => part.flash_as_letter(),
                Tei::RisingFan => part.rise_as_fan(),
                Tei::SpiralBlade => part.collapse_in_a_spiral(),
                Tei::FrameFade => part.fade_by_frames(self.pictures.len()),
                Tei::Gi3 => part.rise_as_quarter(),
                Tei::GrandCross2 => part.rise_as_box(),
                Tei::Intimidate => part.fly_as_coin(),
                Tei::ExpandPulse => part.shoot_as_plate(),
                Tei::TeiHit2 => part.fly_as_fleck(),
                Tei::Guard => part.guard(),
                Tei::HitLine => part.bounce_as_spark(),
                Tei::Magnum2 => part.flash_as_cone(),
                Tei::Drop => {}
                Tei::Columns => part.turn_as_columns(index),
                Tei::LowRing => part.widen_as_low_ring(index),
                Tei::Halo if index < 2 => part.breathe_as_halo(),
                Tei::Halo => {}
                Tei::HitLine3 => part.leap_as_spark(),
                Tei::LineLink if index == 0 => part.follow_as_link(),
                Tei::FirstAid if index == 0 => part.glitter_as_aid(),
                Tei::Footprint if index == 0 => part.fade_as_print(),
                Tei::FirstAid | Tei::Footprint | Tei::DriftingSparks => {}
                Tei::DriftingOrbs => {}
                Tei::LineLink => {}
            }
        }
    }

    /// The rectangles of a kind of this file now; none for any other kind. `right` and `up` are the screen's
    /// two directions in the world, for the kinds that face it.
    pub fn rects(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let Shape::Tei(kind) = self.kind else { return Vec::new() };
        let live = self.parts.iter().filter(|part| part.live);
        match kind {
            Tei::FanBeam => live.flat_map(|part| part.fan(self.picture)).collect(),
            Tei::PentaRing => live.flat_map(|part| part.five_sides(self.picture)).collect(),
            Tei::Wind => live.flat_map(|part| part.rising_arc(self.picture, self.size)).collect(),
            Tei::SphereWind | Tei::SphereWind2 => live.flat_map(|part| part.tipped_band(self.picture, self.size)).collect(),
            Tei::FloorSquare => live.flat_map(|part| part.level_square(self.picture)).collect(),
            Tei::StackedRings => live.flat_map(|part| part.stacked_squares(self.picture)).collect(),
            Tei::Spread | Tei::SoulParticle => live.flat_map(|part| part.sparks(self.picture, right, up)).collect(),
            Tei::ShieldBoomerang3 => live.flat_map(|part| part.panel(self.picture, right, up)).collect(),
            Tei::HitLine3 => live.flat_map(|part| part.bow(self.picture)).collect(),
            Tei::LineLink => live.take(1).flat_map(|part| part.tube(self.picture)).collect(),
            Tei::Ripple | Tei::Columns => live.flat_map(|part| part.casting(self.picture, self.size, 20)).collect(),
            Tei::LowRing => live.flat_map(|part| part.casting(self.picture, self.size, 10)).collect(),
            Tei::Halo | Tei::Footprint => self.parts.iter().enumerate().filter(|(_, part)| part.live).flat_map(|(number, part)| part.sized_square(self.picture, self.size, number as i32)).collect(),
            Tei::Gi3 => self.parts.iter().enumerate().filter(|(_, part)| part.live).flat_map(|(number, part)| part.quarter(self.picture, number, self.size)).collect(),
            Tei::GrandCross2 => live.flat_map(|part| part.open_box(self.picture)).collect(),
            Tei::ShieldBoomerang | Tei::TeiHit2 | Tei::Drop | Tei::FirstAid | Tei::Sky => self.field_panels(right, up),
            Tei::DriftingSparks => self.pillar_squares(right, up),
            Tei::Level99Glow => self.level99_glows(right),
            Tei::Level99Swirl => self.level99_swirls(right),
            Tei::Level99Orb => self.level99_orbs(right, up),
            Tei::DriftingOrbs => self.layered_glow(right, up),
            Tei::HitLine => live.flat_map(|part| part.bouncing(self.picture)).collect(),
            Tei::Magnum2 => live.flat_map(|part| part.cone(self.picture, self.size)).collect(),
            Tei::Intimidate => live.filter_map(|part| part.coin(self.picture, right, up)).collect(),
            Tei::ExpandPulse => live.filter_map(|part| part.plate(self.picture)).collect(),
            Tei::Guard => live.flat_map(|part| part.cage(self.pictures)).collect(),
            Tei::Slash1 => live.flat_map(|part| part.slash(self.picture)).collect(),
            Tei::Gust | Tei::GustChase | Tei::GustSwerve => live.flat_map(|part| part.spinning_square(self.picture)).collect(),
            Tei::GustRing => live.flat_map(|part| part.upright_square(self.picture)).collect(),
            Tei::JitterSweep => self.jitter_sweeps(),
            Tei::Electric => live.flat_map(|part| part.lightning_arc(self.picture)).collect(),
            Tei::Aura => live.flat_map(|part| part.aura(self.picture, right, up)).collect(),
            Tei::Twinkle => live.flat_map(|part| part.colour_panel(self.picture)).collect(),
            Tei::DriftRing => self.trailing_arcs(),
            Tei::SpiralBlade => live.flat_map(|part| part.blades(self.picture)).collect(),
            Tei::AsuraCasting | Tei::RisingFan => live.flat_map(|part| part.halo_layers(self.pictures, self.picture, right, up)).collect(),
            Tei::SoulLink => live.flat_map(|part| part.soul(self.picture, right, up)).collect(),
            Tei::BladeFins => live.flat_map(|part| part.fins(self.picture)).collect(),
            Tei::Hover => live.flat_map(|part| part.jitter_quad(self.picture, self.parts[0].flags[1], right, up)).collect(),
            Tei::TaperedQuad => live.flat_map(|part| part.tapered_quad(self.picture, right, up)).collect(),
            Tei::GlowSquare => live.flat_map(|part| part.glow_square(self.picture, right, up)).collect(),
            Tei::OrbitIcons => live.flat_map(|part| part.orbit_icons(self.pictures, right, up)).collect(),
            Tei::DoubleGumGang => live.flat_map(|part| part.double_signs(self.pictures, right, up)).collect(),
            Tei::Peong => live.flat_map(|part| part.scrap(self.picture, right, up)).collect(),
            Tei::PeongUp => live.flat_map(|part| part.rising_light(self.picture, right, up)).collect(),
            Tei::FlowerCast => live.flat_map(|part| part.casting(self.picture, self.size, 20)).collect(),
            Tei::WanderingFlames | Tei::WanderingFlames2 => live.flat_map(|part| part.twin_glows(self.picture, right, up)).collect(),
            Tei::Sma | Tei::SpiralFade => Vec::new(),
            Tei::Forest => live.flat_map(|part| part.slanting_shaft(self.picture)).collect(),
            Tei::ArcWall => live.flat_map(|part| part.arc_walls(self.picture, self.size)).collect(),
            Tei::FrameFade => live.flat_map(|part| part.aura(self.pictures.get(part.flags[0] as usize).copied().unwrap_or(self.picture), right, up)).collect(),
        }
    }
}
