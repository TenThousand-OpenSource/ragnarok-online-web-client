//! The Tipbox: the client's tips, a page at a time.

use super::*;
use crate::press::Step;
use crate::press::tips::Press;

/// A page of a tip as HTML: its colours (`^rrggbb`) kept, a link to another tip (`<TIPBOX>name<INFO>number
/// </INFO></TIPBOX>`) made a press on its name, the other links of the client (`<NAVI>`, `<ITEM>`, `<URL>`)
/// left as their names, and the sign the lines begin with, which is one of the client's own font, a dot.
fn page_html(page: &str) -> String {
    let dotted: String = page.chars().map(|letter| if letter.is_ascii() || letter.is_alphanumeric() { letter } else { '·' }).collect();
    let mut html = String::new();
    let mut rest = dotted.as_str();
    while let Some(start) = rest.find('<') {
        html.push_str(&coloured(&rest[..start]));
        let Some(end) = rest[start..].find('>').map(|end| start + end) else {
            rest = &rest[start + 1..];
            continue;
        };
        let tag = &rest[start + 1..end];
        let closing = format!("</{tag}>");
        let Some(close) = rest[end..].find(&closing).map(|close| end + close) else {
            rest = &rest[end + 1..];
            continue;
        };
        let inside = &rest[end + 1..close];
        let (name, info) = inside.split_once("<INFO>").map_or((inside, ""), |(name, info)| (name, info.trim_end_matches("</INFO>")));
        match (tag, info.trim().parse::<u32>()) {
            ("TIPBOX", Ok(tip)) => html.push_str(&format!(r#"<span style="color:#3b488c;text-decoration:underline;cursor:pointer" {}>{}</span>"#, pressed(Press::Tip(tip)), escape(name))),
            _ => html.push_str(&escape(name)),
        }
        rest = &rest[close + closing.len()..];
    }
    html.push_str(&coloured(rest));
    html.replace('\n', "<br>")
}

impl App {
    /// The Tipbox (the client's `UITipboxWnd`, 480 by 365): the title (line 3238 of the message table); the
    /// picture `tipbox\TipboxBG.bmp` at the left under it and `bg_tipbox.bmp` at 241; at (253, 22) the line to
    /// search with, 193 wide; the tip's title at (254, 50) in bold blue (44, 77, 160), what stands after a `#`
    /// in it left out; its text in a box 226 by 245 at (250, 70); the page and how many there are at (426,
    /// 320) between the arrows at 415 and 462; the bar `btnbar_bottom.bmp` at 337 with the box "Open on
    /// Startup" at (9, 347) and Close at (415, 341). What a search found is listed in a box 216 by 266 at
    /// (250, 50), each a press. (The client's left half shows a picture that goes with the tip, and its
    /// button that writes a link to the tip into the chat is not here.)
    pub(in crate::web) fn tip_window(&self) -> String {
        let tips = data::tips();
        let mut window = title_bar(480, &Self::words(3238, "TIPBOX"));
        window.push_str(&press_button(465, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Window));
        window.push_str(&art("tipbox/TipboxBG.bmp", 0, 17, 241, 320));
        window.push_str(&art("tipbox/bg_tipbox.bmp", 241, 17, 239, 320));
        window.push_str(&art("tipbox/btnbar_bottom.bmp", 0, 337, 480, 28));
        window.push_str(&format!(
            r#"<form {} class="a" style="{}"><input class="a in" id="{}" maxlength="150" autocomplete="off" placeholder="{}" style="{};background:#fff"><button type="submit" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')"></button></form>"#,
            sent(Form::TipSearch),
            at(253, 21, 222, 18),
            windows::TIPS.field(),
            escape(&Self::words(3253, "Please enter a search term")),
            at(0, 1, 193, 15),
            at(195, 0, 26, 18),
            picture("tipbox/btn_searchbar_normal.bmp"),
            picture("tipbox/btn_searchbar_over.bmp"),
            picture("tipbox/btn_searchbar_press.bmp"),
        ));
        let blue = "rgb(44,77,160)";
        if let Some(found) = &self.tips.found {
            let rows: String = found
                .iter()
                .filter_map(|id| tips.iter().find(|tip| tip.id == *id))
                .map(|tip| format!(r#"<div class="t" style="height:16px;cursor:pointer;white-space:nowrap;overflow:hidden" {}>{}</div>"#, pressed(Press::Tip(tip.id)), escape(tip.title.split('#').next().unwrap_or_default())))
                .collect();
            window.push_str(&format!(r#"<div class="a scroll" style="{};overflow-y:auto;background:#f0f0f0">{rows}</div>"#, at(250, 50, 216, 266)));
        } else if let Some(tip) = tips.iter().find(|tip| tip.id == self.tips.tip) {
            let pages = tip.pages.len().max(1);
            let page = self.tips.page.min(pages - 1);
            window.push_str(&format!(r#"<div class="a t" style="{};color:{blue};{};white-space:normal;line-height:18px">{}</div>"#, at(254, 50, 220, 36), Font(0, 13, true).style(), escape(tip.title.split('#').next().unwrap_or_default())));
            window.push_str(&format!(
                r#"<div class="a t scroll" style="{};overflow-y:auto;white-space:normal;line-height:15px;color:{BLACK}">{}</div>"#,
                at(250, 90, 226, 225),
                tip.pages.get(page).map(|page| page_html(page)).unwrap_or_default()
            ));
            window.push_str(&pictured(415, 319, (11, 12), ["tipbox/bt_arrowL_off.bmp", "tipbox/bt_arrowL_on.bmp", "tipbox/bt_arrowL_on.bmp"], Press::Page(Step::Less), ""));
            window.push_str(&pictured(462, 319, (11, 12), ["tipbox/bt_arrowR_off.bmp", "tipbox/bt_arrowR_on.bmp", "tipbox/bt_arrowR_on.bmp"], Press::Page(Step::More), ""));
            window.push_str(&format!(r#"<div class="a t" style="{};text-align:center;color:{blue}"><b>{}</b>/{pages}</div>"#, at(426, 320, 36, 14), page + 1));
        }
        window.push_str(&format!(
            r#"<div class="a t" style="left:9px;top:347px;height:12px;line-height:12px;padding-left:14px;cursor:default;white-space:nowrap;color:{BLACK};background:url('{}') 0 1px no-repeat" {}>{}</div>"#,
            picture(if self.tips.not_at_start { "tipbox/checkBox.bmp" } else { "tipbox/checkBox_check.bmp" }),
            pressed(Press::Startup),
            escape(&Self::words(3939, "Open on Startup"))
        ));
        window.push_str(&pictured_by(Convention::NormalOverPress, 415, 341, (60, 20), "tipbox/btn_close", Press::Window, ""));
        window.push_str(&format!(r#"<div class="a t" style="{};text-align:center;line-height:20px;pointer-events:none;color:{BLACK}">{}</div>"#, at(415, 341, 60, 20), escape(&Self::words(1299, "Close"))));
        window
    }
}
