//! The client's own commands: typed into the chat after a slash, or run from the Game Settings window.

use super::*;
use crate::net::commands;

mod beside;

impl App {
    /// Does one of the client's commands; `false` for words that are none of them (they are said, then). The
    /// client tells them by the first word, whatever the size of its letters (`CSession::GetTalkType`), from
    /// its table of commands (`crate::commands`).
    pub(super) fn command(&mut self, words: &str) -> bool {
        let Some((first, rest)) = crate::commands::split(words) else { return false };
        if self.socket.is_none() {
            return false;
        }
        let command = first.to_lowercase();
        // With a name after it, `/ex` blocks that character's whispers and `/in` lets them in again
        // (`GetTalkType`: the name is all the rest of the line).
        if matches!(command.as_str(), "/ex" | "/in") && !rest.is_empty() {
            self.whisper_from(rest, command == "/in");
            return true;
        }
        // `/packets` is a command of this project's own, for whoever builds it: the list of the packets this
        // client does not read yet (`clicks/unread.rs`). The client has no command of that name.
        if command == "/packets" {
            self.unread_switched();
            return true;
        }
        // `/farwalk` is this project's own too: it switches walking in stretches to a place pressed farther
        // than the server walks in one go (`world::Options::far_walk`), which is on at first. Off, such a press
        // is asked once and does nothing, as in the client.
        if command == "/farwalk" {
            self.switch_option(super::settings::CLIENT_WALK);
            let now = if self.option(super::settings::CLIENT_WALK) { "far walk is off." } else { "far walk is on." };
            self.chat_line(crate::i18n::tr(now).to_owned(), YELLOW);
            return true;
        }
        // `/hi` with words after it greets the friends with them (`TT_EX_HI`).
        if command == "/hi" && !rest.is_empty() {
            self.greet_friends(rest);
            return true;
        }
        // An emotion by its name (`/ho`, `/delight`, `/e1`): the client's list of them.
        if let Some(emotion) = data::emotion_named(first).or_else(|| data::emotion_named(&command)).and_then(|emotion| u8::try_from(emotion).ok()) {
            self.send(&emotions::show(emotion));
            return true;
        }
        let Some(asks) = crate::commands::asks(&command) else { return false };
        // `/navi`, `/navi2` and `/naviopen` take what stands after them (`clicks/navigation.rs`).
        match asks {
            "TT_NAVIGATION" | "TT_NAVIGATION2" => {
                self.navigation_command(&rest.split_whitespace().collect::<Vec<_>>(), asks == "TT_NAVIGATION2");
                return true;
            }
            "TT_NAVIGATION3" => {
                self.navigate(Wanted::Open);
                return true;
            }
            _ => {}
        }
        if !self.asked(asks) {
            // A command of the client's that this client does nothing for yet is not said aloud. The words are
            // this client's own.
            self.chat_line(format!("{first}: {}", crate::i18n::tr("this command does nothing in this client yet.")), YELLOW);
        }
        true
    }

    /// Does what a command asks for, by the client's name for it (`CGameMode::ExecuteTalkType`, and the
    /// functions `CGameSettingsMgr` runs for the options that are commands); `false` for what is not done here.
    fn asked(&mut self, asks: &str) -> bool {
        match asks {
            // `/sit` and `/stand` both sit the character down or stand it up, whichever it is not doing
            // (`RequestPlayerAction31`); one who walks does not sit down (as with the key).
            "TT_EX_SIT" => {
                if self.hero.sitting || !world::walking() {
                    self.send(&net::actors::sit(!self.hero.sitting));
                }
            }
            "TT_EX_MAP_POS" => self.tell_where(),
            // `/chat` opens the window that makes a chat room, `/q` leaves the room sat in (`clicks/chat_rooms.rs`).
            "TT_EX_MAKE_CHAT_ROOM" => self.room_make_asked(),
            "TT_EX_EXIT_CHAT_ROOM" => self.room_left(),
            // `/hw`: the homunculus comes and stands by, or goes about its own again (`MM_HOMUN_WAIT`).
            "TT_HOMUN_WAIT" => self.companion_stands_by(crate::ai::Kind::Homunculus),
            "TT_EX_REMEMBER_WARPPOINT" => self.send(&commands::remember_warp()),
            "TT_EX_DORIDORI" => self.send(&commands::doridori()),
            // The character turns an eighth of the way round, clockwise or the other way.
            "TT_EX_BANGBANG" | "TT_EX_BINGBING" => {
                if let Some(direction) = world::turn(asks == "TT_EX_BANGBANG") {
                    self.send(&net::actors::face(direction));
                }
            }
            // `/who`, `/w`: how many play (`MM_REQ_NUM_USER`); the answer is told in the chat
            // (`incoming/commands.rs`).
            "TT_REQ_NUM_USER" => {
                self.commands.who_asked = true;
                self.send(&commands::user_count());
            }
            // `/help`, `/h`: the client's list of commands, line by line (`ShowHelpMessages`, in `0xFFFF64`).
            "TT_EX_HELP" => {
                for line in data::command_help() {
                    self.chat_line(line, PALE_CYAN);
                }
            }
            // `/ex` alone asks whose whispers are blocked (`SendWhisperBlockPacket`); `/inall` lets everyone's
            // in and `/exall` blocks everyone's (`EnableWhisperBlock`, `DisableWhisperBlock`).
            "TT_EX_WHISPER_BLOCK_LIST" => self.send(&commands::whisper_list()),
            // Letting everyone's in forgets who was turned away (`EnableWhisperBlock`).
            "TT_EX_WHISPER_ALL_ACCEPT" => {
                self.commands.blocked.clear();
                self.send(&commands::whisper_all(true));
            }
            // `/eqopen`: others may look at what the player wears, or may not (`clicks/viewequip.rs`).
            "TT_REQ_OPEN_EQUIPEDITEM_ONOFF" => self.equipment_open_switched(),
            "TT_EX_WHISPER_ALL_BLOCK" => self.send(&commands::whisper_all(false)),
            // `/bgm` and `/sound` switch the music and the effects' sounds, as the Game Settings' own switches
            // do, and say so (`TT_MUSIC_ON_OFF`, `TT_EFFECT_SOUND_ON_OFF`: `MSI_BGM_ON` and the like).
            "TT_MUSIC_ON_OFF" | "TT_EFFECT_SOUND_ON_OFF" => {
                let (off, said) = if asks == "TT_MUSIC_ON_OFF" { (&mut self.settings.music_off, ["MSI_BGM_ON", "MSI_BGM_OFF"]) } else { (
                    &mut self.settings.effects_off,
                    ["MSI_SOUND_ON", "MSI_SOUND_OFF"]
                ) };
                *off = !*off;
                let said = crate::commands::message(said[usize::from(*off)]).and_then(data::message);
                self.sound_levels();
                self.switched(said);
            }
            // Any other command that is an option of the Game Settings switches it, as its row of the window
            // does, and the option's own message says how it stands now (`GameSettings::ToggleOption`).
            // (`/aura` while auras do not show says that alone: `aura_hidden_said`.)
            option if data::option_switched(option) => {
                self.switch_option(option);
                let said = if self.aura_hidden_said(option) { None } else { data::option_says(option, self.option(option)) };
                self.switched(said);
            }
            _ => return self.asked_beside(asks),
        }
        true
    }

