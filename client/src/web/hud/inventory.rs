//! The inventory and what belongs to its items: the description window, the note of a thing just picked up,
//! the window that asks how many to drop, and the list of what a magnifier can identify.

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
        let Some((named, amount)) = &self.bag.obtained else { return String::new() };
        let Some((_, resource)) = data::item_as(named.id, named.identified) else { return String::new() };
        // The whole name, as in the chat (`UINotifyItemObtainWnd::SendMsg`, `UM_SETTITLE`: `DisplayName(0)`).
        let name = data::item_title(named, false);
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
        // In the middle of its place (260 wide), on a whole pixel.
        let left = (260 - width) / 2;
        [
            format!(r#"<div class="note" style="position:relative;display:block;left:{left}px;width:{width}px;height:32px;"#),
            format!(r#"text-align:left;animation-delay:-{shown:.2}s">{}"#, system_box(0, width, 32)),
            format!(r#"<div class="a cell" style="left:9px;top:1px">{}</div>"#, Self::icon(Some(&resource))),
            format!(r#"<div class="a t" style="left:41px;top:10px;white-space:nowrap">{words}</div></div>"#)
        ].concat()
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
        let large = if resource.is_empty() { String::new() } else { format!(
            r#"<img src="{}" alt="" onerror="this.remove()">"#,
            picture(&format!("collection/{resource}.bmp"))
        ) };
        let kept = data::item_moves(id).map(Self::kept_from).unwrap_or_default();
        let height = if kept.is_empty() { 120 } else { 135 };
        let slots = if identified && named.is_equipment() && !named.made_or_egg() { Self::slots_row(&named, height) } else { String::new() };
        let view = if data::card_picture(id).is_some() { picture_press(6, 6, "btn_view", Press::InfoCard) } else { String::new() };
        // On a book that is carried, "Read" and "Auto Read" stand there (`hud/books.rs`).
        let read = self.book_buttons(id);
        format!(
            r#"{}{}<div class="a" style="{}">{large}</div>
               <div class="a t" style="{};white-space:nowrap;overflow:hidden;color:{}">{}</div>
               <div class="a t viewer" style="{};white-space:normal;user-select:text">{description}</div>{kept}{slots}{view}{read}{}"#,
            system_box(0, 280, height),
            art("basic_interface/collection_bg.bmp", 0, 0, 280, 120),
            at(10, 11, 75, 100),
            at(90, 10, 170, 14),
            if identified { BLACK } else { "#808080" },
            escape(&data::item_title(&named, false)),
            at(91, 31, 182, 80),
            cross(280, Press::InfoClose),
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
                        r#"<div class="a" style="{}" data-info="{card}" {}>{}</div>"#,
                        at(x, top + 2, 24, 24),
                        tip(&data::item(card).map(|(name, _)| name).unwrap_or_default()),
                        Self::item_icon(card, true)
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
            cross(300, Press::IllustClose)
        )
    }

    /// The words of the inventory's four tabs (`UIItemWnd::OnCreate`: the client's lines 1465, 2050, 1471 and
    /// 2051).
    fn inventory_tabs() -> Vec<String> {
        [(1465, "Use"), (2050, "Eqp"), (1471, "Etc"), (2051, "Fav")].into_iter().map(|(line, otherwise)| Self::words(line, otherwise)).collect()
    }

    /// How tall the inventory's body is, between its title and the bar under it: as tall as its tabs, which is
    /// the least the client lets the window be (`UIItemWnd::SendMsg`, `UM_RESIZE`: its height is that of the
    /// tabs, whole rows of cells more, and 38). The window is not resized here, so this is its size.
    pub(in crate::web) fn inventory_body() -> i32 {
        tabs::side_tabs_tall(&Self::inventory_tabs(), 10, true)
    }

    /// The inventory (`UIItemWnd`): its title, its body with the four tabs at its left, the cells of what is
    /// carried under the open tab, and the bar at its bottom.
    pub(in crate::web) fn inventory_window(&self) -> String {
        let width = 40 + 32 * ITEM_COLUMNS + 16;
        let body = Self::inventory_body();
        let mut window = title_bar(width, "Inventory");
        window.push_str(&cross(width, Press::Window));
        window.push_str(&self.inventory_ground(width, body));
        window.push_str(&grid_of(ITEM_COLUMNS, body / 32, 17, &self.inventory_cells()));
        window.push_str(&self.inventory_bar(width, 17 + body));
        window
    }

    /// The inventory's body and its tabs.
    fn inventory_ground(&self, width: i32, body: i32) -> String {
        // The body (`UIItemWnd::Paint`): the piece at its left, white from 20 in to the cells and under their
        // last whole row, a grey line down at 20, and the piece at its right.
        let mut ground = format!(
            r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#,
            at(0, 17, 20, body),
            picture("basic_interface/itemwin_left.bmp")
        );
        ground.push_str(&format!(
            r#"<div class="a" style="{};background:#fff;border-left:1px solid #c0c0c0;box-sizing:border-box"></div>"#,
            at(20, 17, width - 24, body)
        ));
        ground.push_str(&format!(
            r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#,
            at(width - 20, 17, 20, body),
            picture("basic_interface/itemwin_right.bmp")
        ));
        // The four tabs (`UIItemWnd::OnCreate`: a `UITabControl` at 3, 18, with 10 round each tab's letters
        // and every tab as tall as the tallest; `hud/tabs.rs`), the last on the ground the client gives it
        // (`colorchip.bmp`, the cell 5, 2). A thing dragged onto one is kept under "Fav" from then, or no
        // more (`UIItemWnd::DragAndDrop`).
        let open = [Tab::Usable, Tab::Equipment, Tab::Other, Tab::Favourite].iter().position(|tab| *tab == self.bag.tab).unwrap_or(0);
        let tabs: Vec<(String, Option<&str>, String)> = Self::inventory_tabs()
            .into_iter()
            .enumerate()
            .map(|(index, words)| (
                words,
                (index == 3).then_some("#cedeff"),
                format!("{} {}", pressed(Press::Tab(index)), dropped(Drop::InventoryTab(index)))
            ))
            .collect();
        ground.push_str(&tabs::side_tabs(3, 18, 10, true, open, &tabs));
        ground
    }

    /// The cells of what is carried under the open tab (the grid shows rows of empty cells below the last
    /// item, so the window is never shorter than it shows).
    fn inventory_cells(&self) -> Vec<String> {
        // What is worn is in the Equipment window, as in the original; the arrows readied stay here too
        // (`Item::in_bag`). What is kept under "Fav" is on that tab
        // alone (`UIItemWnd::SendMsg`, `UM_REFRESHITEM`), in the order of its kinds and names while "Align
        // items" is on (`UIItemWnd::RefreshItems`): `inventory::shown`.
        crate::inventory::shown(&self.bag.items, self.bag.tab, &self.bag.favourites, self.bag.aligned, Self::item_name)
            .iter()
            .enumerate()
            .map(|(place, item)| {
                // With the storage open a double click puts the thing into it.
                let marks = format!(
                    "{} {}",
                    dragged(Drag::Item(item.id)),
                    used(if self.storage.open.is_some() { Use::Put(item.index) } else { Use::Item(item.index) })
                );
                grid_cell(place, ITEM_COLUMNS, item, item.amount, &marks, &amount_tip(&Self::item_name(item), item.amount))
            })
            .collect()
    }

    /// The bar at the inventory's bottom, `footer` down: how many kinds are carried, and "Align items".
    fn inventory_bar(&self, width: i32, footer: i32) -> String {
        let mut bar = button_bar(width, footer + 21);
        let kinds = self.bag.items.iter().filter(|item| item.in_bag()).count();
        // How many kinds of thing are carried, of how many there is room for: the smaller text
        // (`UIItemWnd::Paint`, (0, 11) at 35 in and 15 from the bottom).
        bar.push_str(&text_in(SMALL, 35, footer + 5, if kinds >= 100 { RED } else { "#232323" }, &format!("{kinds} / 100")));
        // "Align items" (`UIItemWnd::RefreshSortButton`: `inventory\bt_sort.bmp` on and `bt_sort_off.bmp` off, 25
        // by 17, 157 in and 19 from the bottom; it says the client's line 3007 with ON or OFF).
        let align = if self.bag.aligned { "inventory/bt_sort" } else { "inventory/bt_sort_off" };
        let says = format!(r#" title="{} {}""#, escape(&Self::words(3007, "Align items")), if self.bag.aligned { "ON" } else { "OFF" });
        bar.push_str(&pictured_by(Convention::OverPress, 157, footer + 2, (25, 17), align, Press::Align, &says));
        bar
    }

    /// The items a magnifier can identify (the client's `UIItemIdentifyWnd`, titled with its message 521): the
    /// picked one is identified with OK (or with a double click); none is picked at first.
    pub(in crate::web) fn identify_window(&self) -> String {
        let Some(items) = &self.bag.identify else { return String::new() };
        let does = Picking {
            pick: |row| Press::IdentifyPick(row).into(),
            used: Use::Identify,
            ok: Press::IdentifyOk.into(),
            cancel: Press::IdentifyCancel.into()
        };
        pick_list("Identify Item", &self.carried_rows(items), self.bag.identify_picked, does)
    }
}
