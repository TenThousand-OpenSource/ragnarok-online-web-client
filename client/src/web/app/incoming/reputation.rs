//! What the server tells of the reputation (`crate::net::reputation`).

use super::*;
use crate::net::reputation;
use crate::reputation::Filter;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn reputation_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Another character of the account: the points of the one before are forgotten.
            packets::ZC_ACCEPT_ENTER => {
                self.reputation = Reputation::default();
                return false;
            }
            // `CGameMode::ReceiveReputeInfo`: each reputation told has those points now, and the window, which
            // said that it was loading until the points were there, lists them.
            reputation::ZC_REPUTE_INFO => {
                if let Some((success, told)) = reputation::info(packet) {
                    for (kind, points) in told {
                        match self.reputation.points.iter_mut().find(|(known, _)| *known == kind) {
                            Some(known) => known.1 = points,
                            None => self.reputation.points.push((kind, points)),
                        }
                    }
                    self.reputation.told = success;
                    self.redraw(&[&wnd::REPUTATION]);
                }
            }
            // `CReputeMgr::OpenRepute`: a script opens the window on a group, one reputation marked.
            reputation::ZC_REPUTE_OPEN => {
                if let Some((group, marked)) = reputation::opened(packet) {
                    let tables = data::reputation();
                    let place = tables.1.iter().position(|known| known.id == group);
                    if group == 0 || place.is_some() {
                        (self.reputation.group, self.reputation.filter) = (place, place.map_or(Filter::All, Filter::Group));
                        (self.reputation.page, self.reputation.marked) = (1, Vec::new());
                    }
                    if tables.0.iter().any(|repute| repute.id == marked) {
                        self.reputation.marked.push(marked);
                    }
                    (self.reputation.open, self.reputation.choosing) = (true, false);
                    self.render();
                }
            }
            _ => return false,
        }
        true
    }
}
