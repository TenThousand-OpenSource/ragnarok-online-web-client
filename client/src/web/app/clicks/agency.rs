//! The Adventurer's Agency: what its windows do when pressed, and the asking of the game's web service (the
//! client's `UIMenuIconWnd::SendMsg` with `ID_TOGGLE_PARTY_BOARD`, `UIAdvenPartyBoardWnd::SendMsg`,
//! `UIRegisterPartyWnd::SendMsg`, `UIApplyForPartyWnd::SendMsg`, `UIRequestJoinPartyWnd::SendMsg`, and
//! `CAdventurerAgencyMgr`'s requests).

use super::*;
use crate::agency::{Asker, Fields, NOTICE_FIELDS, Notice, Request, SEARCH_FIELDS};
use crate::press::agency::Press;

/// Posts a form to the web service through the gateway and gives what it answered; none when it could not be
/// asked or the gateway could not reach it (its 502), which is the client's request that failed.
async fn posted(path: &'static str, body: String) -> Option<String> {
    let window = web_sys::window()?;
    let headers = web_sys::Headers::new().ok()?;
    headers.set("Content-Type", &format!("multipart/form-data; boundary={}", crate::agency::BOUNDARY)).ok()?;
    let options = web_sys::RequestInit::new();
    options.set_method("POST");
    options.set_headers(&headers);
    options.set_body(&wasm_bindgen::JsValue::from_str(&body));
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str_and_init(path, &options)).await.ok()?.dyn_into().ok()?;
    if response.status() == 502 {
        return None;
    }
    JsFuture::from(response.text().ok()?).await.ok()?.as_string()
}

impl App {
    /// A press of this system (`crate::press::agency`).
    pub(super) fn agency_pressed(&mut self, press: Press) {
        self.agency_fields_kept();
        match press {
            Press::Window if self.agency.open => self.agency.open = false,
            Press::Window => self.agency_opened(),
            Press::Close => self.agency.open = false,
            Press::Role(role) => self.agency.roles[role.min(3)] ^= true,
            Press::Purposes => self.agency.purposes_open ^= true,
            Press::Purpose(purpose) => (self.agency.purpose, self.agency.purposes_open) = (purpose, false),
            Press::Search => self.agency_search(true),
            Press::Reset => self.agency_search_reset(),
            Press::Reload => self.agency_reload(),
            Press::Before | Press::After => {
                self.agency.page += if press == Press::After { 1 } else { -1 };
                if self.agency.searching { self.agency_search(false) } else { self.agency_page_asked() }
            }
            Press::Row(row) => self.agency_row_pressed(row),
            Press::Own => self.agency_writes(self.agency.own.clone()),
            Press::Recruit => self.agency_writes(None),
            // `RequestStop`: only a notice there is can be taken down.
            Press::Stop if self.agency.registered => {
                self.agency.stopping = self.own_account();
                self.agency_asked(Request::Stop, self.agency_asker().stop(self.own_account()));
            }
            Press::Stop => {}
            _ => self.agency_side_pressed(press),
        }
        self.redraw(&wnd::AGENCY_WINDOWS);
    }

    /// A press in one of the windows beside the board.
    fn agency_side_pressed(&mut self, press: Press) {
        match press {
            Press::NoticeRole(role) => self.agency_writing(|writing| writing.roles[role.min(3)] ^= true),
            Press::NoticePurposes => self.agency_writing(|writing| writing.purposes_open ^= true),
            Press::NoticePurpose(purpose) => self.agency_writing(|writing| (writing.purpose, writing.purposes_open) = (purpose, false)),
            Press::NoticeOk => self.agency_notice_sent(),
            Press::NoticeClose => self.agency.writing = None,
            Press::Talk => {
                if let Some(name) = self.agency.reading.as_ref().map(|notice| notice.name.clone()) {
                    self.whisper_to(&name);
                }
            }
            // `RequestInfo`: the notice is read anew before its leader is asked.
            Press::Join => {
                if let Some(account) = self.agency.reading.as_ref().map(|notice| notice.account) {
                    self.agency_asked(Request::Info, self.agency_asker().info(account));
                }
            }
            Press::ReadClose => self.agency.reading = None,
            Press::Block => self.agency.block ^= true,
            Press::Accept | Press::Refuse => self.agency_join_answered(press == Press::Accept),
            _ => {}
        }
    }

