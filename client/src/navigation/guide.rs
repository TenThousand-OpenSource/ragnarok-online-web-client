//! What the navigation is doing now (the client's `CNavigationManager`): what was searched for and found, the
//! target looked at and the one being gone to, the route across the maps and the way over the map one is on,
//! and how its window shows all that. The ways of getting somewhere are in `guide/going.rs`.

use super::route::Allowed;
use super::search::{self, Words};
use super::{Atlas, Target, What};

mod going;

/// How far the navigation has got (`viewMode`): nothing asked, something searched for, on the way.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Idle,
    Sought,
    Going,
}

/// What the window shows (`displayMode`): the lists of what was found with the target's picture, the map of
/// the way, or a map to press a place on for the chat.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    #[default]
    Lists,
    Way,
    Pick,
}

/// What the navigation has to say in the chat (`ShowRouteResult`, `CancelRoute`, `SelectNodeState`): the app
/// writes each in the words of the client's message table.
#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    /// How an attempt ended, by the client's number for it (`texts::result_line`), after "Navigation >: ".
    Result(i32),
    /// "Talk to [name]": the next way out is an NPC.
    TalkTo(String),
    /// "Please move to [map]": where that NPC takes one.
    MoveTo(String),
    /// "Please move to the Airship".
    Airship,
    /// "Use the Warp Portal to move to the next area."
    Portal,
    /// "Arrived at the target map", as the way is over.
    Arrived,
    /// The map one is on is not in the navigation's tables (the client says so in a box of its own).
    Unsupported,
    /// The map gone to is one the search does not list (line 4195 of the message table).
    Unlisted,
}

/// The navigation's state.
#[derive(Debug, Default)]
pub struct Guide {
    /// What the last search found (`states`), the groups it is listed in (`nodePaths`), and which group and
    /// which of its members are picked (`selectedPath`, `selectedPathNode`).
    pub found: Vec<Target>,
    pub groups: Vec<Vec<usize>>,
    pub group: Option<usize>,
    pub member: Option<usize>,
    /// The target looked at (`currentTarget`) and the one being gone to (`savedTarget`).
    pub current: Option<Target>,
    pub saved: Option<Target>,
    /// What walking the route costs (`routeResult`), the ways out to take (`nodes`), and which of them is
    /// gone to on this map (`selectedNode`).
    pub cost: i32,
    pub nodes: Vec<usize>,
    pub node: Option<usize>,
    /// The way over the map one is on, cell by cell from where one stood (`movementRoute`); whether it is
    /// drawn (`drawRoute`), whether arriving near its end ends the navigation (`routeFlag`), and whether its
    /// end is the place sought itself (`drawDestination`).
    pub way: Vec<(i32, i32)>,
    pub shown: bool,
    pub arriving: bool,
    pub at_goal: bool,
    /// Where the navigation leads (`CNaviData::info` with its cell; -1 for a map or a monster).
    pub goal: Option<(usize, (i32, i32))>,
    /// The ways into the map sought from which the place is reached, with what walking on costs (`endLinks`).
    end: Vec<(usize, i32)>,
    pub mode: Mode,
    pub display: Display,
    /// The window is there (`g_windowMgr.navigationWnd`), is not shown (`hideWindow`), is the small one
    /// (`expandedMap`).
    pub window: bool,
    pub hidden: bool,
    pub small: bool,
    /// What the list is narrowed to (`searchType`: 0 all, 1 maps, 2 NPCs, 3 monsters) and the text sought.
    pub kind: i32,
    pub sought: String,
    pub allowed: Allowed,
    /// Which of the eight sets of arrows marks the way (`viewIcon`, 1 to 8).
    pub icon: u8,
    /// What there is to say, until the app has said it.
    pub said: Vec<Line>,
    pub words: Words,
}

impl Guide {
    /// The navigation as the client starts with it: the first set of arrows.
    pub fn new() -> Self {
        Self { icon: 1, ..Self::default() }
    }

