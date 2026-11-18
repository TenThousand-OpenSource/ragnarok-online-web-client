//! What the world keeps between frames: the map and who is on it, what the player is doing, the effects under
//! way, what the page is shown.

use super::*;

#[derive(Default)]
pub(super) struct State {
    pub(super) gpu: Option<Gpu>,
    pub(super) scene: Option<Scene>,
    pub(super) hooks: Option<Hooks>,
    /// Counts the maps entered: a map that finishes loading after the next was asked for is dropped.
    pub(super) entered: u32,
    /// The right button is held: where the pointer was last.
    pub(super) turning: Option<(f32, f32)>,
    /// When the right button was last pressed: twice in a moment puts the camera back.
    pub(super) right_pressed: f64,
    /// The player's options the world goes by.
    pub(super) options: Options,
    /// The skill being aimed is one of help, which goes on a monster only with Shift held (unless the client's
    /// No Shift option is on).
    pub(super) aim_guarded: bool,
    /// How the monster being fought is fought: 0 until it is down, 1 while the left button is held, 2 with one
    /// blow as soon as it is near enough (the button was let go before any blow was asked for).
    pub(super) fight: u8,
    /// What is written by the pointer while a skill is aimed: the skill and the level it will be used at.
    pub(super) aim_words: String,
    /// The camera stays on the character without following it gradually (the client's `/camera` option).
    pub(super) camera_fixed: bool,
    /// Where the right button went down, until it comes up.
    pub(super) right_at: Option<(f32, f32)>,
    pub(super) listening: bool,
    /// Those the server showed while the map was still being read: put on it once it is there.
    pub(super) early: Vec<Unit>,
    /// The map being read, until it shows; and where the character is to stand on it when it does, if the
    /// server named another place meanwhile.
    pub(super) reading: Option<String>,
    pub(super) start: Option<(i32, i32)>,
    /// Where on the page the pointer is, while it is over the world with no button held.
    pub(super) pointer: Option<(f32, f32)>,
    /// Who the pointer is on: their ID, and whether it is a monster.
    pub(super) over: Option<(u32, bool)>,
    /// The left button is held on the ground: where the pointer is, and when a walk was last asked for.
    pub(super) held: Option<(f32, f32)>,
    /// When the left button went down.
    pub(super) held_since: f64,
    pub(super) held_asked: f64,
    /// The monster being fought: its ID, when it was last looked after, and whether the server was told to strike
    /// since it came near.
    pub(super) chase: Option<(u32, f64, bool)>,
    /// When the player's own character last struck, and when the server was last asked to strike: a fight that
    /// does not start is asked for again.
    pub(super) own_struck: f64,
    /// Until when the player's own character is in the motion of a blow it deals: it does not walk meanwhile.
    pub(super) own_striking_until: f64,
    /// Until when the player's own character cannot walk for a blow it took (the server's walk delay).
    pub(super) own_stopped_until: f64,
    pub(super) strike_asked: f64,
    /// How near the character must stand to strike with what it holds, in cells, once the server has said so
    /// (it says when a blow was asked for from too far).
    pub(super) strike_range: Option<i32>,
    /// The thing on the ground the character is walking to, to pick it up, and when it was last looked after.
    pub(super) fetch: Option<(u32, f64)>,
    /// The glitter of First Aid playing: on whom (none: the player's own character), since when, and the two
    /// angles it starts at (how it is turned, and where its slow pulse is), which the client draws by lot.
    pub(super) aids: Vec<(Option<u32>, f64, f32, f32)>,
    /// The effects the client draws by code that are under way.
    pub(super) bursts: Vec<prims::Burst>,
    /// The weapon the player's own character holds (kept across maps).
    pub(super) weapon: u32,
    /// The player's own life and spirit, left and most: the bars under the character.
    pub(super) own: [i64; 4],
    /// Whether life bars are on the page.
    pub(super) bars: bool,
    /// Blows on their way: when each lands, on whom, the player's own ID, how much, and what it sounds like.
    pub(super) landing: Vec<(f64, u32, u32, i32, Option<String>)>,
    /// Where the pointer in the world stands this frame (see the frame's code).
    pub(super) target_at: Option<Vector>,
    /// The numbers in the air.
    pub(super) flying: Vec<Flying>,
    /// Bolts falling, and the effects and sounds that are to start at a moment yet to come.
    pub(super) bolts: Vec<Bolt>,
    pub(super) effects_due: Vec<(f64, String, Option<u32>)>,
    pub(super) sounds_due: Vec<(f64, String)>,
    /// Arrows in flight: when each leaves, from where, and at whom.
    pub(super) arrows: Vec<(f64, Vector, u32)>,
    /// The effects playing, and the effect files read so far, by name.
    pub(super) effects: Vec<Playing>,
    pub(super) effect_files: HashMap<String, Rc<Shown>>,
    /// Hits to show as numbers: on whom, and how much.
    pub(super) hits: Vec<(u32, i32)>,
    /// A skill is being aimed: at a place (`true`) or at someone.
    pub(super) aiming: Option<bool>,
    /// How far the skill being aimed reaches, in cells.
    pub(super) aim_range: f32,
    /// The skill was aimed at someone or somewhere out of its reach: the character walks there, and the skill is
    /// used once it is near enough. On whom (or else at which cell), the skill's reach, and when a walk was last
    /// asked for.
    pub(super) reach: Option<(Option<u32>, (i32, i32), f32, f64)>,
    /// Which pointer and whose name the page shows now.
    pub(super) pointing: usize,
    /// Since when it is that one: its pictures run from then.
    pub(super) pointing_since: f64,
    pub(super) named: Option<u32>,
    /// The game's pointers as CSS, once read (see `POINTERS`): the pictures of each, in turn, and how long each
    /// picture shows.
    pub(super) pointers: Vec<(Vec<String>, f64)>,
    /// Which picture of the pointer the page shows now.
    pub(super) pointer_frame: usize,
    pub(super) frame: Option<Closure<dyn FnMut()>>,
}
