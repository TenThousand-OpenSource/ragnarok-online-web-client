//! The navigation: what is asked of it from its window, from the links of a text, from the World Map, from
//! a command and from the server (`crate::navigation`), and what follows the character's own steps.

use std::rc::Rc;

use super::*;
use crate::map::Gat;
use crate::navigation::guide::{Display, Line, Mode};
use crate::navigation::{Atlas, Ground, Here, Target, What, texts};
use crate::press::navigation::Press;

/// The colour the navigation speaks in (`0x62B0FF` as the client writes a colour: blue first).
const SAID: &str = "#ffb062";
/// The colour of its word that a map is not one the search lists (`0x0A0AFF`).
const UNLISTED: &str = "#ff0a0a";

impl App {
    /// The map the character is on, as the navigation's tables name one, and its cell.
    fn navigation_here(&self) -> Option<(String, (i32, i32))> {
        self.game.as_ref().map(|game| (crate::maps::bare(&game.map).to_owned(), (i32::from(game.x), i32::from(game.y))))
    }

    /// Whether the navigation's own window shows: it may be there and kept away (`hideWindow`).
    pub(in crate::web) fn navigation_open(&self) -> bool {
        self.navigation.guide.window && !self.navigation.guide.hidden
    }

    /// Asks something of the navigation. What needs the tables, or the ground of another map, waits until
    /// they are read (`navigation_news`).
    pub(in crate::web::app) fn navigate(&mut self, wanted: Wanted) {
        let Some(atlas) = data::atlas() else {
            self.navigation.waits = Some(wanted);
            return data::want_atlas(|| dispatch(Action::Navigation(News::Tables)));
        };
        let Some((map, cell)) = self.navigation_here() else { return };
        let here = Here { map: &map, cell };
        // The way to a place on another map asks for that map's ground.
        let place = match &wanted {
            Wanted::Start { map, kind: 0, .. } => atlas.find(map).map(|map| Target::of(What::Place(map))),
            Wanted::Find => self.navigation.guide.current.clone(),
            _ => None,
        };
        let needed = self.navigation.guide.ground_needed(&atlas, place.as_ref(), here).map(str::to_owned);
        if let Some(name) = needed.as_ref().filter(|name| !self.navigation.grounds.contains_key(*name)) {
            self.navigation.waits = Some(wanted);
            return ground_wanted(name.clone());
        }
        let there = needed.and_then(|name| self.navigation.grounds.get(&name).cloned().flatten());
        let was = self.navigation_windows();
        self.navigation_acts(&atlas, wanted, here, there);
        self.navigation_told(was);
    }

    /// Does what was asked, with the grounds it needs.
    fn navigation_acts(&mut self, atlas: &Atlas, wanted: Wanted, here: Here, there: Option<Rc<Gat>>) {
        let guide = &mut self.navigation.guide;
        let there = there.as_deref().map(|ground| ground as &dyn Ground);
        match wanted {
            Wanted::Search { text, kind, hide } => {
                guide.look_for(atlas, &text, kind, hide);
            }
            Wanted::Point { map, x, y } => {
                guide.point_at(atlas, &map, x, y);
            }
            Wanted::Open => (guide.window, guide.hidden) = (true, false),
            Wanted::Start { map, kind, mode, hide, x, y, id } => {
                world::with_ground(|ground| guide.start(atlas, (&map, kind, mode, hide), (x, y, id), here, ground, there));
            }
            Wanted::Find => {
                world::with_ground(|ground| guide.find(atlas, here, ground, there));
            }
            Wanted::Resume => {
                world::with_ground(|ground| guide.resume(atlas, here, ground));
            }
        }
    }

    /// Which of the navigation's windows show: so that the screen is written anew only when that changes.
    fn navigation_windows(&self) -> [bool; 5] {
        [self.navigation_open(), self.navigation.guide.small, self.navigation.route, self.navigation.icons, self.navigation.asks]
    }

