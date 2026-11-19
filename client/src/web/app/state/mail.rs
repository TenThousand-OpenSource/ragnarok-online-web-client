//! What the app keeps of the mail.

use crate::net::inventory::Item;
use crate::net::mail::Receiver;
use crate::mail::Store;

/// The colour of what the chat is told about the mail (the client's 0x64FFFF, which it writes blue first).
pub(in crate::web) const MAIL_SAID: &str = crate::said::colour::PALE_YELLOW;

/// The mail (the client's RODEX): the letters as the server has listed them (`RodexMailStore`), the mailbox's
/// window (`UIRodexInboxWnd`), the letter being read (`UIRodexContentsWnd`) and the one being written
/// (`UIRodexComposeWnd`).
#[derive(Default)]
pub(in crate::web) struct Mail {
    /// The server says there is mail not read: the notice by the corner of the screen shows (the client's
    /// `UIOpenMailBoxWnd`), until the mailbox is closed or the server says there is none.
    pub notice: bool,
    /// The menu's icon wears its mark of something new, until the last unread letter is read or deleted.
    pub marked: bool,
    pub store: Store,
    /// How many of the letters listed are not read (`unreadCount`).
    pub unread: i32,
    /// The mailbox's window is open; the server has listed all of it (`isPageComplete`); the mailbox was asked
    /// for anew and the buttons wait for it.
    pub open: bool,
    pub loaded: bool,
    pub waiting: bool,
    /// Which kind of letters is shown (`inboxType`), or what was searched for; the page of six; which of its
    /// rows are ticked.
    pub tab: u8,
    pub searching: bool,
    pub page: usize,
    pub checked: [bool; crate::mail::PAGE],
    /// The search goes through the senders, not the titles; and what it last found.
    pub by_sender: bool,
    pub found: Vec<(u8, i64)>,
    /// How many letters were sent today, as the server counts them (the state `EFST_DAILYSENDMAILCNT`).
    pub sent_today: i32,
    /// The letter being read: its kind and number.
    pub reading: Option<(u8, i64)>,
    /// Its things, and its zeny, were asked for and the server has not answered: the button waits.
    pub taking: (bool, bool),
    /// The letter being written.
    pub writing: Option<Writing>,
    /// What is asked before something is done.
    pub asks: Option<MailAsk>,
    /// The thing whose count is asked for before it goes into the letter: where it is carried, and how many
    /// there are.
    pub counting: Option<(u16, u16)>,
}

/// A letter being written.
#[derive(Default)]
pub(in crate::web) struct Writing {
    /// What was typed, as of the last press in the window: the receiver, the title, the text and the zeny.
    /// Between presses the page's fields have it.
    pub to: String,
    pub title: String,
    pub body: String,
    pub zeny: String,
    /// The receiver, once the server has said who that is.
    pub receiver: Option<Receiver>,
    /// The things put in (each with where it is carried and how many went in), and what they weigh together,
    /// as the server last said.
    pub items: Vec<Item>,
    pub weight: u16,
}

/// What the mail asks before it acts, in the client's message box. A letter is its kind and its number.
#[derive(Clone, PartialEq)]
pub(in crate::web) enum MailAsk {
    /// Delete every letter of the tab that has nothing in it?
    DeleteAll,
    /// Take what is in every letter of the tab?
    ClaimAll,
    /// Delete the letters ticked?
    Delete(Vec<(u8, i64)>),
    /// Take what is in the letters ticked?
    Claim(Vec<(u8, i64)>),
    /// Delete the letter being read?
    DeleteRead,
    /// Read a letter of someone who is neither a friend nor of the guild?
    Stranger(u8, i64),
}

impl Mail {
    /// Every letter of the tab shown, in the order of the list.
    pub fn listed(&self, now: f64) -> Vec<(u8, i64)> {
        if self.searching { self.found.clone() } else { self.store.listed(self.tab, now) }
    }

    /// The rows of the page shown: the kind and the number of the letter in each.
    pub fn rows(&self, now: f64) -> Vec<(u8, i64)> {
        self.listed(now).into_iter().skip(self.page * crate::mail::PAGE).take(crate::mail::PAGE).collect()
    }

    /// The letters of the rows that are ticked, without those the server has deleted.
    pub fn ticked(&self, now: f64) -> Vec<(u8, i64)> {
        self.rows(now)
            .into_iter()
            .enumerate()
            .filter(|(row, (tab, id))| self.checked[*row] && self.store.get(*tab, *id).is_some_and(|letter| !letter.gone))
            .map(|(_, letter)| letter)
            .collect()
    }

    /// The page is another, or what it lists is: nothing of it is ticked.
    pub fn turn_to(&mut self, page: usize) {
        (self.page, self.checked) = (page, [false; crate::mail::PAGE]);
    }
}
