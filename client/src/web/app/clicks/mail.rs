//! The mail: opening the mailbox, its tabs, pages and search, reading a letter, taking what is in it, deleting
//! it (the client's `UIRodexInboxWnd::SendMsg` and `UIRodexContentsWnd::SendMsg`). The letter being written is
//! in `mail/writing.rs`.

use super::*;
use crate::net::mail;
use crate::mail::{has, tab};
use crate::press::Step;
use crate::press::mail::Press;

mod writing;

impl App {
    /// A press of this system (`crate::press::mail`).
    pub(super) fn mail_pressed(&mut self, press: Press) {
        match press {
            Press::Window => self.mail_toggled(),
            Press::Tab(shown) => self.mailbox(|app| {
                (app.mail.searching, app.mail.tab) = if shown < 3 { (false, shown) } else { (true, app.mail.tab) };
                app.mail.turn_to(0);
            }),
            // The page before; the next, if the list goes on past this one.
            Press::Page(step) => self.mailbox(|app| {
                let page = app.mail.page;
                if step == Step::Less && page > 0 {
                    app.mail.turn_to(page - 1);
                } else if step == Step::More && app.mail.listed(epoch()).len() > (page + 1) * crate::mail::PAGE {
                    app.mail.turn_to(page + 1);
                }
            }),
            // The mailbox is asked for anew: what the server has deleted is forgotten, and the buttons wait
            // for the list.
            Press::Refresh => self.mailbox(|app| {
                app.mail.store.purge();
                app.mail.checked = [false; crate::mail::PAGE];
                app.mail.waiting = true;
                app.mail_asked_for(true);
            }),
            Press::Write => self.mailbox(|app| app.mail_write_to("")),
            Press::Check(row) => self.mailbox(|app| {
                if row < app.mail.rows(epoch()).len() {
                    app.mail.checked[row] = !app.mail.checked[row];
                }
            }),
            Press::Title(row) => self.mailbox(|app| app.mail_title_pressed(row)),
            Press::Sender(row) => self.mailbox(|app| app.mail_sender_pressed(row)),
            // The letter of someone not known is to go back to its sender (`ID_REPORT_RODEX`); the server
            // here does nothing about it.
            Press::GiveBack(row) => self.mailbox(|app| {
                if let Some((tab::NORMAL, id)) = app.mail.rows(epoch()).get(row).copied().filter(|_| app.mail.tab == tab::NORMAL) {
                    app.send_on_map(&mail::give_back(id));
                }
            }),
            // For the rows ticked while any is (and nothing at all if those are all deleted already), for
            // the whole tab otherwise: each is asked about first.
            Press::Delete | Press::Claim => self.mailbox(|app| {
                let ticked = app.mail.ticked(epoch());
                app.mail.asks = match (app.mail.checked.contains(&true), press == Press::Delete) {
                    (true, _) if ticked.is_empty() => None,
                    (true, true) => Some(MailAsk::Delete(ticked)),
                    (true, false) => Some(MailAsk::Claim(ticked)),
                    (false, true) => Some(MailAsk::DeleteAll),
                    (false, false) => Some(MailAsk::ClaimAll),
                };
            }),
            Press::SearchIn(by_sender) => self.mailbox(|app| app.mail.by_sender = by_sender),
            // The search's button and the OK of the count send the forms they stand in: the press itself
            // does nothing.
            Press::Search | Press::CountOk => {}
            Press::ReadClose => {
                (self.mail.reading, self.mail.taking) = (None, (false, false));
                self.redraw(&[&wnd::MAIL_READ, &wnd::MAIL]);
            }
            Press::TakeItems | Press::TakeZeny => self.mail_take(press == Press::TakeItems),
            Press::ReadDelete => self.mail_read_delete(),
            Press::Reply => {
                if let Some(sender) = self.mail.reading.and_then(|(kind, id)| self.mail.store.get(kind, id)).map(|letter| letter.sender.clone()) {
                    self.mail_write_to(&sender);
                }
            }
            Press::ReadGiveBack => {
                if let Some((_, id)) = self.mail.reading {
                    self.send_on_map(&mail::give_back(id));
                }
            }
            Press::WriteClose => self.mail_write_closed(),
            Press::Validate => self.mail_validate(),
            Press::Send => self.mail_send(),
            Press::Yes | Press::No => self.mail_answered(press == Press::Yes),
        }
    }

