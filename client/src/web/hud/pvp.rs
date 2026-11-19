//! The standing on a PVP map: "rank / total" in the corner of the screen, in the digits of the client's sprite
//! `이팩트\RankFont` (`CGameMode::DrawMiniMapCoordinates`).

use super::*;

impl App {
    /// The line as it stands now: each piece a picture, its origin where the client puts it, counted from the
    /// right edge and the bottom of the screen (`crate::net::pvp::rank_line`). Nothing off a PVP map, or until
    /// the pictures are read.
    pub(in crate::web) fn rank_line(&self) -> String {
        if !self.pvp.zone.ranked() {
            return String::new();
        }
        crate::net::pvp::rank_line(self.pvp.rank, self.pvp.total)
            .into_iter()
            .filter_map(|(action, x, y)| {
                let (picture, _, origin) = self.pvp.digits.get(action)?;
                Some(format!(r#"<img alt="" draggable="false" src="{picture}" style="left:calc(100% + {:.0}px);top:calc(100% + {:.0}px)">"#, x as f32 - origin.0, y as f32 - origin.1))
            })
            .collect()
    }
}
