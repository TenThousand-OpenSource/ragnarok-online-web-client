//! The family on the page: the question of one who asks to adopt the character (the client's
//! `UIBabyAcceptWnd` and `UIBabyAcceptWnd2`, laid out as every box of its kind is:
//! `UIExchangeAcceptWnd::OnCreate`).

use super::*;
use crate::press::family::Press;

impl App {
    /// The question: first the asker's name in brackets and the client's line 794; after OK, the client's
    /// warning of what being adopted takes away (the line its `UIBabyAcceptWnd2` shows, 2965 of its table).
    pub(in crate::web) fn family_question(&self) -> String {
        let Some((parent, warned)) = &self.family.asks else { return String::new() };
        let words = if *warned {
            escape(&Self::words(2965, "Are you sure you want to be adopted?")).replace("\\n", "<br>")
        } else {
            escape(&format!("({}){}", parent.name, Self::words(794, " wishes to adopt you. Do you accept?")))
        };
        message_box(Asked::Offer, &words, Press::Yes, Press::No)
    }
}
