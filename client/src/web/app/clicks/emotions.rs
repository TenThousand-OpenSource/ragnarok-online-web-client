//! The window of emotions and the Shortcut List.

use super::*;

/// Where the browser keeps the ten lines of the Shortcut List, one to a line (the client keeps them in
/// `savedata\shortcutlist`).
const KEPT: &str = "roweb.shortcuts";
/// How long the client waits between two emotions (`MM_REQ_EMOTION`), in milliseconds.
const WAIT: f64 = 2500.0;

impl App {
    /// Reads the lines the browser keeps.
    pub(in crate::web::app) fn load_shortcuts(&mut self) {
        let kept = stored(KEPT).unwrap_or_default();
        for (line, kept) in self.emotions.lines.iter_mut().zip(kept.split('\n')) {
            *line = kept.to_owned();
        }
    }

    /// A press of this system (`crate::press::emotions`).
    pub(super) fn emotions_pressed(&mut self, press: crate::press::emotions::Press) {
        use crate::press::emotions::Press;
        match press {
            Press::Window | Press::View => {
                self.emotions.open = !self.emotions.open;
                if self.emotions.open {
                    self.read_emotions();
                }
                self.render();
            }
            // A press on an emotion writes its name where the player types (`CCashEmotionAction_Register`):
            // into the line of the Shortcut List that has the keyboard, else into the chat's line.
            Press::Pick(emotion) => {
                let Some(name) = data::emotion_name(emotion) else { return };
                let typed_in = crate::web::document().and_then(|document| document.active_element()).map(|active| active.id()).filter(
                    |id| self.emotions.shortcuts && id.starts_with(&wnd::SHORTCUTS.field())
                );
                let line = typed_in.unwrap_or_else(|| "chat-input".to_owned());
                set_field(&line, &name);
                if let Some(input) = element(&line).and_then(|input| input.dyn_into::<web_sys::HtmlElement>().ok()) {
                    let _ = input.focus();
                }
            }
            Press::Shortcuts => {
                if self.emotions.shortcuts {
                    self.shortcuts_kept();
                }
                self.emotions.shortcuts = !self.emotions.shortcuts;
                self.render();
            }
        }
    }

    /// The pictures of the emotions for the grid are read, once: of each the middle one of its pictures.
    fn read_emotions(&mut self) {
        if std::mem::replace(&mut self.emotions.asked, true) {
            return;
        }
        let wanted: Vec<(u16, String, usize)> = data::emotions()
            .into_iter()
            .filter_map(|emotion| data::emotion(emotion).map(|(sprite, action)| (emotion, sprite, action)))
            .collect();
        wasm_bindgen_futures::spawn_local(async move {
            let mut read = Vec::new();
            for (emotion, sprite, action) in wanted {
                if figure::frame_picture(&sprite, action, 0).await.is_err() {
                    continue;
                }
                let middle = figure::frames_of(&sprite, action).unwrap_or(1) / 2;
                if let Ok((picture, size, _)) = figure::frame_picture(&sprite, action, middle).await {
                    read.push((emotion, (picture, size)));
                }
            }
            dispatch(Action::EmotionsRead(read));
        });
    }

    /// The pictures were read: the grid shows them.
    pub(in crate::web::app) fn emotions_read(&mut self, pictures: Vec<(u16, (String, (usize, usize)))>) {
        self.emotions.pictures.extend(pictures);
        self.redraw(&[&wnd::EMOTIONS]);
    }

    /// A double click on an emotion shows it (`CCashEmotionAction_Play`), no sooner than 2500 milliseconds
    /// after the last.
    pub(in crate::web::app) fn emotion_shown(&mut self, emotion: u16) {
        let now = epoch();
        let Ok(emotion) = u8::try_from(emotion) else { return };
        if now - self.emotions.last < WAIT {
            return;
        }
        self.emotions.last = now;
        self.send(&crate::net::emotions::show(emotion));
    }

    /// What is typed into the Shortcut List is kept (`UIEmotionWnd::StoreInfo`): a line left empty stays what
    /// it was.
    pub(in crate::web::app) fn shortcuts_kept(&mut self) {
        for place in 0..10 {
            let typed = field(&Self::shortcut_field(place));
            let typed = typed.trim();
            if !typed.is_empty() && typed != self.shortcut_line(place) {
                self.emotions.lines[place] = typed.to_owned();
            }
        }
        store(KEPT, Some(&self.emotions.lines.join("\n")));
    }

    /// A key of the Shortcut List was pressed (the client's `UIWindowMgr::SendEmotionHotkey`): its line is
    /// done as if typed into the chat: one that begins with a slash as an emotion or a command, any other
    /// said aloud. A line with a link to an item in it is not said (the client's line 3667).
    pub(in crate::web::app) fn shortcut_pressed(&mut self, place: usize) {
        let line = self.shortcut_line(place);
        let line = line.trim();
        if line.contains("<ITEML>") {
            return self.chat_line(Self::words(3667, "It cannot be used because it contains an Item Tag."), RED);
        }
        if !line.is_empty() && !self.typed_command(line) {
            self.say_aloud(line);
        }
    }
}
