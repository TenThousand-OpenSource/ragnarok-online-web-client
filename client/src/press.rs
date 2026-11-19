//! What can be pressed on the page: every button, tab and row that does something, as a value.
//!
//! The page is text, and a press comes back from it as text: the name the thing was drawn with (`data-click`)
//! and what went with it (`data-arg`). This is the one place where that text is written and read. A window
//! draws a thing with a `Press` (`web::parts::pressed`), the page gives its name back, `Press::parse` makes the
//! value of it again, and the app matches on the value: a press nobody handles, or a name nobody draws, does
//! not compile.
//!
//! Each system has its presses in a file of its own here, named as its window in `web/hud/` and its handler in
//! `web/app/clicks/`. `presses!` writes the enum and both ways of the text from one list, so the two cannot
//! disagree.

/// A kind of press that has a name on the page.
pub trait Named: Sized {
    /// Every name of this kind.
    const NAMES: &'static [&'static str];
    /// Its name on the page, and what goes with it (empty for nothing).
    fn name(&self) -> (&'static str, String);
    /// The press of that name with that beside it; none if the name is not of this kind or what is beside it
    /// is not what the press takes.
    fn parse(what: &str, arg: &str) -> Option<Self>;
}

/// Writes an enum of presses from a list of `Variant = "name"` and `Variant(Type) = "name"`: the type of what
/// goes with a press is anything that is written and read as text (`Display`, `FromStr`).
macro_rules! presses {
    ($(#[$meta:meta])* $name:ident { $($(#[$about:meta])* $variant:ident $(($with:ty))? = $text:literal,)* }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $($(#[$about])* $variant $(($with))?,)*
        }

        impl $crate::press::Named for $name {
            const NAMES: &'static [&'static str] = &[$($text),*];

            fn name(&self) -> (&'static str, String) {
                match *self {
                    $(presses!(@seen $variant value $($with)?) => ($text, presses!(@written value $($with)?)),)*
                }
            }

            fn parse(what: &str, _arg: &str) -> Option<Self> {
                match what {
                    $($text => presses!(@read $variant _arg $($with)?),)*
                    _ => None,
                }
            }
        }
    };
    (@seen $variant:ident $value:ident) => { Self::$variant };
    (@seen $variant:ident $value:ident $with:ty) => { Self::$variant(ref $value) };
    (@written $value:ident) => { String::new() };
    (@written $value:ident $with:ty) => { $value.to_string() };
    (@read $variant:ident $arg:ident) => { Some(Self::$variant) };
    (@read $variant:ident $arg:ident $with:ty) => { $arg.parse::<$with>().ok().map(Self::$variant) };
}

/// One step of something that goes up and down: a plus or a minus button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    More,
    Less,
}

impl std::fmt::Display for Step {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        to.write_str(if *self == Step::More { "+" } else { "-" })
    }
}

impl std::str::FromStr for Step {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        match text {
            "+" => Ok(Step::More),
            "-" => Ok(Step::Less),
            _ => Err(()),
        }
    }
}

/// Writes `Press`, the press of any system, from the list of systems: each a variant holding that system's own
/// presses.
macro_rules! systems {
    ($($(#[$about:meta])* $variant:ident = $module:ident,)*) => {
        $(pub mod $module;)*

        /// Anything on the page that can be pressed, by the system it belongs to.
        #[derive(Debug, Clone, PartialEq)]
        pub enum Press {
            $($(#[$about])* $variant($module::Press),)*
        }

        impl Press {
            /// The names of every press, system by system.
            pub const NAMES: &'static [&'static [&'static str]] = &[$(<$module::Press as Named>::NAMES),*];

            /// Its name on the page, and what goes with it.
            pub fn name(&self) -> (&'static str, String) {
                match self {
                    $(Self::$variant(press) => press.name(),)*
                }
            }

            /// The press the page names so.
            pub fn parse(what: &str, arg: &str) -> Option<Self> {
                None$(.or_else(|| $module::Press::parse(what, arg).map(Self::$variant)))*
            }
        }

        $(impl From<$module::Press> for Press {
            fn from(press: $module::Press) -> Self {
                Self::$variant(press)
            }
        })*
    };
}

pub mod things;

systems! {
    /// What belongs to no one window: the language, the sound, the minimap, the game's menu.
    General = general,
    /// Logging in, the servers, the characters and making one.
    Account = account,
    /// What an NPC says and asks.
    Talk = talk,
    /// The Status window.
    Status = status,
    /// The inventory, the equipment and what is known of an item.
    Inventory = inventory,
    /// The skills and the hotkey bar.
    Skills = skills,
    /// The chat.
    Chat = chat,
    /// An NPC's shop.
    Shops = shops,
    /// The storage, the Kafra's or a guild's.
    Storage = storage,
    /// The pet.
    Pets = pets,
    /// The quest journal.
    Quests = quests,
    /// The Shortcut Settings window.
    Hotkeys = hotkeys,
    /// The Game Settings window.
    Settings = settings,
    /// The Adventure Guide.
    Guide = guide,
    /// The party.
    Party = party,
    /// The guild.
    Guild = guild,
    /// The menu on another player.
    PlayerMenu = player_menu,
    /// A deal between two players.
    Exchange = exchange,
    /// The mail.
    Mail = mail,
    /// The friends list.
    Friends = friends,
    /// A talk with one player in a window of its own.
    Whispers = whispers,
    /// Looking at another player's equipment.
    ViewEquip = viewequip,
    /// The lists a skill or a card opens to pick one thing of.
    Choosing = choosing,
    /// The alchemist's homunculus.
    Homunculus = homunculus,
    /// A hired mercenary.
    Mercenary = mercenary,
    /// The list of packets this client does not read (`/packets`).
    Unread = unread,
    /// What an NPC's script shows: the picture over the screen.
    Scripted = scripted,
    /// Chat rooms.
    ChatRooms = chat_rooms,
    /// The family: the answer to one who asks to adopt.
    Family = family,
    /// The window of emotions and the Shortcut List.
    Emotions = emotions,
    /// The World Map and the large window of the map one is on.
    WorldMap = worldmap,
    /// The Tipbox.
    Tips = tips,
    /// The windows a fight opens: what Sense tells, a Star Gladiator's place.
    Fight = fight,
    /// The merchant's cart.
    Cart = cart,
    /// A merchant's shop.
    Vending = vending,
    /// The books that are read: the buttons of a book's window, and those an item that is a book has.
    Books = books,
}

#[cfg(test)]
mod tests {
    use super::*;

    presses! {
        /// Presses made up for the tests below.
        Sample {
            Close = "sample-close",
            Tab(usize) = "sample-tab",
            Named(String) = "sample-named",
        }
    }

    #[test]
    fn a_press_is_read_back_from_what_it_is_written_as() {
        for press in [Sample::Close, Sample::Tab(3), Sample::Named("a b".to_owned())] {
            let (what, arg) = press.name();
            assert_eq!(Sample::parse(what, &arg), Some(press));
        }
    }

    #[test]
    fn a_press_with_the_wrong_thing_beside_it_is_no_press() {
        assert_eq!(Sample::parse("sample-tab", "three"), None);
        assert_eq!(Sample::parse("sample-open", ""), None);
    }

    #[test]
    fn no_two_presses_share_a_name() {
        let mut names: Vec<&str> = Press::NAMES.iter().flat_map(|names| names.iter().copied()).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "two presses have one name");
    }
}
