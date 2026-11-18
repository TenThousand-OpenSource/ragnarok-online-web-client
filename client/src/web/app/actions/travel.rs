//! The character moving: over the cells of a map, and from map to map.

use super::*;

impl App {
    /// The character stands on another cell: the place under the minimap and the minimap itself follow.
    pub(super) fn moved(&mut self, x: u16, y: u16) {
        if let Some(game) = &mut self.game {
            (game.x, game.y) = (x, y);
            if let Some(position) = element("position") {
                position.set_text_content(Some(&format!(" {x}  {y}")));
            }
            if let Some((pictured, marked)) = crate::web::view::minimap_place(game, self.minimap_zoom) {
                for (id, style) in [("minimap", pictured), ("marker", marked)] {
                    if let Some(shown) = element(id) {
                        let _ = shown.set_attribute("style", &style);
                    }
                }
            }
            // Brought nearer, the minimap moves under its marks as the character walks.
            if self.minimap_zoom > 1.0 {
                if let Some(marks) = element("map-marks") {
                    marks.set_inner_html(&crate::web::view::minimap_marks(game, self.minimap_zoom, self.minimap_marks_off));
                }
            }
        }
    }

    /// The map is drawn, or could not be.
    pub(super) fn world_ready(&mut self, outcome: Result<(), String>) {
        if !matches!(self.screen, Screen::Game) {
            return;
        }
        // The world goes by some of the player's options from its first frame.
        self.apply_options();
        if outcome.is_ok() {
            self.map_sign();
        }
        if let Err(error) = outcome {
            web_sys::console::error_1(&format!("world: {error}").into());
            self.message = Some(error);
        }
        if let Some(game) = &mut self.game {
            game.map_size = world::map_size();
        }
        self.wait = None;
        self.render();
        // The loading screen goes, and the game comes out of the dark.
        loading(None);
        fade(false);
    }

    /// The server put the character somewhere, on this map or on another.
    pub(super) fn arrived(&mut self, map: String, x: u16, y: u16) {
        if !matches!(self.screen, Screen::Game) {
            return fade(false);
        }
        // Right after the character comes in the server may name its place once more, while the map is
        // still being read: the reading goes on, and the character is put there when the map shows.
        let reading = world::reading(&map);
        let same = self.game.as_ref().is_some_and(|game| game.map == map) && (world::map_size().is_some() || reading);
        if let Some(game) = &mut self.game {
            (game.x, game.y) = (x, y);
            if !same {
                (game.map, game.map_size) = (map, None);
            }
        }
        // A dead character arrives alive.
        self.hero.dead = false;
        self.esc = 0;
        self.dialog = None;
        self.refresh_values();
        self.refresh_dialog();
        if same {
            // The map is already drawn: the character is only put elsewhere on it.
            world::jump((x, y));
            // (While the map is read, its loading screen stays; the dark goes when it shows.)
            if !reading {
                fade(false);
            }
        } else {
            self.enter_world();
        }
        self.send(&packets::map_loaded());
    }
}
