use super::atlas::Rows;
use super::guide::{Display, Line, Mode};
use super::route::{self, Allowed};
use super::search::Words;
use super::texts::{self, Asked, Picture};
use super::*;
use crate::map::Gat;

fn text(words: &str) -> String {
    words.to_owned()
}

/// A town, a field south of it, a cave east of the field and a room of the town: the town's gate (1) leads to
/// the field, the field's (2) back and (3) into the cave, the cave's (4) back to the field; a Kafra in the
/// town (5) takes one into the cave for zeny. A Kafra stands in the cave, Porings and a boss live in the field.
fn world() -> Atlas {
    let map = |name: &str, title: &str, kind: i32| (text(name), text(title), kind, 100, 100);
    let link = |map: &str, id: i32, kind: i32, name: &str, at: (i32, i32), to: &str, there: (i32, i32)| {
        (text(map), id, kind, 99999, text(name), String::new(), at.0, at.1, text(to), there.0, there.1)
    };
    let pass = |kind: &str, id: i32, cost: i32| (text(kind), id, cost);
    Atlas::new(Rows {
        maps: vec![map("town", "The Town", 5001), map("field", "South Field", 5001), map("cave", "Dark Cave", 5001), map("town_in", "A Room", 5003)],
        npcs: vec![
            (text("cave"), 100, 101, 114, text("Kafra Employee"), text("kaf_cave"), 30, 50),
            (text("nowhere"), 101, 101, 114, text("Lost"), String::new(), 1, 1),
        ],
        mobs: vec![
            (text("field"), 200, 300, (30 << 16) | 1002, text("Poring"), text("PORING"), 1, 0x150103),
            (text("field"), 201, 301, (1 << 16) | 1511, text("Amon Ra"), text("AMON_RA"), 88, 0x3d0207),
        ],
        links: vec![
            link("town", 1, 200, "town-field", (50, 10), "field", (50, 90)),
            link("field", 2, 200, "field-town", (50, 95), "town", (50, 15)),
            link("field", 3, 200, "field-cave", (90, 50), "cave", (10, 50)),
            link("cave", 4, 200, "cave-field", (5, 50), "field", (85, 50)),
            link("town", 5, 204, "Kafra", (20, 20), "cave", (50, 50)),
        ],
        ways: vec![
            (text("town"), 1, vec![pass("E", 2, 6), pass("E", 3, 60), pass("P", 5, 31)]),
            (text("field"), 2, vec![pass("E", 1, 5), pass("E", 5, 40)]),
            (text("field"), 3, vec![pass("E", 4, 6)]),
            (text("cave"), 4, vec![pass("E", 2, 50), pass("E", 3, 6)]),
            (text("town"), 5, vec![pass("E", 4, 45)]),
        ],
        npc_ways: vec![(text("cave"), 100, vec![pass("field", 3, 21), pass("town", 5, 20), pass("nowhere", 9, 1)])],
        picks: vec![(text("cave"), 30, 50, [4001, 1, 0, 2, 3, 0, 0, 0, 0])],
    })
}

fn here(map: &str, cell: (i32, i32)) -> Here<'_> {
    Here { map, cell }
}

