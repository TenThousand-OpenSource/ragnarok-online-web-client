//! What the world keeps between frames: the map and who is on it, what the player is doing, the effects under
//! way, what the page is shown.

use super::*;

/// What decides whom the pointer picks and whether a skill aimed goes on the one pressed, besides who is
/// under the pointer (`crate::skills::aim`).
#[derive(Default)]
pub(super) struct Picking {
    /// The skill being aimed is one of help: it goes on a monster only with Shift held, and while Shift is up
    /// the pointer picks a player before a monster.
    pub(super) support: bool,
    /// The skill being aimed is one for a trap (Remove Trap, Spring Trap): it is pointed at a trap standing
    /// on the ground, and at nobody.
    pub(super) traps: bool,
    /// Whether Shift is held, as the last event of the mouse or of the keys told (the client asks the
    /// keyboard every frame: `g_pickModifierKey`).
    pub(super) shift: bool,
}

#[derive(Default)]
pub(super) struct State {
    pub(super) gpu: Option<Gpu>,
    pub(super) scene: Option<Scene>,
    pub(super) hooks: Option<Hooks>,
    /// The server's clock, by which everyone's walk is shown (`crate::server_time`); whether the server has told
    /// its time on this map yet (the first answer sets the clock, the later ones correct it); and the clock's
    /// time at the frame before, none before a map's first frame.
    pub(super) clock: Clock,
    pub(super) clock_told: bool,
    pub(super) stepped: Option<f64>,
    /// When a walk was last asked of the server, and to which cell (the client's `standTick` and the cell its
    /// player is bound for).
    pub(super) walk_asked: f64,
    pub(super) asked: (i32, i32),
    /// A cell pressed while the character's body was in a blow it deals, and when that blow is over: it is
    /// walked to then (the client's `isPreengageStateOfMove`; `input/press.rs`, `pursuit.rs`).
    pub(super) walk_waits: Option<((i32, i32), f64)>,
    /// Counts the maps entered: a map that finishes loading after the next was asked for is dropped.
    pub(super) entered: u32,
    /// The right button is held: where the pointer was last.
    pub(super) turning: Option<(f32, f32)>,
    /// When the right button was last pressed: twice in a moment puts the camera back.
    pub(super) right_pressed: f64,
    /// Fighting other players: the map's kind and the player's friends (`pvp.rs`).
    pub(super) pvp: pvp::Pvp,
    /// The player's homunculus and mercenary: which units they are, and their life (`ai.rs`).
    pub(super) ai: ai::Owned,
    /// The guilds of those in sight: their names and emblems (`guild.rs`).
    pub(super) guilds: guild::Guilds,
    /// The Granny models read and what each one in sight is doing (`granny.rs`).
    pub(super) granny: granny::Models,
    /// The player's options the world goes by.
    pub(super) options: Options,
    /// What of the surroundings is kept from map to map: the switch of the fog, the night (`scenery.rs`).
    pub(super) scenery: scenery::Kept,
    /// What the pointer's pick goes by besides who is under it: the kind of the skill aimed, and Shift.
    pub(super) picking: Picking,
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
    /// Where on the page the pointer is, over the world or over a window.
    pub(super) mouse: Option<(f32, f32)>,
    /// Who the pointer is on: their ID, and whether it is a monster.
    pub(super) over: Option<(u32, bool)>,
    /// The left button is held on the ground: where the pointer is, and when a walk was last asked for.
    pub(super) held: Option<(f32, f32)>,
    /// When the left button went down.
    pub(super) held_since: f64,
    pub(super) held_asked: f64,
    /// A place pressed that is farther than the server walks in one go, and the cell last asked for on the way
    /// to it: the character goes on from there when it gets there.
    pub(super) journey: Option<((i32, i32), (i32, i32))>,
    /// The monster being fought: its ID, when it was last looked after, and whether the server was told to strike
    /// since it came near.
    pub(super) chase: Option<(u32, f64, bool)>,
    /// When the player's own character last struck, and when the server was last asked to strike: a fight that
    /// does not start is asked for again.
    pub(super) own_struck: f64,
    /// Until when the player's own character is in the motion of a blow it deals: it does not walk meanwhile.
    pub(super) own_striking_until: f64,
    pub(super) strike_asked: f64,
    /// How near the character must stand to strike with what it holds, in cells, once the server has said so
    /// (it says when a blow was asked for from too far).
    pub(super) strike_range: Option<i32>,
    /// The thing on the ground the character is walking to, to pick it up, and when it was last looked after.
    pub(super) fetch: Option<(u32, f64)>,
    /// The copies of someone that effects left standing (`ailments.rs`).
    pub(super) copies: Vec<ailments::Ghost>,
    /// The effects the client draws by code that are under way.
    pub(super) bursts: Vec<prims::Burst>,
    /// The shake of the screen under way (`crate::quake`).
    pub(super) quake: Option<crate::quake::Quake>,
    /// The bits of the state the player's own character is in (kept across maps): what it rides.
    pub(super) own_state: u32,
    /// What the player's own character's body and health states make of it (kept across maps), and its two
    /// veils over the screen (`ailments.rs`).
    pub(super) own_ail: ailments::Ail,
    pub(super) veils: ailments::Veils,
    /// The units of skills standing on the ground, and the places skills were used on (`skill_units.rs`).
    pub(super) grounds: skill_units::Grounds,
    /// The weapon the player's own character holds (kept across maps).
    pub(super) weapon: u32,
    /// The shield it has on its arm (kept across maps).
    pub(super) shield: u32,
    /// The player's own life and spirit, left and most: the bars under the character.
    pub(super) own: [i64; 4],
    /// Whether life bars are on the page.
    pub(super) bars: bool,
    /// The hits on their way, the one queue of them: when each lands, on whom, and what it is. As one lands
    /// it sounds and shows its numbers (`combat/landing.rs`), and the one struck stops and flinches
    /// (`frame/sprites.rs`, told by the former).
    pub(super) blows: Vec<(f64, u32, combat::Landing)>,
    /// Where the turning marks in the world stand this frame: at the feet of the monster being fought, and of
    /// another monster the pointer is on.
    pub(super) target_at: [Option<Vector>; 2],
    /// The numbers in the air.
    pub(super) flying: Vec<Flying>,
    /// The circles on the ground of skills being cast on a place.
    pub(super) scopes: Vec<super::scope::Scope>,
    /// The way the navigation marks on the ground (`navigation.rs`).
    pub(super) way: super::navigation::Marks,
    /// The effects and sounds that are to start at a moment yet to come.
    pub(super) effects_due: Vec<(f64, String, Option<u32>, Option<Carrying>)>,
    pub(super) sounds_due: Vec<(f64, String)>,
    /// The sounds that are to start on someone at a moment yet to come, heard from where that one is then
    /// (`earshot.rs`).
    pub(super) heard: Vec<earshot::Due>,
    /// What the server told of those in sight beside their names, and the player's own level (`names.rs`).
    pub(super) cards: HashMap<u32, crate::names::Card>,
    pub(super) own_level: u16,
    /// The words of units on the ground (`written.rs`).
    pub(super) written: written::Words,
    /// Arrows in flight, and what monsters shoot: when each leaves, from where, at whom, and its sprite.
    pub(super) arrows: Vec<(f64, Vector, u32, &'static str)>,
    /// What comes up under those a Mandragora or its like strikes (`crate::blow::shots::root`): since when,
    /// where, the striker's job and the sprite.
    pub(super) roots: Vec<(f64, Vector, u16, &'static str)>,
    /// The effects playing, and the effect files read so far, by name.
    pub(super) effects: Vec<Playing>,
    pub(super) effect_files: HashMap<String, Rc<Shown>>,
    /// Which of its three places the next hit the client puts at one of three is put at (its `CRandomHitEffect`
    /// counts them round in one number for every such effect: `crate::effect::played`).
    pub(super) hit_turn: usize,
    /// The counts towards launching anew the effects that a bit of someone's state keeps going (Sight, Ruwach).
    pub(super) kept: crate::state_effects::Kept,
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
    /// The marks over NPCs that have something for the character (`quests.rs`): the visit to a map they are
    /// of (`entered`), and by NPC which action of the marks' sprite each is.
    pub(super) quest_marks: (u32, HashMap<u32, usize>),
    /// The carts merchants pull (`cart.rs`).
    pub(super) carts: cart::Carts,
    /// The falcons hunters keep (`falcon.rs`).
    pub(super) falcons: falcon::Falcons,
    /// The signs standing over heads: of chat rooms, of shops (`signs.rs`).
    pub(super) signs: signs::Signs,
    /// What an NPC's script shows in the world: the bars that fill over someone (`scripted.rs`).
    pub(super) scripted: scripted::Scripted,
}

impl State {
    /// Whether the pointer picks a player before a monster now (`crate::skills::aim::players_first`).
    pub(super) fn players_first(&self) -> bool {
        crate::skills::aim::players_first(self.aiming == Some(false) && self.picking.support, self.picking.shift)
    }

    /// Whether the skill being aimed at someone would go on that one, pressed now: on a monster or on a player
    /// by the client's rule of Shift (`crate::skills::aim`), on nobody else.
    pub(super) fn aim_goes_on(&self, id: u32, unit: &Mover) -> bool {
        let (support, shift, no_shift) = (self.picking.support, self.picking.shift, self.options.no_shift);
        if unit.companion {
            false
        } else if unit.lively {
            crate::skills::aim::goes_on_monster(support, shift, no_shift)
        } else {
            matches!(unit.look, Look::Character(_)) && crate::skills::aim::goes_on_player(support, shift, no_shift, self.pvp.foe(id, unit))
        }
    }

    /// Forgets what was under way in the fights and effects of the place left: the numbers flying, the arrows,
    /// the effects and sounds due and playing, the blows on their way, the circles of skills aimed, the copies
    /// left standing (`session.rs`: leaving the world, and a jump to another place).
    pub(super) fn forget_fights(&mut self) {
        self.flying.clear();
        self.arrows.clear();
        self.roots.clear();
        self.effects_due.clear();
        self.sounds_due.clear();
        self.heard.clear();
        self.effects.clear();
        self.blows.clear();
        self.scopes.clear();
        self.bursts.clear();
        self.copies.clear();
    }
}
