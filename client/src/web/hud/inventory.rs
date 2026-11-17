//! The inventory and what belongs to its items: the description window, the note of a thing just picked up,
//! and the window that asks how many to drop.

use super::*;

impl App {
    /// Redraws the window that asks how many to drop, where it is on the page, and puts the typing into it.
    pub(in crate::web) fn refresh_throw(&self) {
        let Some(window) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.get_element_by_id("throw")) else { return };
        window.set_inner_html(&self.throw_window());
        if let Some(count) = window.query_selector("input").ok().flatten().and_then(|count| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlInputElement>(count).ok()) {
            let _ = count.focus();
            count.select();
        }
    }

    /// How many of a stack to drop: the original's small box (`UIItemDropCntWnd`, 182 by 46) with the item's
    /// name, the number to type over (all of them at first; a plain field of six characters) and OK.
    pub(in crate::web) fn throw_window(&self) -> String {
        let Some((index, most)) = self.bag.throwing else { return String::new() };
        let name = self.bag.items.iter().find(|item| item.index == index).map(Self::item_name).unwrap_or_default();
        format!(
            r#"{}<form data-submit="throw" class="a" style="{}">
                 <div class="a t" style="left:15px;top:6px;width:104px;overflow:hidden;white-space:nowrap">{}</div>
                 <input id="throw-count" class="a in" style="{};background:#e8e8e8" type="text" inputmode="numeric" maxlength="6" autocomplete="off" value="{most}">
                 {}
               </form>"#,
            system_box(0, 182, 46),
            at(0, 0, 182, 46),
            escape(&name),
            at(15, 22, 80, 16),
            picture_button(127, 14, "btn_ok", "throw-ok").replace(r#"type="button""#, r#"type="submit""#),
        )
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
    /// collection's background, the item's large picture at 10, 11, its name at 90, 10 and its description in a
    /// field of 182 by 80 at 91, 31 that scrolls.
    pub(in crate::web) fn item_info_window(&self) -> String {
        let Some((id, identified)) = self.bag.info else { return String::new() };
        let (name, resource) = data::item_as(id, identified).unwrap_or_else(|| (format!("{} {id}", tr("Item")), String::new()));
        // Until it is identified an item has the description the client's table gives for it unidentified.
        let description = if identified { data::about(id).unwrap_or_default().0 } else { data::unidentified_about(id) };
        let description: String = description.iter().map(|line| format!("<div>{}</div>", coloured(line))).collect();
        let large = if resource.is_empty() { String::new() } else { format!(r#"<img src="{}" alt="" onerror="this.remove()">"#, picture(&format!("collection/{resource}.bmp"))) };
        format!(
            r#"{}{}<div class="a" style="{}">{large}</div>
               <div class="a t" style="{};white-space:nowrap;overflow:hidden">{}</div>
               <div class="a t viewer" style="{};white-space:normal;user-select:text">{description}</div>{}"#,
            system_box(0, 280, 120),
            art("basic_interface/collection_bg.bmp", 0, 0, 280, 120),
            at(10, 11, 75, 100),
            at(90, 10, 170, 14),
            escape(&name),
            at(91, 31, 182, 80),
            button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "item-info-close", ""),
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

    /// An item's name as the game shows it: its refinement before it, its free slots after it.
    pub(in crate::web) fn item_name(item: &Item) -> String {
        let name = data::item_as(item.id, item.identified).map_or_else(|| format!("{} {}", tr("Item"), item.id), |(name, _)| name);
        if item.refine > 0 { format!("+{} {name}", item.refine) } else { name }
    }

    pub(in crate::web) fn inventory_window(&self) -> String {
        let width = 40 + 32 * ITEM_COLUMNS + 16;
        let body = 32 * ITEM_ROWS;
        let mut window = title_bar(width, "Inventory");
        window.push_str(&button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "inventory", ""));
        // The body: the tabs' strip, the cells, and the pieces at their sides.
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c3c3c3"></div>"#, at(0, 17, 40, body)));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(20, 17, 20, body), picture("basic_interface/itemwin_left.bmp")));
        window.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 repeat-y"></div>"#, at(width - 20, 17, 20, body), picture("basic_interface/itemwin_right.bmp")));
        for (index, (tab, label)) in [(Tab::Usable, "Use"), (Tab::Equipment, "Eqp"), (Tab::Other, "Etc")].into_iter().enumerate() {
            let open = if tab == self.bag.tab { " open" } else { "" };
            let letters: String = tr(label).chars().map(|letter| format!("<span>{letter}</span>")).collect();
            window.push_str(&format!(
                r#"<button type="button" class="a tab{open}" style="{}" data-click="inventory-tab" data-arg="{index}">{letters}</button>"#,
                at(3, 18 + index as i32 * 51, 17, 52)
            ));
        }
        let mut cells = String::new();
        // What is worn is in the Equipment window, as in the original.
        let shown: Vec<&Item> = self.bag.items.iter().filter(|item| item.tab() == self.bag.tab && item.worn == 0).collect();
        for (place, item) in shown.iter().enumerate() {
            let (column, row) = (place as i32 % ITEM_COLUMNS, place as i32 / ITEM_COLUMNS);
            let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
            let worn = if item.worn != 0 { " worn" } else { "" };
            cells.push_str(&format!(
                r#"<div class="a cell{worn}" style="left:{}px;top:{}px" draggable="true" data-drag="item:{}" data-use="{}:{}" data-info="{info}" title="{} : {} {}">{}<span class="n">{}</span></div>"#,
                column * 32,
                row * 32,
                item.id,
                if self.deal.storage.is_some() { "put" } else { "item" },
                item.index,
                escape(&Self::item_name(item)),
                item.amount,
                tr("ea"),
                Self::icon(resource.as_deref()),
                item.amount,
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
        window.push_str(&text(35, footer + 5, if kinds >= 100 { "#f00" } else { "#232323" }, &format!("{kinds} / 100")));
        window
    }
}
