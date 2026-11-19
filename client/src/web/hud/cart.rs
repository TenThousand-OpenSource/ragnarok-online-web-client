//! The merchant's cart: its window (`UIMerchantItemWnd`), the box that asks how many go into it or out of it,
//! the Equipment window's button that opens it (`UIEquipWnd`), and the window of the looks it may be given
//! (`UISelectCartWnd`). Here too is the grid of things that the client's windows made of `UIItemBaseWnd`
//! share with it (what the cart has for a shop being made, one's own shop: `hud/vending.rs`).

use super::*;
use crate::press::cart::Press;
use crate::web::app::Carting;

/// A thing in a cell of a grid (`UIItemBaseWnd::Paint`): its picture with how many there are, in the cell at
/// that `place` of a grid `columns` wide; `marks` is what the cell's tag says more (what it does when
/// dragged), and `tip` what it says when pointed at.
pub(super) fn grid_cell(place: usize, columns: i32, item: &Item, amount: u16, marks: &str, tip: &str) -> String {
    let resource = data::item_as(item.id, item.identified).map(|(_, resource)| resource);
    format!(
        r#"<div class="a cell" style="left:{}px;top:{}px" {marks} {} title="{}">{}{}</div>"#,
        place as i32 % columns * 32,
        place as i32 / columns * 32,
        item_marks(item.id, item.identified),
        escape(tip),
        App::icon(resource.as_deref()),
        count(amount)
    )
}

/// The grid of such a window: `columns` cells of 32 across from 40 in, `rows` of them showing under the
/// title, scrolled when it has more (the client scrolls it by a bar at the right).
pub(super) fn grid_of(columns: i32, rows: i32, cells: &[String]) -> String {
    let filled = (cells.len().div_ceil(columns.max(1) as usize) as i32).max(rows);
    format!(
        r#"<div class="a scroll" style="{};background:url('{}')"><div style="position:relative;height:{}px">{}</div></div>"#,
        at(40, 17, 32 * columns, 32 * rows),
        picture("basic_interface/itemwin_mid.bmp"),
        filled * 32,
        cells.concat()
    )
}

/// The cross in the corner of a window `width` wide.
pub(super) fn cross(width: i32, press: impl Into<crate::press::Press>) -> String {
    press_button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", press)
}

impl App {
    /// The cart's window, 280 by 120 as the client first makes it (`UIWindowMgr::EnsureWindow`,
    /// `WID_MERCHANTITEMWND`; the client lets it be drawn out to 320 by 240, this one stays as it is and
    /// scrolls): its title (the client's line 222), seven cells across and two rows, and under them how many
    /// kinds of things are in it of how many fit (`icon_num.bmp` at 10, the number at 23) and what they weigh
    /// of what it bears (`icon_weight.bmp` at 75, the number at 90), each number red once it is all there may
    /// be (`UIMerchantItemWnd::Paint`). The weights are written as the client keeps them, a tenth of the
    /// server's. A thing is taken up out of it unless a shop is being made (`OnLBtnDown`); the window takes
    /// what is dragged out of the inventory and the storage.
    pub(in crate::web) fn cart_window(&self) -> String {
        let (width, height) = (280, 120);
        let taken = self.vending.making.is_none();
        let cells: Vec<String> = self
            .cart
            .items
            .iter()
            .enumerate()
            .map(|(place, item)| {
                let marks = if taken { dragged(Drag::Carted(item.index)) } else { String::new() };
                grid_cell(place, 7, item, item.amount, &marks, &format!("{} : {} {}", Self::item_name(item), item.amount, tr("ea")))
            })
            .collect();
        let fill = self.cart.fill;
        let of = |x: i32, y: i32, has: u32, most: u32| {
            format!(
                r#"<div class="a t" style="left:{x}px;top:{y}px;white-space:nowrap;color:#000;{}"><span style="color:{}">{has}</span>/{most}</div>"#,
                SMALL.style(),
                if has >= most { "#f00" } else { "#232323" }
            )
        };
        format!(
            r#"<div {}>{}{}{}<img class="a" style="left:10px;top:{}px" src="{}" alt="">{}<img class="a" style="left:75px;top:{}px" src="{}" alt="">{}</div>"#,
            dropped(Drop::Cart),
            list_frame(width, height, &Self::words(222, "Cart Items")),
            cross(width, Press::Window),
            grid_of(7, 2, &cells),
            height - 22,
            picture("inventory/icon_num.bmp"),
            of(23, height - 19, u32::from(fill.kinds), u32::from(fill.most_kinds)),
            height - 20,
            picture("inventory/icon_weight.bmp"),
            of(90, height - 19, fill.weight / 10, fill.most_weight / 10),
        )
    }

