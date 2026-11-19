//! The World Map and the large window of the map one is on.

use super::*;

impl App {
    /// A press of this system (`crate::press::worldmap`).
    pub(super) fn worldmap_pressed(&mut self, press: crate::press::worldmap::Press) {
        use crate::press::worldmap::Press;
        match press {
            // The menu's icon, the minimap's last button and Ctrl+~ (`ToggleWindow(WID_ROMAPWND)`): it opens
            // on the world one is in.
            Press::Window => {
                // With no world to show (a gateway that does not give the table yet) it stays closed.
                self.worldmap.open = !self.worldmap.open && !data::worlds().is_empty();
                (self.worldmap.world, self.worldmap.choosing) = (None, false);
                self.render();
            }
            Press::Worlds => {
                self.worldmap.choosing = !self.worldmap.choosing;
                self.redraw(&[&wnd::WORLD_MAP]);
            }
            Press::World(world) => {
                (self.worldmap.world, self.worldmap.choosing) = (Some(world), false);
                self.redraw(&[&wnd::WORLD_MAP]);
            }
            Press::Labels => {
                self.worldmap.labels = !self.worldmap.labels;
                self.redraw(&[&wnd::WORLD_MAP]);
            }
            // The minimap's "mini" button (`WID_MINIMAPWND`).
            Press::Large => {
                self.worldmap.large = !self.worldmap.large;
                self.render();
            }
        }
    }

    /// Tab, while the World Map is open, ticks and empties its box "Monster(Tab)". Returns whether the key
    /// was the map's.
    pub(in crate::web::app) fn worldmap_key(&mut self, key: &str, with: u16) -> bool {
        if !self.worldmap.open || key != "Tab" || with == crate::hotkeys::CTRL {
            return false;
        }
        self.pressed(crate::press::worldmap::Press::Labels);
        true
    }
}
