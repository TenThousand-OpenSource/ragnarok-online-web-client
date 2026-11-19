//! What an NPC's script shows on the page: the picture over the screen (the client's `UIIllustWnd`) and its
//! marks on the minimap (`CGameMode::DrawMiniMap`).

use super::*;
use crate::net::scripted::Stand;
use crate::press::scripted::Press;

impl App {
    /// The picture (`UIIllustWnd::SendMsg`, `UM_SETINFO`): a file of the client's `illust` folder, or of its
    /// `cardbmp` folder when there is none of that name in the first; the window is as large as the picture,
    /// whose magenta is see-through. The kinds in the middle of the screen have a cross, and one of them the
    /// title bar over the picture. A picture neither folder has shows nothing.
    pub(in crate::web) fn cutin_window(&self) -> String {
        let Some((name, stand)) = &self.scripted.cutin else { return String::new() };
        let bar = if *stand == Stand::Titled {
            [
                format!(r#"<div style="display:flex;height:17px"><img src="{}" alt="" draggable="false">"#, picture("basic_interface/titlebar_left.bmp")),
                format!(
                    r#"<div style="flex:1;background:url('{}') repeat-x"></div><img src="{}" alt="" draggable="false"></div>"#,
                    picture("basic_interface/titlebar_mid.bmp"),
                    picture("basic_interface/titlebar_right.bmp")
                )
            ].concat()
        } else {
            String::new()
        };
        // The cross stands 14 from the right edge and 3 down (`UIFrameWnd::AddCloseButton`).
        let cross = if matches!(stand, Stand::Titled | Stand::Centre) {
            format!(
                r#"<div class="a" style="right:14px;top:3px;width:0;height:0">{}</div>"#,
                press_button(0, 0, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::CutinClose)
            )
        } else {
            String::new()
        };
        [
            format!(r#"<div style="position:relative;display:inline-block;vertical-align:top">{bar}"#),
            format!(
                r#"<img src="{}" data-next="{}" alt="" draggable="false" style="display:block" "#,
                picture(&format!("illust/{}", escape(name))),
                picture(&format!("cardbmp/{}", escape(name)))
            ),
            format!(r#"onerror="if(this.dataset.next){{this.src=this.dataset.next;delete "#),
            format!(r#"this.dataset.next}}else{{this.parentNode.remove()}}">{cross}</div>"#)
        ].concat()
    }

    /// The script's marks on the minimap (`CGameMode::DrawMiniMap`, `DrawMiniMapMarker` of kind 3): a plus
    /// in the mark's colour, a bar 8 by 2 across one 2 by 8, its middle on the cell, shown for half of every
    /// second. One outside the part of the map shown is left out.
    pub(in crate::web) fn minimap_compass(&self) -> String {
        let Some(game) = self.game.as_ref().filter(|_| self.scripted.compass.0 == crate::web::world::visit()) else { return String::new() };
        self.scripted
            .compass
            .1
            .values()
            .filter_map(|mark| {
                let (x, y) = crate::web::view::minimap_spot(game, self.minimap_zoom, (mark.cell.0 as f32, mark.cell.1 as f32))?;
                let colour = crate::said::colour::css(mark.colour);
                let bar = |width: i32, height: i32| format!(
                    r#"<div style="position:absolute;left:{}px;top:{}px;width:{width}px;height:{height}px;background:{colour}"></div>"#,
                    -width / 2,
                    -height / 2
                );
                Some(format!(r#"<div class="blink" style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:0;height:0">{}{}</div>"#, bar(8, 2), bar(2, 8)))
            })
            .collect()
    }
}
