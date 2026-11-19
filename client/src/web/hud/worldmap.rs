//! The World Map, and the large window of the map one is on.

use super::*;
use crate::press::worldmap::Press;
use data::{Field, World};

/// A rectangle of a world's picture as a style, in hundredths of the picture: the client scales the table's
/// points by the picture's size to the window's (`CWorldViewMgr`), which this does for any size of window.
fn spot(world: &World, [left, top, right, bottom]: [f32; 4]) -> String {
    let (width, height) = (world.size.0.max(1.0), world.size.1.max(1.0));
    format!("left:{:.3}%;top:{:.3}%;width:{:.3}%;height:{:.3}%", left / width * 100.0, top / height * 100.0, (right - left) / width * 100.0, (bottom - top) / height * 100.0)
}

/// What shows by a map under the pointer (`DrawSelectedMapPreview`): its small map, 130 by 130 on a
/// see-through black, 10 to the right of the rectangle and 10 down (to the left of it where it would leave
/// the picture, and kept from its bottom), with what the client calls the map at (3, 3) in gold and the map
/// itself at (3, 115) in white, both bold. It is only fetched when it shows.
fn preview(world: &World, field: &Field) -> String {
    let side = if field.at[2] / world.size.0.max(1.0) > 0.86 { " wv-left" } else { "" };
    let low = if field.at[1] / world.size.1.max(1.0) > 0.8 { " wv-low" } else { "" };
    format!(
        r#"<div class="wv-view{side}{low}" style="--v:url('{}')"><b style="position:absolute;left:3px;top:3px;color:#ffd200">{}</b><b style="position:absolute;left:3px;top:115px;color:#fff">{}</b></div>"#,
        picture(&format!("map/{}.bmp", field.map)),
        escape(&field.name),
        escape(&field.map)
    )
}

impl App {
    /// The world the World Map shows: the one chosen, or the first that has the map one is on (`FindWorld`),
    /// or the first of all.
    pub(in crate::web) fn world_shown(&self, worlds: &[World]) -> usize {
        let map = self.game.as_ref().map(|game| game.map.trim_end_matches(".gat").to_owned()).unwrap_or_default();
        self.worldmap.world.filter(|world| *world < worlds.len()).or_else(|| worlds.iter().position(|world| world.fields.iter().any(|field| field.map == map))).unwrap_or(0)
    }

