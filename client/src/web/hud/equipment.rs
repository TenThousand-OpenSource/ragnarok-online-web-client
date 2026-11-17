//! The Equipment window: what is worn, around the character.

use super::*;

impl App {
    /// What the character wears, laid out as the original's window (its sizes as rustnarok-client lists them):
    /// two columns of five places, the character between them. A double click on a place takes the thing off.
    pub(in crate::web) fn equipment_window(&self) -> String {
        // Where a thing is worn (the server's bits) and where that shows: column and row.
        const PLACES: [(u32, i32, i32); 10] = [(256, 0, 0), (512, 1, 0), (1, 0, 1), (16, 1, 1), (2, 0, 2), (32, 1, 2), (4, 0, 3), (64, 1, 3), (8, 0, 4), (128, 1, 4)];
        let mut window = title_bar(280, "Equipment");
        window.push_str(&button(266, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", "equipment", ""));
        window.push_str(&format!(r#"<div class="a" style="{};background:#fff;border-left:1px solid #c8c8c8;border-right:1px solid #c8c8c8;box-sizing:border-box"></div>"#, at(0, 17, 280, 20)));
        window.push_str(&art("basic_interface/equipwin_bg2.bmp", 0, 37, 280, 150));
        // The character stands on the shadow drawn into the picture.
        window.push_str(&format!(
            r#"<div class="a" style="{};overflow:hidden;pointer-events:none"><canvas id="pc-equip" width="{}" height="{}" style="position:absolute;left:0;top:-25px"></canvas></div>"#,
            at(61, 37, super::super::figure::WIDTH as i32, 150),
            super::super::figure::WIDTH,
            super::super::figure::HEIGHT
        ));
        for (place, column, row) in PLACES {
            // A thing worn in two places (a two-handed weapon) shows in both.
            let Some(item) = self.bag.items.iter().find(|item| item.worn & place != 0) else { continue };
            let resource = data::item(item.id).map(|(_, resource)| resource);
            let (icon_x, name_x) = if column == 0 { (6, 33) } else { (251, 172) };
            window.push_str(&format!(
                r#"<div class="a cell" style="left:{icon_x}px;top:{}px;width:24px;height:24px" data-use="item:{}" data-info="{}" title="{}"><div style="position:absolute;left:-4px;top:-4px">{}</div></div>
                   <div class="a t" style="left:{name_x}px;top:{}px;width:74px;line-height:12px;white-space:normal;overflow:hidden;max-height:24px;{}">{}</div>"#,
                38 + row * 26,
                item.index,
                item.id,
                escape(&Self::item_name(item)),
                Self::icon(resource.as_deref()),
                40 + row * 26,
                if column == 0 { "" } else { "text-align:right" },
                escape(&Self::item_name(item)),
            ));
        }
        // The ammunition in use (arrows): its picture at 115, 64 under the character's hands and its name under
        // it, 40 wide, as the client's window has them. A double click takes it off, as with anything worn.
        if let Some(item) = self.bag.items.iter().find(|item| item.kind == 10 && item.worn != 0) {
            let resource = data::item(item.id).map(|(_, resource)| resource);
            window.push_str(&format!(
                r#"<div class="a cell" style="left:115px;top:64px;width:24px;height:24px" data-use="item:{}" data-info="{}" title="{} : {} {}"><div style="position:absolute;left:-4px;top:-4px">{}</div><span class="n">{}</span></div>
                   <div class="a t" style="left:107px;top:88px;width:40px;line-height:12px;text-align:center;white-space:nowrap;overflow:hidden">{}</div>"#,
                item.index,
                item.id,
                escape(&Self::item_name(item)),
                item.amount,
                tr("ea"),
                Self::icon(resource.as_deref()),
                item.amount,
                escape(&Self::item_name(item)),
            ));
        }
        window
    }
}