    /// Something of the mailbox itself was pressed. It takes no press while the server has not listed it,
    /// while it waits for the list asked for anew, and while a letter is being read or written or something
    /// is asked (`UIRodexInboxWnd::SendMsg` gives up at once then); otherwise `does` is done and the mailbox
    /// and what it asks are drawn anew.
    fn mailbox(&mut self, does: impl FnOnce(&mut Self)) {
        let mail = &self.mail;
        if !mail.open || !mail.loaded || mail.waiting || mail.reading.is_some() || mail.writing.is_some() || mail.asks.is_some() {
            return;
        }
        does(self);
        self.redraw(&[&wnd::MAIL, &wnd::MAIL_ASKS]);
    }

    /// The key of the mailbox (`HK_MAIL_ONOFF`) does what the menu's icon does.
    pub(in crate::web::app) fn mail_key(&mut self) -> bool {
        self.pressed(Press::Window);
        true
    }

    /// The search was sent (`ID_SEARCH`): every kind of letter is gone through for the words typed, in the
    /// titles or the senders, and the tab of what was found opens.
    pub(in crate::web::app) fn mail_searched(&mut self) {
        self.mailbox(|app| {
            app.mail.found = app.mail.store.search(app.mail.by_sender, &field("mail-sought"), epoch());
            app.mail.searching = true;
            app.mail.turn_to(0);
        });
    }

    /// CZ_OPEN_MAILBOX2, or CZ_REQ_REFRESH_MAIL_LIST2 for the mailbox anew: with the last number the client
    /// has of each kind of letter.
    fn mail_asked_for(&self, anew: bool) {
        let store = &self.mail.store;
        self.send_on_map(&mail::open(anew, [store.last(tab::NORMAL), store.last(tab::RETURNED), store.last(tab::ACCOUNT)]));
    }

    /// The mailbox opens, asking the server for its letters, or closes (`UIMenuIconWnd`, `UIOpenMailBoxWnd`,
    /// and the `WID_RODEXINBOXWND` arms of `UIWindowMgr`). Opened, it forgets what the server has deleted and
    /// shows the picture of waiting until the list is there. Closed, the notice of mail not read goes with it,
    /// and so do a letter being read or written. (The client's own cross does nothing while a letter is open,
    /// and its icon then closes the mailbox alone; here both close everything, as its Escape does.)
    fn mail_toggled(&mut self) {
        if self.mail.open {
            if self.mail.writing.is_some() {
                self.mail_write_closed();
            }
            let mail = &mut self.mail;
            (mail.open, mail.notice, mail.reading, mail.asks, mail.waiting) = (false, false, None, None, false);
            self.send_on_map(&mail::close());
        } else {
            let mail = &mut self.mail;
            mail.store.purge();
            (mail.open, mail.loaded, mail.searching) = (true, false, false);
            mail.turn_to(0);
            self.mail_asked_for(false);
        }
        self.render();
    }

    /// The title of a row: the letter is asked for. One of a sender not known is asked about first, and the
    /// chat warns of it in red (`RODEX_BUTTON_SUBJECT`).
    fn mail_title_pressed(&mut self, row: usize) {
        let Some((kind, id)) = self.mail.rows(epoch()).get(row).copied() else { return };
        let Some(letter) = self.mail.store.get(kind, id).filter(|letter| !letter.gone) else { return };
        if letter.known {
            return self.send_on_map(&mail::ask(kind, id));
        }
        self.mail.asks = Some(MailAsk::Stranger(kind, id));
        self.chat_line(Self::words(3867, "The sender of this RODEX is not a friend or guild member. Please be careful."), RED);
    }

    /// The sender of a row: a letter is written back, for a letter a player sent (`RODEX_BUTTON_SENDER`:
    /// only among the letters of the first kind).
    fn mail_sender_pressed(&mut self, row: usize) {
        let Some((tab::NORMAL, id)) = self.mail.rows(epoch()).get(row).copied().filter(|_| self.mail.tab == tab::NORMAL) else { return };
        let sender = self.mail.store.get(tab::NORMAL, id).filter(|letter| letter.has & has::NPC == 0 && !letter.sender.is_empty() && !letter.gone).map(|letter| letter.sender.clone());
        if let Some(sender) = sender {
            self.mail_write_to(&sender);
        }
    }

