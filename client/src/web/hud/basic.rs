//! The Basic Information window with its gauges, and the strip of menu icons under it.

use super::*;

/// A number with commas between its thousands.
pub(super) fn grouped(number: i64) -> String {
    let digits = number.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if number < 0 { format!("-{grouped}") } else { grouped }
}

impl App {
    /// A gauge of the Basic Information window: its back, the fill up to `now` of `most` (red when under a
    /// quarter), and the two numbers over it.
    pub(in crate::web) fn gauge(&self, x: i32, y: i32, now: i64, most: i64) -> String {
        let mut gauge = art("basic_interface/gze_bg1.bmp", x, y, 135, 9);
        // Values that make no sense (more than the most) are not drawn at all.
        if most <= 0 || now > most {
            return gauge;
        }
        let colour = if now * 4 < most { "red" } else { "blue" };
        let fill = (127 * now.max(0) / most) as i32;
        gauge.push_str(&art(&format!("basic_interface/gze{colour}_left.bmp"), x, y, 4, 8));
        gauge.push_str(&art(&format!("basic_interface/gze{colour}_mid.bmp"), x + 4, y, fill, 8));
        gauge.push_str(&art(&format!("basic_interface/gze{colour}_right.bmp"), x + 4 + fill, y, 4, 8));
        gauge.push_str(&format!(r#"<div class="a t centre" style="{};white-space:pre">{now:3}  /  {most:3}</div>"#, at(x, y - 2, 135, 13)));
        gauge
    }

    /// An experience bar: how much of the way to the next level is done.
    pub(in crate::web) fn experience(&self, x: i32, y: i32, now: i64, next: i64) -> String {
        if next <= 0 {
            return String::new();
        }
        let fill = (110 * now.clamp(0, next) / next) as i32;
        format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #afafaf" title="{:.1} %"><div style="width:{fill}px;height:4px;background:#567fcf"></div></div>"#,
            at(x, y, 112, 6),
            now as f64 * 100.0 / next as f64
        )
    }

    pub(in crate::web) fn percent(now: i64, most: i64) -> String {
        if most > 0 { format!("{}%", now * 100 / most) } else { String::new() }
    }

    /// The strip of icons that open the windows, as the original lays it out: five to a row, under a bar that
    /// folds it away. The icons of windows not made yet are there, faded, and do nothing.
    pub(in crate::web) fn menu_icons(&self) -> String {
        const ICONS: [(&str, &str, &str); 23] = [
            ("status", "status", "Status (Alt+A)"),
            ("equip", "equipment", "Equipment (Alt+Q)"),
            ("item", "inventory", "Inventory (Alt+E)"),
            ("skill", "skills", "Skills (Alt+S)"),
            ("party", "", "Party (Alt+Z)"),
            ("guild", "", "Guild (Alt+G)"),
            ("battle", "", "Battlegrounds"),
            ("quest", "", "Quests (Alt+U)"),
            ("map", "", "World map (Ctrl+~)"),
            ("navigation", "", "Navigation"),
            ("option", "", "Options (Alt+O)"),
            ("bank", "", "Bank"),
            ("rec", "", "Replay"),
            ("mail", "", "Mail"),
            ("achievement", "", "Achievements"),
            ("tip", "", "Tips"),
            ("shop", "", "Cash shop"),
            ("keyboard", "", "Shortcut settings"),
            ("attendance", "", "Attendance"),
            ("adventurerAgency", "", "Adventurer agency"),
            ("repute", "", "Reputation"),
            ("adventureguide", "", "Adventure guide"),
            ("probability", "", "Probabilities"),
        ];
        let bar = |name: &str| {
            format!(
                r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="menu" title="Menu"></button>"#,
                at(0, 4, 219, 9),
                picture(&format!("menu_icon/bt_menu_{name}normal.bmp")),
                picture(&format!("menu_icon/bt_menu_{name}over.bmp")),
                picture(&format!("menu_icon/bt_menu_{name}press.bmp"))
            )
        };
        if self.menu_closed {
            return bar("");
        }
        let mut strip = bar("close_");
        for (position, (name, click, title)) in ICONS.iter().enumerate() {
            let name = if *name == "status" && self.game.as_ref().is_some_and(|game| data::is_doram(game.job)) { "status_doram" } else { name };
            let (x, y) = ((position % 5 * 42 + 9) as i32, (position / 5 * 43 + 23) as i32);
            let (normal, pressed) = (picture(&format!("menu_icon/bt_{name}.bmp")), picture(&format!("menu_icon/bt_{name}_press.bmp")));
            let dim = if click.is_empty() { " dim" } else { "" };
            strip.push_str(&format!(
                r#"<button type="button" class="a b{dim}" style="{};--n:url('{normal}');--o:url('{normal}');--p:url('{pressed}')" data-click="{click}" title="{}"></button>"#,
                at(x, y, 32, 34),
                tr(title)
            ));
        }
        strip
    }

    pub(in crate::web) fn basic_info(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        let job = data::job_name(game.job);
        let (hp, max_hp, sp, max_sp) = (self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        let (base_level, job_level) = (self.value(parameter::BASE_LEVEL), self.value(parameter::JOB_LEVEL));
        let (base_exp, next_base) = (self.value(parameter::BASE_EXP), self.value(parameter::NEXT_BASE_EXP));
        let (job_exp, next_job) = (self.value(parameter::JOB_EXP), self.value(parameter::NEXT_JOB_EXP));

        if self.hero.basic_small {
            // The small form: a title bar with the name, and two lines.
            let exp = if next_base > 0 { base_exp as f64 * 100.0 / next_base as f64 } else { 0.0 };
            let low = |now: i64, most: i64| if now * 4 < most { "#f00" } else { BLACK };
            return format!(
                r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0"></div>{}{}{}
                   <div class="a t" style="left:9px;top:36px"><span style="color:{}">HP. {hp} / {max_hp}</span> | <span style="color:{}">SP. {sp} / {max_sp}</span></div>"#,
                at(0, 17, 220, 38),
                title_bar(220, &game.name),
                button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", "basic-size", ""),
                text(9, 20, BLACK, &format!("{lv}{base_level:2} / {job} / {lv}{job_level:2} / {} {exp:.1} %", tr("Exp."), lv = tr("Lv."))),
                low(hp, max_hp),
                low(sp, max_sp),
            );
        }

        let mut window = art("basic_interface/basewin_bg2.bmp", 0, 0, 220, 134);
        window.push_str(&format!(r#"<div class="a t titled" style="left:17px;top:2px">{}</div>"#, tr("Basic Information")));
        window.push_str(&button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", "basic-size", ""));
        window.push_str(&text(9, 22, BLACK, &game.name));
        window.push_str(&text(9, 36, BLACK, &job));
        window.push_str(&text(17, 51, BLACK, "HP"));
        window.push_str(&text(17, 65, BLACK, "SP"));
        window.push_str(&self.gauge(35, 53, hp, max_hp));
        window.push_str(&self.gauge(35, 68, sp, max_sp));
        window.push_str(&text(173, 51, BLACK, &Self::percent(hp, max_hp)));
        window.push_str(&text(173, 65, BLACK, &Self::percent(sp, max_sp)));
        window.push_str(&text(17, 84, BLACK, &format!("{} {base_level}", tr("Base Lv."))));
        window.push_str(&text(17, 96, BLACK, &format!("{} {job_level}", tr("Job Lv."))));
        window.push_str(&self.experience(85, 89, base_exp, next_base));
        window.push_str(&self.experience(85, 101, job_exp, next_job));
        // Weight is kept in tenths; carrying half of what one can is already too much to recover by resting.
        let (weight, max_weight) = (self.value(parameter::WEIGHT), self.value(parameter::MAX_WEIGHT));
        let heavy = if max_weight > 0 && weight * 2 >= max_weight { "#f00" } else { BLACK };
        window.push_str(&format!(
            r#"<div class="a t" style="right:5px;top:116px;white-space:pre"><span style="color:{heavy}">{} : {:3} / {:3}</span>  {} : {}</div>"#,
            tr("Weight"),
            weight / 10,
            max_weight / 10,
            tr("Zeny"),
            grouped(self.value(parameter::ZENY))
        ));
        window
    }
}
