//! The Basic Information window with its gauges, and the strip of menu icons under it.

use super::*;
use crate::web::app::Game;

/// What is left of a gauge of Basic Information once its back is drawn (`gze_bg1.bmp`, 135 by 9): the fill up
/// to `now` of `most` and the two numbers over it.
fn gauge_over(back: String, x: i32, y: i32, now: i64, most: i64, red: Red) -> String {
    format!("{back}{}{}", filling(x, y, 127, now, most, red), numbers_over(x, y, 135, now, most))
}

/// A gauge of the Basic Information window: its back, the fill up to `now` of `most` (red when under a
/// quarter), and the two numbers over it.
fn gauge(x: i32, y: i32, now: i64, most: i64) -> String {
    let back = art("basic_interface/gze_bg1.bmp", x, y, 135, 9);
    // Values that make no sense (more than the most) are not drawn at all.
    if most <= 0 || now > most {
        return back;
    }
    gauge_over(back, x, y, now, most, Red::UnderQuarter)
}

/// The gauge of Activity Points (the client's `UIBarGraphPlayerAP`): the same frame, blue, and red once it
/// is full; with none, nothing in it.
fn ap_gauge(x: i32, y: i32, now: i64, most: i64) -> String {
    let back = art("basic_interface/gze_bg1.bmp", x, y, 135, 9);
    if most <= 0 || now > most || now <= 0 {
        return back;
    }
    gauge_over(back, x, y, now, most, Red::Full)
}

/// An experience bar: how much of the way to the next level is done.
fn experience(x: i32, y: i32, now: i64, next: i64) -> String {
    if next <= 0 {
        return String::new();
    }
    let fill = (110 * now.clamp(0, next) / next) as i32;
    format!(
        r#"<div class="a" style="{};background:#fff;border:1px solid #afafaf" data-tip="{:.1} %"><div style="width:{fill}px;height:4px;background:#567fcf"></div></div>"#,
        at(x, y, 112, 6),
        // Under the pointer the bar tells how far it is, in tenths of a hundredth cut short, as the client's
        // does (`UIINT64BarGraph::OnMouseHover`).
        (now.clamp(0, next) as f64 / next as f64 * 1000.0).floor() / 10.0
    )
}

/// How many hundredths of `most` there are, as the window writes it beside a gauge.
fn percent(now: i64, most: i64) -> String {
    if most > 0 { format!("{}%", now * 100 / most) } else { String::new() }
}

