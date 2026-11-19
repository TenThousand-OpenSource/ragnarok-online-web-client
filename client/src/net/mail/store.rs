//! How the letters are kept and ordered as the server lists them, and the rules of the mailbox that need no page: the colour of a title, the time a letter has left, the fee, the zeny typed.

use super::*;

/// A letter as the mailbox keeps it (the client's `RodexMail`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Letter {
    pub read: bool,
    /// What is in it (`has`).
    pub has: u8,
    pub sender: String,
    pub title: String,
    /// When it is deleted by itself, in seconds since 1970.
    pub expires: f64,
    /// Its text, zeny and things, once it was read.
    pub body: String,
    pub zeny: i64,
    pub items: Vec<Item>,
    /// The server has deleted it: it stays listed, greyed, until the mailbox is asked for again (the client's
    /// `isReturned`, cleared by `PurgeFlagged`).
    pub gone: bool,
    /// The sender is one of the player's own characters, a friend or of the guild (`isKnownSender`).
    pub known: bool,
}

/// A row of ZC_ACK_MAIL_LIST3.
#[derive(Debug, Clone, PartialEq)]
pub struct Listed {
    pub tab: u8,
    pub id: i64,
    pub read: bool,
    pub has: u8,
    pub sender: String,
    /// The seconds until it is deleted by itself.
    pub expires_in: u32,
    pub title: String,
}

/// The letters, by kind and number (the client's `RodexMailStore`: three maps, the newest number first).
#[derive(Debug, Default)]
pub struct Store {
    pub tabs: [BTreeMap<i64, Letter>; 3],
}

impl Store {
    pub fn get(&self, tab: u8, id: i64) -> Option<&Letter> {
        self.tabs.get(usize::from(tab))?.get(&id)
    }

    pub fn get_mut(&mut self, tab: u8, id: i64) -> Option<&mut Letter> {
        self.tabs.get_mut(usize::from(tab))?.get_mut(&id)
    }

    /// Takes a listed letter in. One the mailbox has already stays as it is (`ReceiveRodexMailList` inserts
    /// into a map, which keeps what is there). Gives back whether it is new and not read.
    pub fn add(&mut self, listed: Listed, now: f64, known: bool) -> bool {
        let Some(letters) = self.tabs.get_mut(usize::from(listed.tab)) else { return false };
        if letters.contains_key(&listed.id) {
            return false;
        }
        let unread = !listed.read;
        letters.insert(
            listed.id,
            Letter { read: listed.read, has: listed.has, sender: listed.sender, title: listed.title, expires: now + f64::from(listed.expires_in), known, ..Letter::default() },
        );
        unread
    }

    /// The number the requests for the mailbox name for a kind: the last of the client's map, which is the
    /// lowest it has; 0 for none.
    pub fn last(&self, tab: u8) -> i64 {
        self.tabs.get(usize::from(tab)).and_then(|letters| letters.keys().next().copied()).unwrap_or(0)
    }

    /// Forgets the letters the server has deleted (`RodexMailStore::PurgeFlagged`).
    pub fn purge(&mut self) {
        for letters in &mut self.tabs {
            letters.retain(|_, letter| !letter.gone);
        }
    }

    /// The letters of a kind in the order the mailbox lists them (`RodexMailEntryLess`): those not read first,
    /// then the newer first. The client compares the times they were sent, which this list of the server does
    /// not tell, so all are equal there and the order of its map stays: the higher number first. One whose
    /// time has run out is not listed (`DrawExpireTime` drops it).
    pub fn listed(&self, tab: u8, now: f64) -> Vec<(u8, i64)> {
        let Some(letters) = self.tabs.get(usize::from(tab)) else { return Vec::new() };
        let mut rows: Vec<(bool, i64)> = letters.iter().filter(|(_, letter)| letter.expires > now).map(|(id, letter)| (letter.read, *id)).collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
        rows.into_iter().map(|(_, id)| (tab, id)).collect()
    }

    /// The letters of every kind whose title, or whose sender, has the words in it (`SearchByTitle`,
    /// `SearchBySender`), in the same order.
    pub fn search(&self, by_sender: bool, words: &str, now: f64) -> Vec<(u8, i64)> {
        let mut rows: Vec<(bool, i64, u8)> = Vec::new();
        for tab in [tab::NORMAL, tab::ACCOUNT, tab::RETURNED] {
            for (id, letter) in &self.tabs[usize::from(tab)] {
                if letter.expires > now && if by_sender { &letter.sender } else { &letter.title }.contains(words) {
                    rows.push((letter.read, *id, tab));
                }
            }
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
        rows.into_iter().map(|(_, id, tab)| (tab, id)).collect()
    }
}

/// The colour of a letter's title in the list (`UIRodexInboxWnd::SetMailRow`, written here as the page wants
/// it): from a sender not known, red until it is read and pale red after; otherwise black, and grey once read.
/// Only the letters of the first kind have a sender who may be unknown.
pub fn title_colour(tab: u8, letter: &Letter) -> &'static str {
    if letter.gone {
        return "#c8c8c8";
    }
    match (tab == tab::NORMAL && !letter.known, letter.read) {
        (true, false) => "#ff0000",
        (true, true) => "#e6c8c8",
        (false, true) => "#c8c8c8",
        (false, false) => "#000000",
    }
}

/// How long a letter still lies in the mailbox, as its row says it (`DrawExpireTime`): the line of the
/// client's message table with its words in English, and the number that goes into it.
pub fn time_left(seconds: i64) -> (usize, &'static str, i64) {
    match seconds {
        86400.. => (2852, "%d day", seconds / 86400),
        3600.. => (2851, "%d hour", seconds / 3600),
        _ => (2850, "%d minute", seconds / 60),
    }
}

/// What sending a letter costs (`UIRodexComposeWnd::Paint`): 2500 zeny a thing, and two hundredths of the
/// zeny sent.
pub fn fee(items: usize, zeny: i64) -> i64 {
    items as i64 * 2500 + zeny * 2 / 100
}

/// The zeny typed into the letter (`UIRodexComposeWnd::GetZeny`): the digits it starts with, commas between
/// them or not; nothing for anything else.
pub fn zeny_typed(text: &str) -> i64 {
    let digits: String = text.trim().chars().filter(|letter| *letter != ',').take_while(char::is_ascii_digit).collect();
    digits.parse().unwrap_or(if digits.is_empty() { 0 } else { i64::MAX })
}
