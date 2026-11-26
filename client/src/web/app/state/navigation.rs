//! What the app keeps of the navigation (`crate::navigation`).

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::map::Gat;
use crate::navigation::Guide;

/// The navigation: what it is doing (`Guide`, the client's `CNavigationManager`), and the windows about it.
pub(in crate::web) struct Navigation {
    pub guide: Guide,
    /// The windows beside the navigation's own: its help (`UINavigationHelpWnd`), the sets of arrows
    /// (`UINavigationroadiconWnd`), the list of the route (`UINavigationRuideWnd`).
    pub help: bool,
    pub icons: bool,
    pub route: bool,
    /// "Do you want to cancel navigation?" is being asked.
    pub asks: bool,
    /// The list of the box that narrows what is listed is open.
    pub kinds: bool,
    /// The box "Minimize" is ticked: the window becomes the small one as the character is sent walking.
    pub fold: bool,
    /// Until when the map to press a place on says what it is for (the page's clock).
    pub notice: f64,
    /// The grounds of other maps read so far, for the way to a place on one; none for a map that could not
    /// be read.
    pub grounds: HashMap<String, Option<Rc<Gat>>>,
    /// What waits for the navigation's tables, or for the ground of a map, to be read.
    pub waits: Option<Wanted>,
    /// The pictures of the NPCs and monsters looked at, by job, as addresses the page shows; and those asked for.
    pub pictures: HashMap<i32, (String, (usize, usize), (f32, f32))>,
    pub pictures_asked: HashSet<i32>,
    /// What was typed into the World Map's line of search, kept while the map is drawn anew.
    pub world_sought: String,
    /// Where the navigation's window stood as a window beside it was last opened: those open by it.
    pub at: Option<(i32, i32)>,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            guide: Guide::new(),
            help: false,
            icons: false,
            route: false,
            asks: false,
            kinds: false,
            fold: false,
            notice: 0.0,
            grounds: HashMap::new(),
            waits: None,
            pictures: HashMap::new(),
            pictures_asked: HashSet::new(),
            world_sought: String::new(),
            at: None,
        }
    }
}

/// Something asked of the navigation (`App::navigate`).
#[derive(Clone)]
pub(in crate::web) enum Wanted {
    /// A navigation started from outside the window (`Guide::start`).
    Start { map: String, kind: i32, mode: u8, hide: bool, x: i32, y: i32, id: i32 },
    /// A search (`Guide::look_for`).
    Search { text: String, kind: i32, hide: bool },
    /// A place of a link of the chat (`Guide::point_at`).
    Point { map: String, x: i32, y: i32 },
    /// The window's button that finds the way to what is looked at (`Guide::find`).
    Find,
    /// The window opened with nothing asked.
    Open,
    /// A map was entered (`Guide::resume`).
    Resume,
}

/// What comes back to the navigation some time after it was asked for.
pub(in crate::web) enum News {
    /// The tables were read.
    Tables,
    /// The ground of a map was read, or could not be.
    Ground(String, Option<Rc<Gat>>),
    /// The picture of an NPC or a monster was made: its job, and the picture.
    Picture(i32, (String, (usize, usize), (f32, f32))),
    /// The time the map to press a place on says what it is for is over.
    Noticed,
}
