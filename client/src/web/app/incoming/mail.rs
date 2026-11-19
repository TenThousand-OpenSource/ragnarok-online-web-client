//! The mail: what the map server tells of it. What the chat says of each answer, and in which colour, is the
//! client's (`ReceiveMailboxAvailability`, `ReceiveRodexMailList`, `ReceiveRodexReadMail`,
//! `ReceiveRodexMailReturn`, `ReceiveRodexItemReturn`, `ReceiveRodexMailDeleted`, `ReceiveRodexOpen`,
//! `ReceiveRodexRecipient`, `ReceiveItemReward2`, `ReceiveStackedItemReturn`, `ReceiveRodexComposeResult`).

use super::*;
use crate::net::mail::{self, added};
use crate::mail::{has, tab};
use crate::net::statuses;

/// The state that carries how many letters were sent today (`EFST_DAILYSENDMAILCNT`), in its second value.
const SENT_TODAY: u16 = 824;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other. Three packets of other systems
    /// are overheard here and left to their own readers: entering the game, where the mail of the character
    /// before is forgotten; a state going on, for the count of letters sent today; and an item leaving the
    /// inventory, which closes a letter being written.
    pub(super) fn mail_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            packets::ZC_ACCEPT_ENTER => {
                self.mail = Mail::default();
                return false;
            }
            statuses::ZC_MSG_STATE_CHANGE2 | statuses::ZC_MSG_STATE_CHANGE3 => {
                if let Some(change) = statuses::changed(packet).filter(|change| change.state == SENT_TODAY && change.on && change.who == self.own_account()) {
                    self.mail.sent_today = change.values[1];
                    self.redraw(&[&wnd::MAIL]);
                }
                return false;
            }
            // The client closes the letter being written when anything leaves the inventory
            // (`ReceiveItemConsumed`): a thing in the letter might be the one that went.
            net::inventory::ZC_DELETE_ITEM_FROM_BODY => {
                if self.mail.writing.is_some() {
                    self.chat_line(Self::words(2734, "Consuming item in inventory will close mail compose window."), MAIL_SAID);
                    self.mail_write_closed();
                }
                return false;
            }
            // There is mail not read, or none any more: the notice by the corner shows or goes, and the
            // menu's icon is marked or not as the server says (it says so again after a letter is read or
            // deleted: rAthena's `clif_Mail_new`, which counts letters not read and those with something
            // still to take). Between its tellings the mark also goes with the last unread letter here.
            mail::ZC_NOTIFY_UNREADMAIL => {
                self.mail.notice = mail::unread(packet) == Some(true);
                self.mail.marked = self.mail.notice;
                self.redraw(&[&wnd::MAIL_NOTICE, &wnd::MENU]);
            }
            mail::ZC_MAIL_LIST => self.mail_listed(packet),
            mail::ZC_ACK_READ_RODEX => self.mail_opened(packet),
            mail::ZC_ACK_ZENY_FROM_MAIL | mail::ZC_ACK_ITEM_FROM_MAIL => self.mail_taken(id == mail::ZC_ACK_ITEM_FROM_MAIL, packet),
            // A letter was deleted: it stays in the list, greyed, until the mailbox is asked for again.
            mail::ZC_ACK_DELETE_MAIL => {
                if let Some(letter) = mail::deleted(packet).and_then(|(kind, id)| self.mail.store.get_mut(kind, id)) {
                    let unread = !letter.read;
                    letter.gone = true;
                    if unread {
                        self.mail_one_less();
                    }
                }
                self.redraw(&[&wnd::MAIL, &wnd::MENU]);
            }
            // A letter may be written: its window opens, the name asked with in the receiver's field. While
            // one is open already, nothing happens.
            mail::ZC_ACK_OPEN_WRITE_MAIL => match mail::write_opened(packet) {
                Some((to, true)) => {
                    if self.mail.writing.is_none() {
                        self.mail.writing = Some(Writing { to, title: "TITLE".to_owned(), zeny: "0".to_owned(), ..Writing::default() });
                        self.redraw(&[&wnd::MAIL_WRITE]);
                    }
                }
                Some((_, false)) => self.chat_line(Self::words(2604, "You cannot open the mail."), MAIL_SAID),
                None => {}
            },
            // Who the receiver is; a character of 0 is nobody.
            mail::ZC_CHECKNAME => match mail::receiver(packet) {
                Some(receiver) if receiver.character == 0 => self.chat_line(Self::words(2615, "The recipient's name does not exist."), MAIL_SAID),
                Some(receiver) => {
                    if let Some(writing) = &mut self.mail.writing {
                        writing.receiver = Some(receiver);
                        self.redraw(&[&wnd::MAIL_WRITE]);
                    }
                }
                None => {}
            },
            mail::ZC_ACK_ADD_ITEM_RODEX => self.mail_attach_answered(packet),
            mail::ZC_ACK_REMOVE_ITEM_MAIL => self.mail_detach_answered(packet),
            mail::ZC_ACK_WRITE_MAIL => self.mail_sent(packet),
            _ => return false,
        }
        true
    }

    /// An unread letter was read or deleted: with none left the menu's icon loses its mark.
    fn mail_one_less(&mut self) {
        self.mail.unread = (self.mail.unread - 1).max(0);
        if self.mail.unread == 0 {
            self.mail.marked = false;
        }
    }

    /// Whether the client knows a name as one of the player's own characters or of the guild, which is what
    /// it tells the server of a receiver and goes by for a sender (`CSession::HasCharacterName`,
    /// `IsKnownName`). The client knows the player's friends too; this client has no list of them yet.
    pub(in crate::web::app) fn mail_knows(&self, name: &str) -> bool {
        self.characters.iter().any(|character| character.name == name) || self.guild.members.iter().any(|member| member.name == name)
    }

    /// ZC_ACK_MAIL_LIST3: the letters are taken in, those the mailbox has already staying as they are. With
    /// the list whole the mailbox shows its first page again, nothing ticked, and its buttons wait no more.
    fn mail_listed(&mut self, packet: &[u8]) {
        let Some((whole, letters)) = mail::list(packet) else { return };
        let now = epoch();
        for letter in letters {
            // Only a letter a player sent has a sender who may be unknown.
            let known = letter.tab != tab::NORMAL || letter.has & has::NPC != 0 || self.mail_knows(&letter.sender);
            if self.mail.store.add(letter, now, known) {
                self.mail.unread += 1;
            }
        }
        if whole {
            (self.mail.loaded, self.mail.waiting) = (true, false);
            self.mail.turn_to(0);
            self.redraw(&[&wnd::MAIL]);
        }
    }

    /// ZC_ACK_READ_RODEX: a letter's text, zeny and things: it is read now, and its window opens. The
    /// mailbox turns to the kind of letter it is, as the client's does. (The client adds the text to what it
    /// has of the letter, so a letter read twice would say everything twice; here it is the text.)
    fn mail_opened(&mut self, packet: &[u8]) {
        let Some(read) = mail::read(packet) else { return };
        let Some(letter) = self.mail.store.get_mut(read.tab, read.id) else { return };
        let unread = !std::mem::replace(&mut letter.read, true);
        data::want_items(read.items.iter().map(|item| item.id), || dispatch(Action::ItemsKnown));
        (letter.body, letter.zeny, letter.items) = (read.body, read.zeny, read.items);
        if unread {
            self.mail_one_less();
        }
        (self.mail.tab, self.mail.reading, self.mail.taking) = (read.tab, Some((read.tab, read.id)), (false, false));
        self.redraw(&[&wnd::MAIL, &wnd::MAIL_READ, &wnd::MENU]);
    }

    /// ZC_ACK_ZENY_FROM_MAIL and ZC_ACK_ITEM_FROM_MAIL: the zeny, or the things, of a letter were taken, and
    /// the letter has them no more; or why not.
    fn mail_taken(&mut self, things: bool, packet: &[u8]) {
        let Some((kind, id, result)) = mail::taken(packet) else { return };
        if things { self.mail.taking.0 = false } else { self.mail.taking.1 = false }
        let words = match (things, result) {
            (false, 0) => Self::words(2591, "Zeny received."),
            (false, 1) => Self::words(2592, "Failed to get zeny."),
            (false, 2) => Self::words(2593, "Exceeded the limit of zeny."),
            (true, 0) => Self::words(2588, "This item has been moved to the inventory."),
            (true, 1) => Self::words(2589, "Failed to get items."),
            (true, 2) => Self::words(2590, "Please empty your inventory."),
            _ => return self.redraw(&[&wnd::MAIL_READ]),
        };
        if let Some(letter) = self.mail.store.get_mut(kind, id).filter(|_| result == 0) {
            if things {
                letter.has &= !has::ITEMS;
                letter.items.clear();
            } else {
                letter.has &= !has::ZENY;
                letter.zeny = 0;
            }
        }
        self.chat_line(words, MAIL_SAID);
        self.redraw(&[&wnd::MAIL, &wnd::MAIL_READ]);
    }

    /// ZC_ACK_ADD_ITEM_RODEX: a thing went into the letter being written (more of one that is in it already
    /// is added to it), or why it did not.
    fn mail_attach_answered(&mut self, packet: &[u8]) {
        let Some((result, item, weight)) = mail::add_answered(packet) else { return };
        let words = match result {
            added::DONE => Self::words(2594, "Item successfully attached."),
            added::OVERWEIGHT => Self::words(2630, "The weight of the items that can be attached to mail has been exceeded."),
            added::FAILED => Self::words(2629, "Failed to attach an item."),
            added::TOO_MANY => Self::words(2697, "The number of items that can be attached is exceeded."),
            added::NOT_ALLOWED => Self::words(2699, "Unable to attach the items."),
            _ => return,
        };
        if let (added::DONE, Some(writing)) = (result, &mut self.mail.writing) {
            data::want_items(std::iter::once(item.id), || dispatch(Action::ItemsKnown));
            match writing.items.iter_mut().find(|put| put.index == item.index) {
                Some(put) => put.amount = put.amount.saturating_add(item.amount),
                None => writing.items.push(item),
            }
            writing.weight = weight;
        }
        self.chat_line(words, MAIL_SAID);
        self.redraw(&[&wnd::MAIL_WRITE]);
    }

    /// ZC_ACK_REMOVE_ITEM_MAIL: so many of a thing came back out of the letter being written.
    fn mail_detach_answered(&mut self, packet: &[u8]) {
        let Some((done, index, amount, weight)) = mail::remove_answered(packet) else { return };
        if !done {
            return self.chat_line(Self::words(2602, "Please free the window possessing space."), MAIL_SAID);
        }
        if let Some(writing) = &mut self.mail.writing {
            if let Some(item) = writing.items.iter_mut().find(|item| item.index == index) {
                item.amount = item.amount.saturating_sub(amount);
            }
            writing.items.retain(|item| item.amount > 0);
            writing.weight = weight;
            self.redraw(&[&wnd::MAIL_WRITE]);
        }
    }

    /// ZC_ACK_WRITE_MAIL: the letter was sent, or why not. For one not sent the client tells the server once
    /// more that nothing is being written.
    fn mail_sent(&mut self, packet: &[u8]) {
        let words = match mail::sent(packet) {
            Some(0) => Self::words(2587, "Your mail has been sent."),
            Some(1) => Self::words(2597, "Mail delivery failed."),
            Some(2) => Self::words(2603, "Sending mail count exceeded."),
            Some(3) => Self::words(2598, "Transmission has failed to inappropriate items."),
            Some(4) => Self::words(2599, "Information of the recipient does not exist."),
            _ => return,
        };
        if mail::sent(packet) != Some(0) {
            self.send_on_map(&mail::cancel_write());
        }
        self.chat_line(words, MAIL_SAID);
        self.redraw(&[&wnd::MAIL]);
    }
}
