//! The Adventurer's Agency on the page: the board of notices (the client's `UIAdvenPartyBoardWnd`, 480 by
//! 340), a row of it (`UIPartyInfoWnd`), the window a notice is written in (`UIRegisterPartyWnd`) and the one
//! it is read in (`UIApplyForPartyWnd`), both on `bg_agency_register.bmp`, 220 by 190, and the question to a
//! leader whether to let someone in (`UIRequestJoinPartyWnd`, 220 by 160).

use super::*;
use crate::agency::{NOTICE_FIELDS, Notice, PURPOSES, ROLES, ROLE_PICTURES, ROWS, SEARCH_FIELDS};
use crate::press::Press as AnyPress;
use crate::press::agency::Press;

/// Where the agency's pictures are.
const ART: &str = "UIadventurerAgency/";

/// How wide words are in the usual text, in pixels.
fn wide(words: &str) -> i32 {
    crate::web::text_widths()(words) as i32
}

/// A strip that is a picture in three states with something on it (the client's `UIBmpTextButton`, and a row
/// of the board): pressed, or with no press the picture of one that cannot be.
fn strip((x, y, width, height): (i32, i32, i32, i32), pictures: [&str; 4], press: Option<AnyPress>, more: &str, inside: &str) -> String {
    let url = |name: &str| picture(&format!("{ART}{name}.bmp"));
    match press {
        Some(press) => format!(
            r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}');padding:0;text-align:left" {}{more}>{inside}</button>"#,
            at(x, y, width, height),
            url(pictures[0]),
            url(pictures[1]),
            url(pictures[2]),
            pressed(press)
        ),
        None => format!(r#"<div class="a" style="{};background:url('{}') no-repeat"{more}>{inside}</div>"#, at(x, y, width, height), url(pictures[3])),
    }
}

/// Words in the middle of a strip so wide, as `UIBmpTextButton::Paint` writes them: black over a grey copy.
fn middle_words(width: i32, height: i32, words: &str) -> String {
    format!(
        r#"<span class="t" style="display:block;width:{width}px;line-height:{height}px;text-align:center;color:#000;text-shadow:1px 1px 0 #d7d7d7">{}</span>"#,
        escape(words)
    )
}

/// The four pictures of the roles from that place, 18 apart: lit for a role wanted. With `press` each is
/// pressed to switch it, and tells under the pointer which role it is.
fn roles((x, y): (i32, i32), wanted: [bool; 4], press: Option<fn(usize) -> Press>) -> String {
    (0..4)
        .map(|role| {
            let name = format!("{ART}{}", ROLE_PICTURES[role][usize::from(!wanted[role])]);
            let tip = escape(&App::words(ROLES[role], ""));
            match press {
                Some(press) => format!(
                    r#"<button type="button" class="a b" style="{};--n:url('{url}');--o:url('{url}');--p:url('{url}')" {} data-tip="{tip}"></button>"#,
                    at(x + 18 * role as i32, y, 16, 16),
                    pressed(press(role)),
                    url = picture(&name)
                ),
                None => format!(
                    r#"<div class="a" style="{};background:url('{}')" data-tip="{tip}"></div>"#,
                    at(x + 18 * role as i32, y, 16, 16),
                    picture(&name)
                ),
            }
        })
        .collect()
}

