//! A talk with one player in a window of its own: opening one, what is typed into it, its cross and its menu
//! (the client's `UIM_MAKE_WHISPER_WINDOW` of `UIWindowMgr`, `UIWindowMgr::ShowWhisperFrom` and
//! `UIWhisperWnd::SendMsg`).

use super::*;
use crate::whispers::{self, Typed};

/// How many lines a talk keeps: as many as the chat does. (How many the client's own list keeps was not read.)
const KEPT: usize = crate::web::hud::CHAT_KEPT;

impl App {
    /// A press of this system (`crate::press::whispers`).
    pub(super) fn whispers_pressed(&mut self, press: crate::press::whispers::Press) {
        use crate::press::whispers::Press;
        match press {
            Press::Close(place) => self.whisper_closed(place),
            // The menu's row: the other of the talk is asked to be a friend.
            Press::Friend => {
                let with = self.whispers.menu.take().and_then(|(_, place)| self.whispers.talks.get(place)?.as_ref().map(|talk| talk.with.clone()));
                self.redraw(&[&wnd::WHISPER_MENU]);
                if let Some(name) = with {
                    self.friend_wanted(&name);
                }
            }
        }
    }

    /// What the browser keeps of the Friend Setup's boxes is read: each is on unless it was switched off.
    pub(in crate::web::app) fn load_whispers(&mut self) {
        let on = |name: &str| stored(name).as_deref() != Some("off");
        (self.whispers.strangers, self.whispers.friends, self.whispers.sound) = (on("roweb.whisper.strangers"), on("roweb.whisper.friends"), on("roweb.whisper.sound"));
    }

    /// A box of the Friend Setup window was pressed (`UIFriendOptionWnd::SendMsg`, `ID_CHECK`): 0 whether a
    /// whisper of one who is no friend opens a window, 1 whether a friend's does, 2 whether a window opened
    /// so chimes. Kept in the browser.
    pub(in crate::web::app) fn whisper_option_switched(&mut self, which: u8) {
        let (option, name) = match which {
            0 => (&mut self.whispers.strangers, "roweb.whisper.strangers"),
            1 => (&mut self.whispers.friends, "roweb.whisper.friends"),
            2 => (&mut self.whispers.sound, "roweb.whisper.sound"),
            _ => return,
        };
        *option = !*option;
        store(name, (!*option).then_some("off"));
    }

    /// What the other of a talk is to the player, for the window's title: a friend (in the words of the
    /// client's message table) or one of the guild (`GuildMember`, as the client writes it).
    fn whisper_kin(&self, name: &str) -> Option<String> {
        if self.friends.has(name) {
            Some(Self::words(102, "Friend"))
        } else {
            self.guild.members.iter().any(|member| member.name == name).then(|| "GuildMember".to_owned())
        }
    }

    /// Makes the window of a talk with that player, unless there is one (`UIM_MAKE_WHISPER_WINDOW`): `true`
    /// for a window made. With every place of the list of windows taken none is made.
    fn whisper_window_made(&mut self, name: &str) -> bool {
        if self.whispers.with(name).is_some() {
            return false;
        }
        let Some(place) = self.whispers.talks.iter().position(Option::is_none) else { return false };
        let open = self.whispers.talks.iter().flatten().count();
        let scale = crate::web::stage_scale();
        let view = crate::web::page_size();
        let at = whispers::place(open, ((view.0 / scale) as i32, (view.1 / scale) as i32));
        self.whispers.talks[place] = Some(Talk { with: name.to_owned(), title: whispers::title(name, self.whisper_kin(name).as_deref()), lines: Vec::new(), at });
        self.open_at(wnd::WHISPERS[place], &self.whisper_style(place));
        true
    }

    /// A window for a talk with that player is asked for (the menus' "Open 1:1 Chat", a double click on a
    /// friend): it is made if there is none, and the keyboard goes to its line (`UIWhisperWnd::OnCreate`).
    /// With no place left for a window the name goes into the chat's box of whom to whisper to instead.
    pub(in crate::web::app) fn whisper_to(&mut self, name: &str) {
        self.whisper_window_made(name);
        let line = match self.whispers.with(name) {
            Some(place) => format!("whisper-input-{place}"),
            None => {
                set_field("chat-to", name);
                "chat-input".to_owned()
            }
        };
        if let Some(line) = element(&line).and_then(|line| line.dyn_into::<web_sys::HtmlElement>().ok()) {
            let _ = line.focus();
        }
    }

