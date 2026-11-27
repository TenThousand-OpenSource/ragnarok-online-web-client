//! The old party booking on the page: the list of ads (the client's `UISeekPartyListWnd`, 550 by 350), its
//! help (`UIPartyBookingHelpWnd`, 320 by 199), the window an ad is registered in (`UISeekPartyMBWnd`, 360 by
//! 350), the window of one's own ad (`UISeekPartyWnd`, 152 by 199) and the one its jobs are changed in
//! (`UIJobListWnd`, 152 by 199). The places and sizes are those of each window's `OnCreate` and `Paint`.

use super::*;
use crate::booking::{self as rules, line, Row, LEVEL, OWN_LEVEL};
use crate::press::booking::{Choice, Press};

/// The ground of a field and of a box, and their rim (`Background(0xE8, 0xE8, 0xE8)`, `Border(0x64, 0x64,
/// 0x64)`).
const FIELD: &str = "box-sizing:border-box;background:#e8e8e8;border:1px solid #646464";
/// The rim of a row of the list (the client's `0xCECECE`) and the ground of the row picked (`0xEF9C73`, which
/// is blue first).
const ROW_RIM: &str = "#cecece";
const PICKED: &str = "#739cef";
/// How high the lists of the boxes drop (`dropdownHeight`).
const DROP: i32 = 80;