    fn agency_writing(&mut self, change: impl FnOnce(&mut NoticeDraft)) {
        if let Some(writing) = &mut self.agency.writing {
            change(writing);
        }
    }

    /// What stands in the fields of the board and of a notice being written is kept, so that a window drawn
    /// anew has it again.
    fn agency_fields_kept(&mut self) {
        if self.agency.open && element(SEARCH_FIELDS[0]).is_some() {
            let [least, most, memo] = SEARCH_FIELDS.map(field);
            (self.agency.least, self.agency.most, self.agency.memo) = (least, most, memo);
        }
        if let Some(writing) = self.agency.writing.as_mut().filter(|_| element(NOTICE_FIELDS[0]).is_some()) {
            let [least, most, memo] = NOTICE_FIELDS.map(field);
            (writing.least, writing.most, writing.memo) = (least, most, memo);
        }
    }

    /// Who asks the web service.
    pub(in crate::web::app) fn agency_asker(&self) -> Asker {
        let account = self.account.as_ref();
        Asker {
            account: self.own_account(),
            character: self.own_character(),
            world: account.and_then(|account| account.char_servers.get(self.server_picked)).map(|server| server.name.clone()).unwrap_or_default(),
            token: account.map(|account| account.token.chars().take(16).collect()).unwrap_or_default(),
        }
    }

    /// Asks the web service; its answer comes back as an action.
    pub(in crate::web::app) fn agency_asked(&self, request: Request, fields: Fields) {
        let body = crate::agency::form(&fields);
        wasm_bindgen_futures::spawn_local(async move {
            let answer = posted(request.path(), body).await;
            dispatch(Action::AgencyAnswered(request, answer));
        });
    }

    /// The levels the windows begin with.
    fn agency_levels(&self) -> (String, String) {
        let (least, most) = crate::agency::levels_around(self.agency_level());
        (least.to_string(), most.to_string())
    }

    /// The board opens (`UIAdvenPartyBoardWnd::OnCreate`): a search of nothing around the player's level, the
    /// first page asked for.
    pub(in crate::web::app) fn agency_opened(&mut self) {
        let (least, most) = self.agency_levels();
        let kept = std::mem::take(&mut self.agency);
        let (registered, asking, waiting, blocked) = (kept.registered, kept.asking, kept.waiting, kept.blocked);
        self.agency = Agency { open: true, least, most, registered, asking, waiting, blocked, ..Agency::default() };
        self.agency_page_asked();
    }

    /// `LoadPage`: the page is asked for, and a leader's own notice with it; the arrows and the button that
    /// asks anew are dim until it comes.
    fn agency_page_asked(&mut self) {
        self.agency.loading = true;
        let asker = self.agency_asker();
        self.agency_asked(Request::List, asker.list(self.agency.page));
        if self.party_led() {
            self.agency_asked(Request::Mine, asker.mine());
        }
    }

    /// The button that asks for the first page anew (`ID_PARTY_RELOAD_FIRST_PAGE`): the search is over.
    pub(in crate::web::app) fn agency_reload(&mut self) {
        (self.agency.page, self.agency.searching) = (1, false);
        self.agency_page_asked();
    }

    /// `ResetSearch`: the search is as the board began with it.
    fn agency_search_reset(&mut self) {
        let (least, most) = self.agency_levels();
        let agency = &mut self.agency;
        (agency.least, agency.most, agency.memo, agency.roles, agency.purpose, agency.searching) = (least, most, String::new(), [false; 4], 0, false);
    }