    /// After the navigation did something: what it has to say goes into the chat, the way is marked on the
    /// ground, the picture of what is looked at is asked for, and its windows are drawn anew.
    fn navigation_told(&mut self, was: [bool; 5]) {
        for line in std::mem::take(&mut self.navigation.guide.said) {
            match line {
                Line::Result(result) => {
                    let words = texts::result_line(result).map_or_else(|| "NoMsg.".to_owned(), |(line, otherwise)| Self::words(line, otherwise));
                    self.chat_line(crate::numbers::filled(&Self::words(2315, "Navigation >: %s"), &[&words]), SAID);
                }
                Line::TalkTo(name) => self.chat_line(crate::numbers::filled(&Self::words(2316, "Navigation >: Talk to [%s]"), &[&name]), SAID),
                Line::MoveTo(map) => {
                    self.chat_line(crate::numbers::filled(&Self::words(2317, "Navigation >: Please move to [%s] map"), &[&map]), SAID);
                }
                Line::Airship => self.chat_line(Self::words(2319, "Navigation >: Please move to the Airship"), SAID),
                Line::Portal => self.chat_line(Self::words(2320, "Navigation >: Use the Warp Portal to move to the next area."), SAID),
                Line::Arrived => self.chat_line(Self::words(2309, "Arrived at the target map"), SAID),
                Line::Unsupported => self.chat_line(Self::words(2293, "That does not support the navigation area"), SAID),
                Line::Unlisted => self.say_filled(said::GENERAL, 4195, &[], UNLISTED),
            }
        }
        if self.navigation.guide.nodes.is_empty() {
            self.navigation.route = false;
        }
        self.navigation_marked();
        self.navigation_pictured();
        if self.navigation_windows() != was {
            self.render();
        } else {
            self.redraw(&[&wnd::NAVIGATION, &wnd::NAVIGATION_ROUTE, &wnd::NAVIGATION_ICONS, &wnd::WORLD_MAP]);
        }
    }

    /// The way is marked on the ground as the navigation has it now (`CNavigationManager::DrawRoute`).
    fn navigation_marked(&self) {
        let guide = &self.navigation.guide;
        let on_goal_map = || {
            let (atlas, here) = (data::atlas()?, self.navigation_here()?);
            Some(guide.goal.map(|(map, _)| map) == atlas.find(&here.0))
        };
        world::way_marked((guide.shown && !guide.way.is_empty()).then(|| world::Marked {
            cells: guide.way.clone(),
            icon: guide.icon,
            at_goal: guide.at_goal && on_goal_map().unwrap_or(false),
        }));
    }

