//! Fighting other players: which map the character is on (`crate::pvp::Zone`) and who its friends are,
//! as the app tells them, and whether someone on the map is fought as a monster is. The clicking and the pointer
//! go by `foe`.

use super::*;
use crate::pvp::Zone;

/// What the world knows of it.
#[derive(Default)]
pub(super) struct Pvp {
    /// What holds on this map.
    zone: Zone,
    /// The accounts of the player's party, the player's own guild, and the guilds allied to it.
    party: Vec<u32>,
    guild: u32,
    allies: Vec<u32>,
}

/// The server said what holds on the map the character is on.
pub fn zone(zone: Zone) {
    STATE.with_borrow_mut(|state| state.pvp.zone = zone);
}

/// Who the player's friends are: the accounts of the party, the player's guild (0 for none) and the guilds
/// allied to it.
pub fn sides(party: Vec<u32>, guild: u32, allies: Vec<u32>) {
    STATE.with_borrow_mut(|state| (state.pvp.party, state.pvp.guild, state.pvp.allies) = (party, guild, allies));
}

impl Pvp {
    /// Whether that one is another player who is fought on this map (`Zone::fights`): then a press attacks
    /// them and the pointer over them is the sword, as with a monster. A guild is the same as the player's only
    /// when there is one (the client's `IsLocalPlayerAid` is false for no guild).
    pub(super) fn foe(&self, id: u32, unit: &Mover) -> bool {
        matches!(unit.look, Look::Character(_))
            && !unit.companion
            && unit.dying.is_none()
            && self.zone.fights(self.party.contains(&id), unit.guild.0 != 0 && unit.guild.0 == self.guild, self.allies.contains(&unit.guild.0))
    }
}