/// The client's box of purposes (`UIComboBoxDef2Wnd`): the purpose picked 2 in and 3 down, the button that
/// opens its list at its right (`btn_dropdown_*`, 18 square), and the list under it while it is open, a row
/// to a purpose, 18 high.
fn purposes((x, y, width): (i32, i32, i32), picked: i32, open: bool, opens: Press, picks: fn(i32) -> Press) -> String {
    let name = |purpose: i32| App::words(PURPOSES[purpose.clamp(0, 6) as usize], "");
    let mut combo = text(x + 2, y + 3, BLACK, &name(picked));
    combo.push_str(&pictured_by(Convention::OutOverPress, x + width - 18, y, (18, 18), &format!("{ART}btn_dropdown"), opens, ""));
    if open {
        let rows: String = (0..7).map(|purpose| format!(r#"<div style="height:18px">{}</div>"#, menu_row(&name(purpose), picks(purpose)))).collect();
        combo.push_str(&format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #9a9a9a;box-sizing:border-box;z-index:2">{rows}</div>"#,
            at(x, y + 18, width, 128)
        ));
    }
    combo
}

/// The lines of a notice as a row of the board writes them (`UIPartyInfoWnd::Paint`): the levels at 11, the
/// purpose at 64, the roles from 109, the leader's name at 192 in 119, the note at 315.
fn row_words(notice: &Notice) -> String {
    [
        text(11, 5, BLACK, &notice.levels()),
        text(64, 5, BLACK, &App::words(PURPOSES[notice.purpose.clamp(0, 6) as usize], "")),
        roles((109, 3), notice.roles(), None),
        fitted(192, 5, 119, BLACK, &notice.name),
        fitted(315, 5, 160, BLACK, &notice.memo),
    ]
    .concat()
}

/// The note in red under a notice written or read (the client's line 3370, cut into lines of 40 letters, 12
/// apart from 136 down, in its smaller text).
fn rules_note() -> String {
    crate::broadcast::wrapped(&App::words(3370, "* Please respect the rules of the server."), 40)
        .iter()
        .enumerate()
        .map(|(row, line)| text_in(SMALL, 9, 136 + 12 * row as i32, RED, line))
        .collect()
}

impl App {
    /// The character's level, which the agency's windows begin their levels around.
    pub(in crate::web) fn agency_level(&self) -> i32 {
        self.value(parameter::BASE_LEVEL) as i32
    }

    /// The board (`UIAdvenPartyBoardWnd::OnCreate`, `Paint`, `RefreshRows`, `UpdatePagination`,
    /// `Registration`): its picture `bg_agencyList.bmp` with the title (line 3504) and the bar of buttons;
    /// the search along the top (two levels at 10 and 46 with "~" between, the purposes at 76, the roles from
    /// 160, the note at 238, Search at 354, Reset at 401, the button that asks anew at 452); at (1, 49) the
    /// player's own notice, or the strip that puts one up; ten rows from (1, 73), 24 apart; the page between
    /// its arrows at 319 down; Close 10 from the right at 317 and "Cancel Recruitment" 5 before it.
    pub(in crate::web) fn agency_board(&self) -> String {
        let agency = &self.agency;
        let mut board = art(&format!("{ART}bg_agencyList.bmp"), 0, 0, 480, 340);
        board.push_str(&title_bar(480, &Self::words(3504, "Adventurer's Agency")));
        board.push_str(&cross(480, Press::Close));
        board.push_str(&button_bar(480, 340));
        board.push_str(&numeric_field(SEARCH_FIELDS[0], (10, 24, 25, 14), &agency.least, 3, "background:#f2f2f2;padding:0 1px"));
        board.push_str(&text(36, 25, BLACK, "~"));
        board.push_str(&numeric_field(SEARCH_FIELDS[1], (46, 24, 25, 14), &agency.most, 3, "background:#f2f2f2;padding:0 1px"));
        board.push_str(&roles((160, 23), agency.roles, Some(Press::Role)));
        board.push_str(&format!(
            r#"<input id="{}" class="a in" style="{};background:#fff" type="text" maxlength="150" autocomplete="off" placeholder="{}" value="{}">"#,
            SEARCH_FIELDS[2],
            at(238, 24, 116, 14),
            escape(&Self::words(3419, "Enter search details")),
            escape(&agency.memo)
        ));
        board.push_str(&strip(
            (354, 22, 40, 18),
            ["btn_search", "btn_search_over", "btn_search_press", "btn_search_disable"],
            Some(Press::Search.into()),
            "",
            &middle_words(40, 18, &Self::words(3366, "Search"))
        ));
        board.push_str(&strip(
            (401, 22, 44, 18),
            ["btn_initialize", "btn_initialize_over", "btn_initialize_press", "btn_initialize_disable"],
            Some(Press::Reset.into()),
            "",
            &middle_words(44, 18, &Self::words(3367, "Reset"))
        ));
        let reload = ["btn_refresh", "btn_refresh_over", "btn_refresh_press", "btn_refresh_disable"];
        board.push_str(&strip((452, 22, 18, 18), reload, (!agency.loading).then(|| Press::Reload.into()), "", ""));
        board.push_str(&self.agency_own_strip());
        for row in 0..ROWS {
            let place = (1, 73 + 24 * row as i32, 478, 22);
            if let Some(notice) = agency.entries.get(row) {
                let pictures = ["btn_list", "btn_list_over", "btn_list_press", "btn_list_disable"];
                board.push_str(&strip(place, pictures, Some(Press::Row(row).into()), "", &row_words(notice)));
            }
        }
        board.push_str(&self.agency_pages());
        board.push_str(&basic_button(428, 317, 42, &Self::words(3045, "Close"), Some(Press::Close.into())));
        if agency.settled && agency.registered && self.party_led() {
            let stop = Self::words(3375, "Cancel Recruitment");
            let width = wide(&stop) + 10;
            board.push_str(&basic_button(428 - width - 5, 317, width, &stop, Some(Press::Stop.into())));
        }
        // (Last, so that its list lies over what is under it.)
        board.push_str(&purposes((76, 22, 76), agency.purpose, agency.purposes_open, Press::Purposes, Press::Purpose));
        board
    }

    /// What stands at (1, 49) once a list has come (`Registration`): a leader's own notice, pressed to change
    /// it; or the strip that puts a notice up, which one who leads no party cannot press and is told why
    /// under the pointer (line 2047).
    fn agency_own_strip(&self) -> String {
        let agency = &self.agency;
        let place = (1, 49, 478, 22);
        if !agency.settled {
            return String::new();
        }
        let recruit = ["btn_recruit_normal", "btn_recruit_over", "btn_recruit_press", "btn_recruit_disable"];
        if !self.party_led() {
            let tip = format!(r#" data-tip="{}""#, escape(&Self::words(2047, "Only party leader can register party ad.")));
            return strip(place, recruit, None, &tip, "");
        }
        match (&agency.own, agency.registered) {
            (Some(own), true) => {
                let pictures = ["btn_recruit_on", "btn_recruit_on", "btn_recruiting_press", "btn_recruiting_disable"];
                strip(place, pictures, Some(Press::Own.into()), "", &row_words(own))
            }
            (None, true) => String::new(),
            (_, false) => strip(place, recruit, Some(Press::Recruit.into()), "", ""),
        }
    }

    /// The page and how many there are, about the middle of the board, between its arrows, each 3 from the
    /// words (`RefreshRows`); an arrow with no page to go to, or while a page is awaited, is the dim one.
    fn agency_pages(&self) -> String {
        let agency = &self.agency;
        let words = format!("{} / {}", agency.page, agency.pages);
        let half = wide(&words) / 2;
        let (before, after) = crate::agency::arrows(agency.page, agency.pages);
        let arrow = |x: i32, name: &'static str, dim: &'static str, press: Press, live: bool| {
            strip((x, 319, 16, 16), [name, name, name, dim], (live && !agency.loading).then(|| press.into()), "", "")
        };
        [
            text(240 - half, 319, BLACK, &words),
            arrow(240 - 16 - half - 3, "btn_list_left", "btn_list_left_disable", Press::Before, before),
            arrow(240 + half + 3, "btn_list_right", "btn_list_right_disable", Press::After, after),
        ]
        .concat()
    }

    /// The window a notice is written in (`UIRegisterPartyWnd::OnCreate`, `Paint`): the title (line 3356);
    /// the leader (3415) with the name at 60; the levels (line 408) in two fields at 60 and 96 with "~"
    /// between; the purpose (3369) in its box at (60, 70); the roles (line 504) from (60, 95); the note
    /// (3359) in a field of 24 letters at (60, 119); the red note; Register or Edit (3371, 3376) at (108, 165)
    /// and Close at (173, 165).
    pub(in crate::web) fn agency_notice(&self) -> String {
        let Some(writing) = &self.agency.writing else { return String::new() };
        let ok = Self::words(if writing.changes { 3376 } else { 3371 }, if writing.changes { "Edit" } else { "Register" });
        let grey = "background:#e8e8e8;padding:0 1px";
        [
            art(&format!("{ART}bg_agency_register.bmp"), 0, 0, 220, 190),
            title_bar(220, &Self::words(3356, "Register in the Adventurer's Agency")),
            cross(220, Press::NoticeClose),
            text(10, 23, BLACK, &Self::words(3415, "Leader")),
            fitted(60, 23, 119, BLACK, &writing.name),
            text(10, 49, BLACK, &Self::words(408, "Level")),
            numeric_field(NOTICE_FIELDS[0], (60, 47, 25, 16), &writing.least, 3, grey),
            text(86, 50, BLACK, "~"),
            numeric_field(NOTICE_FIELDS[1], (96, 47, 25, 16), &writing.most, 3, grey),
            text(10, 73, BLACK, &Self::words(3369, "Purpose")),
            text(10, 97, BLACK, &Self::words(504, "Job")),
            roles((60, 95), writing.roles, Some(Press::NoticeRole)),
            text(10, 121, BLACK, &Self::words(3359, "Note")),
            field(NOTICE_FIELDS[2], (60, 119, 139, 14), &writing.memo, 24, "background:#e8e8e8"),
            rules_note(),
            basic_button(108, 165, wide(&ok) + 10, &ok, Some(Press::NoticeOk.into())),
            picture_press(173, 165, "basic_interface/btn_close", Press::NoticeClose),
            purposes((60, 70, 63), writing.purpose, writing.purposes_open, Press::NoticePurposes, Press::NoticePurpose),
        ]
        .concat()
    }

    /// The window a notice is read in (`UIApplyForPartyWnd::OnCreate`, `Paint`): the title (line 3368); the
    /// leader, the levels, the purpose, the roles and the note, each from 60 beside its word; the red note;
    /// and the buttons for a talk with the leader (3374) at 51, for asking to join (3373) at 108 and Close at
    /// 173, all 165 down.
    pub(in crate::web) fn agency_read(&self) -> String {
        let Some(notice) = &self.agency.reading else { return String::new() };
        let button = |x: i32, line: usize, otherwise: &str, press: Press| {
            let words = Self::words(line, otherwise);
            basic_button(x, 165, wide(&words) + 10, &words, Some(press.into()))
        };
        [
            art(&format!("{ART}bg_agency_register.bmp"), 0, 0, 220, 190),
            title_bar(220, &Self::words(3368, "Registration for Party")),
            cross(220, Press::ReadClose),
            text(10, 25, BLACK, &Self::words(3415, "Leader")),
            fitted(60, 25, 119, BLACK, &notice.name),
            text(10, 49, BLACK, &Self::words(408, "Level")),
            text(60, 49, BLACK, &notice.levels()),
            text(10, 73, BLACK, &Self::words(3369, "Purpose")),
            text(60, 73, BLACK, &Self::words(PURPOSES[notice.purpose.clamp(0, 6) as usize], "")),
            text(10, 97, BLACK, &Self::words(504, "Job")),
            roles((60, 95), notice.roles(), None),
            text(10, 121, BLACK, &Self::words(3359, "Note")),
            fitted(60, 121, 155, BLACK, &notice.memo),
            rules_note(),
            button(51, 3374, "PM", Press::Talk),
            button(108, 3373, "Join", Press::Join),
            button(173, 3045, "Close", Press::ReadClose),
        ]
        .concat()
    }

    /// The question to a leader (`UIRequestJoinPartyWnd::OnCreate`, `UM_SET_JOIN_PARTY_MSG`): on
    /// `bg_agency_admit.bmp` with the title (line 3505), the picture of the job of the one who asks at
    /// (12, 29), 25 square, with the level under its middle at 56; who asks (line 3378) in a box 167 wide at
    /// (43, 30); the box that blocks this player's requests at (7, 81), 24 by 47, with what it does (3518)
    /// beside it; Accept (3489) at (122, 136) and Refuse (3490) at (169, 136).
    pub(in crate::web) fn agency_question(&self) -> String {
        let Some(asking) = &self.agency.asking else { return String::new() };
        let asks = crate::message_box::lines(&Self::words(3378, "[%s] has requested to join the party.").replacen("%s", &asking.name, 1), 22);
        let level = asking.level.to_string();
        let blocks = crate::message_box::lines(&Self::words(3518, "Block party request from this player."), 30);
        let lines = |lines: &[String], height: i32| -> String {
            lines.iter().map(|line| format!(r#"<div style="height:{height}px;line-height:{height}px;white-space:pre">{}</div>"#, escape(line))).collect()
        };
        let block = format!("{ART}btn_block_{}.bmp", if self.agency.block { "on" } else { "off" });
        let button = |x: i32, line: usize, otherwise: &str, press: Press| {
            let words = Self::words(line, otherwise);
            basic_button(x, 136, wide(&words) + 10, &words, Some(press.into()))
        };
        [
            art(&format!("{ART}bg_agency_admit.bmp"), 0, 0, 220, 160),
            title_bar(220, &Self::words(3505, "Request to join the party")),
            cross(220, Press::Refuse),
            format!(
                r#"<div class="a" style="{};background:url('{}') no-repeat" data-tip="{}"></div>"#,
                at(12, 29, 25, 25),
                picture(&format!("renewalparty/icon_jobs_{}.bmp", asking.job)),
                escape(&data::job_name(asking.job))
            ),
            text(25 - wide(&level) / 2, 56, BLACK, &level),
            format!(r#"<div class="a t" style="{}">{}</div>"#, at(43, if asks.len() == 3 { 20 } else { 30 }, 167, 60), lines(&asks, 18)),
            pictured(7, 81, (24, 47), [block.as_str(); 3], Press::Block, ""),
            format!(
                r#"<div class="a t" style="{};background:rgb(188,198,215)"><div style="padding-left:3px">{}</div></div>"#,
                at(31, 81, 182, 47),
                lines(&blocks, 15)
            ),
            button(122, 3489, "Accept", Press::Accept),
            button(169, 3490, "Refuse", Press::Refuse),
        ]
        .concat()
    }
}
