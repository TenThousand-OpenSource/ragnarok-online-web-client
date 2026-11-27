//! The old party booking: what its windows do when pressed (the client's `UISeekPartyListWnd::SendMsg`,
//! `UISeekPartyMBWnd::SendMsg`, `UISeekPartyWnd::SendMsg`, `UIJobListWnd::SendMsg`), and the two commands that
//! open them.

use super::*;
use crate::booking::{self as rules, Row, Search, LEVEL, OWN_LEVEL};
use crate::net::booking;
use crate::press::booking::{Choice, Press};

impl App {
    /// A press of this system (`crate::press::booking`).
    pub(super) fn booking_pressed(&mut self, press: Press) {
        self.booking_typed_kept();
        let state = &mut self.booking;
        match press {
            // `ID_CLOSE`: how the box "Notice" stands is kept for when the list is closed.
            Press::Close => (state.board.alarm, state.list, state.dropped) = (state.alarm, false, None),
            Press::Size => (state.small, state.dropped) = (!state.small, None),
            Press::RegisterSize => (state.register_small, state.dropped) = (!state.register_small, None),
            Press::OwnSize => {
                if let Some(own) = &mut state.own {
                    own.small = !own.small;
                }
            }
            Press::Search => self.booking_search(false),
            Press::Row(index) => state.picked = Some(index),
            Press::Talk => self.booking_talk(),
            Press::Recruit => self.booking_register_switched(),
            Press::Help => state.help = true,
            Press::HelpClose => state.help = false,
            Press::Alarm => state.alarm = !state.alarm,
            Press::Opens(choice) => state.dropped = (state.dropped != Some(choice)).then_some(choice),
            Press::Pick(place) => self.booking_picked(place),
            Press::Tick(job) => {
                if !state.ticked.remove(&job) {
                    state.ticked.insert(job);
                }
            }
            Press::Register => self.booking_register_asked(),
            Press::RegisterClose => (state.register, state.changing, state.dropped) = (false, None, None),
            // `ID_CLOSE` of one's own ad: the server is asked to take it away, and its answer closes the window.
            Press::Withdraw => {
                state.changing = None;
                self.send_on_map(&booking::delete());
            }
            // `ID_CHARGE`: the jobs of the ad as they stand, to be changed.
            Press::Jobs => state.changing = Some(state.jobs.clone()),
            Press::JobTick(place) => {
                if let Some((_, sought)) = state.changing.as_mut().and_then(|jobs| jobs.get_mut(place)) {
                    *sought = !*sought;
                }
            }
            Press::JobsOk => self.booking_jobs_changed(),
            Press::JobsClose => state.changing = None,
        }
        self.redraw(&wnd::BOOKING_WINDOWS);
    }

    /// What is typed into the two fields of the level is kept, so that a window drawn anew shows it still.
    fn booking_typed_kept(&mut self) {
        if self.booking.list && !self.booking.small {
            self.booking.level = field(LEVEL);
        }
        if self.booking.register && !self.booking.register_small {
            self.booking.own_level = field(OWN_LEVEL);
        }
    }

    /// What was typed into the chat, when it opens a window of the booking: `/recruit` the window an ad is
    /// registered in, `/booking` the list (the client's `TT_SEEK_PARTY` and `TT_REQ_RECRUIT`:
    /// `MM_OPEN_SEEKPARTY_MEMBER`, `MM_OPEN_SEEKPARTY_LIST`). The two words are those the client's own help of
    /// the booking gives (`PartyBookingHelp` of `party_booking_config.lub`); the client of 2026 has no word
    /// left for either in its table of commands. `false` for anything else.
    pub(in crate::web::app) fn booking_typed(&mut self, words: &str) -> bool {
        if self.game.is_none() {
            return false;
        }
        match words.trim().to_lowercase().as_str() {
            "/recruit" => self.booking_register_switched(),
            "/booking" => self.booking_list_switched(),
            _ => return false,
        }
        self.redraw(&wnd::BOOKING_WINDOWS);
        true
    }

    /// The list opens as a new window, or closes if it was open (`UIWindowMgr::EnsureWindow`,
    /// `WID_SEEKPARTYLISTWND`). The ads it showed last are in it still: they are the session's.
    fn booking_list_switched(&mut self) {
        let state = &mut self.booking;
        let board = std::mem::take(&mut state.board);
        *state = Booking {
            list: !state.list,
            alarm: board.alarm,
            board,
            register: state.register,
            register_small: state.register_small,
            own_level: std::mem::take(&mut state.own_level),
            own_region: state.own_region,
            own_map: state.own_map,
            own_group: state.own_group,
            ticked: std::mem::take(&mut state.ticked),
            sent: state.sent,
            own: state.own.take(),
            jobs: std::mem::take(&mut state.jobs),
            changing: state.changing.take(),
            help: state.help,
            dropped: state.dropped.filter(|choice| matches!(choice, Choice::OwnRegion | Choice::OwnMap | Choice::OwnGroup)),
            ..Booking::default()
        };
    }

