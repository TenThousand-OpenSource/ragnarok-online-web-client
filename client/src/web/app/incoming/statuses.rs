//! The states someone is in (buffs and debuffs) going on and off: the character's own are listed at the side
//! of the screen; three of them say what anyone rides or keeps.

use super::*;
use crate::net::statuses::{self, Change};

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn statuses_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // A state goes on or off (the client's `ApplyStatusChange`, and `ReceiveStatusChange` for the
            // form without a time); or someone comes into sight in one (`ReceiveStatusChangeEntry`), which
            // is the same to the client: a state that is on, with its time.
            statuses::ZC_MSG_STATE_CHANGE | statuses::ZC_MSG_STATE_CHANGE2 | statuses::ZC_MSG_STATE_CHANGE3 | statuses::ZC_EFST_SET_ENTER2 => {
                if let Some(change) = statuses::changed(packet) {
                    self.rides(&change);
                    // The effects that live for as long as the state (`crate::state_effects`).
                    world::state_effects(
                        (Some(change.who) != self.account.as_ref().map(|account| account.account_id)).then_some(change.who),
                        u32::from(change.state),
                        change.on
                    );
                    // Playing dead (EFST_TRICKDEAD, 29) lays anyone down.
                    if change.state == 29 {
                        world::plays_dead(change.who, self.own_account(), change.on);
                    }
                    if Some(change.who) == self.account.as_ref().map(|account| account.account_id) {
                        self.state_changed(&change);
                    }
                }
            }
            _ => return false,
        }
        true
    }

    /// Three of the states say what someone rides or keeps, which is how the server tells a player of its own
    /// mount or falcon as it logs in: riding (27: a bird or a dragon, by the job), riding a wolf (357) and
    /// the falcon (EFST_FALCON, 28). rAthena's `clif_parse_LoadEndAck` sends the player nothing else of its own
    /// effect state then (its ZC_STATE_CHANGE goes out only with a health state), so without this the
    /// character's own falcon was there only after a change of its options, never after logging in. (Where
    /// the Windows client takes its own `StateFlag04` from at that moment was not found in the decompiled
    /// code: this is decided here, as the mounts were.)
    fn rides(&mut self, change: &Change) {
        // The bits of the state the client's table of mounts goes by: 5 a bird, 19 a dragon, 21 a wolf; and
        // the falcon's (`crate::falcon::BIT`).
        let mask = match change.state {
            27 => 1 << 5 | 1 << 19,
            28 => crate::falcon::BIT,
            357 => 1 << 21,
            _ => return,
        };
        let own = Some(change.who) == self.account.as_ref().map(|account| account.account_id);
        world::stated((!own).then_some(change.who), if change.on { mask } else { 0 }, Some(mask));
        if own {
            // The character's own state keeps the one bit its job rides by: a dragon if the client's table has
            // one for the job, else a bird.
            let job = self.own_figure().job;
            let rides = if change.state != 27 || crate::mounts::drawn_job(job, 1 << 19) == job { mask & !(1 << 19) } else { 1 << 19 };
            self.hero.state = self.hero.state & !mask | if change.on { rides } else { 0 };
            self.render();
        }
    }

    /// One of the character's own states went on or off (`ApplyStatusChange`). The wait after a skill is no
    /// state to show: it holds the skills back (`cooldowns`). Any other is listed, the newest last, with the
    /// time it runs out when the server gave one.
    fn state_changed(&mut self, change: &Change) {
        let now = now();
        if change.on && change.state == statuses::POSTDELAY {
            self.cooldowns.waits.begin(
                crate::cooldowns::Wait { skill: crate::cooldowns::ANY, end: now.saturating_add(change.remaining), duration: change.total }
            );
            self.redraw(&[&wnd::HOTKEYS]);
        }
        let end = change.timed().then(|| now.saturating_add(change.remaining));
        self.state_set(u32::from(change.shown()), change.on, end, change.total);
        self.statuses_changed();
    }

    /// Puts a state on the list, in place of itself if it was there, or takes it off.
    fn state_set(&mut self, state: u32, on: bool, end: Option<u32>, total: u32) {
        self.statuses.on.retain(|active| active.state != state);
        if on {
            self.statuses.on.push(Active { state, end, total });
        }
    }

    /// ZC_STATE_CHANGE told the character's own body and health states: the client lists a state for each
    /// (`ApplyStatusIcons`): those of the health state's bits go on and off, the one of the body state before
    /// goes off and the new one's on, none with a time.
    pub(super) fn body_stated(&mut self, packet: &[u8]) {
        let Some((_, body, health)) = statuses::body_and_health(packet) else { return };
        for (state, on) in crate::statuses::of_health(health) {
            self.state_set(state, on, None, 0);
        }
        if let Some(before) = crate::statuses::of_body(self.statuses.body) {
            self.state_set(before, false, None, 0);
        }
        if let Some(state) = crate::statuses::of_body(body) {
            self.state_set(state, true, None, 0);
        }
        self.statuses.body = body;
        self.statuses_changed();
    }

    /// The list of states is drawn anew; while one of them has a time, it is again every second, for the time
    /// written beside the one pointed at (the client writes it every frame).
    fn statuses_changed(&mut self) {
        self.redraw(&[&wnd::STATUSES]);
        if !self.statuses.ticking && self.statuses.on.iter().any(|active| active.end.is_some()) {
            self.statuses.ticking = true;
            after(1000, Action::StatusTick);
        }
    }

    /// A second has passed: the times left are written anew, and again in a second while any is left. A state
    /// whose time is over stays until the server takes it off, as in the client.
    pub(in crate::web::app) fn status_tick(&mut self) {
        let now = now();
        self.statuses.ticking = false;
        if matches!(self.screen, Screen::Game) && self.statuses.on.iter().any(|active| active.end.is_some_and(|end| end > now)) {
            self.statuses_changed();
        }
    }
}
