//! The Equipment window (the client's `UIEquipWnd`): what is worn, around the character, on its page "General",
//! and the costumes on its page "Costume".

use super::*;
use crate::press::inventory::Press;

/// Where a thing is worn (the server's bits) and where that shows: column and row.
const PLACES: [(u32, i32, i32); 10] = [(256, 0, 0), (512, 1, 0), (1, 0, 1), (16, 1, 1), (2, 0, 2), (32, 1, 2), (4, 0, 3), (64, 1, 3), (8, 0, 4), (128, 1, 4)];

/// The same for the page of costumes (`CSession`'s `costumeItems`, drawn in the same ten places by
/// `UIEquipWnd::DrawEquipSlots`): the three places of the head and the garment. The six places between them
/// are the client's shadow gear, which the classic game does not have.
const COSTUMES: [(u32, i32, i32); 4] = [(0x400, 0, 0), (0x800, 1, 0), (0x1000, 0, 1), (0x2000, 0, 3)];

impl App {
    /// The things worn among `items`, each in its place with its name beside it. The player's `own` are
    /// taken up and taken off; another's, in the window that shows them (`hud/viewequip.rs`), are only looked
    /// at.
    pub(super) fn worn_places(items: &[Item], own: bool) -> String {
        Self::worn_in(items, own, &PLACES)
    }