/// What every window here has over its ground: the title bar with its words, the button that makes the window
/// small or large again 28 from the right (`AddMinimizeButton`: `sys_mini_off.bmp`), and the cross.
fn head(width: i32, title: &str, size: Option<Press>, close: Press) -> String {
    let size = size.map(|size| press_button(width - 28, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", size));
    [title_bar(width, title), size.unwrap_or_default(), cross(width, close)].concat()
}

/// The three words down the left of the list and of registering, as the client writes them.
fn labels() -> String {
    [text(10, 23, BLACK, "LEVEL"), text(10, 45, BLACK, "MAP"), text(10, 70, BLACK, "JOB")].concat()
}

/// The names of a job's group, by its place among the eight.
fn group_name(group: usize) -> String {
    App::words(rules::GROUP_LINE + group, "")
}

/// What a job of an ad is called.
fn job_name(job: i16) -> String {
    rules::job_line(job).map(|line| App::words(line, "")).unwrap_or_default()
}

/// Lines of a box's list, all in black.
fn plain(names: impl IntoIterator<Item = String>) -> Vec<(String, String)> {
    names.into_iter().map(|name| (name, BLACK.to_owned())).collect()
}

/// The maps of a region as the lines of a box's list, each in its colour.
fn map_lines(region: Option<&rules::Region>) -> Vec<(String, String)> {
    let coloured = |(name, [red, green, blue]): &(String, [u8; 3])| (name.clone(), format!("#{red:02x}{green:02x}{blue:02x}"));
    region.map(|region| region.maps.iter().map(coloured).collect()).unwrap_or_default()
}

impl App {
    /// The four boxes over the list, each with what is picked in it and the lines it drops
    /// (`UISeekPartyListWnd::InitRegionFilter`, `InitJobFilter`): the regions with "all maps" first and the
    /// maps of the one picked; the groups of jobs with "all" first and the jobs of the one picked.
    fn booking_list_boxes(&self) -> [(Choice, (i32, i32, i32), Vec<(String, String)>, usize); 4] {
        let state = &self.booking;
        let regions = data::booking_regions();
        let any = Self::words(line::ANY, "Select All");
        let groups = std::iter::once(any.clone()).chain((0..rules::GROUPS.len()).map(group_name));
        let jobs = match state.group.checked_sub(1) {
            Some(group) => plain(rules::jobs_of(group).map(job_name)),
            None => plain([any]),
        };
        [
            (Choice::Region, (50, 43, 150), plain(regions.iter().map(|region| region.name.clone())), state.region),
            (Choice::Map, (220, 43, 200), map_lines(regions.get(state.region)), state.map),
            (Choice::Group, (50, 68, 150), plain(groups), state.group),
            (Choice::Job, (220, 68, 200), jobs, state.job),
        ]
    }

    /// The three boxes of registering (`UISeekPartyMBWnd::LoadRegionMaps`, `InitJobCategoryCombo`): the
    /// regions without "all maps", the maps of the one picked, the groups of jobs.
    fn booking_register_boxes(&self) -> [(Choice, (i32, i32, i32), Vec<(String, String)>, usize); 3] {
        let state = &self.booking;
        let regions = data::booking_regions();
        [
            (Choice::OwnRegion, (50, 43, 100), plain(regions.iter().skip(1).map(|region| region.name.clone())), state.own_region),
            (Choice::OwnMap, (155, 43, 190), map_lines(regions.get(state.own_region + 1)), state.own_map),
            (Choice::OwnGroup, (50, 68, 150), plain((0..rules::GROUPS.len()).map(group_name)), state.own_group),
        ]
    }

    /// Boxes drawn: each with what is picked in it, and last the list of the one that is dropped, over the
    /// others.
    fn booking_boxes(&self, boxes: &[(Choice, (i32, i32, i32), Vec<(String, String)>, usize)]) -> String {
        let mut drawn: String = boxes
            .iter()
            .map(|(choice, place, lines, picked)| combo(*place, lines.get(*picked).map_or("", |(words, _)| words.as_str()), Press::Opens(*choice)))
            .collect();
        if let Some((_, place, lines, picked)) = boxes.iter().find(|(choice, ..)| self.booking.dropped == Some(*choice)) {
            drawn.push_str(&combo_dropped(*place, DROP, lines, *picked, Press::Pick));
        }
        drawn
    }

    /// The list (`UISeekPartyListWnd::OnCreate` and `Paint`): its title (line 1689, or what was last sought);
    /// the level sought typed at (50, 20), the boxes of the place and of the job, the box "Notice" at (430,
    /// 43) and Search at (480, 37); the heads of the columns 104 down; five rows of 40 from 124 with the bar
    /// at (515, 120); and on the bar at the bottom "Recruit" at 10, the button of a talk with the leader at 70
    /// with its words at 100, Help 110 from the right and OK 45 from it.
    pub(in crate::web) fn booking_list(&self) -> String {
        let state = &self.booking;
        let title = state.title.clone().unwrap_or_else(|| Self::words(line::LIST, "Party Booking List"));
        let mut window = head(550, &title, Some(Press::Size), Press::Close);
        if state.small {
            return window;
        }
        window.push_str(&fill(0, 17, 550, 312, WHITE));
        window.push_str(&labels());
        window.push_str(&numeric_field(LEVEL, (50, 20, 40, 16), &state.level, 5, FIELD));
        window.push_str(&check_words(430, 43, state.alarm, &Self::words(line::ALARM, "Notice"), Press::Alarm));
        window.push_str(&picture_press(480, 37, "btn_find", Press::Search));
        for (x, head) in [(33, line::TIME), (106, line::NAME), (220, line::LEVEL), (365, line::MAP)] {
            window.push_str(&text(x, 104, BLACK, &Self::words(head, "")));
        }
        window.push_str(&self.booking_rows());
        window.push_str(&button_bar(550, 350));
        window.push_str(&format!(
            r#"<div class="a t hov" style="left:10px;top:328px;color:{BLACK};--over:#00b7ef;cursor:default" {} data-tip="{}">{}</div>"#,
            pressed(Press::Recruit),
            escape(&Self::words(line::RECRUIT_TIP, "")),
            escape(&Self::words(line::RECRUIT, "Recruit"))
        ));
        window.push_str(&pictured(70, 326, (30, 20), ["basic_interface/direct_chat.bmp"; 3], Press::Talk, ""));
        window.push_str(&text(100, 330, BLACK, &Self::words(line::TALK, "Chat Dialog")));
        window.push_str(&picture_press_sized(440, 326, (54, 20), "battle_field/btn_help", Press::Help));
        window.push_str(&picture_press(505, 326, "btn_ok", Press::Close));
        window.push_str(&self.booking_boxes(&self.booking_list_boxes()));
        window
    }

    /// The rows of the list (`UISeekPartyListWnd::Paint`): each 510 by 40 from 20 across, a grey rim and
    /// white inside, the one picked blue all over; the time 2 in, the name at 56 (130 wide at most), the
    /// levels at 195 and the place at 260, all 3 down, and the jobs sought 23 down. The row that asks for the
    /// next page shows `more_info.bmp`. A press picks a row, a double click opens a talk with its leader.
    fn booking_rows(&self) -> String {
        let state = &self.booking;
        let highest = data::booking_highest();
        let last = state.board.rows.len().saturating_sub(1);
        let rows: String = state
            .board
            .rows
            .iter()
            .enumerate()
            .map(|(place, (index, row))| {
                let picked = state.picked == Some(*index);
                let under = if place == last { 1 } else { 0 };
                let look = if picked { format!("background:{PICKED};border:1px solid {PICKED}") } else { format!(
                    "background:{WHITE};border:1px solid {ROW_RIM};border-bottom-width:{under}px"
                ) };
                let inside = match row {
                    Row::More => art("basic_interface/more_info.bmp", 1, 1, 200, 30),
                    Row::Ad(entry) => {
                        let jobs: Vec<String> = entry.jobs.iter().filter(|job| **job >= 0).map(|job| job_name(*job)).collect();
                        let time = js_sys::Date::new(&(f64::from(entry.time) * 1000.0).into());
                        [
                            text(1, 2, BLACK, &rules::clock(time.get_hours(), time.get_minutes())),
                            format!(r#"<div class="a t" style="left:55px;top:2px;width:130px;overflow:hidden">{}</div>"#, escape(&entry.name)),
                            text(194, 2, BLACK, &rules::levels(entry.level, highest)),
                            text(259, 2, BLACK, &entry.location),
                            text(1, 22, BLACK, &jobs.join(",")),
                        ]
                        .concat()
                    }
                };
                format!(
                    r#"<div style="position:relative;height:40px;box-sizing:border-box;overflow:hidden;cursor:default;{look}" {} {}>{inside}</div>"#,
                    pressed(Press::Row(*index)),
                    used(Use::Ad(*index))
                )
            })
            .collect();
        [
            format!(r#"<div class="a scroll" style="{}" {}>{rows}</div>"#, at(20, 124, 510, 200), rows_stepped(40, rules::ROWS)),
            scroll_bar(515, 120, 200, 40),
        ]
        .concat()
    }

    /// The help (`UIPartyBookingHelpWnd::OnCreate` and `Paint`): the title (line 1716) and the lines the
    /// client's table has for it, in a box 300 by 164 at (10, 25).
    pub(in crate::web) fn booking_help(&self) -> String {
        let lines: String = data::booking_help()
            .lines()
            .map(|line| format!(r#"<div style="min-height:18px;white-space:normal">{}</div>"#, escape(line)))
            .collect();
        [
            title_bar(320, &Self::words(line::HELP, "Party recruitment related command")),
            base_button(3, 3),
            cross(320, Press::HelpClose),
            panel(320, 199),
            format!(r#"<div class="a t bar" style="{};line-height:18px"><div style="padding:0 0 0 3px">{lines}</div></div>"#, at(10, 25, 300, 164)),
        ]
        .concat()
    }

    /// The window an ad is registered in (`UISeekPartyMBWnd::OnCreate`, `CreateJobChecks` and `Paint`): the
    /// title (line 1688); the lowest level typed at (50, 20), the boxes of the place, the box of the jobs'
    /// groups; in a box with a black rim from (50, 95) to (310, 293) the jobs of the group shown, each to be
    /// ticked, eight down from (54, 99) 25 apart and the rest from (180, 99); OK and Cancel on the bar.
    pub(in crate::web) fn booking_register(&self) -> String {
        let state = &self.booking;
        let mut window = head(360, &Self::words(line::REGISTER, "Party Recruitment"), Some(Press::RegisterSize), Press::RegisterClose);
        if state.register_small {
            return window;
        }
        window.push_str(&fill(0, 17, 360, 312, WHITE));
        window.push_str(&labels());
        let level = numeric_field(OWN_LEVEL, (50, 20, 40, 16), &state.own_level, 5, FIELD);
        window.push_str(&level.replacen("<input ", &format!(r#"<input data-tip="{}" "#, escape(&Self::words(line::LEVEL_TIP, ""))), 1));
        window.push_str(&fill(50, 95, 260, 198, BLACK));
        window.push_str(&fill(51, 96, 258, 196, "#f0f0f0"));
        for (place, job) in rules::jobs_of(state.own_group).enumerate() {
            let place = place as i32;
            let (x, y) = if place < 8 { (54, 99 + place * 25) } else { (180, 99 + (place - 8) * 25) };
            window.push_str(&check_words(x, y, state.ticked.contains(&job), &job_name(job), Press::Tick(job)));
        }
        window.push_str(&button_bar(360, 350));
        window.push_str(&picture_press(267, 326, "btn_ok", Press::Register));
        window.push_str(&picture_press(314, 326, "btn_cancel", Press::RegisterClose));
        window.push_str(&self.booking_boxes(&self.booking_register_boxes()));
        window
    }

    /// The window of one's own ad (`UISeekPartyWnd::OnCreate` and `Paint`): the title with the levels sought;
    /// a picture that moves, 150 square at (1, 18); and on the bar how long the ad has stood, 10 in, the
    /// button that changes its jobs 92 from the right and Cancel 46 from it. Made small it is 44 high: the
    /// title and the bar.
    pub(in crate::web) fn booking_own(&self) -> String {
        let Some(own) = &self.booking.own else { return String::new() };
        let height = if own.small { 44 } else { 199 };
        let lasted = crate::web::app::now().wrapping_sub(own.since);
        let title = rules::own_title(&Self::words(line::OWN, "Recruit party members"), own.level, data::booking_highest());
        let mut window = head(152, &title, Some(Press::OwnSize), Press::Withdraw);
        window.push_str(&fill(0, 17, 152, height - 38, WHITE));
        if !own.small {
            window.push_str(&art(&rules::frame_file(own.kind, rules::frame(own.kind, lasted)), 1, 18, 150, 150));
        }
        window.push_str(&button_bar(152, height));
        window.push_str(&text(10, height - 20, BLACK, &rules::lasted(lasted)));
        window.push_str(&picture_press(60, height - 24, "btn_rewrite", Press::Jobs));
        window.push_str(&picture_press(106, height - 24, "btn_cancel", Press::Withdraw));
        window
    }

    /// The window the jobs of one's ad are changed in (`UIJobListWnd::OnCreate`, `RefreshJobs` and `Paint`):
    /// "Job List"; the jobs the ad was registered with, 25 apart from (10, 25), each with the mark of one
    /// sought (`grp_online.bmp`) or not (`grp_offline.bmp`), its name 3 after the mark; OK 92 from the right.
    pub(in crate::web) fn booking_jobs(&self) -> String {
        let jobs: String = self
            .booking
            .changing
            .iter()
            .flatten()
            .enumerate()
            .map(|(place, (job, sought))| {
                let mark = picture(if *sought { "basic_interface/grp_online.bmp" } else { "basic_interface/grp_offline.bmp" });
                [
                    format!(r#"<div class="a t" style="left:10px;top:{}px;width:120px;height:15px;padding-left:18px;"#, (place as i32 + 1) * 25),
                    format!(
                        r#"box-sizing:border-box;cursor:default;color:{BLACK};background:url('{mark}') 0 0 no-repeat" {}>{}</div>"#,
                        pressed(Press::JobTick(place)),
                        escape(&job_name(*job))
                    ),
                ]
                .concat()
            })
            .collect();
        [
            head(152, "Job List", None, Press::JobsClose),
            fill(0, 17, 152, 161, WHITE),
            jobs,
            button_bar(152, 199),
            picture_press(60, 175, "btn_ok", Press::JobsOk),
        ]
        .concat()
    }
}