#[test]
fn the_tables_are_put_together_by_map() {
    let atlas = world();
    let (town, field, cave) = (atlas.find("town").unwrap(), atlas.find("field.gat").unwrap(), atlas.find("cave").unwrap());
    assert_eq!((atlas.maps[town].links.len(), atlas.maps[field].mobs.len(), atlas.maps[cave].npcs.len()), (2, 2, 1));
    // A row of a map the tables do not have is passed by.
    assert_eq!((atlas.npcs.len(), atlas.find("nowhere")), (1, None));
    // A copy of a map made for a party is the map.
    assert_eq!(atlas.find("0011@cave"), None);
    let gate = atlas.link_of(town, 1).unwrap();
    assert_eq!(atlas.links[gate].onward, [(atlas.link_of(field, 2).unwrap(), 6), (atlas.link_of(field, 3).unwrap(), 60)]);
    assert_eq!(atlas.links[gate].near, [(atlas.link_of(town, 5).unwrap(), 31)]);
    let kafra = atlas.npc_at(cave, 30, 50).unwrap();
    assert_eq!(atlas.npcs[kafra].ways, [(atlas.link_of(field, 3).unwrap(), 21), (atlas.link_of(town, 5).unwrap(), 20)]);
    assert_eq!(atlas.npcs[kafra].look, Some([4001, 1, 0, 2, 3, 0, 0, 0, 0]));
    let poring = atlas.mob_of(field, 1002).unwrap();
    assert_eq!((atlas.mobs[poring].job(), atlas.mobs[poring].many(), atlas.mob_titled("AMON_RA")), (1002, 30, Some(poring + 1)));
    assert_eq!(atlas.ways_into(cave), [atlas.link_of(town, 5).unwrap(), atlas.link_of(field, 3).unwrap()]);
    assert!(atlas.maps[town].searchable() && !atlas.maps[atlas.find("town_in").unwrap()].searchable());
}

#[test]
fn a_search_finds_maps_monsters_and_npcs_by_every_word() {
    let atlas = world();
    let words = Words::default();
    let kinds = |query: &str| atlas.search(query, 0, &words).map(|found| found.iter().map(|target| (target.what.kind(), target.name.clone())).collect::<Vec<_>>());
    assert_eq!(kinds(""), None);
    assert_eq!(kinds("   "), None);
    // A map found lists its monsters under it; a monster is found by its own name too.
    assert_eq!(kinds("south FIELD"), Some(vec![(1, text("field")), (3, text("Poring")), (3, text("Amon Ra"))]));
    assert_eq!(kinds("por"), Some(vec![(3, text("Poring"))]));
    assert_eq!(kinds("kafra employee"), Some(vec![(2, text("Kafra Employee"))]));
    // A room indoors is not found by its name.
    assert_eq!(kinds("room"), Some(vec![]));
    assert_eq!(kinds("$$lv80~"), Some(vec![(3, text("Amon Ra"))]));
    assert_eq!(kinds("$$lv1"), Some(vec![(3, text("Poring"))]));
    assert_eq!(kinds("$$DB PORING"), Some(vec![(3, text("Poring"))]));
    assert_eq!(kinds("$$all").map(|found| found.len()), Some(3 + 1 + 2));
    // Levels that are none are words like any other, and find nothing.
    assert_eq!(kinds("$$lv9~3"), Some(vec![]));
}

#[test]
fn what_was_found_is_listed_in_groups() {
    let atlas = world();
    let mut guide = Guide::new();
    guide.sought = text("o");
    assert!(guide.search(&atlas));
    // Ordered by name, the later first: the town and the field are found as maps ("o" is in what the client
    // calls each), the field's monsters under it, and the Poring, Amon Ra and the Kafra by their own names.
    let rows: Vec<(i32, String)> = guide.groups.iter().filter_map(|group| texts::group_row(&atlas, &guide.found, group)).collect();
    assert_eq!(rows, [
        (1, text("[1]town")),
        (1, text("[3]field")),
        (3, text("[1]Poring")),
        (2, text("[1]Kafra Employee")),
        (3, text("[1]Amon Ra")),
    ]);
    guide.pick_group(&atlas, 1);
    let density = |many: i32| texts::density_line(many).map(|(_, words)| words.to_owned());
    let members: Vec<(i32, String)> = guide.groups[1].iter().map(|index| texts::member_row(&atlas, &guide.found[*index], density)).collect();
    assert_eq!(members, [(1, text("South Field")), (3, text("Poring (Average)")), (3, text("Amon Ra (Very Low)"))]);
    assert_eq!(guide.current.as_ref().map(|target| target.what), Some(What::Map(1)));
    guide.pick_member(&atlas, 1);
    assert_eq!(guide.current.as_ref().map(|target| (target.what.kind(), target.under_its_map())), Some((3, true)));
    // A member that is not there, and the first is looked at.
    guide.pick_member(&atlas, 9);
    assert_eq!((guide.member, guide.current.as_ref().map(|target| target.what)), (Some(0), Some(What::Map(1))));
    // Narrowed to NPCs.
    guide.kind = 2;
    guide.regroup(&atlas);
    assert_eq!(guide.groups.len(), 1);
    // The group named as the text sought comes first.
    (guide.kind, guide.sought) = (0, text("cave"));
    guide.search(&atlas);
    assert_eq!(texts::group_row(&atlas, &guide.found, &guide.groups[0]), Some((1, text("[1]cave"))));
}

