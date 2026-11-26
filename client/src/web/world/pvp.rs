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
    /// On a battleground: the player's own team, and the team of each of the others the server has named one
    /// of, by account (the client's `playerTeam` and each actor's `battleTeam`).
    team: u8,
    teams: HashMap<u32, u8>,
}

/// The server said what holds on the map the character is on.
pub fn zone(zone: Zone) {
    STATE.with_borrow_mut(|state| state.pvp.zone = zone);
}

/// Another map: the teams of those on the map before are forgotten with them (an actor's `battleTeam` goes
/// with the actor; the player's own team stays, as the client's does).
pub fn teams_left() {
    STATE.with_borrow_mut(|state| state.pvp.teams.clear());
}

/// The player's own team on a battleground, as the server named it.
pub fn own_team(team: u8) {
    STATE.with_borrow_mut(|state| state.pvp.team = team);
}

/// The team of another on a battleground.
pub fn team(id: u32, team: u8) {
    STATE.with_borrow_mut(|state| state.pvp.teams.insert(id, team));
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
        if !matches!(unit.look, Look::Character(_)) || unit.companion || unit.dying.is_some() {
            return false;
        }
        // On a battleground the two teams fight, whatever party or guild one is of.
        if self.zone.battlefield {
            return crate::battleground::foe(self.team, self.teams.get(&id).copied().unwrap_or(0));
        }
        self.zone.fights(self.party.contains(&id), unit.guild.0 != 0 && unit.guild.0 == self.guild, self.allies.contains(&unit.guild.0))
    }
}