    /// The picture of the NPC or the monster looked at is made, the first time it is wanted.
    fn navigation_pictured(&mut self) {
        let Some(atlas) = data::atlas() else { return };
        let job = match self.navigation.guide.current.as_ref().map(|target| target.what) {
            Some(What::Npc(npc)) => atlas.npcs[npc].job(),
            Some(What::Mob(mob)) => atlas.mobs[mob].job(),
            _ => return,
        };
        if self.navigation.pictures.contains_key(&job) || !self.navigation.pictures_asked.insert(job) {
            return;
        }
        // (Out of the folder of its job: `crate::creatures::sprite`.)
        let Some(sprite) = u16::try_from(job).ok().and_then(|job| Some(crate::creatures::sprite(job, &data::sprite(job)?))) else { return };
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(picture) = figure::frame_picture(&sprite, 0, 0).await {
                dispatch(Action::Navigation(News::Picture(job, picture)));
            }
        });
    }

    /// Something the navigation waited for has come.
    pub(in crate::web::app) fn navigation_news(&mut self, news: News) {
        match news {
            News::Tables => {}
            News::Ground(map, ground) => {
                self.navigation.grounds.insert(map, ground);
            }
            News::Picture(job, picture) => {
                self.navigation.pictures.insert(job, picture);
                return self.redraw(&[&wnd::NAVIGATION]);
            }
            News::Noticed => return self.redraw(&[&wnd::NAVIGATION]),
        }
        if let Some(wanted) = self.navigation.waits.take() {
            self.navigate(wanted);
        }
    }

    /// A map is shown (`CGameMode`'s `ResumeRoute`): the tables are asked for as the game is first entered,
    /// and a navigation under way goes on from here.
    pub(in crate::web::app) fn navigation_entered(&mut self) {
        if data::atlas().is_none() {
            return data::want_atlas(|| dispatch(Action::Navigation(News::Tables)));
        }
        if self.navigation.guide.mode != Mode::Idle {
            self.navigate(Wanted::Resume);
        }
    }

    /// The character leaves a map: the marks of the way are of the map left.
    pub(in crate::web::app) fn navigation_left_map(&mut self) {
        world::way_marked(None);
    }

    /// The character is on another cell: strayed from the marks, the way is found anew; and the window that
    /// shows the way shows where the character is.
    pub(in crate::web::app) fn navigation_stepped(&mut self) {
        if self.navigation.guide.mode != Mode::Going {
            return;
        }
        let Some((_, cell)) = self.navigation_here() else { return };
        let guide = &mut self.navigation.guide;
        if world::with_ground(|ground| guide.strayed(cell, ground)).unwrap_or(false) {
            self.navigation_marked();
        }
        if self.navigation_open() && self.navigation.guide.display == Display::Way {
            self.redraw(&[&wnd::NAVIGATION]);
        }
    }

    /// The server sent the character walking to a cell (`CheckArrival`): near the end of the way the
    /// navigation is over.
    pub(in crate::web::app) fn navigation_walked(&mut self, to: (i32, i32)) {
        let was = self.navigation_windows();
        if self.navigation.guide.arrival(to) {
            self.navigation_told(was);
        }
    }

    /// The ground was pressed to walk (`UpdateNavigationList`): with the box "Minimize" ticked, the window
    /// becomes the small one.
    pub(in crate::web::app) fn navigation_walk_asked(&mut self) {
        if self.navigation_open() && self.navigation.fold && !self.navigation.guide.small {
            self.pressed(Press::Small);
        }
    }

    /// Back to the characters (`ZC_RESTART_ACK`): the navigation forgets everything.
    pub(in crate::web::app) fn navigation_forgotten(&mut self) {
        self.navigation.guide.left();
        (self.navigation.route, self.navigation.icons, self.navigation.asks, self.navigation.kinds) = (false, false, false, false);
        self.navigation.waits = None;
        world::way_marked(None);
    }

    /// The line of the window was sent (`ID_SEARCH`): what was typed is searched for.
    pub(in crate::web::app) fn navigation_searched(&mut self) {
        let text = field(&wnd::NAVIGATION.field());
        let Some(atlas) = data::atlas().filter(|_| !text.is_empty() && !self.navigation.guide.small) else { return };
        let guide = &mut self.navigation.guide;
        guide.sought = text;
        guide.search(&atlas);
        guide.current = None;
        self.redraw(&[&wnd::NAVIGATION, &wnd::WORLD_MAP]);
    }

    /// The World Map's line was sent (its `ID_SEARCH`): the search is made with the window kept away, and the
    /// map marks what was found.
    pub(in crate::web::app) fn navigation_world_searched(&mut self) {
        let text = field(&wnd::WORLD_MAP.field());
        if !text.is_empty() {
            self.navigation.world_sought = text.clone();
            self.navigate(Wanted::Search { text, kind: 0, hide: true });
        }
    }

    /// A window beside the navigation's opens by where that one stands now (`UIWindowMgr::EnsureWindow`).
    fn navigation_beside(&mut self) {
        if let Some((x, y, ..)) = Self::window_rect(wnd::NAVIGATION.id) {
            self.navigation.at = Some((x, y));
        }
    }

    /// A press of this system (`crate::press::navigation`).
    pub(super) fn navigation_pressed(&mut self, press: Press) {
        let was = self.navigation_windows();
        match press {
            // The menu's icon and the key (`MENUICON_NAVIGATION`, `HK_NAVIGATION_ONOFF`): the window is closed
            // as its cross closes it, or opened.
            Press::Window if self.navigation.guide.window => return self.navigation_pressed(Press::Close),
            Press::Window => return self.navigate(Wanted::Open),
            // The cross: a navigation under way is asked about first.
            Press::Close if self.navigation.guide.mode == Mode::Going => self.navigation.asks = true,
            Press::Close => {
                self.navigation.guide.closed();
                (self.navigation.route, self.navigation.icons, self.navigation.kinds) = (false, false, false);
            }
            Press::Yes => {
                self.navigation.guide.give_up();
                (self.navigation.asks, self.navigation.route) = (false, false);
            }
            Press::No => self.navigation.asks = false,
            Press::Small => self.navigation.guide.small = true,
            // (Also the World Map's button that opens the window: `EnsureWindow`, then `ID_BASE`.)
            Press::Large => {
                let guide = &mut self.navigation.guide;
                (guide.small, guide.hidden, guide.window) = (false, false, true);
            }
            Press::Help => {
                self.navigation.help = !self.navigation.help;
                return self.render();
            }
            Press::Kinds => self.navigation.kinds = !self.navigation.kinds,
            Press::Fold => self.navigation.fold = !self.navigation.fold,
            Press::Service => {
                let on = !self.navigation.guide.allowed.zeny;
                self.navigation.guide.serviced(on);
            }
            Press::Icons => {
                self.navigation.icons = !self.navigation.icons;
                self.navigation_beside();
            }
            Press::Icon(icon) if (1..=8).contains(&icon) => self.navigation.guide.icon = icon,
            Press::Icon(_) => {}
            // (`NAVI_BUTTON_ROAD_LIST`: only with a route to list.)
            Press::Route => {
                self.navigation.route = !self.navigation.route && !self.navigation.guide.nodes.is_empty();
                self.navigation_beside();
            }
            Press::Find => return self.navigate(Wanted::Find),
            // The private airship and the files of places are not made here (`docs/DEBTS.md`).
            Press::Airship | Press::Files => {}
            Press::Go(info) => return self.navigation_linked(&info),
            Press::Chat(link) => {
                if let Some((map, x, y)) = texts::chat_linked(&link) {
                    return self.navigate(Wanted::Point { map: map.to_owned(), x, y });
                }
            }
            // A map of the World Map (`UIRoMapWnd::OnLBtnUp`): what the map has is searched for and the
            // first of it looked at, in the large window.
            Press::Field(map) => {
                self.navigation.guide.small = false;
                self.navigate(Wanted::Search { text: map, kind: 0, hide: false });
                return self.navigation_pressed(Press::Group(0));
            }
            // A mark of the minimap (`UIMinimapZoomWnd`'s buttons of a town): the way to that cell of this map.
            Press::Mark(place) => {
                let Some((map, _)) = self.navigation_here() else { return };
                return self.navigate(Wanted::Start { map, kind: 0, mode: 5, hide: true, x: place.0, y: place.1, id: 1002 });
            }
            Press::Turn | Press::Kind(_) | Press::Group(_) | Press::Member(_) | Press::Share | Press::Pick(_) => self.navigation_listed(press),
        }
        self.navigation_told(was);
    }

    /// The presses that go by the tables: the lists, the arrow that turns the window, the sharing of a place.
    fn navigation_listed(&mut self, press: Press) {
        let Some(atlas) = data::atlas() else { return };
        let guide = &mut self.navigation.guide;
        match press {
            Press::Turn => guide.turned(&atlas),
            Press::Kind(kind) if (0..=3).contains(&kind) => {
                (guide.kind, self.navigation.kinds) = (kind, false);
                guide.regroup(&atlas);
            }
            Press::Group(group) if group < guide.groups.len() => guide.pick_group(&atlas, group),
            Press::Member(member) => guide.pick_member(&atlas, member),
            // (`ID_PREVIEW`): a map is shown to press a place on, and says so for five seconds; an NPC or a
            // monster goes into the chat's line as a link.
            Press::Share => match guide.current.as_ref().map(|target| target.what) {
                Some(What::Place(_) | What::Map(_)) => {
                    guide.display = Display::Pick;
                    self.navigation.notice = crate::web::now() + 5000.0;
                    after(5000, Action::Navigation(News::Noticed));
                }
                Some(What::Npc(npc)) => chat_linked(&atlas.maps[atlas.npcs[npc].map].name, atlas.npcs[npc].x, atlas.npcs[npc].y),
                Some(What::Mob(mob)) => chat_linked(&atlas.maps[atlas.mobs[mob].map].name, 1000, 1000),
                Some(What::Node(_)) | None => {}
            },
            // A place pressed on the map shown so (`UINavigationV4Wnd::OnLBtnUp`).
            Press::Pick(place) if guide.display == Display::Pick => {
                let Some(map) = guide.current.as_ref().map(|target| &atlas.maps[atlas.map_of(target.what)]) else { return };
                if let Some((x, y)) = texts::pressed_cell(map.width, map.height, (place.0, place.1)) {
                    chat_linked(&map.name, x, y);
                }
            }
            _ => {}
        }
    }

    /// A link `<NAVI>` of a text was pressed (`OpenTextButtonNavigation`).
    fn navigation_linked(&mut self, info: &str) {
        let fields: Vec<String> = info.split(',').map(|field| field.trim().to_owned()).collect();
        match texts::linked(&fields) {
            Some(texts::Asked::Start { map, kind, mode, hide, x, y, id }) => self.navigate(Wanted::Start { map, kind, mode, hide, x, y, id }),
            Some(texts::Asked::Search { text, kind }) => self.navigate(Wanted::Search { text, kind, hide: false }),
            None => {}
        }
    }

    /// `/navi <map> <x>/<y>` and `/navi2 <map> <x>/<y> <scroll>/<zeny>/<airship>` (`TT_NAVIGATION`,
    /// `TT_NAVIGATION2`): the way to a cell of a map, with the window kept away; `/naviopen` opens the window.
    pub(in crate::web::app) fn navigation_command(&mut self, words: &[&str], switched: bool) {
        let cell = words.get(1).and_then(|cell| texts::cell(cell));
        let mode = if switched { words.get(2).and_then(|switches| texts::switches(switches)) } else { Some(5) };
        match (words.first(), cell, mode) {
            (Some(map), Some((x, y)), Some(mode)) => self.navigate(Wanted::Start { map: (*map).to_owned(), kind: 0, mode, hide: true, x, y, id: 1002 }),
            _ => {
                let was = self.navigation_windows();
                self.navigation.guide.said.push(Line::Result(-100));
                self.navigation_told(was);
            }
        }
    }
}

/// Asks for the ground of a map (its `.gat`), and tells the app when it is read or cannot be.
fn ground_wanted(map: String) {
    wasm_bindgen_futures::spawn_local(async move {
        let read = crate::web::fetch_asset(&format!("data/{map}.gat")).await.ok().and_then(|bytes| Gat::parse(&bytes).ok());
        dispatch(Action::Navigation(News::Ground(map, read.map(Rc::new))));
    });
}

/// A link to a place is written into the chat's line (`UINavigationV4Wnd::InsertNavigationLink`), which is
/// given the keyboard.
fn chat_linked(map: &str, x: i32, y: i32) {
    set_field("chat-input", &format!("{}{}", field("chat-input"), texts::chat_link(map, x, y)));
    if let Some(line) = element("chat-input").and_then(|line| line.dyn_into::<web_sys::HtmlElement>().ok()) {
        let _ = line.focus();
    }
}
