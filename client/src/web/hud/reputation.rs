//! The reputation's window (the client's `UIReputeWnd`, 640 by 450).
//!
//! The client writes its colours blue first: each is written here as the page takes it, with the client's
//! own number beside it.

use super::*;
use crate::press::Step;
use crate::press::reputation::Press;
use crate::reputation::{self as rules, Filter, PER_PAGE, Repute};

/// The window's pictures (`유저인터페이스\Reputation\`).
fn piece(name: &str) -> String {
    format!("reputation/{name}.bmp")
}

/// The grounds of the window's boxes (239 grey) and their rims (99 grey).
const BOX: &str = "#efefef";
const BOX_RIM: &str = "#636363";

impl App {
    /// The reputations the window lists now, as numbers.
    pub(in crate::web) fn reputation_listed(&self) -> Vec<u64> {
        let tables = data::reputation();
        rules::listed(&tables.0, &tables.1, &self.reputation.points, &self.reputation.filter)
    }

    /// The window (`UIReputeWnd::OnCreate` and `Paint`): the title bar (line 3827) with its two small
    /// buttons; the drop-down of the groups at 8, 25 (122 by 18), the line to look for a name with at 136, 25
    /// (200 by 18, twenty letters) and its button (line 3777) at 336; eight reputations to a page in two
    /// columns; the page and how many there are between the two arrows at 567 and 623, 402 down; and Close
    /// (line 3045) at 589, 425 on the bar at the bottom. Until the server has told the points it says that it
    /// is loading instead of listing anything.
    pub(in crate::web) fn reputation_window(&self) -> String {
        let state = &self.reputation;
        let mut window = title_bar(640, &Self::words(3827, "Reputation Status"));
        window.push_str(&base_button(3, 3));
        window.push_str(&cross(640, Press::Window));
        // (The rim under the title is 0xC6C6C6, the line under the title black.)
        window.push_str(&format!(
            r#"<div class="a" style="{};box-sizing:border-box;background:#fff;border:1px solid #c6c6c6;border-top-color:#000"></div>"#,
            at(0, 16, 640, 415)
        ));
        window.push_str(&button_bar(640, 450));
        window.push_str(&Self::reputation_search());
        window.push_str(&basic_button(589, 425, 42, &Self::words(3045, "Close"), Some(Press::Window.into())));
        if !state.told {
            window.push_str(&Self::reputation_loading());
            window.push_str(&self.reputation_groups());
            return window;
        }
        let tables = data::reputation();
        let listed = self.reputation_listed();
        let (page, pages) = rules::paged(listed.len(), state.page);
        for (place, repute) in listed.iter().skip((page - 1) * PER_PAGE).take(PER_PAGE).filter_map(|id| tables.0.iter().find(|one| one.id == *id)).enumerate() {
            window.push_str(&self.reputation_card(repute, place as i32));
        }
        if state.group.is_some() && !matches!(state.filter, Filter::Sought(_)) {
            window.push_str(&Self::reputation_total(rules::total(&listed, &state.points)));
        }
        window.push_str(&Self::reputation_pages(page, pages));
        // Over the list: its lines lie on the first reputations while it is open.
        window.push_str(&self.reputation_groups());
        window
    }