#[test]
fn a_route_takes_the_first_way_in_it_comes_upon() {
    let atlas = world();
    let (town, field, cave) = (0, 1, 2);
    let link = |map: usize, id: i32| atlas.link_of(map, id).unwrap();
    let start = [(link(town, 1), 12), (link(town, 5), 40)];
    let plain = Allowed::default();
    assert_eq!(route::find(&atlas, &start, &[], cave, true, plain), Some((72, vec![link(town, 1), link(field, 3)])));
    // For zeny the Kafra takes one straight there.
    let zeny = Allowed { zeny: true, ..plain };
    assert_eq!(route::find(&atlas, &start, &[], cave, true, zeny), Some((40, vec![link(town, 5)])));
    // To a place, only a way in from which it is reached will do, and its walk is counted.
    let end = [(link(field, 3), 21)];
    assert_eq!(route::find(&atlas, &start, &end, cave, false, zeny), Some((93, vec![link(town, 1), link(field, 3)])));
    assert_eq!(route::find(&atlas, &start, &[], cave, false, zeny), None);
    assert_eq!(route::find(&atlas, &[], &[], cave, true, zeny), None);
    // Back and forth: the town from the cave.
    assert_eq!(route::find(&atlas, &[(link(cave, 4), 3)], &[], town, true, plain), Some((53, vec![link(cave, 4), link(field, 2)])));
}

#[test]
fn a_navigation_to_an_npc_leads_from_map_to_map() {
    let atlas = world();
    let ground = Gat::open(100, 100);
    let mut guide = Guide::new();
    assert!(guide.start(&atlas, ("cave", 2, 0, false), (30, 50, 1002), here("town", (50, 12)), &ground, None));
    assert_eq!((guide.mode, guide.display, guide.window, guide.node), (Mode::Going, Display::Way, true, Some(0)));
    assert_eq!(guide.said, [Line::Result(5), Line::Portal]);
    assert_eq!((guide.way.first(), guide.way.last(), guide.shown, guide.arriving), (Some(&(50, 12)), Some(&(50, 10)), true, false));
    assert_eq!(guide.route_maps(&atlas), ["town", "field", "cave"]);
    assert_eq!(guide.way_lines(&atlas, "Destination"), [text("The Town"), text("town 50 10"), text("[Destination] -> cave 30 50")]);
    assert_eq!(guide.small_lines(&atlas), [text("town 50 10..."), text(">.cave 30 50")]);
    // Walking towards the gate ends nothing.
    assert!(!guide.arrival((50, 10)));
    // On the field, arrived where the gate leads, the next way out is gone to.
    guide.said.clear();
    guide.resume(&atlas, here("field", (50, 90)), &ground);
    assert_eq!((guide.node, guide.way.last(), guide.said.as_slice()), (Some(1), Some(&(90, 50)), [Line::Result(5), Line::Portal].as_slice()));
    assert_eq!(guide.current.as_ref().map(|target| target.what.kind()), Some(6));
    // In the cave the way leads to the NPC, and near it the navigation is over.
    guide.said.clear();
    guide.resume(&atlas, here("cave", (10, 50)), &ground);
    assert_eq!((guide.node, guide.way.last(), guide.said.as_slice(), guide.at_goal), (None, Some(&(30, 50)), [Line::Result(7)].as_slice(), true));
    assert!(guide.nodes.is_empty());
    assert!(!guide.arrival((20, 50)));
    assert!(guide.arrival((27, 53)));
    assert_eq!((guide.mode, guide.saved.is_some(), guide.shown, guide.said.last()), (Mode::Sought, false, false, Some(&Line::Arrived)));
}

