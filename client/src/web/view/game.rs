//! The game's screen: the minimap in its corner with the character's place under it, over the game's own windows.

use super::*;

/// The minimap's marks of who stands where in a town (the client's `CGameMode::DrawMiniMap` and
/// `UIMinimapZoomWnd::AddTownButton`): each a small picture by its kind, its middle on the cell, with the name
/// shown under the pointer. Those outside the part of the map shown are left out. (In the client a press on one
/// starts the navigation there; this client has no navigation yet.)
pub(in super::super) fn minimap_marks(game: &Game, zoom: f32, hidden: bool) -> String {
    const PICTURES: [&str; 8] = ["store", "weaponshop", "armorshops", "smithy", "guide", "inn", "kafra", "style"];
    if hidden {
        return String::new();
    }
    let Some((width, height)) = game.map_size else { return String::new() };
    let zoom = zoom.clamp(1.0, 4.0);
    let scale = 100.0 / width.max(height) as f32;
    let on_picture = |x: f32, y: f32| (50.0 + (x - width as f32 / 2.0) * scale, 50.0 - (y - height as f32 / 2.0) * scale);
    let (own_x, own_y) = on_picture(f32::from(game.x), f32::from(game.y));
    let shown = 100.0 / zoom;
    let from = |at: f32| (at - shown / 2.0).clamp(0.0, 100.0 - shown);
    let (left, top) = (from(own_x), from(own_y));
    data::town_marks(&game.map)
        .iter()
        .filter_map(|(name, x, y, kind)| {
            let picture_name = PICTURES.get(usize::try_from(*kind).ok()?)?;
            let (x, y) = on_picture(*x as f32, *y as f32);
            let (x, y) = ((x - left) * zoom, (y - top) * zoom);
            ((0.0..=100.0).contains(&x) && (0.0..=100.0).contains(&y)).then(|| {
                format!(
                    r#"<img class="mark" src="{}" alt="" title="{}" draggable="false" style="left:{x:.2}%;top:{y:.2}%" onerror="this.remove()">"#,
                    picture(&format!("information/{picture_name}.bmp")),
                    escape(name)
                )
            })
        })
        .collect()
}

/// How the minimap's picture and the character's mark on it are placed, as styles. The minimap is a square
/// picture with the map in its middle, north up; nearer than whole (the client's zoom, 1 to 4) it shows the
/// part around the character, as much of it as fits, never past the picture's edge (`CGameMode::DrawMiniMap`).
pub(in super::super) fn minimap_place(game: &Game, zoom: f32) -> Option<(String, String)> {
    let (width, height) = game.map_size?;
    let zoom = zoom.clamp(1.0, 4.0);
    let scale = 100.0 / width.max(height) as f32;
    let (x, y) = (50.0 + (f32::from(game.x) - width as f32 / 2.0) * scale, 50.0 - (f32::from(game.y) - height as f32 / 2.0) * scale);
    // What is shown is a square of the picture, this many hundredths of it a side, from here.
    let shown = 100.0 / zoom;
    let from = |at: f32| (at - shown / 2.0).clamp(0.0, 100.0 - shown);
    let (left, top) = (from(x), from(y));
    Some((
        format!("position:absolute;width:{:.2}%;height:{:.2}%;left:{:.2}%;top:{:.2}%", zoom * 100.0, zoom * 100.0, -left * zoom, -top * zoom),
        format!("left:{:.2}%;top:{:.2}%", (x - left) * zoom, (y - top) * zoom),
    ))
}

impl App {
    /// In the game: the game's own windows over the world, and the minimap in its corner with the character's
    /// place under it.
    pub(super) fn game_window(&self, view: (f64, f64)) -> String {
        let Some(game) = &self.game else { return String::new() };
        // A map name goes into a URL: only what map names are made of passes.
        let map: String = game.map.chars().filter(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '@')).collect();
        // Under the minimap, as the original has it: the character's cell, and the minimap's buttons (which do
        // nothing yet).
        // Plus and minus bring the map nearer and take it back (the client's `UIMinimapZoomWnd`), the first
        // button shows and hides the marks of who stands where in a town; the others (the small map's window,
        // the world map) do nothing yet.
        let buttons: String = [("object", 1, "marks"), ("plus", 73, "+"), ("minus", 87, "-"), ("mini", 101, ""), ("viewon", 115, "")]
            .iter()
            .map(|(name, x, zoom)| {
                let press = match *zoom {
                    "" => String::new(),
                    "marks" => r#" data-click="map-marks""#.to_owned(),
                    zoom => format!(r#" data-click="map-zoom" data-arg="{zoom}""#),
                };
                format!(
                    r#"<img src="{}" alt="" draggable="false" style="position:absolute;left:{x}px;top:0;{}" onerror="this.remove()"{press}>"#,
                    picture(&format!("minimap/i_{name}_1.bmp")),
                    if zoom.is_empty() { "" } else { "pointer-events:auto;cursor:pointer" }
                )
            })
            .collect();
        // The buttons under the minimap show when the player has them so (the client's `/minimap`).
        let buttons = if self.option("TT_SHOW_MINIMAP_BUTTON_ONOFF") { buttons } else { String::new() };
        let (pictured, marked) = minimap_place(game, self.minimap_zoom).unwrap_or_else(|| (String::new(), "display:none".to_owned()));
        format!(
            r#"<div class="minimap" style="overflow:hidden"><img id="minimap" src="{}" alt="" style="{pictured}" onerror="this.remove()"><i id="marker" style="{}"></i><div id="map-marks">{marks}</div></div>
               <div class="mapname"><span id="position"> {}  {}</span>{buttons}</div>{}"#,
            picture(&format!("map/{map}.bmp")),
            marked,
            game.x,
            game.y,
            self.hud(view),
            marks = minimap_marks(game, self.minimap_zoom, self.minimap_marks_off),
        )
    }
}