    /// The line to look for a name with, and its button, a picture that says line 3777 when pointed at
    /// (`UIReputeWnd::OnCreate`; Enter in the line does what the button does: `m_message0`). That the
    /// button's label is told and not written on it is how the client's other picture buttons have theirs:
    /// `UIBitmapButton`'s drawing of a label was not read.
    fn reputation_search() -> String {
        let pictures = ["btn_search", "btn_search_over", "btn_search_press"].map(piece);
        [
            format!(r#"<form {} class="a" style="{}">"#, sent(Form::ReputeSearch), at(136, 25, 233, 18)),
            field(&windows::REPUTATION.field(), (0, 0, 200, 18), "", 20, &format!("box-sizing:border-box;background:{BOX};border:1px solid {BOX_RIM}")),
            format!(
                r#"<button type="submit" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-tip="{}"></button></form>"#,
                at(200, 0, 33, 18),
                picture(&pictures[0]),
                picture(&pictures[1]),
                picture(&pictures[2]),
                escape(&Self::words(3777, "Search"))
            )
        ].concat()
    }

    /// The drop-down of the groups (`UIComboBoxDefWnd`, 122 by 18 at 8, 25, its list up to 160 high): the
    /// group shown, and while it is open "all" (line 3829) and the groups under it. How the client draws the
    /// box's arrow was not read: the box alone here.
    fn reputation_groups(&self) -> String {
        let tables = data::reputation();
        let state = &self.reputation;
        let all = Self::words(3829, "Entire");
        let name = |place: Option<usize>| place.and_then(|place| tables.1.get(place)).map_or(all.clone(), |group| group.name.clone());
        let choice = |y: i32, words: &str, press: Press| {
            [
                format!(r#"<div class="a t opt" style="{};line-height:16px;padding:0 3px;box-sizing:border-box;background:{BOX};"#, at(8, y, 122, 18)),
                format!(r#"border:1px solid {BOX_RIM};cursor:default;overflow:hidden" {}>{}</div>"#, pressed(press), escape(words))
            ].concat()
        };
        let mut groups = choice(25, &name(state.group), Press::Groups);
        if state.choosing {
            let lines: String = std::iter::once(all.clone())
                .chain(tables.1.iter().map(|group| group.name.clone()))
                .enumerate()
                .map(|(place, words)| choice(17 * place as i32, &words, Press::Group(place)).replace("left:8px", "left:0px"))
                .collect();
            let height = 17 * (tables.1.len() as i32 + 1) + 1;
            groups.push_str(&format!(r#"<div class="a scroll" style="{};max-height:160px" {}>{lines}</div>"#, at(8, 42, 122, height), lines_stepped(17)));
        }
        groups
    }

    /// What the window shows until the points are there (`UIReputeWnd::Paint`): a rounded box 190 by 140 in
    /// its middle (rim 0xCE9484, ground 0xFFE7DE, white inside), the poring of the client's notices, and line
    /// 3830 under it, larger and bold (0xC89482).
    fn reputation_loading() -> String {
        [
            format!(
                r#"<div class="a" style="{};box-sizing:border-box;border-radius:4px;border:1px solid #8494ce;background:#dee7ff"></div>"#,
                at(225, 155, 190, 140)
            ),
            format!(r#"<div class="a" style="{};border-radius:3px;background:#fff"></div>"#, at(231, 161, 178, 128)),
            art("basic_interface/notice_poring.bmp", 288, 182, 65, 49),
            text_centred(Font(0, 15, true), 320, 260, "#8294c8", &Self::words(3830, "NOW LOADING..")),
        ]
        .concat()
    }

    /// One reputation (`UIReputeWnd::Paint`): its ground 310 by 82 (another for one a script marked), 316
    /// apart across and 86 down from 7, 50; its name 5 in and 6 down in bold; a light for every thousand it
    /// may reach 14 apart from 5, 22, so many of them lit, in red for points against; the bar of what is over
    /// the last whole thousand at 5, 50, 6 high and 299 at its most (0xCD9279, or red); and that remainder
    /// written into line 3828, ending at 304, 64 down, in bold.
    fn reputation_card(&self, repute: &Repute, place: i32) -> String {
        let (left, top) = ((place & 1) * 316 + 7, (place >> 1) * 86 + 50);
        let marked = self.reputation.marked.contains(&repute.id);
        let drawn = rules::drawn(repute, rules::points(&self.reputation.points, repute.id));
        let mut card = art(&piece(if marked { "bg_highlight" } else { "bg_info" }), 0, 0, 310, 82);
        card.push_str(&text_in(BOLD, 5, 6, BLACK, &repute.name));
        let lit = piece(if drawn.against { "light_red" } else { "light_blue" });
        for light in 0..drawn.lights.min(22) {
            card.push_str(&art(if light < drawn.lit { &lit } else { "reputation/light_empty.bmp" }, 5 + light as i32 * 14, 22, 12, 12));
        }
        card.push_str(&fill(5, 50, drawn.bar, 6, if drawn.against { RED } else { "#7992cd" }));
        card.push_str(&text_ending_in(BOLD, 304, 310, 64, BLACK, &crate::numbers::filled(&Self::words(3828, "%u / 1000 P"), &[&drawn.remainder.to_string()])));
        format!(r#"<div class="a" style="{}">{card}</div>"#, at(left, top, 310, 82))
    }

    /// The points of a group together (`UIReputeWnd::Paint`, while a group is picked): a box 258 by 18 at
    /// 374, 25 with "GROUP :" in it, a light by which way the sum goes, and the sum at its right, in bold.
    fn reputation_total(total: i64) -> String {
        let light = match total {
            0 => "light_empty",
            1.. => "light_blue",
            _ => "light_red",
        };
        [
            format!(r#"<div class="a" style="{};box-sizing:border-box;background:{BOX};border:1px solid {BOX_RIM}"></div>"#, at(374, 25, 258, 18)),
            text_in(BOLD, 377, 29, BLACK, "GROUP :"),
            art(&piece(light), 439, 28, 12, 12),
            text_ending_in(BOLD, 627, 640, 29, BLACK, &format!("{total} P")),
        ]
        .concat()
    }

    /// The page and how many there are, about 598, 402 down, in bold, between the arrow back at 567 and the
    /// arrow on at 623 (`UIReputeWnd::Paint`, `SetPage`): an arrow is there only while there is a page that
    /// way, and says where it leads when pointed at (lines 1053 and 1054).
    fn reputation_pages(page: usize, pages: usize) -> String {
        let arrow = |x: i32, name: &str, step: Step, line: usize, otherwise: &str| {
            let picture = piece(name);
            pictured(x, 402, (7, 11), [picture.as_str(); 3], Press::Page(step), &format!(r#" data-tip="{}""#, escape(&Self::words(line, otherwise))))
        };
        let mut pager = text_centred(BOLD, 598, 402, BLACK, &format!("{page} / {pages}"));
        if page > 1 {
            pager.push_str(&arrow(567, "arrow_prev", Step::Less, 1053, "Prev"));
        }
        if pages > page {
            pager.push_str(&arrow(623, "arrow_next", Step::More, 1054, "Next"));
        }
        pager
    }
}