    /// Forgets the target and the way (`Reset`); with `all`, what was found too. The switches of which ways
    /// out may be taken, the set of arrows and the window stay.
    pub fn reset(&mut self, all: bool) {
        if all {
            self.groups.clear();
            (self.group, self.member) = (None, None);
            self.found.clear();
        }
        (self.saved, self.current) = (None, None);
        (self.cost, self.node) = (0, None);
        self.nodes.clear();
        (self.arriving, self.shown) = (false, false);
        self.way.clear();
        self.mode = Mode::Idle;
        self.kind = 0;
        self.sought.clear();
        self.allowed.side = 0;
    }

    /// Leaving the game for the characters (`ZC_RESTART_ACK`): everything is forgotten.
    pub fn left(&mut self) {
        self.reset(true);
        (self.at_goal, self.goal) = (false, None);
        self.end.clear();
        self.display = Display::Lists;
    }

    /// Searches for the text sought (`UpdateSearchResults`): what is found is ordered and grouped, the group
    /// named as the text first. False for a text of no words.
    pub fn search(&mut self, atlas: &Atlas) -> bool {
        self.display = Display::Lists;
        let Some(found) = atlas.search(&self.sought, 0, &self.words) else { return false };
        self.found = found;
        search::ordered(&mut self.found);
        self.regroup(atlas);
        search::first_named(&self.found, &mut self.groups, &self.sought);
        true
    }

    /// Groups what was found anew (`BuildNodePaths`), as when the list is narrowed.
    pub fn regroup(&mut self, atlas: &Atlas) {
        self.groups = atlas.groups(&self.found, self.kind);
    }

    /// A search asked for from outside the window (`SearchNavigation`: a link, the World Map): the text is
    /// searched for and the window brought up, unless it is to stay away.
    pub fn look_for(&mut self, atlas: &Atlas, text: &str, kind: i32, hide: bool) -> bool {
        if !(0..=4).contains(&kind) {
            return false;
        }
        (self.sought, self.hidden, self.kind) = (text.to_owned(), hide, kind);
        self.search(atlas);
        if !self.hidden {
            if self.window {
                (self.small, self.hidden) = (false, false);
            }
            self.window = true;
        }
        true
    }

    /// A place of a link in the chat is looked at (`SetDestinationMap`): the map alone when the link has no
    /// cell (1000), and the window is brought up on its lists.
    pub fn point_at(&mut self, atlas: &Atlas, map: &str, x: i32, y: i32) -> bool {
        let Some(map) = atlas.find_exactly(map) else { return false };
        let name = &atlas.maps[map].name;
        self.current = Some(if x == 1000 {
            Target { what: What::Map(map), x: -1, y: -1, name: name.clone() }
        } else {
            Target { what: What::Place(map), x, y, name: format!("{name} {x},{y}") }
        });
        (self.display, self.small, self.hidden) = (Display::Lists, false, false);
        if self.mode != Mode::Going {
            self.mode = Mode::Sought;
        }
        self.window = true;
        true
    }

    /// The target of the group and member picked becomes the one looked at (`SelectNodeState`); with none
    /// picked nothing is looked at, and a map the search does not list is told of.
    pub fn look_at_picked(&mut self, atlas: &Atlas) -> bool {
        let picked = self.group.and_then(|group| self.groups.get(group)).zip(self.member).and_then(|(group, member)| group.get(member));
        if let Some(target) = picked.and_then(|index| self.found.get(*index)) {
            self.current = Some(target.clone());
            return true;
        }
        self.current = None;
        if self.goal.is_some_and(|(map, _)| !atlas.maps[map].searchable()) {
            self.said.push(Line::Unlisted);
        }
        false
    }

    /// A group of the list is picked (`ID_SELECT_ITEM`), and its first member looked at.
    pub fn pick_group(&mut self, atlas: &Atlas, group: usize) {
        if self.groups.is_empty() {
            return;
        }
        (self.group, self.member) = (Some(group), Some(0));
        self.look_at_picked(atlas);
    }

    /// A member of the group picked is looked at (`SelectRouteNode`); one that is not there, and the first is.
    pub fn pick_member(&mut self, atlas: &Atlas, member: usize) {
        self.member = Some(member);
        if !self.look_at_picked(atlas) {
            self.member = Some(0);
            self.look_at_picked(atlas);
        }
    }

