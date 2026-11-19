//! The client's commands beside those that switch a row of the Game Settings: the options that have a command
//! and no row, the commands that switch several at once, and those that do something of their own (`/hi`,
//! `/savechat`, the rankings, `/tip`, `/buildinfo`).

use super::*;

/// The options the client has a command for and no row of its Game Settings: the client's name for each, how
/// it starts, and the messages said when it goes on and off (`MSI_…`, `crate::commands::message`). That the
/// two first start on was not found in the client: it is how its players know them.
const BESIDE: [(&str, bool, [&str; 2]); 3] = [
    ("TT_SKILLFAIL_ON_OFF", true, ["MSI_SKILLFAIL_ON", "MSI_SKILLFAIL_OFF"]),
    ("TT_LOGINOUT_ON_OFF", true, ["MSI_LOGINOUT_ON", "MSI_LOGINOUT_OFF"]),
    ("TT_BATTLE_ON_OFF", false, ["MSI_BATTLE_ON", "MSI_BATTLE_OFF"]),
];

/// What `/set1`, `/set2` and `/set3` switch, each as the client lists it (`ExecuteTalkType`: its options 113,
/// 108, 109, 141; with 120 and 121; and 113, 108, 109, 141, 165, 135, 167).
const SET1: [&str; 4] = ["TT_SKILLFAIL_ON_OFF", "TT_BOLD_NAME_TYPE_ON_OFF", "TT_NOCTRL_ON_OFF", "TT_WINDOW_SNAP_ON_OFF"];
const QUICK: [&str; 2] = ["TT_QUICKSPELL_ON_OFF", "TT_QUICKSPELL2_ON_OFF"];
const SET3: [&str; 3] = ["TT_LOGINOUT_ON_OFF", "TT_BATTLE_ON_OFF", "TT_SHOPPING_ON_OFF"];

impl App {
    /// How an option with no row of the Game Settings starts, if it is one of those.
    pub(in crate::web::app) fn starts_beside(id: &str) -> Option<bool> {
        BESIDE.iter().find(|(name, ..)| *name == id).map(|(_, on, _)| *on)
    }

    /// Switches an option, as its command does, and says how it stands now: one of the Game Settings in the
    /// words its row names, one beside them in its own.
    fn switch_saying(&mut self, option: &str) {
        self.switch_option(option);
        let on = self.option(option);
        let said = match BESIDE.iter().find(|(name, ..)| *name == option) {
            Some((_, _, said)) => crate::commands::message(said[usize::from(!on)]).and_then(data::message),
            None => data::option_says(option, on),
        };
        self.switched(said);
    }

    /// Does what one of these commands asks for; `false` for what is not done here.
    pub(super) fn asked_beside(&mut self, asks: &str) -> bool {
        match asks {
            // `/skillfail`, `/loginout`, `/bm`. Battle mode keeps the chat's line closed until Enter, so that
            // the keys go to the hotkeys (`chatModeChanged`): this client's line is closed until Enter as it
            // is, so the switch changes nothing but what it says.
            option if Self::starts_beside(option).is_some() => self.switch_saying(option),
            // `/q3` switches both quick skills.
            "TT_QUICKSPELL3_ON_OFF" => QUICK.into_iter().for_each(|option| self.switch_saying(option)),
            "TT_SET1_ON_OFF" => SET1.into_iter().for_each(|option| self.switch_saying(option)),
            "TT_SET2_ON_OFF" => SET1.into_iter().chain(QUICK).for_each(|option| self.switch_saying(option)),
            "TT_SET3_ON_OFF" => SET1.into_iter().chain(SET3).for_each(|option| self.switch_saying(option)),
            // `/tip` opens the Tipbox, or closes it.
            "TT_OPEN_TIP" => self.pressed(crate::press::tips::Press::Window),
            // `/buildinfo`: the client's four lines, in yellow; the build is this client's own.
            "TT_SHOW_BUILD_INFO" => {
                for line in [String::new(), format!("ver : roweb {}", env!("CARGO_PKG_VERSION")), "s/n : ".to_owned(), "verification : XXXXXXX".to_owned()] {
                    self.chat_line(line, YELLOW);
                }
            }
            "TT_EX_HI" => self.greet_friends(""),
            "TT_EX_SAVE_CHAT" => self.save_chat(),
            // The ten first of a ranking are asked for (`CZ_REQ_RANKING`); the answer is told in the chat
            // (`incoming/commands.rs`).
            "TT_EX_RANK_BLACKSMITH" => self.send(&commands::ranking_ask(0)),
            "TT_EX_RANK_ALCHEMIST" => self.send(&commands::ranking_ask(1)),
            "TT_EX_RANK_TAEKWON" => self.send(&commands::ranking_ask(2)),
            // The client does nothing for `/stat`.
            "TT_STAT" => {}
            _ => return false,
        }
        true
    }

    /// `/hi`: every friend is whispered "(Hi) " and the words after the command, or the client's smile when
    /// there are none, and the chat tells of it ("[ To Friends ] : (Hi) …", in the client's `0x1EFFFF`).
    pub(super) fn greet_friends(&mut self, words: &str) {
        let greeting = format!("(Hi) {}", if words.is_empty() { "*^_^*" } else { words });
        let friends: Vec<String> = self.friends.list.iter().filter(|friend| !friend.offline).map(|friend| friend.name.clone()).collect();
        for friend in friends {
            self.send(&crate::net::whispers::whisper(&friend, &greeting));
        }
        self.chat_line(format!("[ To Friends ] : {greeting}"), "#ffff1e");
    }

    /// `/savechat`: what each tab of the chat lists is given to the player as a text file, a line to a line
    /// (the client writes `Chat\Chat_<tab>.txt`), and the chat says so ("<file> is Saved.", in green).
    fn save_chat(&mut self) {
        for tab in self.chat_tabs() {
            let lines: Vec<&str> = self.chat.listed(&tab).map(|line| line.words.as_str()).collect();
            if lines.is_empty() {
                continue;
            }
            let name: String = format!("Chat_{}.txt", tab.name).chars().map(|sign| if "\\/:*?\"<>|".contains(sign) { '_' } else { sign }).collect();
            let text = String::from(js_sys::encode_uri_component(&(lines.join("\n") + "\n")));
            give_file(&name, &format!("data:text/plain;charset=utf-8,{text}"));
            self.chat_line(format!("{name} is Saved."), "#00ff00");
        }
    }
}
