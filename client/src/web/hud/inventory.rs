//! The inventory and what belongs to its items: the description window, the note of a thing just picked up,
//! and the window that asks how many to drop.

use super::*;
use crate::item_names::Named;
use crate::press::inventory::Press;

impl App {
    /// How many of a stack to drop: the box that asks how many (`count_box`), in its first state.
    pub(in crate::web) fn throw_window(&self) -> String {
        self.count_box(&windows::THROW, self.bag.throwing, Form::Throw, Press::ThrowOk)
    }

    /// The note of a thing just picked up: its picture and "name - N obtained", fading after a moment.
    pub(in crate::web) fn obtained_note(&self) -> String {
        let Some((id, amount, identified)) = self.bag.obtained else { return String::new() };
        let Some((name, resource)) = data::item_as(id, identified) else { return String::new() };
        // It shows for five seconds from the moment the thing was picked up, however often the page is drawn
        // meanwhile: drawn anew, it goes on from where it was.
        let shown = (js_sys::Date::now() - self.bag.obtained_at) / 1000.0;
        if !(0.0..5.0).contains(&shown) {
            return String::new();
        }
        // The client's `UINotifyItemObtainWnd`: a system box 32 high, the item's picture 13 in and 5 down, the
        // words from 41 in and 10 down; it stays five seconds.
        let words = format!("{} - {amount} {}", escape(&name), tr("obtained."));
        let width = 41 + words.chars().count() as i32 * 6 + 12;
        format!(
            r#"<div class="note" style="position:relative;display:block;left:{left}px;width:{width}px;height:32px;text-align:left;animation-delay:-{shown:.2}s">{}<div class="a cell" style="left:9px;top:1px">{}</div><div class="a t" style="left:41px;top:10px;white-space:nowrap">{words}</div></div>"#,
            system_box(0, width, 32),
            Self::icon(Some(&resource)),
            // In the middle of its place (260 wide), on a whole pixel.
            left = (260 - width) / 2,
        )
    }