    /// The window of registering opens as a new one, or closes if it was open; it does not open while one's
    /// own ad stands (`EnsureWindow`, `WID_SEEKPARTYMBWND`).
    fn booking_register_switched(&mut self) {
        let state = &mut self.booking;
        if state.own.is_some() {
            return;
        }
        state.register = !state.register;
        (state.register_small, state.own_region, state.own_map, state.own_group) = (false, 0, 0, 0);
        state.own_level.clear();
        state.ticked.clear();
        state.dropped = None;
    }

    /// A line of the list a box dropped was picked (`UM_COMBOBOX_SELECTED`): a region shows its first map, a
    /// group of jobs its first job.
    fn booking_picked(&mut self, place: usize) {
        let state = &mut self.booking;
        match state.dropped.take() {
            Some(Choice::Region) => (state.region, state.map) = (place, 0),
            Some(Choice::Map) => state.map = place,
            Some(Choice::Group) => (state.group, state.job) = (place, 0),
            Some(Choice::Job) => state.job = place,
            Some(Choice::OwnRegion) => (state.own_region, state.own_map) = (place, 0),
            Some(Choice::OwnMap) => state.own_map = place,
            Some(Choice::OwnGroup) => state.own_group = place,
            None => {}
        }
    }

    /// The list's Search, or its row that asks for the next page (`ID_PARTY_SEARCH`,
    /// `SEEKPARTY_SEARCH_NEXT_PAGE`): what is typed and picked is looked at, and the server asked; the title
    /// says what is sought.
    fn booking_search(&mut self, next: bool) {
        let state = &mut self.booking;
        let now = match rules::sought(&state.level, data::booking_highest(), (state.region, state.map), (state.group, state.job)) {
            Ok(now) => now,
            Err(refused) => {
                let (line, number) = refused.said();
                return self.booking_told(line, number);
            }
        };
        let asked = if next { rules::next_page(&mut state.board) } else { rules::search_pressed(&mut state.board, state.sought, now) };
        let Search::Asked(last) = asked else { return };
        let regions = data::booking_regions();
        let place = regions.get(state.region).filter(|_| state.region != 0);
        let place = place.map(|region| (region.name.as_str(), region.maps.get(state.map).map_or("", |(name, _)| name.as_str())));
        let job = (state.group != 0).then(|| rules::job_line(now.job).map(|line| Self::words(line, "")).unwrap_or_default());
        state.title = Some(rules::title(&Self::words(rules::line::SEARCHING, "Searching - "), now.level, place, job.as_deref()));
        (state.sought, state.board.sought) = (now, now);
        self.send_on_map(&booking::search(now.level, now.place, now.job, last, rules::PAGE));
    }

    /// The button of a talk with the leader, and a double click on a row (`ID_WHISPER_PARTY_LEADER`): the row
    /// that asks for the next page asks for it and is gone; an ad of another opens the window of a talk with
    /// that one alone.
    pub(in crate::web::app) fn booking_talk(&mut self) {
        let Some(index) = self.booking.picked else { return };
        match self.booking.board.rows.get(&index).cloned() {
            Some(Row::More) => {
                self.booking_search(true);
                self.booking.board.rows.remove(&index);
            }
            Some(Row::Ad(entry)) => {
                if self.game.as_ref().is_none_or(|game| game.name != entry.name) {
                    self.whisper_to(&entry.name);
                }
            }
            None => {}
        }
    }

    /// The OK of registering (`ID_OK`): what is ticked and typed is looked at, a box says what is wrong with
    /// it, and the ad is sent.
    fn booking_register_asked(&mut self) {
        let state = &mut self.booking;
        let ticked: Vec<i16> = state.ticked.iter().copied().collect();
        match rules::registered(&ticked, &state.own_level, data::booking_highest(), state.own_region, state.own_map) {
            Ok((level, place, jobs)) => {
                state.sent = Some((level, jobs));
                // (The client closes the window of one's ad here if it stands, and then sends nothing.)
                if state.own.take().is_none() {
                    self.send_on_map(&booking::register(level, place, &jobs));
                }
            }
            Err(refused) => {
                let (line, number) = refused.said();
                self.booking_told(line, number);
            }
        }
    }

    /// The OK of the window the jobs are changed in (`UIJobListWnd::SendMsg`): the jobs stand as they are
    /// ticked from now; with none left sought a box says so and the window stays, otherwise they are sent.
    fn booking_jobs_changed(&mut self) {
        let Some(jobs) = self.booking.changing.clone() else { return };
        self.booking.jobs = jobs;
        match rules::changed(&self.booking.jobs) {
            Some(jobs) => {
                self.booking.changing = None;
                self.send_on_map(&booking::update(&jobs));
            }
            None => self.booking_told(rules::line::NONE_LEFT, None),
        }
    }
}
