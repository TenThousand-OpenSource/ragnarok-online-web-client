//! What the pointer did on the map, as the world reports it (`world::Hooks`): someone pressed, a cell to walk to.

use super::*;

impl App {
    /// The player pressed someone on the map.
    pub(in crate::web::app) fn unit_pressed(&mut self, id: u32, monster: bool) {
        // A monster is fought until it or the character is down; anyone else is talked to.
        self.send_on_map(&if monster { units::attack(id) } else { npc::contact(id) });
    }

    /// The player pressed a cell of the map to walk to.
    pub(in crate::web::app) fn walk_asked(&mut self, x: u16, y: u16) {
        self.close_menus();
        // No walk is asked for while the character's own blow is in motion, as in the original client; a
        // button held on the ground asks again once it is over.
        if world::striking() {
            return;
        }
        self.send_on_map(&packets::walk_to(x, y));
    }
}
