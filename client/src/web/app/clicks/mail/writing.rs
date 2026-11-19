//! A letter being written: asking to write one, the receiver asked about, things dragged in and out, and the
//! sending (the client's `UIRodexComposeWnd::SendMsg` and `DragAndDrop`, and the `MM_OPEN_WRITE_RODEX`,
//! `MM_ADD_MAIL_ITEM`, `MM_REMOVE_RODEX_ITEM` and `MM_SEND_MAIL` arms of `CGameMode::SendMsg`).

use super::*;

impl App {
    /// A letter is to be written, to that one if a name is given (`MM_OPEN_WRITE_RODEX`): the server is
    /// asked, and its answer opens the window. A name of 24 letters and more is turned down in a message box.
    pub(in crate::web::app) fn mail_write_to(&mut self, name: &str) {
        if name.len() >= 24 {
            self.message = Some(Self::words(531, "Name is too long. Please enter a name up to 23 english characters."));
            return self.render();
        }
        self.send_on_map(&mail::write(name));
    }

    /// The letter's window closes (`WID_RODEXCOMPOSEWND` of `UIWindowMgr::CloseWindow`): the server is told
    /// that nothing is being written, and what was put in is the character's again.
    pub(in crate::web::app) fn mail_write_closed(&mut self) {
        if self.mail.writing.take().is_some() {
            self.mail.counting = None;
            self.send_on_map(&mail::cancel_write());
            self.redraw(&[&wnd::MAIL_WRITE, &wnd::MAIL_COUNT]);
        }
    }

    /// What is typed into the letter's fields now is kept, each field that is on the page.
    fn mail_keep_typed(&mut self) {
        let Some(writing) = &mut self.mail.writing else { return };
        for (id, kept) in [
            ("mail-to", &mut writing.to),
            ("mail-title", &mut writing.title),
            ("mail-body", &mut writing.body),
            ("mail-zeny", &mut writing.zeny)
        ] {
            if let Some(typed) = written(id) {
                *kept = typed;
            }
        }
    }

    /// The button by the receiver's name (`ID_CHECK`): the server is asked who that is, and told whether the
    /// client knows the name. No name, or one of 24 letters and more, is turned down in the chat.
    pub(super) fn mail_validate(&mut self) {
        self.mail_keep_typed();
        let Some(name) = self.mail.writing.as_ref().filter(|writing| writing.receiver.is_none()).map(|writing| writing.to.clone()) else { return };
        if !name.is_empty() && name.len() < 24 {
            return self.send_on_map(&mail::check_receiver(&name, self.mail_knows(&name)));
        }
        if name.len() >= 24 {
            self.chat_line(Self::words(2599, "Information of the recipient does not exist."), MAIL_SAID);
        }
        self.chat_line(Self::words(2595, "The name of the recipient must be included."), MAIL_SAID);
    }

    /// Send (`ID_SEND`): the letter must have a receiver, a title of 4 to 50 letters, a text of a thousand at
    /// most, and the character the zeny sent and the fee; and the receiver must be one the server has
    /// answered for. Each thing amiss is said in the chat. A letter that goes closes its window at once; the
    /// server's answer comes to the chat.
    pub(super) fn mail_send(&mut self) {
        self.mail_keep_typed();
        let Some(writing) = &self.mail.writing else { return };
        let has = self.hero.zeny();
        let zeny = crate::mail::zeny_typed(&writing.zeny);
        let fee = crate::mail::fee(writing.items.len(), zeny);
        let amiss = if writing.to.is_empty() && writing.receiver.is_none() {
            Some((2595, "The name of the recipient must be included."))
        } else if !crate::mail::TITLE.contains(&writing.title.len()) {
            Some((2596, "The length of the title must be 4 to 50 characters long."))
        } else if writing.body.len() > crate::mail::MOST_TEXT {
            Some((2628, "The maximum length of the message content is 1K."))
        } else if zeny > 0 && zeny > has {
            Some((2613, "You cannot send any more zeny."))
        } else if (zeny > 0 || fee > 0) && zeny.saturating_add(fee) > has {
            Some((2643, "Please check the fees."))
        } else if writing.receiver.is_none() {
            Some((2611, "Press the OK button to confirm the C-CODE."))
        } else {
            None
        };
        if let Some((line, words)) = amiss {
            return self.chat_line(Self::words(line, words), MAIL_SAID);
        }
        if let (Some(receiver), Some(game)) = (&writing.receiver, &self.game) {
            self.send_on_map(&mail::send(receiver, &game.name, zeny, &writing.title, &writing.body));
        }
        self.mail_write_closed();
    }

    /// A thing was dragged out of its window and let go over the letter (`DragAndDrop`): an item of the
    /// inventory goes in, one of it at once, more after its count is asked for. More of a thing that is in
    /// the letter already can follow it, as far as there is more.
    pub(in crate::web::app) fn mail_dropped(&mut self, dragged: Drag) {
        let (Drag::Item(id), Some(writing)) = (dragged, &self.mail.writing) else { return };
        let put = |index: u16| writing.items.iter().find(|item| item.index == index).map_or(0, |item| item.amount);
        let Some((index, left)) =
            self.bag.items.iter().filter(|item| item.id == id && item.worn == 0).map(|item| (item.index, item.amount.saturating_sub(put(item.index)))).find(
                |(_, left)| *left > 0
            ) else { return };
        if left == 1 {
            self.send_on_map(&mail::attach(index, 1));
        } else {
            self.mail.counting = Some((index, left));
            self.redraw(&[&wnd::MAIL_COUNT]);
        }
    }

    /// The count asked for was given. As the window that asks how many to drop: a number that is none, or
    /// more than there is, puts nothing in.
    pub(in crate::web::app) fn mail_counted(&mut self) {
        if let Some((index, most)) = self.mail.counting.take() {
            if let Some(count) = Self::counted(&wnd::MAIL_COUNT, most).filter(|_| self.mail.writing.is_some()) {
                self.send_on_map(&mail::attach(index, count));
            }
        }
        self.redraw(&[&wnd::MAIL_COUNT]);
    }

    /// A thing of the letter was dragged out of it and let go over the inventory (`UM_DECITEM`): all of it
    /// comes back out.
    pub(in crate::web::app) fn mail_taken_out(&mut self, index: u16) {
        if let Some(item) = self.mail.writing.as_ref().and_then(|writing| writing.items.iter().find(|item| item.index == index)) {
            self.send_on_map(&mail::detach(index, item.amount));
        }
    }
}
