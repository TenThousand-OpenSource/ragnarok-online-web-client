//! What the client asks for its own character among those in sight: to attack, to stop, to sit down and stand
//! up, to turn.

use super::*;

/// CZ_CANCEL_LOCKON (0x0118): the server stops the blows it was dealing on its own.
pub fn stop_attack() -> Vec<u8> {
    Writer::new(0x0118).done()
}

/// CZ_REQUEST_ACT2 (0x0437): `u32 target, u8 action`: 7 attacks until told otherwise, 0 once.
pub fn attack(target: u32) -> Vec<u8> {
    request(target, 7)
}

/// The same request for one blow only.
pub fn attack_once(target: u32) -> Vec<u8> {
    request(target, 0)
}

/// The same request to sit down (2) or to stand up (3): no target.
pub fn sit(down: bool) -> Vec<u8> {
    request(0, if down { 2 } else { 3 })
}

fn request(target: u32, action: u8) -> Vec<u8> {
    Writer::new(0x0437).u32(target).u8(action).done()
}

/// CZ_CHANGE_DIRECTION (0x0361 for this client): `u16 head` (how the head is turned: 0 ahead, 1 to the
/// right, 2 to the left), `u8 direction` (0 north, turning left).
pub fn face(direction: u8) -> Vec<u8> {
    turn(0, direction)
}

/// The same request with the head turned.
pub fn turn(head: u8, direction: u8) -> Vec<u8> {
    Writer::new(0x0361).u16(u16::from(head.min(2))).u8(direction % 8).done()
}
