//! The windows of the game itself: Basic Information, the hotkey bar, Status, Inventory, the skill list, and chat.
//!
//! Sizes, places, pictures and colours are the original client's, as rustnarok-client lists them
//! (`src/ui/basic_info.rs`, `hotkeys.rs`, `status.rs`, `inventory.rs`, `skills.rs`, `chat.rs`, `window.rs`; MIT OR
//! Apache-2.0). The inventory and the skill list are simpler than the original's: fixed sizes, the skills as one
//! list without the tree, and worn equipment shown among the items (until there is an Equipment window).
//!
//! Each window lives in an element of its own, so a value that changes redraws only its window: the chat's
//! input line is never rebuilt while someone types in it.

use super::app::{App, escape};
use super::data;
use crate::net::items::{Hotkey, Item, Tab};
use crate::net::packets::parameter;

const UI: &str = "data/texture/유저인터페이스/";

/// How many lines the chat shows at each press of its size key; 0 leaves only the input line.
pub const CHAT_STEPS: [usize; 6] = [3, 6, 9, 12, 15, 0];
/// How many chat lines are kept.
pub const CHAT_KEPT: usize = 240;

const BLACK: &str = "#000";
/// How many columns and rows of items the inventory shows, and how many skills the list shows.
const ITEM_COLUMNS: i32 = 7;
const ITEM_ROWS: i32 = 5;
const SKILL_ROWS: i32 = 6;

fn picture(name: &str) -> String {
    format!("/image/{}", String::from(js_sys::encode_uri(&format!("{UI}{name}"))))
}

fn at(x: i32, y: i32, width: i32, height: i32) -> String {
    format!("left:{x}px;top:{y}px;width:{width}px;height:{height}px")
}