#[test]
fn a_navigation_says_why_it_came_to_nothing() {
    let atlas = world();
    let ground = Gat::open(100, 100);
    let mut guide = Guide::new();
    // No NPC on that cell, no such monster, no such map.
    assert!(!guide.start(&atlas, ("cave", 2, 0, true), (1, 1, 0), here("town", (50, 12)), &ground, None));
    assert!(!guide.start(&atlas, ("field", 3, 0, true), (0, 0, 4000), here("town", (50, 12)), &ground, None));
    assert!(!guide.start(&atlas, ("moon", 1, 0, true), (0, 0, 0), here("town", (50, 12)), &ground, None));
    assert_eq!(std::mem::take(&mut guide.said), [Line::Result(-94), Line::Result(-94)]);
    // From inside a copy of a map only that map is gone to.
    assert!(!guide.start(&atlas, ("cave", 1, 0, true), (0, 0, 0), here("0011@tower", (5, 5)), &ground, None));
    assert_eq!(std::mem::take(&mut guide.said), [Line::Result(-90)]);
    // A place on another map asks for that map's ground.
    assert_eq!(guide.ground_needed(&atlas, Some(&Target::of(What::Place(2))), here("town", (1, 1))), Some("cave"));
    assert_eq!(guide.ground_needed(&atlas, Some(&Target::of(What::Place(0))), here("town", (1, 1))), None);
    assert!(!guide.start(&atlas, ("cave", 0, 0, true), (40, 40, 0), here("town", (50, 12)), &ground, None));
    assert_eq!(std::mem::take(&mut guide.said), [Line::Result(-97)]);
    assert!(guide.start(&atlas, ("cave", 0, 0, true), (40, 40, 0), here("town", (50, 12)), &ground, Some(&ground)));
    assert_eq!((guide.window, guide.hidden, guide.nodes.len()), (false, true, 2));
    // A map and a monster are arrived at as their map is entered.
    let mut guide = Guide::new();
    assert!(guide.start(&atlas, ("field", 3, 0, true), (0, 0, 1002), here("town", (50, 12)), &ground, None));
    guide.said.clear();
    guide.resume(&atlas, here("field", (50, 90)), &ground);
    assert_eq!((guide.said.as_slice(), guide.mode), ([Line::Arrived, Line::Result(8)].as_slice(), Mode::Idle));
    // From a map the tables do not have there is no way.
    let mut guide = Guide::new();
    assert!(!guide.start(&atlas, ("cave", 1, 0, true), (0, 0, 0), here("moon", (5, 5)), &ground, None));
    assert_eq!(guide.said, [Line::Result(-93)]);
}

