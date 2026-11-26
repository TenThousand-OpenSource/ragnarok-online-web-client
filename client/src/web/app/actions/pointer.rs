//! What the pointer did on the map, as the world reports it (`world::Hooks`): someone pressed, a cell to walk to.

use super::*;

impl App {
    /// Whether the map takes no press of the mouse now: while an NPC is talked to or its shop is open, and
    /// while the storage is open. The client does nothing with the mouse on the world then (`CGameMode`'s
    /// frame, rodecomp Source/Client/Mode/GameMode.cpp:9018: `isOnQuest == 0 … && itemStoreWnd == nullptr`;
    /// `isOnQuest` is set by every packet of an NPC's talk and of its shop and cleared when they close).
    fn world_shut(&self) -> bool {
        self.dialog.is_some() || self.shops.shop.is_some() || self.storage.open.is_some()
    }

    /// The player pressed someone on the map.
    pub(in crate::web::app) fn unit_pressed(&mut self, id: u32, monster: bool) {
        if self.world_shut() {
            return;
        }
        // A monster is fought until it or the character is down; anyone else is talked to.
        self.send_on_map(&if monster { net::actors::attack(id) } else { net::talk::contact(id) });
    }

    /// The player pressed a cell of the map to walk to.
    pub(in crate::web::app) fn walk_asked(&mut self, x: u16, y: u16) {
        self.close_menus();
        if self.world_shut() {
            return;
        }
        // (While the character's own blow is in motion the world asks for no walk: `world/pursuit.rs`,
        // `world/input/press.rs`.)
        self.send_on_map(&packets::walk_to(x, y));
        // (`UIWindowMgr::UpdateNavigationList`: the navigation's window may fold as the ground is pressed.)
        self.navigation_walk_asked();
    }
}
