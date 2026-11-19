//! What changes about someone in sight: the state it is in, what it wears and holds, what it is, its life, and
//! one who lay dead standing up again.

use super::*;

/// ZC_STATE_CHANGE (0x0229): `u32 id, u16 body state, u16 health state, u32 effect state, u8 pk`. Someone's
/// state changed: the effect state says what they ride, among other things. (Its body state and health state
/// are read by `net::statuses::body_and_health`.)
pub const ZC_STATE_CHANGE: u16 = 0x0229;
/// ZC_NPC_SHOWEFST_UPDATE (0x028a): `u32 id, u32 effect state, u32 level, u32 third option`: the states that
/// show on someone in sight changed (rAthena's `clif_changeoption2`, which it sends for a monster or an NPC).
/// Whose, the bits of the state it is in and those of its third option.
pub const ZC_NPC_SHOWEFST_UPDATE: u16 = 0x028a;
/// ZC_RESURRECTION (0x0148): `u32 id, u16 kind`: someone who lay dead stands up again.
pub const ZC_RESURRECTION: u16 = 0x0148;
/// ZC_SPRITE_CHANGE (0x01d7): `u32 id, u8 kind, u32 value, u32 second`: what someone wears or holds
/// changes. With the kind 2 the value is the weapon and the second the shield.
pub const ZC_SPRITE_CHANGE: u16 = 0x01d7;
/// ZC_NPCSPRITE_CHANGE (0x01b0): `u32 who, u8 kind, u32 class`. A monster becomes another (the server has a
/// Fabre turn into a Pupa and a Pupa into a Creamy by itself, as a skill of theirs).
pub const ZC_NPCSPRITE_CHANGE: u16 = 0x01b0;
/// ZC_HP_INFO (0x0977): `u32 id, i32 hp, i32 most`: how much life a monster has left.
pub const ZC_HP_INFO: u16 = 0x0977;

/// Reads ZC_STATE_CHANGE: whose state changed, and the bits of the state they are in now.
pub fn state_changed(packet: &[u8]) -> Option<(u32, u32)> {
    (packet.len() >= 14).then(|| (u32_at(packet, 2), u32_at(packet, 10)))
}

/// Reads ZC_NPC_SHOWEFST_UPDATE: whose, the bits of the state it is in and those of its third option.
pub fn third_option(packet: &[u8]) -> Option<(u32, u32, u32)> {
    (packet.len() >= 18).then(|| (u32_at(packet, 2), u32_at(packet, 6), u32_at(packet, 14)))
}

/// Reads ZC_RESURRECTION: who stands up again.
pub fn risen(packet: &[u8]) -> Option<u32> {
    (packet.len() >= 6).then(|| u32_at(packet, 2))
}

/// What ZC_SPRITE_CHANGE tells: whose look, the kind of the change and its two values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    pub who: u32,
    pub kind: u8,
    pub value: u32,
    pub second: u32,
}

/// Reads ZC_SPRITE_CHANGE.
pub fn look(packet: &[u8]) -> Option<Look> {
    (packet.len() >= 15)
        .then(|| Look { who: u32_at(packet, 2), kind: packet[6], value: u32_at(packet, 7), second: u32_at(packet, 11) })
}

/// Reads ZC_NPCSPRITE_CHANGE: who becomes another, and the class it is of from now on.
pub fn became(packet: &[u8]) -> Option<(u32, u32)> {
    (packet.len() >= 11).then(|| (u32_at(packet, 2), u32_at(packet, 7)))
}

/// Reads ZC_HP_INFO: whose life, how much is left and the most it can be.
pub fn life(packet: &[u8]) -> Option<(u32, i32, i32)> {
    Some((long(packet, 2)?, int(packet, 6)?, int(packet, 10)?))
}
