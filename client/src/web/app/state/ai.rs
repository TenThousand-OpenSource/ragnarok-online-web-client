//! What the app keeps of the scripts that move the homunculus and the mercenary (`crate::ai`): the scripts
//! running, and whom everyone in sight last struck.

use std::collections::HashMap;

use crate::ai::Ai;

/// The script of one of the two.
#[derive(Default)]
pub(in crate::web) struct Script {
    /// The script, once loaded: for as long as its homunculus or mercenary is out (the client makes a new
    /// `CMercenaryAI` for each actor).
    pub running: Option<Ai>,
    /// Whether it is the player's own script (the client's `/hoai` and `/merai`) that runs.
    pub user: bool,
    /// The script could not be loaded or failed: it is not tried again until its owner is out anew or the
    /// player's choice of script changes.
    pub failed: bool,
}

/// The scripts, and what they are told that the world does not keep.
#[derive(Default)]
pub(in crate::web) struct Scripts {
    /// The homunculus's, then the mercenary's.
    pub of: [Script; 2],
    /// Whom each one in sight last struck or aimed a skill at (the client's `hitTargetAid`).
    pub targets: HashMap<u32, u32>,
    /// The timer that calls the scripts is set.
    pub ticking: bool,
}