impl App {
    /// The strip of icons that open the windows, as the original lays it out (`UIMenuIconWnd::OnCreate`): five
    /// to a row, under a bar that folds it away. Under the pointer an icon tells what it opens, in the words
    /// of the client's message table, with the key that opens it too where there is one
    /// (`UIMenuIcon::OnMouseMove`: the line of the table and the entry of the interface's keys are beside each
    /// icon here; the game's options tell of Escape). The icons of what only later content has are there,
    /// faded, and are pressed for nothing.
    pub(in crate::web) fn menu_icons(&self) -> String {
        use crate::hotkeys::interface as key;
        use crate::press::{Press, general, guide, inventory, skills, status};
        // The icon's picture, what a press on it does, the line of the message table, the same in this
        // client's words until the table is read, and its key.
        let icons: [(&str, Option<Press>, usize, &str, Option<usize>); 23] = [
            ("status", Some(status::Press::Window.into()), 105, "Status", Some(key::STATUS)),
            ("equip", Some(inventory::Press::Equipment.into()), 104, "Equipment", Some(key::EQUIPMENT)),
            ("item", Some(inventory::Press::Window.into()), 106, "Inventory", Some(key::INVENTORY)),
            ("skill", Some(skills::Press::Window.into()), 283, "Skill Tree", Some(key::SKILLS)),
            ("party", Some(crate::press::party::Press::Window.into()), 103, "Party", Some(key::PARTY)),
            ("guild", Some(crate::press::guild::Press::Window.into()), 3134, "Guild", Some(key::GUILD)),
            ("battle", None, 2003, "Battleground", None),
            ("quest", Some(crate::press::quests::Press::Window.into()), 1317, "Quest List", Some(key::QUESTS)),
            ("map", Some(crate::press::worldmap::Press::Window.into()), 2859, "World Map", Some(key::WORLD_MAP)),
            ("navigation", None, 2353, "Navigation", None),
            ("option", Some(general::Press::Options.into()), 265, "Option", None),
            ("bank", None, 3135, "Bank", None),
            ("rec", None, 2350, "Replay", None),
            ("mail", Some(crate::press::mail::Press::Window.into()), 3136, "Mail", Some(key::MAIL)),
            ("achievement", None, 2646, "Achievements", None),
            ("tip", Some(crate::press::tips::Press::Window.into()), 3239, "Tipbox", Some(key::TIPS)),
            ("shop", None, 3137, "Cash Shop", None),
            // The client's icon opens its guide to the keys (`WID_HOTKEYGUIDEWND`), a picture of a keyboard;
            // here it opens the Shortcut Settings, which the client has in the game's menu.
            ("keyboard", Some(crate::press::hotkeys::Press::Open.into()), 3128, "Hotkey Guide", None),
            ("attendance", None, 3473, "Attendance check", None),
            ("adventurerAgency", None, 3514, "Adventurer's Agency", None),
            ("repute", None, 3827, "Reputation Status", None),
            ("adventureguide", Some(guide::Press::Open.into()), 4053, "Adventure guide", None),
            ("probability", None, 4119, "Probabilities", None),
        ];
        let bar = |name: &str| {
            pictured(0, 4, (219, 9), [&format!("menu_icon/bt_menu_{name}normal.bmp"), &format!("menu_icon/bt_menu_{name}over.bmp"), &format!("menu_icon/bt_menu_{name}press.bmp")], general::Press::Menu, r#" title="Menu""#)
        };
        if self.menu_closed {
            return bar("");
        }
        let mut strip = bar("close_");
        for (position, (name, press, line, otherwise, key)) in icons.into_iter().enumerate() {
            let name = if name == "status" && self.game.as_ref().is_some_and(|game| crate::jobs::doram(game.job)) { "status_doram" } else { name };
            let (x, y) = ((position % 5 * 42 + 9) as i32, (position / 5 * 43 + 23) as i32);
            // With mail not read the mail's icon is the one that wears a mark, six higher and 35 by 40
            // (`UIMenuIconWnd::OnCreate`, `notificationButtons`).
            let (name, y, size) = if name == "mail" && self.mail.marked { ("mail_new", y - 6, (35, 40)) } else { (name, y, (32, 34)) };
            let (normal, down) = (picture(&format!("menu_icon/bt_{name}.bmp")), picture(&format!("menu_icon/bt_{name}_press.bmp")));
            let keys = match (name, key) {
                ("option", _) => "Esc".to_owned(),
                (_, Some(place)) => crate::hotkeys::written(self.bound(crate::hotkeys::table::INTERFACE, place), self.keys.mac, data::key_name),
                _ => String::new(),
            };
            let tells = if keys.is_empty() { Self::words(line, otherwise) } else { format!("{} ({keys})", Self::words(line, otherwise)) };
            let (dim, click) = match press {
                Some(press) => ("", pressed(press)),
                None => (" dim", String::new()),
            };
            strip.push_str(&format!(
                r#"<button type="button" class="a b{dim}" style="{};--n:url('{normal}');--o:url('{normal}');--p:url('{down}')" {click} data-tip="{}"></button>"#,
                at(x, y, size.0, size.1),
                escape(&tells)
            ));
        }
        strip
    }

    /// How tall the Basic Information window is now: closed or open, and taller for a job with Activity Points.
    pub(in crate::web) fn basic_height(&self) -> i32 {
        let ap = self.game.as_ref().is_some_and(|game| crate::jobs::has_ap(crate::mounts::drawn_job(game.job, self.hero.state)));
        match (self.hero.basic_small, ap) {
            (true, false) => 55,
            (true, true) => 71,
            (false, false) => 134,
            (false, true) => 149,
        }
    }

    /// The Basic Information window (`UIBasicInfoWnd`): its small form, a title bar and two lines, or its open
    /// one with the gauges.
    pub(in crate::web) fn basic_info(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        // The fourth jobs have a third gauge, of Activity Points, and the window is taller for it (the client's
        // `UIBasicInfoWnd`: 149 for 134 open and 71 for 55 closed, what is under the gauges 15 lower).
        let ap = crate::jobs::has_ap(crate::mounts::drawn_job(game.job, self.hero.state)).then(|| (self.value(parameter::AP), self.value(parameter::MAX_AP)));
        if self.hero.basic_small { self.basic_small(game, ap) } else { self.basic_open(game, ap) }
    }

    /// The small form: a title bar with the name, and two lines; a third for Activity Points. Its lines are
    /// written as (1, 14), as the window's words are when it is open (`UIBasicInfoWnd::Paint`).
    fn basic_small(&self, game: &Game, ap: Option<(i64, i64)>) -> String {
        let job = data::job_name(game.job);
        let (hp, max_hp, sp, max_sp) = (self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        let (base_level, job_level) = (self.value(parameter::BASE_LEVEL), self.value(parameter::JOB_LEVEL));
        let (base_exp, next_base) = (self.value(parameter::BASE_EXP), self.value(parameter::NEXT_BASE_EXP));
        let figures = FIGURES.style();
        let exp = if next_base > 0 { base_exp as f64 * 100.0 / next_base as f64 } else { 0.0 };
        let low = |now: i64, most: i64| if now * 4 < most { RED } else { BLACK };
        format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0"></div>{}{}{}
                   <div class="a t" style="left:9px;top:36px;{figures}"><span style="color:{}">HP. {hp} / {max_hp}</span> | <span style="color:{}">SP. {sp} / {max_sp}</span></div>{}"#,
            at(0, 17, 220, if ap.is_some() { 54 } else { 38 }),
            title_bar(220, &game.name),
            press_button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", crate::press::status::Press::BasicSize),
            text_in(FIGURES, 9, 20, BLACK, &format!("{lv}{base_level:2} / {job} / {lv}{job_level:2} / {} {exp:.1} %", tr("Exp."), lv = tr("Lv."))),
            low(hp, max_hp),
            low(sp, max_sp),
            ap.map(|(ap, max_ap)| text_in(FIGURES, 9, 52, BLACK, &format!("AP. {ap} / {max_ap}"))).unwrap_or_default(),
        )
    }

    /// The open form: the client's picture, the name and the job, the gauges, the levels with how far each is,
    /// and what is carried.
    fn basic_open(&self, game: &Game, ap: Option<(i64, i64)>) -> String {
        let job = data::job_name(game.job);
        let (hp, max_hp, sp, max_sp) = (self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        let (base_level, job_level) = (self.value(parameter::BASE_LEVEL), self.value(parameter::JOB_LEVEL));
        let (base_exp, next_base) = (self.value(parameter::BASE_EXP), self.value(parameter::NEXT_BASE_EXP));
        let (job_exp, next_job) = (self.value(parameter::JOB_EXP), self.value(parameter::NEXT_JOB_EXP));
        let lower = if ap.is_some() { 15 } else { 0 };
        // The taller window is the same picture twice: its lower part moved down, its upper part over that.
        let mut window = match ap {
            Some(_) => format!(r#"{}<div class="a" style="{};overflow:hidden">{}</div>"#, art("basic_interface/basewin_bg2.bmp", 0, 15, 220, 134), at(0, 0, 220, 82), art("basic_interface/basewin_bg2.bmp", 0, 0, 220, 134)),
            None => art("basic_interface/basewin_bg2.bmp", 0, 0, 220, 134),
        };
        window.push_str(&titled(17, 2, tr("Basic Information")));
        window.push_str(&press_button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", crate::press::status::Press::BasicSize));
        // The name is a text of its own in the usual font (`UIBasicInfoWnd::OnCreate`: `SetFont(0, 12, 0, 0)`);
        // everything under it is written as (1, 14) (`UIBasicInfoWnd::Paint`; the job's name is (0, 12) only
        // in the Korean service).
        window.push_str(&text(9, 22, BLACK, &game.name));
        window.push_str(&text_in(FIGURES, 9, 36, BLACK, &job));
        window.push_str(&text_in(FIGURES, 17, 51, BLACK, "HP"));
        window.push_str(&text_in(FIGURES, 17, 65, BLACK, "SP"));
        window.push_str(&gauge(35, 53, hp, max_hp));
        window.push_str(&gauge(35, 68, sp, max_sp));
        window.push_str(&text_in(FIGURES, 173, 51, BLACK, &percent(hp, max_hp)));
        window.push_str(&text_in(FIGURES, 173, 65, BLACK, &percent(sp, max_sp)));
        if let Some((ap, max_ap)) = ap {
            window.push_str(&text_in(FIGURES, 17, 80, BLACK, "AP"));
            window.push_str(&ap_gauge(35, 83, ap, max_ap));
        }
        window.push_str(&text_in(FIGURES, 17, 84 + lower, BLACK, &format!("{} {base_level}", tr("Base Lv."))));
        window.push_str(&text_in(FIGURES, 17, 96 + lower, BLACK, &format!("{} {job_level}", tr("Job Lv."))));
        window.push_str(&experience(85, 89 + lower, base_exp, next_base));
        window.push_str(&experience(85, 101 + lower, job_exp, next_job));
        window.push_str(&self.basic_carried(lower));
        window
    }

    /// The foot of the open window, `lower` down for a job with Activity Points: the weight carried of what
    /// can be, and the zeny.
    fn basic_carried(&self, lower: i32) -> String {
        // Weight is kept in tenths; carrying the share of what one can that the server names (half, until it
        // names another: `m_overweightThresholdPercent`) is already too much to recover by resting.
        let (weight, max_weight) = (self.value(parameter::WEIGHT), self.value(parameter::MAX_WEIGHT));
        let heavy = if max_weight > 0 && weight * 100 / max_weight >= self.notices.overweight { RED } else { BLACK };
        // The window's bottom left, 100 wide and 20 high, tells under the pointer how much of what can be carried
        // is carried (`UIBasicInfoWnd::OnMouseHover`).
        let mut foot = format!(r#"<div class="a" style="{}" data-tip="Weight {}%"></div>"#, at(0, 114 + lower, 100, 20), if max_weight > 0 { weight * 100 / max_weight } else { 0 });
        foot.push_str(&format!(
            r#"<div class="a t" style="right:5px;top:{}px;{};white-space:pre"><span style="color:{heavy}">{} : {:3} / {:3}</span>  {} : {}</div>"#,
            116 + lower,
            FIGURES.style(),
            tr("Weight"),
            weight / 10,
            max_weight / 10,
            tr("Zeny"),
            grouped(self.value(parameter::ZENY))
        ));
        foot
    }
}