    /// A line of a talk is to be shown (`UIWindowMgr::ShowWhisperFrom`): `true` when a window took it. One
    /// opens for it by itself if the player has it so for friends or for others, with the chime if that is
    /// on (the client has a sound of its own for it, `Msg_NewMsg.wav`, which its files do not hold). Nothing
    /// is shown of a talk with the player's own character. The window that took the line comes to the front.
    pub(in crate::web::app) fn whisper_shown(&mut self, name: &str, line: String, colour: &'static str) -> bool {
        let friend = self.friends.has(name);
        if (if friend { self.whispers.friends } else { self.whispers.strangers }) && self.whisper_window_made(name) && self.whispers.sound {
            self.chime();
        }
        if self.game.as_ref().is_some_and(|game| game.name == name) {
            return false;
        }
        let Some(place) = self.whispers.with(name) else { return false };
        if let Some(talk) = &mut self.whispers.talks[place] {
            talk.lines.push((line, colour));
            if talk.lines.len() > KEPT {
                talk.lines.remove(0);
            }
        }
        self.redraw(&[wnd::WHISPERS[place]]);
        self.window_pressed(wnd::WHISPERS[place].id);
        true
    }

    /// Words go to one player alone (the client's `MM_WHISPERMSG`): not to the player's own character. Whom
    /// and what is kept for the server's answer, which shows the line (`incoming/whispers.rs`).
    pub(in crate::web::app) fn whisper_send(&mut self, to: &str, words: &str) {
        if self.game.as_ref().is_some_and(|game| game.name == to) {
            return;
        }
        self.whispers.last = (to.to_owned(), words.to_owned());
        self.send_on_map(&npc::whisper(to, words));
    }

    /// A line was typed into a talk's window and sent (`UIWhisperWnd::SendMsg`, `ID_OK`): a command is done
    /// as the chat's line does it, and `/q` closes the window; anything else is said to the other.
    pub(in crate::web::app) fn whisper_typed(&mut self, place: usize) {
        let line = format!("whisper-input-{place}");
        let words = field(&line);
        set_field(&line, "");
        match whispers::typed(&words) {
            Typed::Nothing => {}
            Typed::Close => self.whisper_closed(place),
            Typed::Command(command) => {
                self.typed_command(command);
            }
            Typed::Said(words) => {
                if let Some(with) = self.whispers.talks.get(place).and_then(Option::as_ref).map(|talk| talk.with.clone()) {
                    self.whisper_send(&with, words);
                }
            }
        }
    }

    /// A talk's window is put away (`ID_CLOSE`), and its menu with it.
    fn whisper_closed(&mut self, place: usize) {
        if let Some(talk) = self.whispers.talks.get_mut(place) {
            *talk = None;
        }
        self.whispers.menu = None;
        self.redraw(&[wnd::WHISPERS[place], &wnd::WHISPER_MENU]);
    }

    /// The right button on a talk's window: its menu opens there (`UIWhisperWnd::OnRBtnUp`), if it has a row
    /// to show.
    pub(in crate::web::app) fn whisper_menu_opens(&mut self, place: usize, at: (f32, f32)) {
        let Some(talk) = self.whispers.talks.get(place).and_then(Option::as_ref) else { return };
        if self.friends.has(&talk.with) {
            return;
        }
        let at = Self::on_stage(at);
        self.whispers.menu = Some((at, place));
        self.open_at(&wnd::WHISPER_MENU, &wnd::menu_style(at));
    }

    /// Puts the menu of a talk's window away, if it is open.
    pub(in crate::web::app) fn close_whisper_menu(&mut self) {
        if self.whispers.menu.take().is_some() {
            self.redraw(&[&wnd::WHISPER_MENU]);
        }
    }
}