#[test]
fn a_place_on_this_map_is_walked_to_and_reached() {
    let atlas = world();
    let mut ground = Gat::open(100, 100);
    let mut guide = Guide::new();
    assert!(guide.start(&atlas, ("town", 0, 5, false), (60, 60, 0), here("town", (50, 50)), &ground, None));
    assert_eq!((guide.said.as_slice(), guide.way.len(), guide.goal), ([Line::Result(9)].as_slice(), 11, Some((0, (60, 60)))));
    assert_eq!((guide.allowed.zeny, guide.allowed.airship, guide.allowed.scrolls), (true, true, false));
    // Strayed from every mark of the way, it is found anew from where one stands.
    assert!(!guide.strayed((52, 52), &ground));
    assert!(guide.strayed((20, 20), &ground));
    assert_eq!((guide.way.first(), guide.way.last()), (Some(&(20, 20)), Some(&(60, 60))));
    // Standing by it already.
    let mut guide = Guide::new();
    assert!(guide.start(&atlas, ("town", 0, 0, true), (51, 51, 0), here("town", (50, 50)), &ground, None));
    assert_eq!(guide.said, [Line::Result(11)]);
    // The window's own button, with a target looked at; a wall all round the place, and no way is found.
    for (x, y) in [(69, 69), (70, 69), (71, 69), (69, 70), (71, 70), (69, 71), (70, 71), (71, 71)] {
        ground.block(x, y);
    }
    let mut guide = Guide::new();
    assert!(guide.point_at(&atlas, "town", 70, 70));
    assert_eq!((guide.mode, guide.window, guide.current.as_ref().map(|target| target.name.clone())), (Mode::Sought, true, Some(text("town 70,70"))));
    guide.find(&atlas, here("town", (50, 50)), &ground, None);
    assert_eq!(guide.said, [Line::Result(-97)]);
    assert!(guide.point_at(&atlas, "cave", 1000, 1000));
    assert_eq!(guide.current.as_ref().map(|target| target.what), Some(What::Map(2)));
    guide.said.clear();
    guide.find(&atlas, here("town", (50, 50)), &ground, None);
    assert_eq!((guide.said.as_slice(), guide.display), ([Line::Result(5), Line::Portal].as_slice(), Display::Way));
    // Given up from the window: what was gone to is forgotten, the window stays.
    guide.give_up();
    assert_eq!((guide.mode, guide.saved.is_none(), guide.shown, guide.window), (Mode::Sought, true, false, true));
    guide.closed();
    assert_eq!((guide.mode, guide.window), (Mode::Idle, false));
}

#[test]
fn the_ground_gives_a_way_with_both_its_ends() {
    let ground = Gat::open(10, 10);
    assert_eq!(Ground::way(&ground, (1, 1), (3, 3)), Some(vec![(1, 1), (2, 2), (3, 3)]));
    assert_eq!(Ground::way(&ground, (1, 1), (1, 1)), None);
    assert_eq!(Ground::way(&ground, (1, 1), (30, 3)), None);
}

#[test]
fn the_commands_and_the_links_are_read() {
    assert_eq!((texts::cell("100/90"), texts::cell("100"), texts::cell("/9"), texts::cell("501/1"), texts::cell("0/500")), (Some((100, 90)), None, None, None, Some((0, 500))));
    assert_eq!((texts::switches("1/0/1"), texts::switches("0/1/0"), texts::switches("1/1"), texts::switches("2/0/0")), (Some(6), Some(1), None, None));
    assert_eq!((texts::mode_of(0), texts::mode_of(1), texts::mode_of(100), texts::mode_of(111), texts::mode_of(10)), (0, 4, 1, 7, 2));
    let fields = |info: &str| info.split(',').map(str::to_owned).collect::<Vec<_>>();
    assert_eq!(texts::linked(&fields("prontera,150,180")), Some(Asked::Start { map: text("prontera"), kind: 2, mode: 5, hide: false, x: 150, y: 180, id: 1002 }));
    assert_eq!(
        texts::linked(&fields("prontera,150,180,0,101,0,0")),
        Some(Asked::Start { map: text("prontera"), kind: 0, mode: 5, hide: true, x: 150, y: 180, id: 0 })
    );
    assert_eq!(texts::linked(&fields("PORING,0,0,3,-222")), Some(Asked::Search { text: text("$$DB PORING"), kind: 3 }));
    assert_eq!(texts::linked(&fields("prontera,0,0,1,-222")), Some(Asked::Search { text: text("prontera"), kind: 1 }));
    assert_eq!(texts::linked(&[]), None);
}

