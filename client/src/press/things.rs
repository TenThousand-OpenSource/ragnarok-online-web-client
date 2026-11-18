//! What is done to the things in the windows other than pressing them: a double click on one (`data-use`),
//! one dragged out of its place (`data-drag`), a form sent (`data-submit`). Each is written on the page as its
//! name, then a colon and what goes with it, and read back from that.

use super::Named;

presses! {
    /// A double click on a thing.
    Use {
        /// An item carried, by its place in the inventory: it is used, worn or taken off.
        Item(u16) = "item",
        /// A skill, by its number: it is used.
        Skill(u16) = "skill",
        /// A slot of the hotkey bar: what is on it is used.
        Slot(usize) = "slot",
        /// A row of a shop's list: it goes into the basket.
        ShopAdd(usize) = "shop-add",
        /// A row of the basket: it goes back.
        ShopRemove(usize) = "shop-remove",
        /// An item carried, with the storage open: all of it goes in.
        Put(u16) = "put",
        /// An item in the storage: all of it comes out.
        Take(u16) = "take",
        /// A row of the eggs to hatch: it is picked and confirmed.
        Egg(usize) = "egg",
        /// A row of the items to identify: it is picked and confirmed.
        Identify(usize) = "identify",
    }
}

/// A skill and the level it is at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leveled {
    pub id: u32,
    pub level: u16,
}

impl std::fmt::Display for Leveled {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(to, "{}:{}", self.id, self.level)
    }
}

impl std::str::FromStr for Leveled {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let (id, level) = text.split_once(':').ok_or(())?;
        Ok(Leveled { id: id.parse().map_err(|_| ())?, level: level.parse().map_err(|_| ())? })
    }
}

presses! {
    /// A thing dragged out of its place.
    Drag {
        /// An item, by its number (what it is, not where it is carried).
        Item(u32) = "item",
        /// A skill at a level.
        Skill(Leveled) = "skill",
        /// What is on a slot of the hotkey bar.
        Slot(usize) = "slot",
    }
}

presses! {
    /// A form of the page, sent.
    Form {
        Login = "login",
        Register = "register",
        Recover = "recover",
        Reset = "reset",
        /// The making of a character.
        Create = "create",
        /// How many of a thing to drop.
        Throw = "throw",
        /// A line of chat.
        Chat = "chat",
    }
}

/// A thing of one of these kinds as the page has it: its name, and after a colon what goes with it, if anything
/// does.
pub fn written(thing: &impl Named) -> String {
    match thing.name() {
        (name, with) if with.is_empty() => name.to_owned(),
        (name, with) => format!("{name}:{with}"),
    }
}

/// The thing the page's text stands for.
pub fn read<Thing: Named>(text: &str) -> Option<Thing> {
    let (name, with) = text.split_once(':').unwrap_or((text, ""));
    Thing::parse(name, with)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_is_read_back_from_what_it_is_written_as() {
        for thing in [Use::Item(7), Use::Skill(28), Use::Slot(35), Use::ShopAdd(2), Use::ShopRemove(0), Use::Put(3), Use::Take(9), Use::Egg(1), Use::Identify(4)] {
            assert_eq!(read::<Use>(&written(&thing)), Some(thing));
        }
        for thing in [Drag::Item(501), Drag::Skill(Leveled { id: 19, level: 10 }), Drag::Slot(8)] {
            assert_eq!(read::<Drag>(&written(&thing)), Some(thing));
        }
        for thing in [Form::Login, Form::Register, Form::Recover, Form::Reset, Form::Create, Form::Throw, Form::Chat] {
            assert_eq!(read::<Form>(&written(&thing)), Some(thing));
        }
    }

    #[test]
    fn they_are_written_as_the_page_has_always_had_them() {
        assert_eq!(written(&Use::Item(7)), "item:7");
        assert_eq!(written(&Drag::Skill(Leveled { id: 19, level: 10 })), "skill:19:10");
        assert_eq!(written(&Form::Chat), "chat");
        assert_eq!(read::<Use>("item:x"), None);
        assert_eq!(read::<Drag>("skill:19"), None);
    }
}