    /// How many of a thing go into the cart or out of it: the box that asks how many, with the thing's name.
    pub(in crate::web) fn cart_count_window(&self) -> String {
        let Some((carting, most)) = self.cart.counting else { return String::new() };
        let item = match carting {
            Carting::In(index) => self.bag.items.iter().find(|item| item.index == index),
            Carting::Out(index) | Carting::ToStorage(index) => self.cart.items.iter().find(|item| item.index == index),
            Carting::FromStorage(index) => self.storage.open.as_ref().and_then(|storage| storage.items.iter().find(|item| item.index == index)),
        };
        count_box_of(&windows::CART_COUNT, &item.map(Self::item_name).unwrap_or_default(), u32::from(most), Form::CartCount, Press::CountOk)
    }

    /// The Equipment window's button that opens the cart's window, for one who pushes a cart (`UIEquipWnd`:
    /// `btn_items_off.bmp`, `btn_items_on.bmp` under the pointer; 14 left of the window's middle and 60 above
    /// the line its "off" stands 40 above; it says the client's line 237). Nothing for one without a cart.
    pub(super) fn cart_equipment_button(&self) -> String {
        if self.cart.kind == 0 {
            return String::new();
        }
        format!(
            r#"<button type="button" class="a" style="left:126px;top:107px;border:0;padding:0;background:none" {} data-tip="{}"><img src="{}" alt="" draggable="false" onmouseover="this.src='{}'" onmouseout="this.src='{}'"></button>"#,
            pressed(Press::Window),
            escape(&Self::words(237, "Cart Items [Alt+W]")),
            picture("basic_interface/btn_items_off.bmp"),
            picture("basic_interface/btn_items_on.bmp"),
            picture("basic_interface/btn_items_off.bmp"),
        )
    }

    /// The window of the looks a cart may be given (`UISelectCartWnd`): 385 wide and 98 high for every five
    /// looks, its title the client's line 659; each look a button 100 square with the first picture of that
    /// cart's sprite (`AddCart`). Where in its button the client's sprite button draws the cart was not
    /// found: here the cart stands on the button's middle.
    pub(in crate::web) fn cart_looks_window(&self) -> String {
        let Some(looks) = &self.cart.looks else { return String::new() };
        let height = (looks.kinds.len().max(1) as i32 - 1) / 5 * 98 + 98;
        let buttons: String = looks
            .kinds
            .iter()
            .enumerate()
            .filter_map(|(place, kind)| {
                let (_, (x, y)) = crate::carts::offered(*kind, place)?;
                let cart = looks.pictures.get(kind).map_or(String::new(), |(picture, origin)| {
                    format!(r#"<img src="{picture}" alt="" draggable="false" style="position:absolute;left:{:.0}px;top:{:.0}px;image-rendering:pixelated">"#, 50.0 - origin.0, 50.0 - origin.1)
                });
                Some(format!(r#"<button type="button" class="a" style="{};border:0;padding:0;background:none;overflow:hidden" {}>{cart}</button>"#, at(x, y, 100, 100), pressed(Press::Look(*kind))))
            })
            .collect();
        format!(
            r#"{}<div class="a" style="{};background:#fff;border:1px solid #c8c8c8;border-top:0;box-sizing:border-box"></div><div class="a" style="{};overflow:hidden">{buttons}</div>{}"#,
            title_bar(385, &Self::words(659, "Cart List")),
            at(0, 17, 385, height - 17),
            at(0, 0, 385, height),
            cross(385, Press::LooksClose),
        )
    }
}
