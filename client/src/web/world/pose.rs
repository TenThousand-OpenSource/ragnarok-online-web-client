//! How those in sight are posed and shown, as the server tells of each: turning on the spot and turning the
//! head, lying dead and rising, hiding and coming out of it. (What `mover.rs` keeps of it: `Pose`.)
//!
//! Beside these: the spirit spheres about one are `spirits.rs`, the falcon over a hunter's shoulder
//! `falcon.rs`, the aura of the highest level `aura.rs`; a blade caught and the frames of a combo `combat.rs`.

use super::*;

/// The client's `EF_BASH`, which it shows on one that hides (but not on a ninja: jobs 25 and 4222).
const EF_BASH: u32 = 16;
/// The puff of smoke of one that hides or comes out (`MAKETYPE_SMOKE_EFFECT`: the first action of
/// `이팩트\smoke`, played once as the breaking ice is).
pub(super) const SMOKE: (&str, usize) = ("이팩트/smoke", 0);

impl Mover {
    /// Whether its sprite is not drawn (`crate::pose::hidden`): it hides, cloaks or is invisible.
    /// The client leaves the sprite of the player's own character out as well.
    pub(super) fn hidden(&self) -> bool {
        crate::pose::hidden(self.state)
    }

    /// Whether a shadow lies under it (`crate::pose::shadowed`): not under one who sits or lies dead or
    /// vanishes, nor under another who hides, cloaks or is invisible, nor under the player's own character
    /// cloaked or invisible.
    pub(super) fn shadowed(&self, own: bool) -> bool {
        let character = matches!(self.look, Look::Character(_));
        let down = crate::pose::down(character, self.dying.is_some(), self.dead, self.sitting);
        crate::pose::shadowed(down, self.pose.vanishes.is_some(), own, self.state)
    }

    /// Its state changes from `was`, as news (`CGameActor::SetStatusStates` with the word that it is):
    /// hiding or coming out of it puffs smoke where it stands. Gives back whether the client's `EF_BASH`
    /// shows on it: on one that hides, unless a ninja.
    pub(super) fn hides(&mut self, was: u32, time: f64) -> bool {
        if !crate::pose::hiding_changed(was, self.state) {
            return false;
        }
        self.pose.smoke = Some((time, self.at));
        crate::pose::bashed_hiding(self.state, self.job)
    }

    /// It falls dead and lies (`ST_DEAD`): through the pictures of its fall once, each for as long as the
    /// sprite says, and then in the last.
    pub(super) fn falls(&mut self, time: f64) {
        let lasts = match &self.look {
            Look::Character(figure) => {
                figure::prepare(figure, crate::pose::LIES);
                figure::body_timing(figure.job, figure.sex, crate::pose::LIES)
            }
            Look::Creature(_) | Look::Model(_) | Look::Unseen => None,
        };
        (self.dead, self.walk, self.sitting) = (true, None, false);
        self.pose.at_ease();
        self.motion = Some((crate::pose::LIES, time, crate::pose::fall_lasts(lasts)));
    }

    /// It stands up again (`AM_RESURRECTION`).
    pub(super) fn rises(&mut self) {
        (self.dead, self.motion) = (false, None);
        self.pose.at_ease();
    }
}

/// Runs something on someone in sight, or on the player's own character when none is named.
fn on(id: Option<u32>, then: impl FnOnce(&mut Mover)) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        if let Some(mover) = scene.mover_mut(id) {
            then(mover);
        }
    });
}

/// Someone turns where it stands, or turns its head (ZC_CHANGE_DIRECTION; none named: the player's own
/// character): the direction as the server counts it (0 north, turning left), the head 0 ahead, 1 to the
/// right, 2 to the left. One who walks faces where it goes.
pub fn turned(id: Option<u32>, head: u8, direction: u8) {
    on(id, |mover| {
        if mover.walk.is_none() {
            (mover.facing, mover.pose.head) = (crate::pose::facing_of(direction), head.min(2));
        }
    });
}

/// The player's own character looks towards a cell without walking there (the client's `AM_REQ_CHANGE_DIR`:
/// a press on the ground with Shift held, or while it sits; `crate::pose::looks`). Gives back what the server
/// is to be told: how the head is turned and the direction as it counts them. Nothing while it walks, is in
/// a motion or lies dead (the client asks for `ST_STAND`, `ST_DAMAGE` or `ST_SIT`), or for its own cell.
pub(super) fn looks_at(state: &mut State, cell: (i32, i32)) -> Option<(u8, u8)> {
    let player = &mut state.scene.as_mut()?.player;
    if player.walk.is_some() || player.dead || player.motion.is_some() || cell == player.cell {
        return None;
    }
    let towards = crate::map::direction(cell.0 - player.cell.0, cell.1 - player.cell.1);
    (player.facing, player.pose.head) = crate::pose::looks(player.facing, player.pose.head, towards);
    Some((player.pose.head, crate::pose::direction_of(player.facing)))
}

impl Scene {
    /// Whether a press on the ground turns the player's own character and does not walk it: it sits.
    pub(super) fn turns_only(&self) -> bool {
        self.player.sitting
    }
}

/// Another player who lay dead stands up again (ZC_RESURRECTION).
pub fn risen(id: u32) {
    on(Some(id), Mover::rises);
}

/// The effects of a state that changed as news, shown once the world's state is let go: `EF_BASH` on one
/// that hid.
pub(super) fn hid(id: Option<u32>, bash: bool) {
    if bash {
        burst(EF_BASH, id, None, 0.0);
    }
}