    /// What is in a letter is asked for (`RequestMailAttachments`): its things, and its zeny.
    fn mail_claim(&self, kind: u8, id: i64) {
        let Some(letter) = self.mail.store.get(kind, id) else { return };
        for (bit, things) in [(has::ITEMS, true), (has::ZENY, false)] {
            if letter.has & bit != 0 {
                self.send_on_map(&mail::take(things, kind, id));
            }
        }
    }

    /// Letters are deleted (`RequestMailAction`, `RequestAllMailActions`): each that has nothing in it; of
    /// the others the chat says that what is in them must be taken first.
    fn mail_delete(&mut self, letters: &[(u8, i64)]) {
        let mut kept = false;
        for (kind, id) in letters {
            match self.mail.store.get(*kind, *id) {
                Some(letter) if letter.has & (has::ZENY | has::ITEMS) != 0 => kept = true,
                Some(letter) if !letter.gone => self.send_on_map(&mail::delete(*kind, *id)),
                _ => {}
            }
        }
        if kept {
            self.chat_line(Self::words(2612, "Please claim all attachments before deleting your mail."), MAIL_SAID);
        }
    }

    /// What the mail asked was answered. For all of a tab, what is in a letter is taken only from senders
    /// that are known (`RODEX_BUTTON_DELETE_ALL` in the client's naming, the button that reads Claim All).
    fn mail_answered(&mut self, yes: bool) {
        let Some(asked) = self.mail.asks.take() else { return };
        if yes {
            let all = self.mail.listed(epoch());
            match asked {
                MailAsk::DeleteAll => self.mail_delete(&all),
                MailAsk::ClaimAll => {
                    for (kind, id) in all {
                        if self.mail.store.get(kind, id).is_some_and(|letter| letter.known) {
                            self.mail_claim(kind, id);
                        }
                    }
                }
                MailAsk::Delete(letters) => self.mail_delete(&letters),
                MailAsk::Claim(letters) => letters.into_iter().for_each(|(kind, id)| self.mail_claim(kind, id)),
                MailAsk::DeleteRead => {
                    if let Some((kind, id)) = self.mail.reading.take() {
                        self.send_on_map(&mail::delete(kind, id));
                    }
                }
                MailAsk::Stranger(kind, id) => self.send_on_map(&mail::ask(kind, id)),
            }
            self.mail.checked = [false; crate::mail::PAGE];
        }
        self.redraw(&[&wnd::MAIL_ASKS, &wnd::MAIL, &wnd::MAIL_READ]);
    }

    /// The things, or the zeny, of the letter being read are asked for (`ID_BUTTON1`, `ID_BUTTON2`); the
    /// button waits for the answer.
    fn mail_take(&mut self, things: bool) {
        let Some((kind, id)) = self.mail.reading else { return };
        let Some(letter) = self.mail.store.get(kind, id) else { return };
        let (there, waits) = if things { (!letter.items.is_empty(), self.mail.taking.0) } else { (letter.zeny > 0, self.mail.taking.1) };
        if there && !waits {
            self.send_on_map(&mail::take(things, kind, id));
            if things { self.mail.taking.0 = true } else { self.mail.taking.1 = true }
            self.redraw(&[&wnd::MAIL_READ]);
        }
    }

    /// Delete, in the letter being read: asked about first; with something still in the letter the chat
    /// says to take it first.
    fn mail_read_delete(&mut self) {
        let Some(letter) = self.mail.reading.and_then(|(kind, id)| self.mail.store.get(kind, id)) else { return };
        if letter.has & (has::ZENY | has::ITEMS) != 0 {
            return self.chat_line(Self::words(2612, "Please claim all attachments before deleting your mail."), MAIL_SAID);
        }
        self.mail.asks = Some(MailAsk::DeleteRead);
        self.redraw(&[&wnd::MAIL_ASKS]);
    }
}
