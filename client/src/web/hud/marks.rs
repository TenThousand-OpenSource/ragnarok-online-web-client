//! The marks on the minimap that the server tells of: the members of the guild on this map, and the boss
//! monster (the client's `CGameMode::DrawMiniMapMembers`, `DrawMiniMap` and `DrawMiniMapMarker`).

use super::*;

/// The colour of a guild member's mark (`MM_ADDGUILDPOS`: `0xFFFFA6D2`).
const GUILD: &str = "#ffa6d2";

impl App {
    /// The marks, as the party's are written (`minimap_party` puts them after its own). A member of the guild
    /// who is not of the party too is a triangle in the guild's colour, its point up, over a larger one of
    /// white let through a third (`DrawMiniMapTriangle` with sizes 2 and 3: the point twice the size over
    /// the cell, the foot one size under it and the root of three sizes to each side). The boss monster is
    /// the client's `map\bossmonster.bmp`, 12 across, while the player is in the state that shows it
    /// (`EFST_CASH_BOSS_ALARM`). The client keeps the boss's mark at the edge of the minimap when the boss is
    /// outside the part shown; here it is left out then.
    pub(in crate::web) fn minimap_server_marks(&self) -> String {
        let Some(game) = self.game.as_ref().filter(|_| self.marks.visit == crate::web::world::visit()) else { return String::new() };
        let spot = |cell: (u16, u16)| crate::web::view::minimap_spot(game, self.minimap_zoom, (f32::from(cell.0), f32::from(cell.1)));
        let triangle = |size: f32, colour: &str| format!(
            r#"<polygon points="0,{:.2} {:.2},{size} {:.2},{size}" fill="{colour}"/>"#,
            -2.0 * size,
            -(3.0f32.sqrt()) * size,
            3.0f32.sqrt() * size
        );
        let mut marks: String = self
            .marks
            .guild
            .iter()
            .filter(|(account, _)| !self.party.places.contains_key(account))
            .filter_map(|(_, cell)| {
                let (x, y) = spot(*cell)?;
                Some([
                    format!(r#"<svg viewBox="-6 -6 12 12" style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:12px;height:12px;"#),
                    format!(r#"margin:-6px 0 0 -6px;overflow:visible">{}{}</svg>"#, triangle(3.0, "rgba(255,255,255,.67)"), triangle(2.0, GUILD))
                ].concat())
            })
            .collect();
        let alarmed = self.statuses.on.iter().any(|active| active.state == crate::net::marks::BOSS_ALARM);
        if let Some((x, y)) = self.marks.boss.filter(|_| alarmed).and_then(spot) {
            marks.push_str(&[
                format!(
                    r#"<img src="{}" alt="" draggable="false" style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:12px;"#,
                    picture("map/bossmonster.bmp")
                ),
                format!(r#"height:12px;margin:-6px 0 0 -6px" onerror="this.remove()">"#)
            ].concat());
        }
        marks
    }
}