    /// Something was switched by a command: the chat says how it stands now, in the client's colour for it
    /// (`NotifyServiceOption`: `0x64FFFF`), and the Game Settings window shows it.
    pub(super) fn switched(&mut self, said: Option<String>) {
        if let Some(said) = said {
            self.chat_line(said, PALE_YELLOW);
        }
        self.redraw(&[&wnd::SETTINGS]);
    }

    /// Where the character is, as the client writes it: the map's full name, its own, and the cell
    /// (`ShowPlayerLocation`).
    fn tell_where(&mut self) {
        let Some(game) = &self.game else { return };
        let map = game.map.trim_end_matches(".gat");
        let line = match data::map_sign(map) {
            Some(sign) => format!("{}({map}) : {}, {}", sign.name, game.x, game.y),
            None => format!("{map} : {}, {}", game.x, game.y),
        };
        self.chat_line(line, PALE_YELLOW);
    }

    /// Asks the server to block the whispers of a character, or to let them in again
    /// (`TT_REQ_WHISPER_PC_EX`, `TT_REQ_WHISPER_PC_IN`): a name longer than 24 is not asked for. The name is
    /// kept for the answer, which tells of it; and who is turned away is kept as the client keeps it, at the
    /// asking (`g_whisperBlockMgr`: the menu on another player goes by it). The client keeps the names with
    /// the account between visits; here they are this visit's.
    pub(super) fn whisper_from(&mut self, name: &str, allow: bool) {
        if name.len() > 24 {
            return;
        }
        if allow {
            self.commands.blocked.remove(name);
        } else {
            self.commands.blocked.insert(name.to_owned());
        }
        self.commands.whisper_name = name.to_owned();
        self.send(&commands::whisper_from(name, allow));
    }

    /// The command behind one of the "run" rows of the Game Settings window, by the client's name for the row.
    pub(in crate::web) fn command_of(option: &str) -> Option<&'static str> {
        Some(match option {
            "TT_EX_MAP_POS" => "/where",
            "TT_EX_REMEMBER_WARPPOINT" => "/memo",
            "TT_EX_DORIDORI" => "/doridori",
            "TT_EX_BANGBANG" => "/bangbang",
            "TT_EX_BINGBING" => "/bingbing",
            "TT_EX_HI" => "/hi",
            "TT_EX_HELP" => "/h",
            "TT_EX_LEAVE_GROUP" => "/leave",
            "TT_EX_SAVE_CHAT" => "/savechat",
            "TT_EX_WHISPER_BLOCK_LIST" => "/ex",
            "TT_EX_WHISPER_ALL_ACCEPT" => "/inall",
            "TT_EX_WHISPER_ALL_BLOCK" => "/exall",
            "TT_EX_RANK_BLACKSMITH" => "/blacksmith",
            "TT_EX_RANK_ALCHEMIST" => "/alchemist",
            "TT_EX_RANK_TAEKWON" => "/taekwon",
            // The chat rooms and the costumes are not here yet: their commands answer that they do nothing,
            // until what they ask for is done (`asked`).
            "TT_EX_EXIT_CHAT_ROOM" => "/q",
            "TT_EX_MAKE_CHAT_ROOM" => "/chat",
            "TT_EX_NO_COS_PLAY" => "/nocosplay",
            _ => return None,
        })
    }
}
