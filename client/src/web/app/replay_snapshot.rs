//! A replay's snapshot in the browser: what the session and the map hold as a recording begins, gathered out
//! of the app's state and the world into the Windows client's sections of the file (`taken.rs`: the client's
//! `MM_START_REPLAY_RECORDING`), and the same given back to the app and the world as a replay begins from a
//! file (`given.rs`: what `CGameMode::Initialize` reads in the replay's state 6). The sections themselves, and
//! every rule of turning one thing into the other, are `crate::replay::{sections, snapshot}`.

use super::*;

mod given;
mod taken;

pub(in crate::web::app) use taken::Taken;
