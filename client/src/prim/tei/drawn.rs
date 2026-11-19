//! What a kind built of the client's parts shows now: which function of which file draws each kind's
//! rectangles (the client's `RenderTeiRect` and the drawing functions that end in it).

use super::*;

impl Prim {
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