#[test]
fn a_place_is_written_into_the_chat_and_read_back() {
    assert_eq!(texts::chat_link("prontera", 150, 180), "<NAVIL>prontera2q2U</NAVIL>");
    assert_eq!(texts::chat_linked("prontera2q2U"), Some(("prontera", 150, 180)));
    assert_eq!(texts::chat_link("moc_fild01", 5, 61), "<NAVIL>moc_fild010500Z</NAVIL>".replace("0500Z", "050Z"));
    assert_eq!(texts::chat_linked("x0g8g8").map(|(map, x, y)| (map, x, y)), Some(("x0", 1000, 1000)));
    assert_eq!(texts::chat_link("cave", -1, -1), "<NAVIL>caveg8g8</NAVIL>");
    assert_eq!(texts::chat_linked("g8g8"), None);
    assert_eq!(texts::chat_parts("go <NAVIL>cave0505</NAVIL> now<NAVIL>x"), [Ok("go "), Err("cave0505"), Ok(" now<NAVIL>x")]);
    assert_eq!(texts::chat_parts("plain"), [Ok("plain")]);
    assert_eq!((texts::chat_link_words("cave", 1000, 1000), texts::chat_link_words("cave", 5, 9)), (text("<cave>"), text("<cave 5,9>")));
}

#[test]
fn a_target_is_shown_with_its_lines_and_its_picture() {
    let atlas = world();
    assert_eq!(texts::wrapped("Kafra"), ["[Kafra]"]);
    assert_eq!(texts::wrapped("Kafra Employee"), ["[Kafra Empl", "oyee]"]);
    assert_eq!(texts::wrapped("0123456789a"), ["[0123456789", "a]"]);
    // Twenty-three letters with the bracket: the client cuts for two lines and loses the last letter.
    assert_eq!(texts::wrapped("0123456789abcdefghijkl"), ["[0123456789", "abcdefghijk]"]);
    assert_eq!(texts::monster_lines(0x310101), ((2271, 2), 2276, 2273));
    assert_eq!(texts::monster_lines(0x000000), ((2272, 0), 2285, 2275));
    assert_eq!(texts::picture(&atlas, Some(&Target::of(What::Map(0)))), Some(Picture::Map(text("town"))));
    assert_eq!(texts::picture(&atlas, Some(&Target::of(What::Place(3)))), Some(Picture::Indoors));
    assert_eq!(texts::picture(&atlas, None), None);
    assert_eq!((texts::result_line(5).map(|line| line.0), texts::result_line(-96).map(|line| line.0), texts::result_line(77)), (Some(2308), Some(2298), None));
}

#[test]
fn the_world_map_marks_each_map_found_once() {
    let marks = texts::map_marks(&[("field", 0), ("field", 30), ("field", 3), ("cave", 10000), ("cave", 120), ("town", 2), ("town", 10000), ("moc", 10000)]);
    let named = |map: &str, picture: &str| (map.to_owned(), picture.to_owned());
    assert_eq!(marks, [named("field", "Mob3"), named("cave", "Npc_Mob5"), named("town", "Npc_Mob1"), named("moc", "Npc")]);
}

#[test]
fn a_cell_has_its_place_on_a_square_picture_of_its_map() {
    assert_eq!(texts::fitted(400, 200, 100.0), (0.25, 0, 25));
    assert_eq!(texts::fitted(100, 200, 100.0), (0.5, 25, 0));
    assert_eq!(texts::on_view(200, 200, (100, 50), 266.0), (133, 199));
    assert_eq!(texts::on_view(400, 200, (0, 0), 100.0), (0, 75));
    assert_eq!(texts::pressed_cell(265, 265, (10, 10)), Some((10, 255)));
    assert_eq!(texts::pressed_cell(530, 265, (10, 132)), Some((20, 134)));
    assert_eq!(texts::pressed_cell(530, 265, (10, 10)), None);
    assert_eq!(texts::pressed_cell(265, 265, (300, 10)), None);
}
