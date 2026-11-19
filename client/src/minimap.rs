//! The minimap as rules: where a cell of the map lies on it, which part of the map it shows when it is shown
//! nearer than whole, and what its buttons and its key change (the client's `CGameMode::DrawMiniMap`,
//! `UIMinimapZoomWnd`). The page's side of it is `web/view/game.rs` and `web/hud/minimap.rs`.
//!
//! The minimap is a square picture with the map in its middle, north up. Places on it are written in
//! hundredths of its side, as the page places things.

/// How near the minimap may show the map: whole, and four times.
pub const ZOOM: [f32; 2] = [1.0, 4.0];

/// Where a cell lies on the picture of a map so many cells wide and high, in hundredths of the picture's
/// side from its left and from its top.
pub fn on_picture((width, height): (usize, usize), (x, y): (f32, f32)) -> (f32, f32) {
    let scale = 100.0 / width.max(height) as f32;
    (50.0 + (x - width as f32 / 2.0) * scale, 50.0 - (y - height as f32 / 2.0) * scale)
}

/// The corner of the part of the picture shown at that zoom around a place on it (`on_picture`): the part
/// follows the place, as much as fits, never past the picture's edge.
pub fn corner((x, y): (f32, f32), zoom: f32) -> (f32, f32) {
    let shown = 100.0 / zoom.clamp(ZOOM[0], ZOOM[1]);
    let from = |at: f32| (at - shown / 2.0).clamp(0.0, 100.0 - shown);
    (from(x), from(y))
}

/// Where a cell of the map is on the minimap, in hundredths of its width and height, the character standing
/// on the cell `own`: none for one outside the part of the map shown.
pub fn spot(size: (usize, usize), own: (f32, f32), zoom: f32, cell: (f32, f32)) -> Option<(f32, f32)> {
    let zoom = zoom.clamp(ZOOM[0], ZOOM[1]);
    let (left, top) = corner(on_picture(size, own), zoom);
    let (x, y) = on_picture(size, cell);
    let (x, y) = ((x - left) * zoom, (y - top) * zoom);
    ((0.0..=100.0).contains(&x) && (0.0..=100.0).contains(&y)).then_some((x, y))
}

/// How the minimap's picture and the character's mark on it are placed, all in hundredths of the minimap's
/// side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// How large the picture is drawn: the minimap's side times the zoom.
    pub side: f32,
    /// Where the picture's corner goes, left and up of the minimap's own (never positive).
    pub corner: (f32, f32),
    /// Where the character's mark stands.
    pub mark: (f32, f32),
}

/// The picture and the mark for a character on the cell `own` of a map of that size.
pub fn placed(size: (usize, usize), own: (f32, f32), zoom: f32) -> Placed {
    let zoom = zoom.clamp(ZOOM[0], ZOOM[1]);
    let (x, y) = on_picture(size, own);
    let (left, top) = corner((x, y), zoom);
    Placed { side: zoom * 100.0, corner: (-left * zoom, -top * zoom), mark: ((x - left) * zoom, (y - top) * zoom) }
}

/// The zoom after a press of the minimap's plus (`nearer`) or minus (the client's `UIMinimapZoomWnd`): nearer
/// by a third of itself and a quarter of that, back by a quarter, between whole and four times.
pub fn zoomed(zoom: f32, nearer: bool) -> f32 {
    let zoom = zoom.max(ZOOM[0]);
    if nearer { (zoom + (zoom / 3.0 + zoom) * 0.25).min(ZOOM[1]) } else { (zoom - zoom * 0.25).max(ZOOM[0]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_lies_on_the_picture_north_up() {
        // A square map fills the picture: its south-west corner is the picture's bottom left.
        assert_eq!(on_picture((200, 200), (0.0, 0.0)), (0.0, 100.0));
        assert_eq!(on_picture((200, 200), (100.0, 100.0)), (50.0, 50.0));
        assert_eq!(on_picture((200, 200), (200.0, 200.0)), (100.0, 0.0));
        // A map wider than high lies in the middle of the picture's height.
        assert_eq!(on_picture((400, 200), (0.0, 0.0)), (0.0, 75.0));
        assert_eq!(on_picture((400, 200), (400.0, 200.0)), (100.0, 25.0));
    }

    #[test]
    fn the_part_shown_follows_the_character_and_stops_at_the_edge() {
        assert_eq!(corner((50.0, 50.0), 1.0), (0.0, 0.0));
        assert_eq!(corner((50.0, 50.0), 2.0), (25.0, 25.0));
        assert_eq!(corner((10.0, 95.0), 2.0), (0.0, 50.0));
        // A zoom out of its range is taken as the nearest in it.
        assert_eq!((corner((50.0, 50.0), 0.2), corner((50.0, 50.0), 9.0)), ((0.0, 0.0), (37.5, 37.5)));
    }

    #[test]
    fn a_cell_outside_the_part_shown_has_no_spot() {
        let size = (200, 200);
        assert_eq!(spot(size, (100.0, 100.0), 1.0, (50.0, 150.0)), Some((25.0, 25.0)));
        // Twice as near around the middle: the quarter around it fills the minimap.
        assert_eq!(spot(size, (100.0, 100.0), 2.0, (100.0, 100.0)), Some((50.0, 50.0)));
        assert_eq!(spot(size, (100.0, 100.0), 2.0, (50.0, 150.0)), Some((0.0, 0.0)));
        assert_eq!(spot(size, (100.0, 100.0), 2.0, (40.0, 100.0)), None);
        assert_eq!(spot(size, (100.0, 100.0), 1.0, (-1.0, 100.0)), None);
    }

    #[test]
    fn the_picture_and_the_mark_are_placed_together() {
        assert_eq!(placed((200, 200), (100.0, 100.0), 1.0), Placed { side: 100.0, corner: (0.0, 0.0), mark: (50.0, 50.0) });
        assert_eq!(placed((200, 200), (100.0, 100.0), 2.0), Placed { side: 200.0, corner: (-50.0, -50.0), mark: (50.0, 50.0) });
        // In a corner of the map the picture stops at its edge and the mark leaves the middle.
        assert_eq!(placed((200, 200), (0.0, 0.0), 2.0), Placed { side: 200.0, corner: (0.0, -100.0), mark: (0.0, 100.0) });
        // The mark stands where the character's own cell has its spot.
        let own = (30.0, 170.0);
        assert_eq!(Some(placed((200, 200), own, 3.0).mark), spot((200, 200), own, 3.0, own));
    }

    #[test]
    fn the_buttons_bring_the_map_nearer_and_back() {
        assert_eq!(zoomed(1.0, true), 1.0 + (1.0 / 3.0 + 1.0) * 0.25);
        assert_eq!((zoomed(3.9, true), zoomed(4.0, true)), (4.0, 4.0));
        assert_eq!((zoomed(2.0, false), zoomed(1.2, false), zoomed(0.0, false)), (1.5, 1.0, 1.0));
    }
}