    /// The maps of a world, each a rectangle that lights up under the pointer and shows its small map
    /// (`UIRoMapWnd::OnMouseMove`: filled with (149, 15, 15) at 70 of 255). The floors of a dungeon (an ID
    /// under 100) share one rectangle, which bears the dungeon's name, with a line from it to the dungeon's
    /// mark (`DrawDungeonLinks`: red at 180 of 255; `wv_dunSelect.bmp` on the name, `wv_dunPos.bmp` on the
    /// mark). With the box "Monster(Tab)" ticked the maps bear their names in green (128, 255, 128) and the
    /// level they are for in bold white, and a dungeon its levels (`DrawFieldLabels`).
    fn world_fields(&self, world: &World, small: i32) -> String {
        let mut fields = String::new();
        let mut lines = String::new();
        for (id, mark, name, levels) in &world.groups {
            let Some(first) = world.fields.iter().find(|field| field.id == *id) else { continue };
            let centre = |at: [f32; 4]| ((at[0] + at[2]) / 2.0, (at[1] + at[3]) / 2.0);
            let (from, to) = (centre(first.at), centre(*mark));
            lines.push_str(&format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="rgba(255,0,0,.7)" stroke-width="1" vector-effect="non-scaling-stroke"/>"#, from.0, from.1, to.0 + 1.0, to.1 + 1.0));
            fields.push_str(&format!(r#"<div class="a" style="{};background:url('{}') 0 0 / 100% 100% no-repeat;opacity:.7"></div>"#, spot(world, *mark), picture("worldview_interface/wv_dunPos.bmp")));
            let levels = if self.worldmap.labels { format!("<br>{}", escape(levels)) } else { String::new() };
            fields.push_str(&format!(
                r#"<div class="wv-field" style="{};background-image:url('{}');background-size:100% 100%"><div style="text-align:center;padding-top:2px;font-size:{small}px;line-height:{}px;color:#c8c8c8;text-shadow:1px 1px 0 #282828;white-space:nowrap">{}{levels}</div>{}</div>"#,
                spot(world, first.at),
                picture("worldview_interface/wv_dunSelect.bmp"),
                small + 2,
                escape(name),
                preview(world, first)
            ));
        }
        for field in world.fields.iter().filter(|field| field.id >= 100) {
            let label = if self.worldmap.labels {
                format!(
                    r#"<div style="position:absolute;left:0;right:0;bottom:3px;text-align:center;font-size:{small}px;line-height:{}px;color:#80ff80;text-shadow:1px 1px 0 #282828,-1px -1px 0 #282828;white-space:nowrap;overflow:hidden">{}<br><b style="color:#fff;text-shadow:1px 1px 0 #000,-1px -1px 0 #000">{}</b></div>"#,
                    small + 2,
                    escape(&field.name),
                    escape(&field.level)
                )
            } else {
                String::new()
            };
            let rim = if self.worldmap.labels { ";outline:1px solid #505050" } else { "" };
            fields.push_str(&format!(r#"<div class="wv-field" style="{}{rim}">{label}{}</div>"#, spot(world, field.at), preview(world, field)));
        }
        format!(r#"<svg class="a" style="left:0;top:0;width:100%;height:100%;pointer-events:none" viewBox="0 0 {} {}" preserveAspectRatio="none">{lines}</svg>{fields}"#, world.size.0, world.size.1)
    }

    /// Where the player and the party's members are, a map each (`RefreshPartyRecords`): a rim round the map
    /// (the one the player is on in `0x23AEE8`, the others in `0x7EE2F7`, as the client writes colours) and
    /// in its middle the face `wv_face1.bmp` or `wv_face.bmp`, as large as the screen's size asks; under the
    /// pointer, who is there, the player's own name first.
    fn world_party(&self, world: &World, face: i32) -> String {
        let Some(game) = &self.game else { return String::new() };
        let own_map = game.map.trim_end_matches(".gat");
        let own = self.own_account();
        let mut there: Vec<(String, Vec<String>)> = vec![(own_map.to_owned(), vec![game.name.clone()])];
        for member in self.party.members.iter().filter(|member| member.account != own && !member.offline) {
            let map = member.map.trim_end_matches(".gat").to_owned();
            match there.iter_mut().find(|(on, _)| *on == map) {
                Some((_, names)) => names.push(member.name.clone()),
                None => there.push((map, vec![member.name.clone()])),
            }
        }
        there
            .iter()
            .filter_map(|(map, names)| Some((map, names, world.fields.iter().find(|field| field.map == *map)?)))
            .map(|(map, names, field)| {
                let here = map == own_map;
                format!(
                    r#"<div class="a" style="{};outline:1px solid {};display:flex;align-items:center;justify-content:center;pointer-events:none"><img src="{}" alt="" draggable="false" style="width:{face}px;height:{face}px;opacity:.93;pointer-events:auto" data-tip="{}"></div>"#,
                    spot(world, field.at),
                    if here { "#e8ae23" } else { "#f7e27e" },
                    picture(if here { "worldview_interface/wv_face1.bmp" } else { "worldview_interface/wv_face.bmp" }),
                    escape(&names.join(", "))
                )
            })
            .collect()
    }

    /// The World Map (the client's `UIRoMapWnd`): as large as the screen; a title bar with no title, at (17,
    /// 0) the box that names the world shown and opens the list of worlds, at the right the box
    /// "Monster(Tab)" (line 2440) and the cross (`world_close_off.bmp`, 18 from the edge); under it the
    /// world's picture drawn out over the rest. A press on a map starts the client's navigation there, which
    /// this client does not have; nor has it the search, the routes and the Kafra's arrows.
    pub(in crate::web) fn worldmap_window(&self) -> String {
        let worlds = data::worlds();
        let shown = self.world_shown(&worlds);
        let Some(world) = worlds.get(shown) else { return String::new() };
        // The size of the screen picks the size of the words and of the faces (the client's four steps: up to
        // 640 wide, 800, 1024, and wider; this client's stage is never under 1024).
        let scale = if crate::web::page_size().0 / crate::web::stage_scale() <= 1024.0 { 2 } else { 3 };
        let piece = |name: &str, place: &str| format!(r#"<div class="a" style="{place};height:17px;background:url('{}') 0 0 / 100% 100% no-repeat"></div>"#, picture(&format!("basic_interface/titlebar_{name}.bmp")));
        let mut window = format!(
            r#"<div class="a" style="left:0;top:0;width:100%;height:100%;background:#000"></div>{}{}{}<div class="a" style="left:0;right:0;top:16px;height:2px;background:#c6dff7"></div>"#,
            piece("left", "left:0;top:0;width:12px"),
            piece("mid", "left:12px;right:12px;top:0"),
            piece("right", "right:0;top:0;width:12px"),
        );
        window.push_str(&format!(
            r#"<div class="a" style="left:0;right:0;top:17px;bottom:0;overflow:hidden"><img src="{}" alt="" draggable="false" style="position:absolute;left:0;top:0;width:100%;height:100%">{}{}</div>"#,
            picture(&world.picture),
            self.world_fields(world, 9 + scale),
            self.world_party(world, 2 * (scale * 2 + 8)),
        ));
        window.push_str(&format!(
            r#"<div class="a t" style="{};background:#e8e8e8;border:1px solid #646464;box-sizing:border-box;line-height:16px;padding-left:3px;cursor:default;color:{BLACK}" {}>{}</div>"#,
            at(17, 0, 110, 18),
            pressed(Press::Worlds),
            escape(&world.name)
        ));
        if self.worldmap.choosing {
            let rows: String = worlds
                .iter()
                .enumerate()
                .map(|(place, world)| format!(r#"<div class="t" style="height:16px;line-height:16px;padding-left:3px;cursor:default;color:{BLACK};background:{}" {}>{}</div>"#, if place == shown { "#c6d6f7" } else { "#e8e8e8" }, pressed(Press::World(place)), escape(&world.name)))
                .collect();
            window.push_str(&format!(r#"<div class="a scroll" style="{};overflow-y:auto;border:1px solid #646464;box-sizing:border-box;background:#e8e8e8">{rows}</div>"#, at(17, 18, 110, 67)));
        }
        // The box stands 82 before the cross, which is 18 from the right edge.
        window.push_str(&format!(
            r#"<div class="a t" style="right:20px;top:3px;width:80px;height:12px;line-height:12px;padding-left:12px;box-sizing:border-box;white-space:nowrap;cursor:default;color:{BLACK};background:url('{}') 0 1px no-repeat" {} data-tip="Tab">{}</div>"#,
            picture(if self.worldmap.labels { "checkbox_1.bmp" } else { "checkbox_0.bmp" }),
            pressed(Press::Labels),
            escape(&Self::words(2440, "Monster(Tab)"))
        ));
        window.push_str(&format!(
            r#"<button type="button" class="a b" style="right:4px;top:0;width:14px;height:16px;--n:url('{off}');--o:url('{on}');--p:url('{on}')" {} data-tip="ctrl+`"></button>"#,
            pressed(Press::Window),
            off = picture("basic_interface/world_close_off.bmp"),
            on = picture("basic_interface/world_close_on.bmp"),
        ));
        window
    }

    /// The large window of the map one is on (the client's `UIMiniMapWnd`, 516 by 559, the minimap's
    /// "mini" button): its title the map as the client calls it, the map itself and the character's cell;
    /// the map's picture 512 by 512 at (2, 19) on grey; on it who of use stands where in a town, the party's
    /// members and the character. (The client's boxes for the marks of quests, towns and the guild, and its
    /// notes on the map, are not here.)
    pub(in crate::web) fn large_map_window(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        let map = game.map.trim_end_matches(".gat");
        let name = data::map_sign(map).map(|sign| sign.name).unwrap_or_default();
        let mut window = title_bar(516, &format!("{name} ( {map} ) {} {}", game.x, game.y));
        window.push_str(&cross(516, Press::Large));
        window.push_str(&panel(516, 559));
        let spot = |cell: (f32, f32)| crate::web::view::minimap_spot(game, 1.0, cell);
        let mut marks = String::new();
        for (name, x, y, kind) in data::town_marks(&game.map) {
            if let (Some(picture_name), Some((x, y))) = (usize::try_from(kind).ok().and_then(|kind| crate::web::view::TOWN_PICTURES.get(kind)), spot((x as f32, y as f32))) {
                marks.push_str(&format!(r#"<img src="{}" alt="" draggable="false" data-tip="{}" style="position:absolute;left:{x:.2}%;top:{y:.2}%;transform:translate(-50%,-50%)">"#, picture(&format!("information/{picture_name}.bmp")), escape(&name)));
            }
        }
        let own = self.own_account();
        for member in self.party.members.iter().filter(|member| member.account != own && !member.offline && member.map.trim_end_matches(".gat") == map) {
            if let Some((x, y)) = self.party.places.get(&member.account).and_then(|(x, y)| spot((f32::from(*x), f32::from(*y)))) {
                marks.push_str(&format!(r#"<span data-tip="{}" style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:6px;height:6px;margin:-4px 0 0 -4px;background:#493a37;border:1px solid rgba(255,255,255,.6)"></span>"#, escape(&member.name)));
            }
        }
        if let Some((x, y)) = spot((f32::from(game.x), f32::from(game.y))) {
            marks.push_str(&format!(r#"<span style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:6px;height:6px;margin:-4px 0 0 -4px;background:#fff;border:1px solid #000"></span>"#));
        }
        window.push_str(&format!(
            r#"<div class="a" style="{};background:#aaa;overflow:hidden"><img src="{}" alt="" draggable="false" style="width:100%;height:100%;display:block" onerror="this.remove()">{marks}</div>"#,
            at(2, 19, 512, 512),
            crate::web::view::map_picture(map)
        ));
        window
    }
}