/// A piece of the window's art at its place and size (stretched to it).
fn art(name: &str, x: i32, y: i32, width: i32, height: i32) -> String {
    format!(r#"<div class="a" style="{};background:url('{}') 0 0 / 100% 100% no-repeat"></div>"#, at(x, y, width, height), picture(name))
}

/// A line of text with its top-left corner at a place.
fn text(x: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="left:{x}px;top:{y}px;color:{colour}">{}</div>"#, escape(words))
}

/// A line of text ending at `right`, in a window `width` wide.
fn text_ending(right: i32, width: i32, y: i32, colour: &str, words: &str) -> String {
    format!(r#"<div class="a t" style="right:{}px;top:{y}px;color:{colour}">{}</div>"#, width - right, escape(words))
}

/// A small picture button whose picture changes under the pointer.
fn button(x: i32, y: i32, size: i32, off: &str, on: &str, click: &str, arg: &str) -> String {
    format!(
        r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="{click}" data-arg="{arg}"></button>"#,
        at(x, y, size, size),
        picture(off),
        picture(on),
        picture(on)
    )
}

/// The standard title bar: caps that keep their size and a middle that stretches, the title in black over a
/// white copy of itself.
fn title_bar(width: i32, title: &str) -> String {
    format!(
        "{}{}{}{}",
        art("basic_interface/titlebar_left.bmp", 0, 0, 12, 17),
        art("basic_interface/titlebar_mid.bmp", 12, 0, width - 24, 17),
        art("basic_interface/titlebar_right.bmp", width - 12, 0, 12, 17),
        format!(r#"<div class="a t titled" style="left:17px;top:1px">{}</div>"#, escape(title))
    )
}

/// A number with commas between its thousands.
fn grouped(number: i64) -> String {
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
    fn value(&self, name: u16) -> i64 {
        self.params.get(&name).copied().unwrap_or(0)
    }

    /// Every window of the game, each in its own element. `view` is the stage's size.
    pub(super) fn hud(&self, view: (f64, f64)) -> String {
        let chat_lines = CHAT_STEPS[self.chat_step % CHAT_STEPS.len()];
        let chat_height = if chat_lines == 0 { 24 } else { 24 + 16 + 14 * (chat_lines as i32 + 1) };
        let hotkeys = if self.hotkeys_hidden { String::new() } else { format!(r#"<div id="hotkeys" class="a hud" style="{}">{}</div>"#, at(230, 0, 290, 76), self.hotkeys()) };
        let status = if self.status_open { format!(r#"<div id="status" class="a hud" style="{}">{}</div>"#, at(185, 300, 280, 141), self.status_window()) } else { String::new() };
        let inventory = if self.inventory_open {
            format!(r#"<div id="inventory" class="a hud" style="{}">{}</div>"#, at(0, 140, 40 + 32 * ITEM_COLUMNS + 16, 17 + 32 * ITEM_ROWS + 21), self.inventory_window())
        } else {
            String::new()
        };
        let skills = if self.skills_open { format!(r#"<div id="skills" class="a hud" style="{}">{}</div>"#, at(242, 80, 330, 34 + 32 * SKILL_ROWS + 32), self.skill_window()) } else { String::new() };
        format!(
            r#"<div id="basic" class="a hud" style="left:0;top:0;width:220px">{}</div>{hotkeys}{status}{inventory}{skills}
               <div id="chat" class="a hud" style="{}">{}</div>"#,
            self.basic_info(),
            at(0, view.1 as i32 - chat_height, 600, chat_height),
            self.chat_window(chat_lines, chat_height)
        )
    }

    /// Redraws the windows that show the character's values, where they are on the page.
    pub(super) fn refresh_values(&self) {
        let document = web_sys::window().and_then(|window| window.document());
        let Some(document) = document else { return };
        if let Some(basic) = document.get_element_by_id("basic") {
            basic.set_inner_html(&self.basic_info());
        }
        if let Some(status) = document.get_element_by_id("status") {
            status.set_inner_html(&self.status_window());
        }
    }

    /// Redraws the windows that show items and skills, where they are on the page.
    pub(super) fn refresh_items(&self) {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else { return };
        for (id, html) in [("inventory", self.inventory_window()), ("skills", self.skill_window()), ("hotkeys", self.hotkeys())] {
            if let Some(element) = document.get_element_by_id(id) {
                // The lists keep their place when they are redrawn.
                let scrolled = element.query_selector(".scroll").ok().flatten().map_or(0, |list| list.scroll_top());
                element.set_inner_html(&html);
                if let Ok(Some(list)) = element.query_selector(".scroll") {
                    list.set_scroll_top(scrolled);
                }
            }
        }
    }

    /// Redraws the chat's lines, the newest at the bottom.
    pub(super) fn refresh_chat(&self) {
        if let Some(log) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("chat-log")) {
            log.set_inner_html(&self.chat_lines(CHAT_STEPS[self.chat_step % CHAT_STEPS.len()]));
        }
    }

    /// A gauge of the Basic Information window: its back, the fill up to `now` of `most` (red when under a
    /// quarter), and the two numbers over it.
    fn gauge(&self, x: i32, y: i32, now: i64, most: i64) -> String {
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
    fn experience(&self, x: i32, y: i32, now: i64, next: i64) -> String {
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

    fn percent(now: i64, most: i64) -> String {
        if most > 0 { format!("{}%", now * 100 / most) } else { String::new() }
    }

    fn basic_info(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        let job = data::job_name(game.job);
        let (hp, max_hp, sp, max_sp) = (self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        let (base_level, job_level) = (self.value(parameter::BASE_LEVEL), self.value(parameter::JOB_LEVEL));
        let (base_exp, next_base) = (self.value(parameter::BASE_EXP), self.value(parameter::NEXT_BASE_EXP));
        let (job_exp, next_job) = (self.value(parameter::JOB_EXP), self.value(parameter::NEXT_JOB_EXP));

        if self.basic_small {
            // The small form: a title bar with the name, and two lines.
            let exp = if next_base > 0 { base_exp as f64 * 100.0 / next_base as f64 } else { 0.0 };
            let low = |now: i64, most: i64| if now * 4 < most { "#f00" } else { BLACK };
            return format!(
                r#"<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0"></div>{}{}{}
                   <div class="a t" style="left:9px;top:36px"><span style="color:{}">HP. {hp} / {max_hp}</span> | <span style="color:{}">SP. {sp} / {max_sp}</span></div>"#,
                at(0, 17, 220, 38),
                title_bar(220, &game.name),
                button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", "basic-size", ""),
                text(9, 20, BLACK, &format!("Lv.{base_level:2} / {job} / Lv.{job_level:2} / Exp. {exp:.1} %")),
                low(hp, max_hp),
                low(sp, max_sp),
            );
        }

        let mut window = art("basic_interface/basewin_bg2.bmp", 0, 0, 220, 134);
        window.push_str(&format!(r#"<div class="a t titled" style="left:17px;top:2px">{}</div>"#, "Basic Information"));
        window.push_str(&button(206, 3, 11, "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", "basic-size", ""));
        window.push_str(&text(9, 22, BLACK, &game.name));
        window.push_str(&text(9, 36, BLACK, &job));
        window.push_str(&text(17, 51, BLACK, "HP"));
        window.push_str(&text(17, 65, BLACK, "SP"));
        window.push_str(&self.gauge(35, 53, hp, max_hp));
        window.push_str(&self.gauge(35, 68, sp, max_sp));
        window.push_str(&text(173, 51, BLACK, &Self::percent(hp, max_hp)));
        window.push_str(&text(173, 65, BLACK, &Self::percent(sp, max_sp)));
        window.push_str(&text(17, 84, BLACK, &format!("Base Lv. {base_level}")));
        window.push_str(&text(17, 96, BLACK, &format!("Job Lv. {job_level}")));
        window.push_str(&self.experience(85, 89, base_exp, next_base));
        window.push_str(&self.experience(85, 101, job_exp, next_job));
        // Weight is kept in tenths; carrying half of what one can is already too much to recover by resting.
        let (weight, max_weight) = (self.value(parameter::WEIGHT), self.value(parameter::MAX_WEIGHT));
        let heavy = if max_weight > 0 && weight * 2 >= max_weight { "#f00" } else { BLACK };
        window.push_str(&format!(
            r#"<div class="a t" style="right:5px;top:116px;white-space:pre"><span style="color:{heavy}">Weight : {:3} / {:3}</span>  Zeny : {}</div>"#,
            weight / 10,
            max_weight / 10,
            grouped(self.value(parameter::ZENY))
        ));
        window
    }

    /// The picture of an item or a skill, or nothing while its name is not known.
    fn icon(resource: Option<&str>) -> String {
        match resource {
            Some(resource) => format!(r#"<img src="{}" alt="" draggable="false" onerror="this.remove()">"#, picture(&format!("item/{resource}.bmp"))),
            None => String::new(),
        }
    }

    /// How many of an item the character carries without wearing them.
    fn carried(&self, id: u32) -> u32 {
        self.inventory.iter().filter(|item| item.id == id && item.worn == 0).map(|item| u32::from(item.amount)).sum()
    }

    /// The bar of slots for skills and items: two rows of nine. F1 to F9 use the first row; an icon dragged from
    /// the inventory or the skill list fills a slot, and a right click empties it.
    fn hotkeys(&self) -> String {
        let mut bar = String::new();
        for row in 0..2 {
            bar.push_str(&art("shortcut/shortcut_frame.bmp", 0, row * 33, 290, 34));
            bar.push_str(&text(6, row * 33 + 13, "rgb(74,74,47)", &(row + 1).to_string()));
            for column in 0..9 {
                let slot = (row * 9 + column) as usize;
                let hotkey = self.hotkeys.get(slot).copied().unwrap_or_default();
                let (icon, count, title) = self.hotkey_look(hotkey);
                let key = if row == 0 { format!("F{} ", column + 1) } else { String::new() };
                bar.push_str(&format!(
                    r#"<div class="a slot" style="{}" data-slot="{slot}" data-use="slot:{slot}" title="{key}{}">{icon}{count}</div>"#,
                    at(16 + 29 * column, row * 33 + 4, 24, 24),
                    escape(&title)
                ));
            }
        }
        bar
    }

    /// A hotkey's picture, the number under it, and what it says when pointed at.
    fn hotkey_look(&self, hotkey: Hotkey) -> (String, String, String) {
        if hotkey.id == 0 {
            return (String::new(), String::new(), String::new());
        }
        if hotkey.skill {
            let known = data::skill(hotkey.id as u16);
            let icon = Self::icon(known.as_ref().map(|(resource, ..)| resource.to_lowercase()).as_deref());
            let name = known.map_or_else(|| format!("Skill {}", hotkey.id), |(_, name, _)| name);
            (icon, format!(r#"<span class="n">{}</span>"#, hotkey.count), format!("{name} Lv {}", hotkey.count))
        } else {
            let known = data::item(hotkey.id);
            let carried = self.carried(hotkey.id);
            // An item that is used up stays on the bar, without its picture, until there is more of it.
            let icon = if carried == 0 { String::new() } else { Self::icon(known.as_ref().map(|(_, resource)| resource.as_str())) };
            let name = known.map_or_else(|| format!("Item {}", hotkey.id), |(name, _)| name);
            (icon, if carried == 0 { String::new() } else { format!(r#"<span class="n">{carried}</span>"#) }, format!("{name} : {carried} ea"))
        }
    }

    /// An item's name as the game shows it: its refinement before it, its free slots after it.
    fn item_name(item: &Item) -> String {
        let name = data::item(item.id).map_or_else(|| format!("Item {}", item.id), |(name, _)| name);
        if item.refine > 0 { format!("+{} {name}", item.refine) } else { name }
    }

    fn inventory_window(&self) -> String {
        let width = 40 + 32 * ITEM_COLUMNS + 16;
        let body = 32 * ITEM_ROWS;
        let mut window = title_bar(width, "Inventory");
        window.push_str(&button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "inventory", ""));
        // The body: the tabs' strip, the cells, and the pieces at their sides.
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c3c3c3"></div>"#, at(0, 17, 40, body)));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(20, 17, 20, body), picture("basic_interface/itemwin_left.bmp")));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(width - 20, 17, 20, body), picture("basic_interface/itemwin_right.bmp")));
        for (index, (tab, label)) in [(Tab::Usable, "Use"), (Tab::Equipment, "Eqp"), (Tab::Other, "Etc")].into_iter().enumerate() {
            let open = if tab == self.inventory_tab { " open" } else { "" };
            let letters: String = label.chars().map(|letter| format!("<span>{letter}</span>")).collect();
            window.push_str(&format!(
                r#"<button type="button" class="a tab{open}" style="{}" data-click="inventory-tab" data-arg="{index}">{letters}</button>"#,
                at(3, 18 + index as i32 * 51, 17, 52)
            ));
        }
        let mut cells = String::new();
        let shown: Vec<&Item> = self.inventory.iter().filter(|item| item.tab() == self.inventory_tab).collect();
        for (place, item) in shown.iter().enumerate() {
            let (column, row) = (place as i32 % ITEM_COLUMNS, place as i32 / ITEM_COLUMNS);
            let resource = data::item(item.id).map(|(_, resource)| resource);
            let worn = if item.worn != 0 { " worn" } else { "" };
            cells.push_str(&format!(
                r#"<div class="a cell{worn}" style="left:{}px;top:{}px" draggable="true" data-drag="item:{}" data-use="item:{}" title="{} : {} ea">{}<span class="n">{}</span></div>"#,
                column * 32,
                row * 32,
                item.id,
                item.index,
                escape(&Self::item_name(item)),
                item.amount,
                Self::icon(resource.as_deref()),
                item.amount
            ));
        }
        // Rows of empty cells below the last item, so the window is never shorter than it shows.
        let rows = (shown.len().div_ceil(ITEM_COLUMNS as usize) as i32).max(ITEM_ROWS);
        window.push_str(&format!(
            r#"<div class="a scroll" style="{};background:url('{}')"><div style="position:relative;height:{}px">{cells}</div></div>"#,
            at(40, 17, 32 * ITEM_COLUMNS, body),
            picture("basic_interface/itemwin_mid.bmp"),
            rows * 32
        ));
        let footer = 17 + body;
        window.push_str(&art("basic_interface/btnbar_left.bmp", 0, footer, 21, 21));
        window.push_str(&art("basic_interface/btnbar_mid.bmp", 21, footer, width - 42, 21));
        window.push_str(&art("basic_interface/btnbar_right.bmp", width - 21, footer, 21, 21));
        let kinds = self.inventory.iter().filter(|item| item.worn == 0).count();
        window.push_str(&text(35, footer + 5, if kinds >= 100 { "#f00" } else { "#232323" }, &format!("{kinds} / 100")));
        window
    }

    fn skill_window(&self) -> String {
        let (width, body) = (330, 32 * SKILL_ROWS);
        let mut window = title_bar(width, "Skill Tree");
        window.push_str(&button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "skills", ""));
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c3c3c3;border-top:0"></div>"#, at(0, 17, width, 17 + body)));
        let points = self.value(parameter::SKILL_POINTS);
        let mut rows = String::new();
        for (row, skill) in self.skills.iter().enumerate() {
            let top = row as i32 * 32;
            let known = data::skill(skill.id);
            let name = known.as_ref().map_or_else(|| format!("Skill {}", skill.id), |(_, name, _)| name.clone());
            let most = known.as_ref().map_or(skill.level, |(.., most)| *most);
            let icon = Self::icon(known.as_ref().map(|(resource, ..)| resource.to_lowercase()).as_deref());
            // Only a skill that is learned and does something can be used and put on the bar.
            let usable = skill.level > 0 && !skill.passive();
            let drag = if usable { format!(r#"draggable="true" data-drag="skill:{}:{}" data-use="skill:{}""#, skill.id, skill.level, skill.id) } else { String::new() };
            rows.push_str(&format!(r#"<div class="a cell" style="left:44px;top:{}px" {drag} title="{}">{icon}</div>"#, top + 2, escape(&name)));
            if skill.raisable && points > 0 && skill.level < most {
                rows.push_str(&format!(
                    r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="raise-skill" data-arg="{}"></button>"#,
                    at(78, top + 2, 24, 24),
                    picture("basic_interface/skill_up_a.bmp"),
                    picture("basic_interface/skill_up_b.bmp"),
                    picture("basic_interface/skill_up_c.bmp"),
                    skill.id
                ));
            }
            let colour = if skill.level > 0 { BLACK } else { "rgb(120,120,120)" };
            rows.push_str(&text(112, top + 1, colour, &name));
            rows.push_str(&text(113, top + 13, colour, &format!("Lv : {:2} / {most}", skill.level)));
            rows.push_str(&text(212, top + 13, colour, &if skill.passive() { "Passive".to_owned() } else { format!("Sp : {}", skill.sp) }));
        }
        window.push_str(&format!(r#"<div class="a scroll" style="{}"><div style="position:relative;height:{}px">{rows}</div></div>"#, at(0, 34, width - 2, body), self.skills.len() as i32 * 32));
        let footer = 34 + body;
        window.push_str(&art("basic_interface/skill_botbar_left.bmp", 0, footer, 12, 32));
        window.push_str(&art("basic_interface/skill_botbar_mid.bmp", 12, footer, width - 24, 32));
        window.push_str(&art("basic_interface/skill_botbar_right.bmp", width - 12, footer, 12, 32));
        window.push_str(&text(30, footer + 9, BLACK, &format!("Skill Point: {points}")));
        window
    }

    fn status_window(&self) -> String {
        let width = 280;
        let mut window = title_bar(width, "Status");
        window.push_str(&button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "status", ""));
        window.push_str(&art("statuswnd/w_statwin_bg.bmp", 0, 17, 280, 124));
        let points = self.value(parameter::STATUS_POINTS);
        for stat in 0..6u16 {
            let top = 23 + 16 * i32::from(stat);
            let (value, bonus, cost) = (self.value(parameter::STR + stat), self.stat_bonus[usize::from(stat)], self.value(parameter::STR_COST + stat));
            window.push_str(&text(36, top, BLACK, &value.to_string()));
            if bonus != 0 {
                window.push_str(&text(if value >= 100 { 54 } else { 49 }, top, BLACK, &format!("{bonus:+}")));
            }
            // A stat can be raised while the points for it are there.
            if cost > 0 && cost <= points {
                window.push_str(&button(78, top + 1, 11, "basic_interface/arw_right.bmp", "basic_interface/arw_right.bmp", "raise", &stat.to_string()));
            }
            if cost > 0 {
                window.push_str(&text_ending(101, width, top, BLACK, &cost.to_string()));
            }
        }
        let pair = |first: u16, second: u16| format!("{} + {}", self.value(first), self.value(second));
        // The server keeps the delay between attacks; the window shows the speed it means.
        let attack_speed = (2000 - self.value(parameter::ATTACK_DELAY)) / 10;
        let left = [pair(parameter::ATK, parameter::ATK_BONUS), pair(parameter::MATK_MIN, parameter::MATK_MAX), self.value(parameter::HIT).to_string(), self.value(parameter::CRITICAL).to_string()];
        let right = [
            pair(parameter::DEF, parameter::DEF_BONUS),
            pair(parameter::MDEF, parameter::MDEF_BONUS),
            pair(parameter::FLEE, parameter::FLEE_BONUS),
            attack_speed.to_string(),
            points.to_string(),
        ];
        for (row, value) in left.iter().enumerate() {
            window.push_str(&text_ending(189, width, 22 + 16 * row as i32, BLACK, value));
        }
        for (row, value) in right.iter().enumerate() {
            window.push_str(&text_ending(275, width, 22 + 16 * row as i32, BLACK, value));
        }
        window
    }

    fn chat_lines(&self, lines: usize) -> String {
        let from = self.chat.len().saturating_sub(lines);
        self.chat[from..].iter().map(|(words, colour)| format!(r#"<div style="color:{colour}">{}</div>"#, escape(words))).collect()
    }

    fn chat_window(&self, lines: usize, height: i32) -> String {
        let log = if lines == 0 { String::new() } else { format!(r#"<div id="chat-log" class="a" style="{}">{}</div>"#, at(5, 18, 592, height - 44), self.chat_lines(lines)) };
        format!(
            r#"{log}<form data-submit="chat" class="a" style="{};background:url('{}') 0 0 no-repeat">
                <input class="a in" id="chat-input" maxlength="200" autocomplete="off" style="{}">
            </form>"#,
            at(0, height - 24, 600, 24),
            picture("basic_interface/dialog_bg.bmp"),
            at(110, 4, 460, 16),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::grouped;

    #[test]
    fn groups_thousands() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        assert_eq!(grouped(1234567), "1,234,567");
        assert_eq!(grouped(-45000), "-45,000");
    }
}
