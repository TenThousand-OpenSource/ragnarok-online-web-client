//! What the server tells of the party's instance (`crate::net::instances`, `crate::instances`).

use super::*;
use crate::instances::{Instance, State};
use crate::net::instances;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other. Entering the game is overheard
    /// and left to its own reader: the instance of the character before is forgotten (`CSession`'s reset).
    pub(super) fn instances_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            packets::ZC_ACCEPT_ENTER => {
                self.instances = Instances::default();
                return false;
            }
            // `ReceiveMemorialDungeonSubscriptionInfo`: the name and the place in the queue; it waits.
            instances::ZC_MEMORIALDUNGEON_SUBSCRIPTION_INFO => {
                let Some((name, queue)) = instances::subscribed(packet) else { return true };
                self.instances.known = Some(Instance { name, queue, limit: 0, idle: 0, state: State::Waiting });
                self.instance_shown();
            }
            // `ReceiveMemorialDungeonSubscriptionNotify`: another place in the queue; under zero the place
            // was given up, which the chat says.
            instances::ZC_MEMORIALDUNGEON_SUBSCRIPTION_NOTIFY => {
                let (Some(queue), Some(instance)) = (instances::queued(packet), self.instances.known.as_mut()) else { return true };
                if queue > -1 {
                    (instance.queue, instance.state) = (queue, State::Waiting);
                    self.instance_shown();
                } else {
                    self.instance_gone(Some(crate::instances::CANCELLED));
                }
            }
            // `ReceiveMemorialDungeonInfo`: the name and the two times.
            instances::ZC_MEMORIALDUNGEON_INFO => {
                let Some((name, limit, idle)) = instances::told(packet) else { return true };
                let before = self.instances.known.as_ref().map(|instance| instance.state);
                let queue = self.instances.known.as_ref().map_or(0, |instance| instance.queue);
                self.instances.known = Some(Instance { name, queue, limit, idle, state: crate::instances::state_of(idle) });
                self.instance_told(before);
            }
            // `ReceiveMemorialDungeonNotify`: the time to enter by changed, or the instance ended.
            instances::ZC_MEMORIALDUNGEON_NOTIFY => {
                let Some((kind, idle)) = instances::noticed(packet) else { return true };
                if kind != 0 {
                    self.instance_gone(crate::instances::ended_line(kind));
                } else if let Some(instance) = self.instances.known.as_mut() {
                    let before = Some(instance.state);
                    (instance.idle, instance.state) = (idle, crate::instances::state_of(idle));
                    self.instance_told(before);
                }
            }
            _ => return false,
        }
        true
    }

    /// The window is brought up (`UIWindowMgr::EnsureWindow`, `WID_MEMORIALDUNWND`) and drawn anew.
    fn instance_shown(&mut self) {
        (self.instances.shown, self.instances.wanted) = (true, true);
        self.redraw(&[&wnd::INSTANCE]);
    }

    /// The instance was told of again: the window comes back unless the player closed it while the instance
    /// ran (`crate::instances::shown_again`); one that shows is drawn anew either way.
    fn instance_told(&mut self, before: Option<State>) {
        let Some(now) = self.instances.known.as_ref().map(|instance| instance.state) else { return };
        if crate::instances::shown_again(before, now, self.instances.wanted) {
            self.instance_shown();
        } else {
            self.redraw(&[&wnd::INSTANCE]);
        }
    }

    /// The instance is no more (`UIWindowMgr::CloseWindow`, which forgets all of it), and the chat says why,
    /// when there is a line for it.
    fn instance_gone(&mut self, line: Option<usize>) {
        self.instance_forgotten();
        if let Some(line) = line {
            self.say_filled(said::GENERAL, line, &[], crate::instances::SAID);
        }
    }

    /// The window is closed and what it told of forgotten, with nothing said: as an instance ends, and as
    /// the player leaves the party it was made for (`CGameMode::ReceivePartyMemberLeft`).
    pub(super) fn instance_forgotten(&mut self) {
        self.instances = Instances::default();
        self.redraw(&[&wnd::INSTANCE]);
    }
}
