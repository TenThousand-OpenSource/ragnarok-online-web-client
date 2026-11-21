//! Everything that happens, as a value: an event of the page, of the world, of a socket or of a timer, given
//! to `App::handle` (`actions.rs`), which says who sees to it.

use super::*;

pub(in crate::web) enum Action {
    /// A form was sent.
    Submit(Form),
    /// Something with `data-click` was pressed: that, and its `data-arg`.
    Click(String, String),
    /// A character slot was double-clicked.
    Open(u8),
    /// Something of the game was double-clicked.
    Use(Use),
    /// What was dragged was dropped on a slot of the hotkey bar; nothing dragged empties the slot.
    Dropped(usize, Option<Drag>),
    /// A window of the game was pressed, anywhere on it: its element's ID.
    WindowPressed(String),
    /// The gateway told what some items are called.
    ItemsKnown,
    Resized,
    /// A moment after the music was asked for: see whether the browser let it play.
    CheckMusic,
    /// The moment to try a login again, if nothing else has happened since (the generation is the same).
    RetryLogin(u32),
    /// From the socket of that generation; older sockets' events are dropped.
    Socket(u32, SocketEvent),
    KeepAlive,
    /// The moment the watch on the connection looks at it, and what is shown for a time is looked over.
    Watch,
    /// The player pressed a cell of the map to walk to.
    WalkTo(u16, u16),
    /// The character looked towards a place: how its head is turned, and the direction it faces.
    Face(u8, u8),
    /// Something was dragged out of its window and let go over the world.
    Thrown(Drag),
    /// Something was dragged out of its window and let go over the window of a deal between two players.
    Offered(Drag),
    /// Something was dragged out of its window and let go over the window of a thing being forged.
    Forged(Drag),
    /// Something was dragged out of its window and let go over a window that takes things (`data-drop`).
    LetGo(Drag, Drop),
    /// A thing was pressed with the right button while Alt was held, with the storage open: all of it goes
    /// into the storage, or comes out of it.
    Stowed(Use),
    /// A thing of the storage (its place there) was dragged out and let go over the inventory.
    Unstored(u16),
    /// Something was dragged out of its window and let go over the letter being written.
    Mailed(Drag),
    /// A thing of the cart (its place there) was dragged out and let go over the inventory.
    Uncarted(u16),
    /// The first picture of the sprite of a look offered for the cart was read: the kind, the picture, and
    /// where in it the cart stands.
    CartLookRead(u8, String, (f32, f32)),
    /// A thing of the letter being written (where it is carried) was dragged out and let go over the inventory.
    Unmailed(u16),
    /// A skill was pressed with the right button: show its description.
    SkillInfo(u16),
    /// An item was pressed with the right button: show what is known of it.
    Info(u32, bool),
    /// The character stands by a thing on the ground it was sent to: pick it up.
    Take(u32),
    /// The player pressed someone on the map: their ID, and whether it is a monster.
    Pressed(u32, bool),
    /// The character is on another cell now.
    Moved(u16, u16),
    /// The map is drawn, or could not be.
    WorldReady(Result<(), String>),
    /// The skill being aimed was pointed at someone (their ID) or at a cell.
    Aimed(Option<u32>, (u16, u16)),
    /// The wheel was turned while a skill is aimed: a level up (1) or down (-1).
    AimLevel(i32),
    /// One blow at that monster.
    StrikeOnce(u32),
    /// The fight is called off.
    StopFight,
    /// A slot of the hotkey bar is used by the mouse.
    Quick(usize),
    /// A slider (`data-slide`) was moved: its name and where it stands, 0 to 100.
    Slid(String, u8),
    /// The right button was pressed on someone on the map: their ID, and where on the page.
    Menu(u32, (f32, f32)),
    /// The right button was pressed on a thing in a window that has a menu: the thing, and where on the page.
    MenuOn(things::Menu, (f32, f32)),
    /// A monster's picture for the Adventure Guide was made: the monster's number, and the picture.
    GuideRead(u32, (String, (usize, usize), (f32, f32))),
    /// The slot machine's pictures were read: those of its waiting, and those of its turning.
    TamingRead(Vec<(String, (usize, usize), (f32, f32))>, Vec<(String, (usize, usize), (f32, f32))>),
    /// The digits of the standing on a PVP map were read (the pictures of `RankFont`, action by action).
    PvpRead(Vec<(String, (usize, usize), (f32, f32))>),
    /// The pictures of the emotions for their window were read: each emotion's picture and its size.
    EmotionsRead(Vec<(u16, (String, (usize, usize)))>),
    /// A picture of the screen was made (or could not be): the name of its file.
    Pictured(Result<String, String>),
    /// A moment of the slot machine has passed.
    TamingTick,
    /// A while after the pet's trick: it may show how it feels standing about.
    PetIdle,
    /// A second has passed while a state of the character has a time running: the times are written anew.
    StatusTick,
    /// A second has passed while a ranking waits for the names of its characters.
    RankingLooked,
    /// A time something of an NPC's script runs by has come: a bar is full, a sound repeats, a mark goes.
    Scripted,
    /// Something of the books has come back: a book's text, a button's picture, the next line read aloud.
    Book(BookNews),
    /// A moment has passed while the player has a homunculus or a mercenary: its script may be due.
    AiTick,
    /// The mouse commanded the homunculus, or the mercenary (true): to fight someone or to walk to a cell, at
    /// once or (true) after its other commands.
    Commanded(bool, Option<u32>, (u16, u16), bool),
    /// The screen has gone dark for a move: the character is put at its new place (a map and a cell).
    Arrive(String, u16, u16),
    /// Something the guild waited for was read: a member's head, a file picked as the emblem.
    Guild(GuildEvent),
    /// The account service answered: the HTTP status and, when it says to wait, for how many seconds.
    Answer(Request, Result<(u16, Option<u64>), String>),
}

/// What was asked of the account service.
#[derive(Clone, Copy)]
pub(in crate::web) enum Request {
    Register,
    Recover,
    Reset,
}