    /// What is gone to is looked at again, or the way out of the route with that number (`SelectTarget`).
    pub fn look_at_node(&mut self, node: Option<usize>) {
        self.current = self.saved.clone();
        if let (Some(current), Some(link)) = (&mut self.current, node.and_then(|node| self.nodes.get(node))) {
            current.what = What::Node(*link);
        }
    }

    /// The target looked at becomes the one to go to (`SaveCurrentTarget`): 0, or why not.
    pub fn save_current(&mut self) -> i32 {
        match &self.current {
            Some(Target { what: What::Node(_), .. }) => -92,
            None => -97,
            Some(target) => {
                self.saved = Some(target.clone());
                0
            }
        }
    }

    /// The way is over (`CancelRoute`): target and way are forgotten, the window goes back to its lists, and
    /// the chat is told.
    pub fn cancel(&mut self) {
        self.reset(false);
        self.display = Display::Lists;
        if !self.hidden {
            self.mode = Mode::Sought;
        }
        self.said.push(Line::Arrived);
    }

    /// The navigation under way is given up from its window (`ID_CLOSE` answered with yes): the target gone
    /// to is forgotten (`ClearSavedTarget`), what was found stays.
    pub fn give_up(&mut self) {
        self.display = Display::Lists;
        self.saved = None;
        (self.arriving, self.shown, self.cost, self.node) = (false, false, 0, None);
        self.nodes.clear();
        self.way.clear();
        self.mode = Mode::Sought;
    }

    /// The window is closed with nothing under way (`ID_CLOSE`): everything is forgotten.
    pub fn closed(&mut self) {
        (self.display, self.small, self.window) = (Display::Lists, false, false);
        self.reset(true);
    }

    /// The arrow at the left of the window's top (`ID_CHARGE`): between the lists and the map of the way.
    pub fn turned(&mut self, atlas: &Atlas) {
        if self.saved.is_none() {
            if self.display == Display::Pick {
                if self.mode == Mode::Going {
                    self.display = Display::Way;
                    self.look_at_node(self.node);
                } else {
                    self.display = Display::Lists;
                    self.look_at_picked(atlas);
                }
            }
        } else if self.display == Display::Way || (self.display == Display::Pick && self.mode != Mode::Going) {
            self.display = Display::Lists;
            self.look_at_picked(atlas);
        } else {
            self.display = Display::Way;
            self.look_at_node(self.node);
        }
    }

    /// The box "service" (`ID_CHECK`): ways out that cost zeny and the airship are taken, or neither.
    pub fn serviced(&mut self, on: bool) {
        self.allowed = Allowed { zeny: on, airship: on, scrolls: false, side: self.allowed.side };
    }

    /// The name of the map the target looked at is on (`GetCurrentTargetName`); empty with none.
    pub fn looked_map<'atlas>(&self, atlas: &'atlas Atlas) -> &'atlas str {
        self.current.as_ref().map_or("", |target| atlas.maps[atlas.map_of(target.what)].name.as_str())
    }

    /// The maps the route leads through, the one sought last (`RefreshWorldMapMarks`); none unless on the way.
    pub fn route_maps<'atlas>(&self, atlas: &'atlas Atlas) -> Vec<&'atlas str> {
        if self.mode != Mode::Going {
            return Vec::new();
        }
        let through = self.nodes.iter().map(|link| atlas.links[*link].map);
        through.chain(self.goal.map(|(map, _)| map)).map(|map| atlas.maps[map].name.as_str()).collect()
    }

    /// The maps of what was found, each with what the World Map marks it by (`StateMapName`): how many of a
    /// monster the map has, 10000 for an NPC, nought for the map itself.
    pub fn found_maps<'atlas>(&self, atlas: &'atlas Atlas) -> Vec<(&'atlas str, i32)> {
        self.found
            .iter()
            .map(|target| {
                let level = match target.what {
                    What::Mob(mob) => atlas.mobs[mob].many(),
                    What::Npc(_) => 10000,
                    _ => 0,
                };
                (atlas.maps[atlas.map_of(target.what)].name.as_str(), level)
            })
            .collect()
    }
}

impl Atlas {
    /// The map of exactly that name (`FindObject` alone, as a link of the chat is looked up).
    pub fn find_exactly(&self, name: &str) -> Option<usize> {
        self.maps.iter().position(|map| map.name == name)
    }
}
