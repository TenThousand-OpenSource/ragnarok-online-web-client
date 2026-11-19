//! The units skills leave on the ground, and skills used on a place: what the server says of them
//! (`net/skill_units.rs`, `world/skill_units.rs`).

use super::*;
use crate::net::skill_units;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn skill_units_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // (A trap that is not for everyone to see is seen by the one who laid it: the client's
            // `AM_SKILLSTANDENTRY`.)
            skill_units::ZC_SKILL_ENTRY5 => {
                if let Some(mut entry) = skill_units::entry(packet) {
                    entry.visible |= entry.creator == self.own_account();
                    world::skill_unit_shown(&entry);
                }
            }
            // A skill strikes: if it is a unit's (a trap going off), the unit is told (the client's `AM_SKILL`
            // to its `CSkill`). Overheard only: the blow itself is the fight's to read.
            skills::ZC_NOTIFY_SKILL => {
                if let Some(landing) = skills::landing(packet) {
                    world::skill_unit_strikes(landing.source);
                }
                return false;
            }
            skill_units::ZC_SKILL_DISAPPEAR => {
                if let Some(id) = skill_units::gone(packet) {
                    world::skill_unit_gone(id);
                }
            }
            // A skill is used on a place (the client's `CGameActor::PlayGroundSkillEffect`): the cast is over,
            // the one who used it goes through the motion the client has for the skill, and the skill's
            // effect plays at the place. (The client also turns the caster to the place, which is not done.)
            skills::ZC_NOTIFY_GROUNDSKILL => {
                if let Some(skills::OnGround { skill, who, cell, .. }) = skills::on_ground(packet) {
                    let own = self.own_account();
                    world::cast_over(who, own, false);
                    world::uses_skill(who, own, effect_tables::skill_state(skill));
                    world::ground_skill(skill, cell, (who != own).then_some(who));
                }
            }
            // A cell of the map can or cannot be walked on from now on (an Ice Wall): the ways walked, the
            // player's and everyone's, go by it.
            skill_units::ZC_UPDATE_MAPINFO => {
                if let Some((cell, kind)) = skill_units::cell_changed(packet) {
                    world::cell_retyped(cell, kind);
                }
            }
            _ => return false,
        }
        true
    }
}