    /// The things worn in those places, each drawn where its place shows.
    fn worn_in(items: &[Item], own: bool, shown: &[(u32, i32, i32)]) -> String {
        let mut places = String::new();
        for (place, column, row) in shown.iter().copied() {
            // A thing worn in two places (a two-handed weapon) shows in both.
            let Some(item) = items.iter().find(|item| item.worn & place != 0) else { continue };
            let (icon_x, name_x) = if column == 0 { (6, 33) } else { (251, 172) };
            places.push_str(&[
                format!(
                    r#"<div class="a cell" style="left:{icon_x}px;top:{}px;width:24px;height:24px" {} data-info="{}" title="{}">"#,
                    38 + row * 26,
                    // Pressed, it is taken up as a thing of the inventory is (the client's `UIEquipWnd::OnLBtnDown`
                    // begins the same drag); a double click takes it off.
                    if own { format!("{} {}", dragged(Drag::Worn(item.index)), used(Use::Item(item.index))) } else { String::new() },
                    item.id,
                    escape(&Self::item_name(item))
                ),
                format!(
                    r#"<div style="position:absolute;left:-4px;top:-4px">{}</div></div>
                   "#,
                    Self::item_icon(item.id, true)
                ),
                format!(r#"<div class="a t" style="left:{name_x}px;top:{}px;width:74px;line-height:12px;white-space:normal;"#, 40 + row * 26),
                format!(r#"overflow:hidden;max-height:24px;{}">{}</div>"#, if column == 0 { "" } else { "text-align:right" }, escape(&Self::item_name(item)))
            ].concat());
        }
        places
    }

    /// What the character wears, laid out as the original's window (its sizes as rustnarok-client lists them):
    /// two columns of five places, the character between them. A double click on a place takes the thing off.
    /// A thing of the inventory let go anywhere on the window is put on (`UIEquipWnd::DragAndDrop`,
    /// `clicks/inventory.rs worn_let_go`).
    pub(in crate::web) fn equipment_window(&self) -> String {
        let mut window = self.equipment_frame();
        if self.bag.costumes {
            // On the page of costumes the client switches the "off" button off; here it is not drawn there.
            window.push_str(&Self::worn_in(&self.bag.items, true, &COSTUMES));
        } else {
            window.push_str(&self.equipment_off_button());
            // One who pushes a cart has the button of the cart's window here too (`hud/cart.rs`).
            window.push_str(&self.cart_equipment_button());
            window.push_str(&Self::worn_places(&self.bag.items, true));
            window.push_str(&self.equipment_ammunition());
        }
        format!("<div {}>{window}</div>", dropped(crate::press::things::Drop::Equipment))
    }

    /// What both pages of the window have: the title, the row of the pages' tabs, the picture with the
    /// character on it, and the box that lets others look.
    fn equipment_frame(&self) -> String {
        let mut window = title_bar(280, "Equipment");
        window.push_str(&cross(280, Press::Equipment));
        window.push_str(&side_rimmed(0, 17, 280, 20));
        window.push_str(&art("basic_interface/equipwin_bg2.bmp", 0, 37, 280, 150));
        // The pages (`UIEquipWnd::OnCreate`: a row of tabs 15 high under the title, the client's lines 1805
        // and 2419; its third, "Title", belongs to the achievements, which the classic game does not have).
        // The client measures a tab by its words; how it colours them was not read: the open one white here.
        for (page, (line, otherwise)) in [(1805, "General"), (2419, "Costume")].into_iter().enumerate() {
            let open = (page == 1) == self.bag.costumes;
            window.push_str(&format!(
                r#"<div class="a t" style="{};line-height:13px;text-align:center;box-sizing:border-box;border:1px solid #c3c3c3;{}cursor:default" {}>{}</div>"#,
                at(page as i32 * 59, 17, 60, 15),
                if open { "background:#fff;border-bottom-color:#fff;" } else { "background:#e8e8e8;" },
                pressed(Press::EquipmentPage(page)),
                escape(&Self::words(line, otherwise))
            ));
        }
        if self.bag.costumes {
            // The frame of the costumes' places (`equipwin_special.bmp`, 14 under where the first page's
            // picture begins less 17: 0, 34).
            window.push_str(&art("basic_interface/equipwin_special.bmp", 0, 34, 280, 133));
        }
        // Under it the window is 20 taller than the one that shows another's equipment (`UIEquipWnd::OnCreate`),
        // for the box that lets others look at this one (`hud/viewequip.rs`).
        window.push_str(&panel_ground(0, 187, 280, 20));
        window.push_str(&self.equipment_open_box(192));
        // The character stands on the shadow drawn into the picture.
        window.push_str(&figure_canvas("pc-equip", (61, 37, crate::web::figure::WIDTH as i32, 150), (0, -25)));
        window
    }

    /// The "off" button under the character (the client's `UIEquipWnd`: `btn_off.bmp`, 36 square, 18 left of the
    /// window's middle and 40 above its 167): it takes the character off its mount or out of its machine, or
    /// lets its falcon go, and says which when pointed at. It is there only while there is one of those to
    /// take away (the client's `IsEquipViewHidden` moves it out of the window otherwise).
    /// With a cart the client shows it too and it says "Take off Cart" (its line 223), or "Mounting/
    /// Dismounting" (3636) when there is a mount or a falcon as well (`UIEquipWnd::OnCreate`).
    fn equipment_off_button(&self) -> String {
        let cart = self.pushes_cart();
        let says = match crate::mounts::off(self.hero.state).1 {
            Some(_) if cart => Some(3636),
            None if cart => Some(223),
            says => says,
        };
        let Some(says) = says else { return String::new() };
        let tip = data::message(says).map(|line| format!(r#" data-tip="{}""#, escape(tr(&line)))).unwrap_or_default();
        format!(
            r#"<button type="button" class="a" style="{};border:0;padding:0;background:url('{}') no-repeat" {}{tip}></button>"#,
            at(122, 127, 36, 36),
            picture("basic_interface/btn_off.bmp"),
            pressed(Press::MountOff)
        )
    }

    /// The arrows readied (`UIEquipWnd::Paint`, after the character is drawn, so over it): their picture
    /// with no number on it, 25 left of the window's middle and 44 under where its page begins (115, 64),
    /// and their name from the same 115, in lines no wider than 40 (`DrawItemIcon`, `DrawItemName`). The
    /// client hands the name's place as 88, and `UIFrameWnd::DrawTextLines` begins the first of its two lines
    /// half a line (6) above the place handed, as it does for every name of this window: so the name begins 5
    /// above 88 here, as a place's name begins 5 above the client's 45 and 71, and lies over the bottom of
    /// the picture. The client does put both there, beside and over the character's head. A double click
    /// takes them off, as with anything worn.
    fn equipment_ammunition(&self) -> String {
        let Some(item) = self.bag.items.iter().find(|item| item.worn == crate::inventory::WORN_AS_AMMO) else { return String::new() };
        [
            format!(
                r#"<div class="a cell" style="left:115px;top:64px;width:24px;height:24px" {} data-info="{}" title="{}">"#,
                // Pressed, it is taken up as a thing of the inventory is (the client's `UIEquipWnd::OnLBtnDown`
                // begins the same drag); a double click takes it off.
                format!("{} {}", dragged(Drag::Worn(item.index)), used(Use::Item(item.index))),
                item.id,
                escape(&amount_tip(&Self::item_name(item), item.amount))
            ),
            format!(
                r#"<div style="position:absolute;left:-4px;top:-4px">{}</div></div>
                   "#,
                Self::item_icon(item.id, true)
            ),
            format!(r#"<div class="a t" style="left:115px;top:83px;width:40px;line-height:12px;white-space:normal;"#),
            format!(r#"overflow:hidden;max-height:24px;pointer-events:none">{}</div>"#, escape(&Self::item_name(item)))
        ].concat()
    }
}
