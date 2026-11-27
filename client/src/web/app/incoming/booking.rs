//! What the server says of the old party booking (`crate::net::booking`): the answers about one's own ad, the
//! ads a search found, and the ads registered, changed and gone since. The client's
//! `CGameMode::ReceivePartyBooking*` (`GameModeHandlers.cpp`) and `PartyBookingMgr`.

use super::*;
use crate::booking::{self as rules, line, Entry};
use crate::net::booking;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn booking_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Overheard: the character is chosen anew (`PartyBookingMgr::Reset`), and the windows are gone
            // with the game they stood in. The server forgets an ad as its character leaves.
            packets::ZC_ACCEPT_ENTER => {
                let mut board = std::mem::take(&mut self.booking.board);
                board.reset();
                self.booking = Booking { board, ..Booking::default() };
                return false;
            }
            booking::ZC_PARTY_BOOKING_ACK_REGISTER => {
                if let Some(result) = booking::result(packet) {
                    self.booking_registered(result);
                }
            }
            booking::ZC_PARTY_BOOKING_ACK_SEARCH => {
                if let Some((more, ads)) = booking::found(packet) {
                    self.booking_found(more, &ads);
                }
            }
            booking::ZC_PARTY_BOOKING_ACK_DELETE => {
                if let Some(result) = booking::result(packet) {
                    self.booking_withdrawn(result);
                }
            }
            booking::ZC_PARTY_BOOKING_NOTIFY_INSERT => {
                if let Some(ad) = booking::inserted(packet) {
                    self.booking_advertised(&ad);
                }
            }
            booking::ZC_PARTY_BOOKING_NOTIFY_UPDATE => {
                if let Some((index, jobs)) = booking::updated(packet) {
                    self.booking.board.set_jobs(index, jobs);
                }
            }
            booking::ZC_PARTY_BOOKING_NOTIFY_DELETE => {
                if let Some(index) = booking::deleted(packet) {
                    self.booking.board.remove(index);
                }
            }
            _ => return false,
        }
        self.redraw(&wnd::BOOKING_WINDOWS);
        true
    }

    /// A line of the message table said in the client's box (`UIWindowMgr::Confirm`).
    pub(in crate::web::app) fn booking_told(&mut self, line: usize, number: Option<i64>) {
        let words = Self::words(line, "");
        self.message = Some(match number {
            Some(number) => crate::numbers::filled(&words, &[&number.to_string()]),
            None => words,
        });
        self.render();
    }

    /// The answer to registering (`ReceivePartyBookingUpdateResult`): the window of one's ad opens, with the
    /// jobs the window of registering was sent with, which closes; or a box says why not.
    fn booking_registered(&mut self, result: i16) {
        if let Some(line) = rules::register_refused(result) {
            return self.booking_told(line, None);
        }
        let state = &mut self.booking;
        let level = state.sent.map_or(1, |(level, _)| level);
        if state.own.is_none() {
            after(1000, Action::BookingTick);
        }
        let kind = (js_sys::Math::random() * f64::from(rules::KINDS)) as u32 % rules::KINDS;
        state.own = Some(OwnAd { since: now(), kind, level, small: false });
        if state.register {
            if let Some((_, jobs)) = state.sent {
                state.jobs = rules::listed(&jobs);
            }
            (state.register, state.dropped) = (false, None);
        }
    }

    /// A second of one's own ad has passed: its window counts on and its picture moves
    /// (`UISeekPartyWnd::OnProcess`).
    pub(in crate::web::app) fn booking_tick(&mut self) {
        if self.booking.own.is_some() {
            self.redraw(&[&wnd::BOOKING_OWN]);
            after(1000, Action::BookingTick);
        }
    }

    /// The ads a search found (`ReceivePartyBookingSearchResult`): none, and a box says so; otherwise they go
    /// into the list. Nothing is listed while the list's window is closed.
    fn booking_found(&mut self, more: bool, ads: &[booking::Ad]) {
        self.booking.board.searched = true;
        if !self.booking.list {
            return;
        }
        if ads.is_empty() {
            self.booking.board.more = false;
            return self.booking_told(line::NOTHING_FOUND, None);
        }
        let regions = data::booking_regions();
        self.booking.board.add(ads.iter().map(|ad| (ad.index, Entry::of(ad, &regions))), more);
    }

    /// The answer to taking one's ad away (`ReceivePartyBookingRegisterResult`): its window closes, and a box
    /// says what went wrong.
    fn booking_withdrawn(&mut self, result: i16) {
        let (line, closes) = rules::deleted(result);
        if closes {
            self.booking.own = None;
        }
        if let Some(line) = line {
            self.booking_told(line, None);
        }
    }

    /// Someone registered an ad (`PartyBookingMgr::OnAdvertisement`): it goes into the list if the list is
    /// open and looks for such, and the player is told with a sound and a line of the chat
    /// (`UISeekPartyListWnd::NotifyNewAd`: `party_alarm.wav`, line 1753 in the client's `0xFFFF00`).
    fn booking_advertised(&mut self, ad: &booking::Ad) {
        let entry = Entry::of(ad, &data::booking_regions());
        let list = self.booking.list.then_some((self.booking.sought, self.booking.alarm));
        if self.booking.board.advertised(ad, entry, list) {
            sound::effect("party_alarm.wav", 1.0);
            self.chat_line(Self::words(line::ADDED, "Party ad has been added."), CYAN);
        }
    }
}
