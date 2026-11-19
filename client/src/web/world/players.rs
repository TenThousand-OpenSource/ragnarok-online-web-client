//! What the app asks about the other players on the map: who one is, and where one of a name stands. The menu
//! on a player, the party and a deal between two go by these.

use super::*;

/// What another player on the map is called; none for anyone who is not a player (the client's menu on an
/// actor is for those `IsPlayerJob` holds for, whose name it knows).
pub fn player_name(id: u32) -> Option<String> {
    STATE.with_borrow(|state| {
        let unit = state.scene.as_ref()?.units.get(&id)?;
        (matches!(unit.look, Look::Character(_)) && !unit.companion && !unit.name.is_empty()).then(|| unit.name.clone())
    })
}

/// The player of that name in sight, if there is one (the client's `nameCard.FindId`, which `/deal` goes by).
pub fn player_named(name: &str) -> Option<u32> {
    STATE.with_borrow(|state| {
        let scene = state.scene.as_ref()?;
        scene.units.iter().find(|(_, unit)| matches!(unit.look, Look::Character(_)) && !unit.companion && unit.name == name).map(|(id, _)| *id)
    })
}
