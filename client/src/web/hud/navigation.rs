//! The navigation's window (the client's `UINavigationV4Wnd`, 272 by 338, and its small form, 134 by 61), in
//! its three faces: the lists of what was found with the target looked at, the map of the way, and a map to
//! press a place on. The windows beside it are in `navigation/beside.rs`.

use super::*;
use crate::navigation::guide::{Display, Mode};
use crate::navigation::texts::{self, Picture};
use crate::navigation::{Atlas, What};
use crate::press::navigation::{Place, Press};

mod beside;

/// The folder of the window's own pictures.
const ART: &str = "navigation_interface3/";
/// The colour of the row picked in a list (the skin's colour for it).
pub(super) const PICKED: &str = "#c6d6f7";
/// The grey of the window's lines and of the ground under its maps (`0xCCCCCC`).
const LINE: &str = "#ccc";

/// What a button tells when pointed at, as its tag says it.
fn tip(words: &str) -> String {
    format!(r#" data-tip="{}""#, escape(words))
}

/// A button of two pictures, as it stands and under the pointer or pressed.
fn two(x: i32, y: i32, size: (i32, i32), off: &str, on: &str, press: Press, tells: &str) -> String {
    pictured(x, y, size, [off, on, on], press, &tip(tells))
}

/// One of the window's own buttons of three pictures (`…_normal`, `…_over`, `…_press`).
fn button(x: i32, y: i32, size: (i32, i32), name: &str, press: Press, tells: &str) -> String {
    pictured_by(Convention::NormalOverPress, x, y, size, &format!("{ART}{name}"), press, &tip(tells))
}

/// A row of one of the window's lists (`UIListBox_NaviSearch::Paint`): 16 high, the picture of its kind 3 in
/// and the words 11 after it; the one picked on the skin's colour for it.
pub(super) fn list_row(icon: &str, words: &str, picked: bool, press: Press) -> String {
    [
        format!(
            r#"<div class="t" style="position:relative;height:16px;line-height:16px;padding-left:14px;white-space:nowrap;cursor:default;color:{BLACK};"#
        ),
        format!(
            r#"background:{} url('{}') 3px 0 no-repeat" {}>{}</div>"#,
            if picked { PICKED } else { "transparent" },
            picture(&format!("{ART}icon_list_{icon}.bmp")),
            pressed(press),
            escape(words)
        )
    ].concat()
}

/// A list of such rows at its place, on its ground, with the line down its right side (`0xEBDEDE`, the
/// blue first).
pub(super) fn list(place: (i32, i32, i32, i32), ground: &str, rows: &str) -> String {
    [
        format!(r#"<div class="a scroll" style="{};overflow-y:auto;overflow-x:hidden;"#, at(place.0, place.1, place.2, place.3)),
        format!(r#"background:{ground};border-right:1px solid #dedeeb;box-sizing:border-box" {}>{rows}</div>"#, lines_stepped(16))
    ].concat()
}

/// The picture of a target's map at a place, drawn out to a square on black (`DrawMapTexture`): the small map
/// of a map that has one, else the client's picture of a place indoors; one that is not there gives way to
/// the client's "no image".
fn map_square(shown: Option<&Picture>, x: i32, y: i32, size: i32) -> String {
    let Some(shown) = shown else { return String::new() };
    let address = match shown {
        Picture::Map(map) => crate::web::view::map_picture(map),
        Picture::Indoors => picture("navigation_interface/Interior.bmp"),
    };
    [
        format!(r#"<div class="a" style="{};background:#000;overflow:hidden">"#, at(x, y, size, size)),
        format!(
            r#"<img src="{address}" alt="" draggable="false" style="width:100%;height:100%;display:block" onerror="this.onerror=null;this.src='{}'"></div>"#,
            picture("navigation_interface/NoImage.bmp")
        )
    ].concat()
}

/// A line written white and bold over a map (`DrawTextAt` with white and bold).
fn over_map(x: i32, y: i32, words: &str) -> String {
    text_in(BOLD, x, y, WHITE, words)
}

impl App {
    /// The navigation's window. (Of the client's: the button that reads a file of places out of the game's
    /// folder and the one of the private airship are drawn and do nothing; the two lines that follow the
    /// pointer over the map to press a place on are not drawn; an NPC drawn as a player is shown by its
    /// job's own picture; and a target's picture stands still.)
    pub(in crate::web) fn navigation_window(&self) -> String {
        let Some(atlas) = data::atlas() else { return self.navigation_frame() };
        let guide = &self.navigation.guide;
        if guide.small {
            return self.navigation_small(&atlas);
        }
        let mut window = self.navigation_frame();
        match (guide.mode, guide.display) {
            (Mode::Going, Display::Way) => window.push_str(&self.navigation_way(&atlas)),
            (_, Display::Pick) => window.push_str(&self.navigation_pick(&atlas)),
            _ => window.push_str(&self.navigation_target(&atlas)),
        }
        if guide.display == Display::Lists {
            window.push_str(&self.navigation_lists(&atlas));
        }
        if self.navigation.kinds {
            window.push_str(&self.navigation_kinds());
        }
        window
    }

    /// What every face of the large window has (`OnCreate`, `LayoutWithoutDetails`, the top of `Paint`): the
    /// title (line 2221) with the question mark, the button that makes the window small and the cross; a grey
    /// line at 43; at 21 the arrow that turns the window, the box that narrows the lists, the line to search
    /// with and its button, and the button of the files; the bar `bg_down.bmp` along the bottom with the box
    /// "Minimize", the button of the route's list and the one of the sets of arrows.
    fn navigation_frame(&self) -> String {
        let guide = &self.navigation.guide;
        let kinds = [(2206, "All"), (2207, "Map"), (2208, "Npc"), (2209, "Mob")];
        let (line, otherwise) = kinds[guide.kind.clamp(0, 3) as usize];
        [
            title_bar(272, &Self::words(2221, "Navigation")),
            panel_ground(0, 17, 272, 321),
            fill(0, 43, 272, 1, LINE),
            art(&format!("{ART}bg_down.bmp"), 0, 314, 272, 24),
            two(230, 3, (11, 11), "navigation_interface/NaviHelp_off.bmp", "navigation_interface/NaviHelp_on.bmp", Press::Help, &Self::words(2205, "Help")),
            two(244, 3, (11, 11), "basic_interface/sys_mini_off.bmp", "basic_interface/sys_mini_on.bmp", Press::Small, &Self::words(2204, "Change to simple UI")),
            cross(272, Press::Close),
            two(4, 21, (22, 18), &format!("{ART}bt_back.bmp"), &format!("{ART}bt_back_on.bmp"), Press::Turn, &Self::words(3208, "Search & description")),
            // The box of kinds (`UIComboBox_BmpWnd`, 61 by 18): its ground and the button that drops its list.
            format!(
                r#"<div class="a t" style="{};background:url('{}') 0 0 no-repeat;line-height:18px;padding-left:4px;box-sizing:border-box;cursor:default;color:{BLACK}" {}>{}</div>"#,
                at(29, 21, 43, 18),
                picture(&format!("{ART}bg_combo.bmp")),
                pressed(Press::Kinds),
                escape(&Self::words(line, otherwise))
            ),
            pictured(72, 21, (18, 18), [&format!("{ART}dropdown_on.bmp"), &format!("{ART}dropdown_on_over.bmp"), &format!("{ART}dropdown_on_press.bmp")], Press::Kinds, ""),
            // The line to search with (`UIBitmapEditCtrl`, 113 by 18 on `bg_search.bmp`, 30 letters), and its button.
            format!(r#"<form {} class="a" style="{}">"#, sent(Form::NavigationSearch), at(94, 21, 138, 18)),
            format!(
                r#"<input class="a in" id="{}" maxlength="30" autocomplete="off" style="{};background:url('{}') 0 0 no-repeat;padding:0 4px;box-sizing:border-box" value="{}">"#,
                windows::NAVIGATION.field(),
                at(0, 0, 113, 18),
                picture(&format!("{ART}bg_search.bmp")),
                escape(&guide.sought)
            ),
            format!(
                r#"<button type="submit" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')"{}></button></form>"#,
                at(113, 0, 25, 18),
                picture(&format!("{ART}btn_searchbar_normal.bmp")),
                picture(&format!("{ART}btn_searchbar_over.bmp")),
                picture(&format!("{ART}btn_searchbar_press.bmp")),
                tip(&Self::words(2196, "Search"))
            ),
            button(236, 21, (30, 18), "btn_navidata_open", Press::Files, &Self::words(2201, "Read external file path information")),
            check_words(9, 321, self.navigation.fold, "Minimize", Press::Fold),
            button(203, 317, (26, 18), "btn_roadlistwnd", Press::Route, &Self::words(3205, "Navigation information")),
            button(237, 317, (26, 18), &format!("btn_roadiocn_select{}", guide.icon.clamp(1, 8)), Press::Icons, &Self::words(3211, "Show navigation icon setting")),
        ].concat()
    }

    /// The list of the box of kinds, dropped under it: all, maps, NPCs, monsters (lines 2206 to 2209).
    fn navigation_kinds(&self) -> String {
        let rows: String = [(2206, "All"), (2207, "Map"), (2208, "Npc"), (2209, "Mob")]
            .into_iter()
            .enumerate()
            .map(|(kind, (line, otherwise))| format!(
                r#"<div class="t" style="height:16px;line-height:16px;padding-left:3px;cursor:default;color:{BLACK};background:{}" {}>{}</div>"#,
                if kind as i32 == self.navigation.guide.kind { PICKED } else { FIELD_GROUND },
                pressed(Press::Kind(kind as i32)),
                escape(&Self::words(line, otherwise))
            ))
            .collect();
        format!(r#"<div class="a" style="{};background:{FIELD_GROUND};border:1px solid #646464;box-sizing:border-box">{rows}</div>"#, at(29, 39, 61, 67))
    }

    /// The two lists of the first face and what stands under them (`LayoutWithDetails`, `FillRouteGroups`,
    /// `SelectRoute`): how many groups were found (line 3210) at (20, 48) by the mark `bg_dot.bmp`; the list
    /// of groups 138 by 244 at (1, 70); the list of the group picked 132 by 80 at (139, 166) on a pale blue;
    /// the box "service" at (146, 247), the buttons that share and that find at 263, and the airship's at 288.
    fn navigation_lists(&self, atlas: &Atlas) -> String {
        let guide = &self.navigation.guide;
        let icon = |kind: i32| match kind {
            2 => "npc",
            3 => "mob",
            _ => "map",
        };
        let groups: String = guide
            .groups
            .iter()
            .enumerate()
            .filter_map(|(place, group)| Some((place, texts::group_row(atlas, &guide.found, group)?)))
            .map(|(place, (kind, words))| list_row(icon(kind), &words, guide.group == Some(place), Press::Group(place)))
            .collect();
        let density = |many: i32| texts::density_line(many).map(|(line, otherwise)| Self::words(line, otherwise));
        let members: String = guide
            .group
            .and_then(|group| guide.groups.get(group))
            .into_iter()
            .flatten()
            .filter_map(|index| guide.found.get(*index))
            .enumerate()
            .map(|(place, target)| {
                let (kind, words) = texts::member_row(atlas, target, density);
                list_row(icon(kind), &words, guide.member == Some(place), Press::Member(place))
            })
            .collect();
        [
            art(&format!("{ART}bg_dot.bmp"), 5, 46, 12, 13),
            text(20, 48, BLACK, &crate::numbers::filled(&Self::words(3210, "Search result[%d]"), &[&guide.groups.len().to_string()])),
            list((1, 70, 138, 244), WHITE, &groups),
            list((139, 166, 132, 80), "rgb(240,244,253)", &members),
            check_words(146, 247, guide.allowed.zeny, "service", Press::Service),
            pictured_by(Convention::NormalOverPress, 145, 263, (58, 20), "private_airplane/btn_share", Press::Share, &tip(&Self::words(3206, "Share current location"))),
            pictured_by(Convention::NormalOverPress, 207, 263, (58, 20), "private_airplane/btn_find", Press::Find, &tip(&Self::words(2197, "Navigation"))),
            pictured_by(Convention::NormalOverPress, 145, 288, (120, 20), "private_airplane/btn_move", Press::Airship, ""),
        ].concat()
    }

    /// The target looked at, at the right of the first face (the end of `Paint`): two grey lines, at 138 down
    /// the window and at 242 across its right part; the map of a map or a place 100 square at (155, 60); an NPC
    /// or a monster as its picture standing in a space 116 square at (147, 45) on the client's round shadow,
    /// with its map 40 square at (227, 120); and over them the lines that tell of it (`texts::labels`), blue
    /// over a pale copy of themselves, the word for a boss red.
    fn navigation_target(&self, atlas: &Atlas) -> String {
        let target = self.navigation.guide.current.as_ref();
        let shown = texts::picture(atlas, target);
        let mut face = [fill(138, 44, 1, 270, LINE), fill(139, 242, 132, 1, LINE)].concat();
        let job = match target.map(|target| target.what) {
            Some(What::Npc(npc)) => Some(atlas.npcs[npc].job()),
            Some(What::Mob(mob)) => Some(atlas.mobs[mob].job()),
            _ => None,
        };
        match job {
            Some(job) => {
                face.push_str(&format!(r#"<div class="a" style="{};overflow:hidden">"#, at(147, 45, 116, 116)));
                face.push_str(&art("Anibackground.bmp", 44, 72, 28, 28));
                if let Some((url, size, feet)) = self.navigation.pictures.get(&job) {
                    face.push_str(&format!(
                        r#"<img src="{url}" alt="" draggable="false" style="position:absolute;left:{:.0}px;top:{:.0}px;width:{}px;height:{}px">"#,
                        58.0 - feet.0,
                        86.0 - feet.1,
                        size.0,
                        size.1
                    ));
                }
                face.push_str("</div>");
                face.push_str(&map_square(shown.as_ref(), 227, 120, 40));
            }
            None => face.push_str(&map_square(shown.as_ref(), 155, 60, 100)),
        }
        for label in target.map(|target| texts::labels(atlas, target, Self::words)).unwrap_or_default() {
            let (under, over) = if label.boss { ("#c8c8c8", RED) } else { ("#dcdcdc", "rgb(84,107,203)") };
            face.push_str(&text(label.x + 1, label.y + 1, under, &label.text));
            face.push_str(&text(label.x, label.y, over, &label.text));
        }
        face
    }

    /// The map of the way (`Paint` on the way): the map of this stretch 266 square at (3, 46) on grey; over it
    /// what the client calls the map and where on it to go, at (10, 48) and (10, 68), and where the navigation
    /// ends, at (10, 296); the way as dots of two pixels (`0xF0F00A`, the blue first) with `ico_destination.bmp`
    /// at its end, and `ico_location.bmp` where the character is.
    fn navigation_way(&self, atlas: &Atlas) -> String {
        let guide = &self.navigation.guide;
        let [title, at_cell, end] = guide.way_lines(atlas, &Self::words(3585, "Destination"));
        let mut face = [fill(1, 44, 270, 270, LINE), map_square(texts::picture(atlas, guide.current.as_ref()).as_ref(), 3, 46, 266)].concat();
        face.push_str(&[over_map(10, 48, &title), over_map(10, 68, &at_cell), over_map(10, 296, &end)].concat());
        let here = self.game.as_ref().and_then(|game| Some((&atlas.maps[atlas.find(&game.map)?], (i32::from(game.x), i32::from(game.y)))));
        let Some((map, cell)) = here else { return face };
        // The dots are counted on a map of 126 and drawn out to this one (`BuildRoute`, then 2.078125 each).
        let mut dots: Vec<(i32, i32)> = Vec::new();
        for step in guide.way.iter().filter(|_| guide.shown) {
            let dot = texts::on_view(map.width, map.height, *step, 126.0);
            if !dots.contains(&dot) {
                dots.push(dot);
            }
        }
        let drawn = |dot: (i32, i32)| ((dot.0 as f32 * 2.078125 + 3.0) as i32, (dot.1 as f32 * 2.078125 + 46.0) as i32);
        for dot in &dots {
            let (x, y) = drawn(*dot);
            face.push_str(&fill(x, y, 2, 2, "rgb(10,240,240)"));
        }
        if let Some(last) = dots.last() {
            let (x, y) = drawn(*last);
            face.push_str(&art(&format!("{ART}ico_destination.bmp"), x - 8, y - 21, 15, 21));
        }
        let (x, y) = texts::on_view(map.width, map.height, cell, 266.0);
        face.push_str(&art(&format!("{ART}ico_location.bmp"), x - 6, y + 29, 17, 17));
        face
    }

    /// The map to press a place on (`Paint` with the third face): the map 266 square at (3, 46) with what the
    /// client calls it and its own name at (9, 50) and (9, 65), white and rimmed black; and for five seconds
    /// what it is for (line 3215), on `back.bmp` let through with the picture of a mouse before it. A press on
    /// the map writes the place into the chat's line.
    fn navigation_pick(&self, atlas: &Atlas) -> String {
        let guide = &self.navigation.guide;
        let mut face = [fill(1, 44, 270, 270, LINE), map_square(texts::picture(atlas, guide.current.as_ref()).as_ref(), 3, 46, 266)].concat();
        let rimmed = |x: i32, y: i32, words: &str| format!(
            r#"<div class="a t" style="left:{x}px;top:{y}px;color:{WHITE};{};{};pointer-events:none">{}</div>"#,
            SMALL_BOLD.style(),
            outlined(BLACK),
            escape(words)
        );
        if let Some(map) = guide.current.as_ref().map(|target| &atlas.maps[atlas.map_of(target.what)]) {
            face.push_str(&[rimmed(9, 50, &map.title), rimmed(9, 65, &map.name)].concat());
        }
        if crate::web::now() < self.navigation.notice {
            let lines = crate::broadcast::wrapped(&Self::words(3215, "Click on the location to share it in a chat room"), 38);
            face.push_str(&format!(
                r#"<div class="a" style="{};opacity:.4;background:url('{}') 0 0 / 100% 100% no-repeat"></div>"#,
                at(12, 272, 248, 30),
                picture(&format!("{ART}back.bmp"))
            ));
            face.push_str(&art(&format!("{ART}icon_mouse.bmp"), 18, 277, 13, 18));
            let top = (41 - lines.len() as i32) * 7;
            for (row, line) in lines.iter().enumerate() {
                face.push_str(&rimmed(42, top + 14 * row as i32, line));
            }
        }
        face.push_str(&format!(r#"<div class="a" style="{};cursor:crosshair" {}></div>"#, at(3, 46, 267, 267), pressed_at(Press::Pick(Place(0, 0)))));
        face
    }

    /// The small window (`Paint` and `RefreshLayout` with `expandedMap`): 134 by 61, let through a third; a
    /// white box with a black one under its title (line 2221, at (3, 3)); in it, on the way, the mark
    /// `ico_destination.bmp` with where to go on this map at (5, 21) and where the navigation ends at (5, 40),
    /// white and bold; the cross and the button that makes it large again at its top right.
    fn navigation_small(&self, atlas: &Atlas) -> String {
        let guide = &self.navigation.guide;
        let [at_cell, end] = guide.small_lines(atlas);
        let mut window = [
            format!(r#"<div class="a" style="{};background:#fff;border-radius:3px"></div>"#, at(0, 0, 134, 61)),
            format!(r#"<div class="a" style="{};background:#000;border-radius:3px"></div>"#, at(1, 17, 132, 43)),
            text(3, 3, BLACK, &Self::words(2221, "Navigation")),
            button(102, 2, (14, 14), "btn_maximize", Press::Large, &Self::words(2203, "Change to basic UI")),
            button(119, 2, (14, 14), "btn_close", Press::Close, &Self::words(3045, "Close")),
        ].concat();
        if !at_cell.is_empty() {
            window.push_str(&[art(&format!("{ART}ico_destination.bmp"), 2, 18, 15, 21), over_map(5, 21, &at_cell), over_map(5, 40, &end)].concat());
        }
        window
    }
}
