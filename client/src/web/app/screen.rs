//! Putting the app on the page: which screen shows, and the characters painted on it.

use super::*;

/// The form fields whose content survives a redraw of the same window.
const KEPT_FIELDS: [&str; 13] =
    ["user", "password", "reg-id", "reg-password", "reg-repeat", "reg-email", "reg-sex", "rec-id", "rec-email", "new-password", "new-repeat", "char-name", "birthdate"];
/// The action of a sitting character, facing south: how one waiting to be deleted is shown.
const SITTING: usize = 16;
/// The narrowest view the client is used in, for now: the width of the original client's smallest screen.
const MIN_WIDTH: f64 = 1024.0;

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
                let job = if make.doram { crate::jobs::SUMMONER } else { 0 };
                let figure = Figure { job, sex: make.sex, hair_style: make.hair_style, hair_colour: make.hair_colour, headgear: [0; 3], body_palette: 0, weapon: 0, shield: 0, action: usize::from(make.direction), frame: 0 };
                figure::paint("pc-make", figure);
            }
            _ => {}
        }
    }

    /// The client's chime (`버튼소리.wav`). The original plays it in a few places only: on logging in, on choosing
    /// a server, on entering the game with a character (and for a notice, a whisper window, someone joining a
    /// chat room). Its buttons, tabs and items make no sound.
    pub(super) fn chime(&self) {
        if self.sound {
            sound::button();
        }
    }

    /// The player's own character as a figure, standing and facing the screen, with the weapon it holds.
    pub(in crate::web) fn own_figure(&self) -> Figure {
        let character = self.zone.as_ref().and_then(|zone| self.characters.iter().find(|character| character.char_id == zone.char_id));
        Figure {
            job: character.map_or(0, |character| character.job),
            sex: character.map_or(1, |character| character.sex),
            hair_style: character.map_or(1, |character| character.hair_style),
            hair_colour: character.map_or(0, |character| u16::from(character.hair_color)),
            headgear: character.map_or([0; 3], |character| character.headgear),
            body_palette: character.map_or(0, |character| character.body_palette),
            weapon: self.bag.items.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id),
            // What is worn on the left hand alone: a weapon of two hands is not a shield.
            shield: self.bag.items.iter().find(|item| item.worn & 0x22 == 0x20).map_or(0, |item| item.id),
            action: 0,
            frame: 0,
        }
    }

    /// The same as the windows show it: on its mount, in its machine, by the state it is in.
    pub(in crate::web) fn shown_figure(&self) -> Figure {
        let figure = self.own_figure();
        Figure { job: crate::mounts::drawn_job(figure.job, self.hero.state), ..figure }
    }

    /// A moment after the music was asked for: whether the browser holds it back until the page is pressed.
    pub(super) fn music_checked(&mut self) {
        let hidden = crate::web::document().is_some_and(|document| document.hidden());
        let held = self.sound && !hidden && !matches!(self.screen, Screen::Game) && !sound::music_playing();
        if held != self.music_held {
            self.music_held = held;
            self.render();
        }
    }
}
