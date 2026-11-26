//! What the Adventurer's Agency is told: by the map server, that someone asks the leader to be let in and
//! what became of the player's own asking (`crate::net::agency`); and by the game's web service, the answers
//! to the board's requests (`crate::agency`). The client's `ID_ZC_PARTY_REQ_MASTER_TO_JOIN` arm with
//! `UIRequestJoinPartyMgr::Add`, `ReceivePartyJoinResult`, and `CAdventurerAgencyMgr::CompleteList`,
//! `CompleteMyParty`, `CompletePartyInfo`, `CompleteRegistration`, `CompleteStopRecruiting`.

use super::*;
use crate::agency::{Named, Request};
use crate::net::agency;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn agency_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // `UIRequestJoinPartyMgr::Add`: one whose requests are blocked is passed by; the question opens,
            // or the request waits behind the one that stands.
            agency::ZC_PARTY_REQ_MASTER_TO_JOIN => {
                if let Some(asking) = agency::asking(packet).filter(|asking| !self.agency.blocked.contains(&asking.account)) {
                    if self.agency.asking.is_none() {
                        self.agency.asking = Some(asking);
                        self.redraw(&[&wnd::AGENCY_ASKS]);
                    } else {
                        self.agency.waiting.push_back(asking);
                    }
                }
            }
            agency::ZC_PARTY_JOIN_REQ_ACK_FROM_MASTER => {
                if let Some((first, second, leader, status)) = agency::joined(packet) {
                    self.agency_join_told(&first, &second, leader, status);
                }
            }
            _ => return false,
        }
        true
    }

    /// `ReceivePartyJoinResult`: a line of the chat, and the notice of a party that turned out to be full is
    /// taken down.
    fn agency_join_told(&mut self, first: &str, second: &str, leader: u32, status: u32) {
        let Some((line, named, good, stops)) = crate::agency::join_result(status, self.party_led()) else { return };
        let words = Self::words(line, "");
        let words = match named {
            Named::Nobody => words,
            Named::First => words.replacen("%s", first, 1),
            Named::Second => words.replacen("%s", second, 1),
        };
        self.chat_line(words, if good { PALE_YELLOW } else { RED });
        if stops {
            self.agency.stopping = leader;
            self.agency_asked(Request::Stop, self.agency_asker().stop(leader));
        }
    }

    /// Whether the player leads a party (the client's `partyLeader`).
    pub(in crate::web) fn party_led(&self) -> bool {
        let own = self.own_account();
        self.party.members.iter().any(|member| member.account == own && member.leader)
    }

    /// The web service has answered a request of the board, or could not be asked (`None`).
    pub(in crate::web::app) fn agency_answered(&mut self, request: Request, answer: Option<String>) {
        match request {
            Request::List | Request::Search => self.agency_listed(answer),
            Request::Mine => self.agency_own_told(answer),
            Request::Info => self.agency_info_told(answer),
            Request::Add => self.agency_added(answer),
            Request::Stop => self.agency_stopped(answer),
        }
    }

    /// `CompleteList`: the notices of the page; the board shows them (it opens for them, as the client's
    /// does), says whether a notice may be put up, and its arrows are live again.
    fn agency_listed(&mut self, answer: Option<String>) {
        let Some((entries, pages)) = answer.as_deref().and_then(crate::agency::listed) else { return };
        self.agency.entries = entries;
        if let Some(pages) = pages {
            self.agency.pages = pages;
        }
        self.agency.page = self.agency.page.min(self.agency.pages);
        (self.agency.open, self.agency.loading, self.agency.settled) = (true, false, true);
        self.redraw(&[&wnd::AGENCY]);
    }

    /// `CompleteMyParty`: the player's own notice, or that there is none.
    fn agency_own_told(&mut self, answer: Option<String>) {
        let Some(own) = answer.as_deref().and_then(crate::agency::mine) else { return };
        self.agency.registered = own.is_some();
        self.agency.own = own;
        (self.agency.open, self.agency.settled) = (true, true);
        self.redraw(&[&wnd::AGENCY]);
    }

    /// `CompletePartyInfo`: the notice of the party one asked to join, read anew. Gone, the chat says so and
    /// its window closes; else the board and the window show it as it is now, and the leader is asked, unless
    /// the level does not fit or the player is in a party already.
    fn agency_info_told(&mut self, answer: Option<String>) {
        let Some(notice) = answer.as_deref().and_then(crate::agency::info) else { return };
        let Some(notice) = notice else {
            self.chat_line(Self::words(3443, "No party can be found."), RED);
            self.agency.reading = None;
            return self.redraw(&[&wnd::AGENCY_READ]);
        };
        if let Some(entry) = self.agency.entries.iter_mut().find(|entry| entry.account == notice.account) {
            *entry = notice.clone();
        }
        if self.agency.reading.is_some() {
            self.agency.reading = Some(notice.clone());
        }
        let level = self.agency_level();
        match crate::agency::join_refused(&notice, level, !self.party.members.is_empty()) {
            Some(line) => self.chat_line(Self::words(line, ""), RED),
            None => {
                self.send_on_map(&crate::net::agency::ask_to_join(notice.character, notice.account));
                self.agency.reading = None;
            }
        }
        self.redraw(&[&wnd::AGENCY, &wnd::AGENCY_READ]);
    }

    /// `CompleteRegistration`: the notice is up (the chat says so, and the board asks for its first page
    /// anew, opening if it was closed), or the chat says it could not be put up.
    fn agency_added(&mut self, answer: Option<String>) {
        match answer.as_deref().and_then(crate::agency::done) {
            Some(true) => {
                self.agency.registered = true;
                self.chat_line(Self::words(3418, "You're party has been registered in the Adventurer's Agency."), PALE_YELLOW);
                if self.agency.open {
                    self.agency_reload();
                } else {
                    self.agency_opened();
                }
                self.redraw(&[&wnd::AGENCY]);
            }
            Some(false) => self.chat_line(Self::words(3417, "Failed to register for the Adventurer's Agency."), RED),
            None => {}
        }
    }

    /// `CompleteStopRecruiting`: the notice is down (of the player's own the chat says so, and the board asks
    /// for its first page anew), or, of the player's own, the chat says it could not be taken down.
    fn agency_stopped(&mut self, answer: Option<String>) {
        let own = self.agency.stopping == self.own_account();
        match answer.as_deref().map(crate::agency::done) {
            Some(Some(true)) => {
                if own {
                    self.chat_line(Self::words(3459, "Party recruitment has been canceled."), PALE_YELLOW);
                }
                (self.agency.registered, self.agency.own) = (false, None);
                if self.agency.open {
                    self.agency_reload();
                    self.redraw(&[&wnd::AGENCY]);
                }
            }
            // An answer that is no JSON says nothing, as in the client.
            Some(None) => {}
            Some(Some(false)) | None => {
                if own {
                    self.chat_line(Self::words(3507, "You can not stop the party recruitment."), RED);
                }
            }
        }
    }
}
