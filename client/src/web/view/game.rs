//! The game's screen: the minimap in its corner with the character's place under it, over the game's own windows.

use super::*;

/// The pictures of the marks of who stands where in a town, by the kind of each (`information\<name>.bmp`).
pub(in super::super) const TOWN_PICTURES: [&str; 8] = ["store", "weaponshop", "armorshops", "smithy", "guide", "inn", "kafra", "style"];

/// The picture of a map as the minimap shows it (`map\<name>.bmp`). A map's name goes into a URL: only what
/// map names are made of passes.
pub(in super::super) fn map_picture(map: &str) -> String {
    let map: String = map.chars().filter(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '@')).collect();
    picture(&format!("map/{map}.bmp"))
}

/// The minimap's marks of who stands where in a town (the client's `CGameMode::DrawMiniMap` and
/// `UIMinimapZoomWnd::AddTownButton`): each a small picture by its kind, its middle on the cell, with the name
/// shown under the pointer. Those outside the part of the map shown are left out. (In the client a press on one
/// starts the navigation there; this client has no navigation yet.)
pub(in super::super) fn minimap_marks(game: &Game, zoom: f32, hidden: bool) -> String {
    if hidden {
        return String::new();
    }
    data::town_marks(&game.map)
        .iter()
        .filter_map(|(name, x, y, kind)| {
            let picture_name = TOWN_PICTURES.get(usize::try_from(*kind).ok()?)?;
            minimap_spot(game, zoom, (*x as f32, *y as f32)).map(|(x, y)| {
                format!(
                    r#"<img class="mark" src="{}" alt="" title="{}" draggable="false" style="left:{x:.2}%;top:{y:.2}%" onerror="this.remove()">"#,
                    picture(&format!("information/{picture_name}.bmp")),
                    escape(name)
                )
            })
        })
        .collect()
}

/// Where a cell of the map is on the minimap, in hundredths of its width and height: none for one outside the
/// part of the map shown (which follows the character when the map is shown nearer than whole). The rule is
/// `crate::minimap::spot`.
pub(in super::super) fn minimap_spot(game: &Game, zoom: f32, cell: (f32, f32)) -> Option<(f32, f32)> {
    crate::minimap::spot(game.map_size?, (f32::from(game.x), f32::from(game.y)), zoom, cell)
}

/// How the minimap's picture and the character's mark on it are placed, as styles. The minimap is a square
/// picture with the map in its middle, north up; nearer than whole (the client's zoom, 1 to 4) it shows the
/// part around the character, as much of it as fits, never past the picture's edge (`CGameMode::DrawMiniMap`:
/// `crate::minimap::placed`).
pub(in super::super) fn minimap_place(game: &Game, zoom: f32) -> Option<(String, String)> {
    let placed = crate::minimap::placed(game.map_size?, (f32::from(game.x), f32::from(game.y)), zoom);
    Some((
        format!("position:absolute;width:{:.2}%;height:{:.2}%;left:{:.2}%;top:{:.2}%", placed.side, placed.side, placed.corner.0, placed.corner.1),
        format!("left:{:.2}%;top:{:.2}%", placed.mark.0, placed.mark.1),
    ))
}

impl App {
    /// In the game: the game's own windows over the world, and the minimap in its corner with the character's
    /// place under it.
    pub(super) fn game_window(&self, view: (f64, f64)) -> String {
        let Some(game) = &self.game else { return String::new() };
        // Under the minimap, as the original has it (`UIMinimapZoomWnd`): the character's cell, and the
        // minimap's buttons: the first shows and hides the marks of who stands where in a town, plus and minus
        // bring the map nearer and take it back, "mini" opens the large window of the map one is on
        // (`WID_MINIMAPWND`) and the last the World Map (`WID_ROMAPWND`). Under the pointer each tells what it
        // does, in the words of lines 2855 to 2859 of the client's message table.
        use crate::press::{Step, general::Press, worldmap};
        let buttons: [(&str, i32, crate::press::Press, usize, &str); 5] = [
            ("object", 1, Press::MapMarks.into(), 2855, "Show information"),
            ("plus", 73, Press::MapZoom(Step::More).into(), 2856, "Zoom in"),
            ("minus", 87, Press::MapZoom(Step::Less).into(), 2857, "Zoom out"),
            ("mini", 101, worldmap::Press::Large.into(), 2858, "Maximize"),
            ("viewon", 115, worldmap::Press::Window.into(), 2859, "World Map"),
        ];
        let buttons: String = buttons
            .into_iter()
            .map(|(name, x, does, line, otherwise)| {
                [
                    format!(r#"<img src="{}" alt="" draggable="false" style="position:absolute;left:{x}px;top:0;"#, picture(&format!(
                        "minimap/i_{name}_1.bmp"
                    ))),
                    format!(
                        r#"pointer-events:auto;cursor:pointer" onerror="this.remove()" {} data-tip="{}">"#,
                        pressed(does),
                        escape(&Self::words(line, otherwise))
                    )
                ].concat()
            })
            .collect();
        // The buttons under the minimap show when the player has them so (the client's `/minimap`).
        let buttons = if self.option("TT_SHOW_MINIMAP_BUTTON_ONOFF") { buttons } else { String::new() };
        let (pictured, marked) = minimap_place(game, self.minimap_zoom).unwrap_or_else(|| (String::new(), "display:none".to_owned()));
        let map = map_picture(&game.map);
        // The character's cell is written in the usual text (`UIMinimapZoomWnd::Paint`: (0, 12) at 19, 128).
        let cell = Font(0, 12, false).style();
        let windows = self.hud(view);
        let marks = minimap_marks(game, self.minimap_zoom, self.minimap_marks_off);
        // Ctrl+Tab runs the minimap through shown, shown fainter (`0xAAFFFFFF`) and away
        // (`HK_MINIMAP_ONOFF`).
        let faint = ["", ";opacity:.67", ";display:none"][usize::from(self.worldmap.minimap.min(2))];
        let away = if self.worldmap.minimap >= 2 { "display:none" } else { "" };
        let (party, quests, compass) = (self.minimap_party(), self.minimap_quests(), self.minimap_compass());
        [
            format!(r#"<div class="minimap" style="overflow:hidden{faint}"><img id="minimap" src="{map}" alt="" style="{pictured}" onerror="this.remove()">"#),
            format!(r#"<i id="marker" style="{marked}"></i><div id="map-marks">{marks}</div><div id="party-marks">{party}</div>"#),
            format!(
                r#"<div id="quest-marks">{quests}</div><div id="compass-marks">{compass}</div></div>
               <div class="mapname" style="{away}">"#
            ),
            format!(r#"<span id="position" style="{cell}"> {}  {}</span>{buttons}</div>{windows}"#, game.x, game.y)
        ].concat()
    }
}
