//! The storage's window (the client's `UIItemStoreWnd`: the Kafra's storage, and a guild's under its own
//! name), the windows that show one of its tabs apart (`UIItemStoreSubWnd`), the one that lists what a search
//! found (`UIItemStoreFindWnd`), and the box that asks how many go in or out.

use super::*;
use crate::storage::{Sorted, TABS, TAB_LINES};
use crate::press::storage::Press;
use crate::web::app::Stowing;

/// The pictures of the eight small buttons under the list, as the client names them (`basic_interface\<name>-1.bmp`
/// pressed in, `-2` not).
const APART: [&str; TABS] = ["소비", "캐시", "방어구", "무기", "의상", "투사체", "카드", "기타"];

impl App {
    /// The things of the storage a list shows, in the order chosen: those under a tab, or all of them, and of
    /// those the ones whose name has what was searched for in it (`UIItemStoreWnd::FilterByName`: as it is
    /// written, capitals and all). `room` is how wide a name may be, 40 from its cell: 180 in the storage's
    /// own list.
    fn storage_rows(&self, tab: Option<usize>, sought: Option<&str>, room: i32) -> String {
        let Some(storage) = &self.storage.open else { return String::new() };
        crate::storage::listed(&storage.items, tab, sought, self.storage.sorted, Self::item_name)
            .iter()
            .map(|(name, item)| {
                let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
                item_row(&Row {
                    resource: resource.as_deref(),
                    amount: Some(u32::from(item.amount)),
                    name,
                    name_at: (40, room),
                    aside: None,
                    marks: &format!("{} {} {}", dragged(Drag::Stored(item.index)), used(Use::Take(item.index)), item_marks(item.id, item.identified)),
                })
            })
            .collect()
    }

    /// The storage (`UIItemStoreWnd::OnCreate` and `Paint`, 280 by 458): the eight tabs down the left side
    /// (`tab_item_NN.bmp`, 20 by 235 from 0, 18, a picture to each tab that is open), twelve rows of the open
    /// tab's things, under them the eight buttons that show a tab in a window of its own (23 by 23, 26 apart
    /// from 5, 52 from the bottom) and the box of the lists' order (214 in, 62 by 18), and on the bar at the
    /// bottom how full the storage is, the search and "close". The title is the name the server gives the
    /// storage. The window takes what is dragged out of the inventory.
    pub(in crate::web) fn storage_window(&self) -> String {
        let Some(storage) = &self.storage.open else { return String::new() };
        let (width, height) = (280, 458);
        let title = if storage.name.is_empty() { Self::words(169, "Storage") } else { storage.name.clone() };
        let mut window = list_frame(width, height, &title);
        window.push_str(&cross(width, Press::Close));
        window.push_str(&art(&format!("basic_interface/tab_item_{:02}.bmp", self.storage.tab + 1), 0, 18, 20, 235));
        for (tab, (line, otherwise)) in TAB_LINES.iter().enumerate() {
            // A tab of the picture is an eighth of its height (`UITab::OnLBtnDown`).
            window.push_str(&format!(
                r#"<div class="a" style="{};cursor:pointer" title="{}" {}></div>"#,
                at(0, 18 + tab as i32 * 29, 20, 29),
                escape(&Self::words(*line, otherwise)),
                pressed(Press::Tab(tab))
            ));
            let shown = if self.storage.apart & (1 << tab) != 0 { 1 } else { 2 };
            window.push_str(&format!(
                r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat;cursor:pointer" title="{}" {}></div>"#,
                at(tab as i32 * 26 + 5, height - 52, 23, 23),
                picture(&format!("basic_interface/{}-{shown}.bmp", APART[tab])),
                escape(&Self::words(*line, otherwise)),
                pressed(Press::Apart(tab))
            ));
        }
        window.push_str(&list_of(40, 225, 12, &self.storage_rows(Some(self.storage.tab), None, 180), ""));
        window.push_str(&self.storage_order(214, height - 52));
        window.push_str(&Self::storage_fullness(storage.count, height));
        // The search (`bg_search.bmp`, 77 in and 23 from the bottom; its button 185 in) and "close" (220 in).
        window.push_str(&[
            format!(
                r#"<form {} class="a" style="{};background:url('{}') 0 0 no-repeat">"#,
                sent(Form::StorageSearch),
                at(77, height - 23, 122, 18),
                picture("bg_search.bmp")
            ),
            format!(
                r#"<input id="{}" class="a in" style="{}" type="text" maxlength="20" autocomplete="off"></form>"#,
                windows::STORAGE.field(),
                at(2, 1, 104, 16)
            )
        ].concat());
        window.push_str(&pictured_by(Convention::NormalOverPress, 185, height - 23, (23, 18), "bt_search", Press::Search, ""));
        window.push_str(&picture_press(220, height - 23, "btn_close", Press::Close));
        format!("<div {}>{window}</div>", dropped(Drop::Storage))
    }

