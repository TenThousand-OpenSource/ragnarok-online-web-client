//! What is done to the things in the windows other than pressing them: a double click on one (`Use`,
//! `data-use`), one dragged out of its place (`Drag`, `data-drag`) and the window it is let go over (`Drop`,
//! `data-drop`), the right button on one that has a menu (`Menu`, `data-menu`), a form sent (`Form`,
//! `data-submit`). Each is written on the page as its name, then a colon and what goes with it, and read back
//! from that.

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
        /// A friend in the list of friends, by its place: a window opens for a talk with that friend alone.
        Friend(usize) = "friend",
        /// A row of a list a skill or a card opened: it is picked and confirmed.
        Choice(usize) = "choice",
        /// The sign of a chat room over its owner's head, by the room's number: the room is entered.
        Room(u32) = "room",
        /// An emotion of the window of emotions: it is shown.
        Emotion(u16) = "emotion",
        /// The sign over a vendor's head, by the vendor's account: what the shop sells is asked for.
        Sign(u32) = "sign",
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
        /// A thing worn, by its place in the inventory: taken out of the Equipment window.
        Worn(u16) = "worn",
        /// A thing put into a letter of the mail, by its place in the inventory: taken out of the letter.
        Attached(u16) = "attached",
        /// A row of a shop's list, counted from 0: taken out of the list towards the basket.
        Offer(usize) = "offer",
        /// A row of a shop's list that is in the basket: taken out of the basket.
        Basket(usize) = "basket",
        /// A thing in the storage, by its place there: taken out of the storage's window.
        Stored(u16) = "stored",
        /// A thing in the cart, by its place there: taken out of the cart's window.
        Carted(u16) = "carted",
        /// A thing of the cart offered to a shop being made, by its place in the cart: taken towards the shop.
        Stock(u16) = "stock",
        /// A row of a shop being made, counted from 0: taken back out of it.
        Stalled(usize) = "stalled",
        /// A row of a vendor's list, counted from 0: taken towards the basket.
        Vended(usize) = "vended",
        /// A row of a vendor's list that is in the basket: taken out of the basket.
        Wanted(usize) = "wanted",
    }
}

presses! {
    /// A thing the right button opens a menu on.
    Menu {
        /// A member of the party, by its place in the party window's list.
        Member(usize) = "member",
        /// A member of the guild, by its place in the guild window's list.
        GuildMember(usize) = "guild-member",
        /// A guild allied to the player's or opposed, by its place among those the server told.
        GuildRelated(usize) = "guild-related",
        /// A friend, by its place in the list of friends.
        Friend(usize) = "friend",
        /// The window of a talk with one player, by its place in the list of windows.
        Whisper(usize) = "whisper",
        /// The window of the chat room sat in, and a member in its list by its place there.
        Room = "room",
        RoomMember(usize) = "room-member",
    }
}

presses! {
    /// A window a thing dragged is let go over, for one that takes things (`data-drop`; the hotkey bar's slots
    /// have their own mark).
    Drop {
        /// The list of a shop.
        Shop = "shop",
        /// The basket of a shop: what is bought, or what is sold.
        Basket = "basket",
        /// The storage, and the windows that show a part of it.
        Storage = "storage",
        /// The cart's window.
        Cart = "cart",
        /// The window of a shop being made, and the one beside it of what the cart has for it.
        Stall = "stall",
        Stock = "stock",
        /// A vendor's list, and the basket of what is bought from it.
        Vendor = "vendor",
        Wants = "wants",
        /// A tab of the inventory, counted from 0: a thing let go on "Fav" is kept under it, on another no more.
        InventoryTab(usize) = "inventory-tab",
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
        /// How many of a thing go into a deal between two players.
        Offer = "offer",
        /// How many of a thing go into a letter of the mail.
        Attach = "attach",
        /// What is searched for in the mailbox.
        MailSearch = "mail-search",
        /// How many of a thing go into a shop's basket or out of it.
        ShopCount = "shop-count",
        /// How many of a thing go into the storage or out of it.
        StorageCount = "storage-count",
        /// What is searched for in the storage.
        StorageSearch = "storage-search",
        /// A line typed into the window of a talk with one player: the window's place in the list of windows.
        Whisper(usize) = "whisper",
        /// A line typed into the window of the chat room sat in.
        Room = "room",
        /// The window that makes or changes a chat room, sent with Enter.
        RoomMake = "room-make",
        /// The password of a chat room, typed into the box that asks for it.
        RoomPassword = "room-password",
        /// How many of a thing go into the cart or out of it.
        CartCount = "cart-count",
        /// How many of a thing go into a shop being made, or into the basket at a vendor's.
        VendingCount = "vending-count",
        /// The shop being made, with its title and prices: it is opened.
        Stall = "stall",
        /// The ten lines of the Shortcut List, kept as they are typed.
        Shortcuts = "shortcuts",
        /// What is looked for in the Tipbox.
        TipSearch = "tip-search",
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

    /// One thing of every name of a kind, made from the kind's own list of names: with nothing beside it, or
    /// with what the things here take (a number, a skill and its level).
    fn every<Thing: Named>() -> Vec<Thing> {
        Thing::NAMES
            .iter()
            .map(|name| ["", "7", "19:10"].iter().find_map(|with| Thing::parse(name, with)).unwrap_or_else(
                || panic!("{name} takes nothing this test knows of")
            ))
            .collect()
    }

    /// Each is read back as itself, and no two of a kind share a name.
    fn read_back<Thing: Named + PartialEq + std::fmt::Debug>() {
        let things = every::<Thing>();
        assert!(!things.is_empty());
        for (at, thing) in things.iter().enumerate() {
            assert_eq!(read::<Thing>(&written(thing)).as_ref(), Some(thing));
            assert_eq!(things.iter().position(|other| other == thing), Some(at), "two things of a kind have one name");
        }
    }

    #[test]
    fn each_is_read_back_from_what_it_is_written_as() {
        read_back::<Use>();
        read_back::<Drag>();
        read_back::<Menu>();
        read_back::<Drop>();
        read_back::<Form>();
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
