//! The presses before the game and on the way out of it: the login and the forms beside it (registering,
//! recovering a password), the question and the wait boxes, the servers, the characters, the making of a
//! character, and leaving.

/// Writes an enum of a few choices, each with the text it is on the page as, and both ways of that text.
macro_rules! choices {
    ($(#[$meta:meta])* $name:ident { $($(#[$about:meta])* $variant:ident = $text:literal,)* }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum $name {
            $($(#[$about])* $variant,)*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
                to.write_str(match self {
                    $(Self::$variant => $text,)*
                })
            }
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(text: &str) -> Result<Self, ()> {
                match text {
                    $($text => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }
    };
}

choices! {
    /// The race of a new character.
    Race {
        Human = "human",
        Doram = "doram",
    }
}

choices! {
    /// The sex of a new character, written as the number the servers know it by.
    Sex {
        Female = "0",
        Male = "1",
    }
}

choices! {
    /// Which way the new character is turned by one of the two arrows under it.
    Turn {
        Left = "-1",
        Right = "1",
    }
}

presses! {
    /// What can be pressed here.
    Press {
        /// The question box: OK.
        Yes = "yes",
        /// The question box: Cancel.
        No = "no",
        /// The wait box: Cancel.
        CancelWait = "cancel-wait",
        /// The login's box for remembering the ID.
        SaveId = "save-id",
        /// The login: to the registration.
        ToRegister = "to-register",
        /// The login: to the recovery of a password.
        ToRecover = "to-recover",
        /// A form beside the login: back to the login.
        ToLogin = "to-login",
        /// A line of the list of servers, by its place in the list.
        Server(usize) = "server",
        /// The servers: OK.
        ServerOk = "server-ok",
        /// The servers: Cancel.
        ServerCancel = "server-cancel",
        /// The game's menu: out of the game altogether.
        Exit = "exit",
        /// Leaving: from the game to the characters, from the characters out.
        Logout = "logout",
        /// The characters: the chosen one is to be deleted (it asks first).
        Delete = "delete",
        /// The characters: the chosen one, whose wait is over, is deleted for good (it asks first).
        DeleteNow = "delete-now",
        /// The characters: the chosen one is not to be deleted after all.
        CancelDelete = "cancel-delete",
        /// A slot of the page of characters, by its number.
        Slot(u8) = "slot",
        /// The characters: into the game with the chosen one.
        Start = "start",
        /// The making of a character: one of the two races.
        Race(Race) = "race",
        /// The making of a character: one of the two sexes.
        Sex(Sex) = "sex",
        /// The making of a character: a hair style, by its number from one.
        Hair(u16) = "hair",
        /// The making of a character: a hair colour, by its number from nought.
        Colour(u16) = "colour",
        /// The making of a character: one of the arrows that turn it.
        Turn(Turn) = "turn",
        /// The making of a character: back to the characters.
        Back = "back",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let presses = [
            Press::Yes,
            Press::No,
            Press::CancelWait,
            Press::SaveId,
            Press::ToRegister,
            Press::ToRecover,
            Press::ToLogin,
            Press::Server(2),
            Press::ServerOk,
            Press::ServerCancel,
            Press::Exit,
            Press::Logout,
            Press::Delete,
            Press::DeleteNow,
            Press::CancelDelete,
            Press::Slot(14),
            Press::Start,
            Press::Race(Race::Human),
            Press::Race(Race::Doram),
            Press::Sex(Sex::Female),
            Press::Sex(Sex::Male),
            Press::Hair(23),
            Press::Colour(8),
            Press::Turn(Turn::Left),
            Press::Turn(Turn::Right),
            Press::Back,
        ];
        // Every name is in the list at least once.
        for name in Press::NAMES {
            assert!(presses.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in presses {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }

    #[test]
    fn a_press_with_the_wrong_thing_beside_it_is_no_press() {
        assert_eq!(Press::parse("race", "elf"), None);
        assert_eq!(Press::parse("turn", "2"), None);
        assert_eq!(Press::parse("slot", "-1"), None);
    }
}