    /// What an item is, as the client shows it (`UIItemCollectionWnd`): its box of 280 by 120 with the
    /// collection's background, the item's large picture at 10, 11, its name at 90, 10 (grey until it is
    /// identified) and its description in a field of 182 by 80 at 91, 31 that scrolls. Under the description
    /// of an item the client's `itemmoveinfov5.txt` names, the underlined words "Item move restrictions"
    /// (line 2796), the box that much higher; under the box of equipment that is identified, a row of its
    /// four slots (`slots_row`); on a card, the button that opens its illustration, at 6, 6.
    pub(in crate::web) fn item_info_window(&self) -> String {
        let Some((id, identified)) = self.bag.info else { return String::new() };
        let resource = data::item_as(id, identified).map(|(_, resource)| resource).unwrap_or_default();
        // The thing itself, where one of that kind is carried: its cards and refinement are in its name and
        // its slots. (What is looked at is told by its number alone: of two alike, the first is shown.)
        let carried = self.bag.items.iter().find(|item| item.id == id && item.identified == identified);
        let named = carried.map_or(Named { id, identified, ..Named::default() }, Named::from);
        // Until it is identified an item has the description the client's table gives for it unidentified.
        let description = if identified { data::about(id).unwrap_or_default().0 } else { data::unidentified_about(id) };
        let description: String = description.iter().map(|line| format!("<div>{}</div>", coloured(line))).collect();
        let large = if resource.is_empty() { String::new() } else { format!(r#"<img src="{}" alt="" onerror="this.remove()">"#, picture(&format!("collection/{resource}.bmp"))) };
        let kept = data::item_moves(id).map(Self::kept_from).unwrap_or_default();
        let height = if kept.is_empty() { 120 } else { 135 };
        let slots = if identified && named.is_equipment() && !named.made_or_egg() { Self::slots_row(&named, height) } else { String::new() };
        let view = if data::card_picture(id).is_some() { picture_press(6, 6, "btn_view", Press::InfoCard) } else { String::new() };
        format!(
            r#"{}{}<div class="a" style="{}">{large}</div>
               <div class="a t" style="{};white-space:nowrap;overflow:hidden;color:{}">{}</div>
               <div class="a t viewer" style="{};white-space:normal;user-select:text">{description}</div>{kept}{slots}{view}{}"#,
            system_box(0, 280, height),
            art("basic_interface/collection_bg.bmp", 0, 0, 280, 120),
            at(10, 11, 75, 100),
            at(90, 10, 170, 14),
            if identified { BLACK } else { "#808080" },
            escape(&data::item_title(&named, false)),
            at(91, 31, 182, 80),
            press_button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::InfoClose),
        )
    }

    /// The words under a description that tell an item is kept from something (`UIItemMoveInfo`: line 2796,
    /// underlined, where the description ends); pointed at, they list from what (the client's round tooltip,
    /// `UM_SET_ITEM_MOVE_TOOLTIP`; here the browser's own). The words show for every item the client's table
    /// names, as in the client.
    fn kept_from(bits: u8) -> String {
        let lines: Vec<String> = crate::item_names::restrictions(bits).into_iter().filter_map(data::message).map(|line| tr(&line).to_owned()).collect();
        format!(
            r#"<div class="a t" style="left:91px;top:112px;white-space:nowrap;text-decoration:underline;cursor:default" title="{}">{}</div>"#,
            escape(&lines.join("\n")).replace("\n", "&#10;"),
            escape(&Self::words(2796, "Item move restrictions"))
        )
    }

    /// The row of slots under the description of equipment (`UIItemCollectionWnd::Paint`): a system box 29
    /// high under the window's own, and in it four places 25 apart from 4 in, 2 down: the picture of the
    /// card set there, `empty_card_slot.bmp` for a slot that is free, `disable_card_slot.bmp` where the item
    /// has no slot. The right button on a card there shows what the card is (`UIItemCollectionWnd::OnRBtnDown`;
    /// the client opens a second window for it, `WID_COMBINEDCARDITEMCOLLECTIONWND`: here the one window
    /// shows the card in place of the item).
    fn slots_row(item: &Named, top: i32) -> String {
        let free = i32::from(data::item_slots(item.id));
        let places: String = (0..4)
            .map(|slot| {
                let x = 4 + slot * 25;
                match item.cards[slot as usize] {
                    0 => art(if slot < free { "empty_card_slot.bmp" } else { "disable_card_slot.bmp" }, x, top + 2, 24, 24),
                    card => format!(
                        r#"<div class="a" style="{}" data-info="{card}" title="{}">{}</div>"#,
                        at(x, top + 2, 24, 24),
                        escape(&data::item(card).map(|(name, _)| name).unwrap_or_default()),
                        Self::icon(data::item(card).map(|(_, resource)| resource).as_deref())
                    ),
                }
            })
            .collect();
        format!("{}{places}", system_box(top, 280, 29))
    }

    /// A card's illustration (`UICardItemIllustWnd`, 300 by 417): a title bar with the card's name at 10, 3
    /// and the cross, and under it the picture of `cardbmp` that `num2cardillustnametable.txt` names for it.
    pub(in crate::web) fn card_illust_window(&self) -> String {
        let Some((card, file)) = self.bag.illust.and_then(|card| Some((card, data::card_picture(card)?))) else { return String::new() };
        let name = data::item(card).map(|(name, _)| name).unwrap_or_default();
        format!(
            r#"{}{}<img class="a" style="left:0;top:17px" src="{}" alt="" draggable="false" onerror="this.remove()">{}"#,
            title_bar(300, ""),
            text(10, 3, BLACK, &name),
            picture(&format!("cardbmp/{file}.bmp")),
            press_button(286, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::IllustClose)
        )
    }

    /// The picture of an item or a skill, or nothing while its name is not known.
    pub(in crate::web) fn icon(resource: Option<&str>) -> String {
        match resource {
            // An item this client's tables do not have is shown as the original shows it: an apple.
            Some(resource) if resource.is_empty() => Self::icon(Some("사과")),
            Some(resource) => format!(r#"<img src="{}" alt="" draggable="false" onerror="this.remove()">"#, picture(&format!("item/{resource}.bmp"))),
            None => String::new(),
        }
    }

    /// How many of an item the character carries without wearing them.
    pub(in crate::web) fn carried(&self, id: u32) -> u32 {
        self.bag.items.iter().filter(|item| item.id == id && item.worn == 0).map(|item| u32::from(item.amount)).sum()
    }

    /// An item's name as the client writes it everywhere (`ITEM_INFO::DisplayName`: `data::item_title`,
    /// `crate::item_names`): its refinement, its cards' words, its slots, who made it.
    pub(in crate::web) fn item_name(item: &Item) -> String {
        data::item_title(&item.into(), false)
    }

    pub(in crate::web) fn inventory_window(&self) -> String {
        let width = 40 + 32 * ITEM_COLUMNS + 16;
        let body = 32 * ITEM_ROWS;
        let mut window = title_bar(width, "Inventory");
        window.push_str(&press_button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Window));
        // The body: the tabs' strip, the cells, and the pieces at their sides.
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c3c3c3"></div>"#, at(0, 17, 40, body)));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(20, 17, 20, body), picture("basic_interface/itemwin_left.bmp")));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(width - 20, 17, 20, body), picture("basic_interface/itemwin_right.bmp")));
        // The four tabs (`UIItemWnd::OnCreate`: the client's lines 1465, 2050, 1471 and 2051). A thing dragged
        // onto one is kept under "Fav" from then, or no more (`UIItemWnd::DragAndDrop`).
        for (index, (tab, line, label)) in [(Tab::Usable, 1465, "Use"), (Tab::Equipment, 2050, "Eqp"), (Tab::Other, 1471, "Etc"), (Tab::Favourite, 2051, "Fav")].into_iter().enumerate() {
            let open = if tab == self.bag.tab { " open" } else { "" };
            let letters: String = Self::words(line, label).chars().map(|letter| format!("<span>{letter}</span>")).collect();
            window.push_str(&format!(
                r#"<button type="button" class="a tab{open}" style="{}" {} {}>{letters}</button>"#,
                // Four tabs down the 160 of the window's body: 40 each, where this client's three had 51.
                format!("{};padding-top:0", at(3, 18 + index as i32 * 40, 17, 40)),
                pressed(Press::Tab(index)),
                dropped(Drop::InventoryTab(index))
            ));
        }
        let mut cells = String::new();
        // What is worn is in the Equipment window, as in the original. What is kept under "Fav" is on that tab
        // alone (`UIItemWnd::SendMsg`, `UM_REFRESHITEM`), in the order of its kinds and names while "Align
        // items" is on (`UIItemWnd::RefreshItems`).
        let favourite = |item: &&Item| self.bag.favourites.contains(&item.index);
        let mut shown: Vec<&Item> = self.bag.items.iter().filter(|item| item.worn == 0 && if self.bag.tab == Tab::Favourite { favourite(item) } else { item.tab() == self.bag.tab && !favourite(item) }).collect();
        if self.bag.tab == Tab::Favourite && self.bag.aligned {
            shown.sort_by_cached_key(|item| (crate::net::items::sort_rank(item.kind), Self::item_name(item)));
        }
        for (place, item) in shown.iter().enumerate() {
            let (column, row) = (place as i32 % ITEM_COLUMNS, place as i32 / ITEM_COLUMNS);
            let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
            let worn = if item.worn != 0 { " worn" } else { "" };
            cells.push_str(&format!(
                r#"<div class="a cell{worn}" style="left:{}px;top:{}px" {} {} data-info="{info}" title="{} : {} {}">{}{}</div>"#,
                column * 32,
                row * 32,
                dragged(Drag::Item(item.id)),
                // With the storage open a double click puts the thing into it.
                used(if self.storage.open.is_some() { Use::Put(item.index) } else { Use::Item(item.index) }),
                escape(&Self::item_name(item)),
                item.amount,
                tr("ea"),
                Self::icon(resource.as_deref()),
                count(item.amount),
                info = if item.identified { item.id.to_string() } else { format!("{}?", item.id) }
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
        let kinds = self.bag.items.iter().filter(|item| item.worn == 0).count();
        // How many kinds of thing are carried, of how many there is room for: the smaller text
        // (`UIItemWnd::Paint`, (0, 11) at 35 in and 15 from the bottom).
        window.push_str(&text_in(SMALL, 35, footer + 5, if kinds >= 100 { "#f00" } else { "#232323" }, &format!("{kinds} / 100")));
        // "Align items" (`UIItemWnd::RefreshSortButton`: `inventory\bt_sort.bmp` on and `bt_sort_off.bmp` off, 25
        // by 17, 157 in and 19 from the bottom; it says the client's line 3007 with ON or OFF).
        let align = if self.bag.aligned { "inventory/bt_sort" } else { "inventory/bt_sort_off" };
        let says = format!(r#" title="{} {}""#, escape(&Self::words(3007, "Align items")), if self.bag.aligned { "ON" } else { "OFF" });
        window.push_str(&pictured(157, footer + 2, (25, 17), [&format!("{align}.bmp"), &format!("{align}_over.bmp"), &format!("{align}_press.bmp")], Press::Align, &says));
        window
    }
}