    /// `Search`: what fits the roles, the levels, the purpose and the note is asked for; from the Search
    /// button its first page, from an arrow the page the arrow went to.
    fn agency_search(&mut self, first: bool) {
        let (least, most) = crate::agency::levels_around(self.agency_level());
        let (least, most) = (crate::agency::level_typed(&self.agency.least, least), crate::agency::level_typed(&self.agency.most, most));
        (self.agency.least, self.agency.most) = (least.to_string(), most.to_string());
        let [tanker, healer, dealer, assist] = self.agency.roles;
        let filter = Notice { tanker, healer, dealer, assist, least, most, purpose: self.agency.purpose, memo: self.agency.memo.clone(), ..Notice::default() };
        (self.agency.searching, self.agency.loading) = (true, true);
        if first {
            self.agency.page = 1;
        }
        self.agency_asked(Request::Search, self.agency_asker().search(&filter, self.agency.page));
    }

    /// A notice of the list is pressed: the player's own is changed, another's is read.
    fn agency_row_pressed(&mut self, row: usize) {
        let Some(notice) = self.agency.entries.get(row).cloned() else { return };
        if notice.account == self.own_account() {
            self.agency_writes(Some(notice));
        } else {
            (self.agency.writing, self.agency.reading) = (None, Some(notice));
        }
    }

    /// The window a notice is written in opens (`UIRegisterPartyWnd::OnCreate`, `SetInfo`): with that notice
    /// in it, or with every role wanted and the levels around the player's own.
    fn agency_writes(&mut self, notice: Option<Notice>) {
        let (least, most) = self.agency_levels();
        let name = self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default();
        let changes = self.agency.registered;
        let fresh = NoticeDraft { name, roles: [true; 4], least, most, changes, ..NoticeDraft::default() };
        self.agency.writing = Some(match notice {
            Some(notice) => NoticeDraft {
                name: notice.name.clone(),
                roles: notice.roles(),
                purpose: notice.purpose,
                least: notice.least.to_string(),
                most: notice.most.to_string(),
                memo: notice.memo,
                ..fresh
            },
            None => fresh,
        });
        self.agency.reading = None;
    }

    /// Register, or Edit (`UIRegisterPartyWnd::SendMsg`): one who leads no party is told so and the window
    /// closes; a note with a word that is not let be written, or levels the wrong way round, are told and the
    /// window stays; else the notice is posted and the window closes.
    fn agency_notice_sent(&mut self) {
        let Some(writing) = self.agency.writing.clone() else { return };
        if !self.party_led() {
            self.chat_line(Self::words(3506, "If you are not a party leader, you can not register party at the Adventurer's Agency."), RED);
            self.agency.writing = None;
            return;
        }
        if data::foul(&writing.memo) {
            return self.chat_line(Self::words(3372, "Contains words that cannot be used."), RED);
        }
        let (least, most) = crate::agency::levels_around(self.agency_level());
        let (least, most) = (crate::agency::level_typed(&writing.least, least), crate::agency::level_typed(&writing.most, most));
        if least > most {
            self.agency_writing(|writing| (writing.least, writing.most) = (least.to_string(), most.to_string()));
            return self.chat_line(Self::words(3516, ""), RED);
        }
        let [tanker, healer, dealer, assist] = writing.roles;
        let notice = Notice { tanker, healer, dealer, assist, least, most, purpose: writing.purpose, memo: writing.memo, ..Notice::default() };
        let name = self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default();
        self.agency_asked(Request::Add, self.agency_asker().add(&notice, &name));
        self.agency.writing = None;
    }

    /// The leader's answer to one who asks to be let in (`UIRequestJoinPartyWnd::SendMsg`): a refusal with
    /// the box ticked blocks that player's requests from then on; then the next who waits is asked about
    /// (`UIRequestJoinPartyMgr::Advance`).
    fn agency_join_answered(&mut self, accept: bool) {
        let Some(asking) = self.agency.asking.take() else { return };
        if !accept && self.agency.block {
            self.agency.blocked.push(asking.account);
        }
        self.send_on_map(&crate::net::agency::answer_join(asking.character, asking.account, accept));
        self.agency.block = false;
        while let Some(next) = self.agency.waiting.pop_front() {
            if !self.agency.blocked.contains(&next.account) {
                self.agency.asking = Some(next);
                break;
            }
        }
    }
}
