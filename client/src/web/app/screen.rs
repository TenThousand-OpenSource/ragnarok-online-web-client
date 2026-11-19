//! Putting the app on the page: which screen shows, and the characters painted on it.

use super::*;

impl App {
    pub(super) fn show(&mut self, screen: Screen) {
        self.screen = screen;
        self.wait = None;
        self.ask = None;
        self.tune();
        self.render();
    }

    /// The title music plays in every window before the game, if sound is on and the page is the one looked at:
    /// a tab left in the background falls silent.
    pub(super) fn tune(&self) {
        let hidden = crate::web::document().is_some_and(|document| document.hidden());
        sound::music(self.sound && !hidden && !matches!(self.screen, Screen::Game));
        // In the game the map has its own music, and everything its sound.
        sound::game(self.sound && !hidden && matches!(self.screen, Screen::Game));
    }

    pub(in super::super) fn render(&self) {
        let Ok(ui) = ui() else { return };
        let view = crate::web::page_size();
        // A window redrawn (a message shown, the view resized) keeps what was typed into it.
        let typed = KEPT_FIELDS.map(|id| (id, field(id)));
        // The windows are laid out for the original client's screen, 1024 by 768, and enlarged together on a
        // larger view, so they keep the share of the screen they have there. A smaller view shrinks each window
        // to fit instead (`view::window`).
        let scale = (view.0 / 1024.0).min(view.1 / 768.0).max(1.0);
        let stage = (view.0 / scale, view.1 / scale);
        // Too narrow a view is covered by a note saying why. The windows stay underneath as they are, so a wider
        // window brings everything back, what was typed included.
        // In the game the canvas shows through, and takes the mouse where no window is.
        let _ = ui.set_attribute("class", if matches!(self.screen, Screen::Game) { "ingame" } else { "" });
        let narrow = if view.0 < MIN_WIDTH {
            format!(r#"<div id="narrow"><p>{}</p></div>"#, fill(Key::TooNarrow.text(self.lang), &[&MIN_WIDTH.to_string()]))
        } else {
            String::new()
        };
        ui.set_inner_html(&format!(
            r#"<div class="stage" style="width:{}px;height:{}px;transform:scale({scale})">{}</div>{narrow}"#,
            stage.0,
            stage.1,
            self.html(stage)
        ));
        for (id, value) in typed {
            if !value.is_empty() {
                set_field(id, &value);
            }
        }
        // `autofocus` only works on page load; here the topmost window's marked element is focused.
        if let Ok(marked) = ui.query_selector_all("[autofocus]") {
            if let Some(last) = marked.length().checked_sub(1).and_then(|index| marked.item(index)) {
                let _ = last.unchecked_into::<web_sys::HtmlElement>().focus();
            }
        }
        self.paint_figures();
        place_moved();
        self.order_windows();
    }

    pub(in super::super) fn paint_figures(&self) {
        // The other player in the window that shows their equipment (`hud/viewequip.rs`).
        if let (Screen::Game, Some(figure)) = (&self.screen, self.seen_figure()) {
            figure::paint("pc-viewequip", figure);
        }
        match self.screen {
            Screen::CharSelect => {
                for character in &self.characters {
                    let figure = Figure {
                        job: character.job,
                        sex: character.sex,
                        hair_style: character.hair_style,
                        hair_colour: u16::from(character.hair_color),
                        headgear: character.headgear,
                        body_palette: character.body_palette,
                        weapon: 0,
                        shield: 0,
                        action: if self.deletions.contains_key(&character.char_id) { SITTING } else { 0 },
                        frame: 0,
                    };
                    figure::paint(&format!("pc{}", character.slot), figure);
                }
            }
            // The character in the Equipment window.
            Screen::Game if self.bag.equipment_open || self.skills.open => {
                if self.bag.equipment_open {
                    figure::paint("pc-equip", self.shown_figure());
                }
                // The character in the skill window: its body and head alone, sitting and turned as the
                // client's action 23 is (`UINewSkillListWnd::DrawCharacter`: neither what it wears on its
                // head nor what it holds, nor what it rides). The client has it look about, a frame of the
                // three every 900 milliseconds; here it shows the first.
                if self.skills.open {
                    figure::paint("pc-skill", Figure { headgear: [0; 3], weapon: 0, shield: 0, action: SITTING + 7, frame: 0, ..self.own_figure() });
                }
            }
            Screen::CharCreate(_) => {
                let make = &self.make;
                let job = if make.doram { data::SUMMONER } else { 0 };
                let figure = Figure { job, sex: make.sex, hair_style: make.hair_style, hair_colour: make.hair_colour, headgear: [0; 3], body_palette: 0, weapon: 0, shield: 0, action: usize::from(make.direction), frame: 0 };
                figure::paint("pc-make", figure);
            }
            _ => {}
        }
    }
}