    /// The box that says how the lists are ordered (`UIItemStoreComboBoxWnd`, 62 by 18): the order chosen,
    /// and while it is open its three choices under it. How the client draws the box was not read: a white
    /// box with a grey rim here.
    fn storage_order(&self, x: i32, y: i32) -> String {
        let choice = |y: i32, sorted: Sorted, press: Press| {
            let (line, otherwise) = sorted.line();
            [
                format!(r#"<div class="a t opt" style="{};line-height:16px;padding:0 3px;box-sizing:border-box;background:#fff;"#, at(x, y, 62, 18)),
                format!(
                    r#"border:1px solid #9a9a9a;cursor:default;overflow:hidden;white-space:nowrap" {}>{}</div>"#,
                    pressed(press),
                    escape(&Self::words(line, otherwise))
                )
            ].concat()
        };
        let mut order = choice(y, self.storage.sorted, Press::Sorting);
        if self.storage.sorting {
            for (place, sorted) in Sorted::ALL.into_iter().enumerate() {
                order.push_str(&choice(y + 17 * (place as i32 + 1), sorted, Press::Sort(place)));
            }
        }
        order
    }

    /// How full the storage is (`UIItemStoreWnd::Paint`): `icon_num.bmp` 6 in and 21 from the bottom, the
    /// number of kinds after it in the smaller text, red once the storage is full, and the most that fit.
    fn storage_fullness((kinds, most): (u16, u16), height: i32) -> String {
        [
            format!(
                r#"{}<div class="a t" style="left:22px;top:{}px;color:#000;white-space:nowrap;{}">"#,
                art("inventory/icon_num.bmp", 6, height - 21, 11, 14),
                height - 19,
                SMALL.style()
            ),
            format!(r#"<span style="color:{}">{kinds}</span><span style="margin-left:1px">/{most}</span></div>"#, if kinds >= most { RED } else { "#232323" })
        ].concat()
    }

    /// A smaller window of the storage's things (`UIItemStoreFindWnd::Paint`, which `UIItemStoreSubWnd` shares):
    /// the title, white under it, the rows with their cell 10 in, the bar at the bottom with a line 16 in and
    /// 20 from the bottom; a name has the window's width less 76. It takes what is dragged out of the
    /// inventory as the storage does.
    fn storage_part(width: i32, height: i32, title: &str, rows: &str, footer: &str, close: Option<Press>) -> String {
        format!(
            "<div {}>{}{}{}{}{}</div>",
            dropped(Drop::Storage),
            title_bar(width, title),
            panel(width, height),
            list_of(10, width - 25, (height - 38) / 32, rows, ""),
            text(16, height - 20, BLACK, footer),
            close.map_or(
                String::new(),
                |close| press_button(width - 16, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", close)
            ),
        )
    }

    /// The window that shows one tab apart (`UIItemStoreSubWnd`, 140 by 145): its title is the tab's name, and
    /// its bar says how full the storage is.
    pub(in crate::web) fn storage_apart_window(&self, tab: usize) -> String {
        let Some(storage) = &self.storage.open else { return String::new() };
        let (line, otherwise) = TAB_LINES[tab.min(TABS - 1)];
        Self::storage_part(
            140,
            145,
            &Self::words(line, otherwise),
            &self.storage_rows(Some(tab), None, 140 - 76),
            &format!("{}/{}", storage.count.0, storage.count.1),
            None
        )
    }

    /// The window of what a search found (`UIItemStoreFindWnd`, 280 by 312; its title is the client's line
    /// 1095): everything in the storage whose name has the words in it.
    pub(in crate::web) fn storage_found_window(&self) -> String {
        let Some(sought) = &self.storage.found else { return String::new() };
        Self::storage_part(280, 312, &Self::words(1095, "Search"), &self.storage_rows(None, Some(sought), 280 - 76), "", Some(Press::FoundClose))
    }

    /// How many of a thing go into the storage or out of it: the box that asks how many, with the thing's name.
    pub(in crate::web) fn storage_count_window(&self) -> String {
        let Some((stowing, most)) = self.storage.counting else { return String::new() };
        let item = match stowing {
            Stowing::In(index) => self.bag.items.iter().find(|item| item.index == index),
            Stowing::Out(index) => self.storage.open.as_ref().and_then(|storage| storage.items.iter().find(|item| item.index == index)),
        };
        count_box_of(&windows::STORAGE_COUNT, &item.map(Self::item_name).unwrap_or_default(), u32::from(most), Form::StorageCount, Press::CountOk)
    }
}
