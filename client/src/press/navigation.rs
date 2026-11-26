//! The presses of the navigation (the client's `UINavigationV4Wnd` with its windows of help, of the sets of
//! arrows and of the route, the menu's icon, and the links that start a navigation).

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon and the key: the window opens, or is closed as its cross closes it.
        Window = "navigation",
        /// The cross (`ID_CLOSE`): a navigation under way is asked about first.
        Close = "navigation-close",
        /// The answers to "Do you want to cancel navigation?".
        Yes = "navigation-yes",
        No = "navigation-no",
        /// The window becomes the small one (`ID_MINI`), and the large one again (`ID_BASE`).
        Small = "navigation-small",
        Large = "navigation-large",
        /// The question mark: the window of help opens and closes.
        Help = "navigation-help",
        /// The arrow at the left: between the lists and the map of the way (`ID_CHARGE`).
        Turn = "navigation-turn",
        /// The box that narrows the lists: its list opens and closes; and a row of it (0 all, 1 maps, 2 NPCs,
        /// 3 monsters).
        Kinds = "navigation-kinds",
        Kind(i32) = "navigation-kind",
        /// A row of the list of what was found, and a row of the list of the group picked.
        Group(usize) = "navigation-group",
        Member(usize) = "navigation-member",
        /// The box "service": ways that cost zeny and the airship are taken, or not.
        Service = "navigation-service",
        /// The button that shares what is looked at (`ID_PREVIEW`): a link to it goes into the chat's line,
        /// or, for a map, the map is shown to press a place on.
        Share = "navigation-share",
        /// A place pressed on that map: where in its picture, counted from the left and the top.
        Pick(Place) = "navigation-pick",
        /// The button that finds the way (`NAVI_BUTTON_FIND`).
        Find = "navigation-find",
        /// The button of the private airship (`NAVI_BUTTON_PRIVATE_AIRPLANE`).
        Airship = "navigation-airship",
        /// The button that reads a file of places (`NAVI_BUTTON_DATA`).
        Files = "navigation-files",
        /// The button that opens the window of the sets of arrows, and a set of that window (1 to 8).
        Icons = "navigation-icons",
        Icon(u8) = "navigation-icon",
        /// The button that opens the list of the route (`NAVI_BUTTON_ROAD_LIST`).
        Route = "navigation-route",
        /// The box "Minimize": the window becomes the small one as the character is sent walking.
        Fold = "navigation-fold",
        /// A link `<NAVI>` in a text: what its `<INFO>` holds, as it is written there.
        Go(String) = "navigation-go",
        /// A link `<NAVIL>` in the chat: what stands between its tags.
        Chat(String) = "navigation-chat",
        /// A map pressed on the World Map: the navigation's window opens on what the map has.
        Field(String) = "navigation-field",
        /// A mark of the minimap (who stands where in a town): the way to that cell of the map one is on.
        Mark(Place) = "navigation-mark",
    }
}

/// A place: two whole numbers, written with a comma between them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place(pub i32, pub i32);

impl std::fmt::Display for Place {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(to, "{},{}", self.0, self.1)
    }
}

impl std::str::FromStr for Place {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let (x, y) = text.split_once(',').ok_or(())?;
        Ok(Place(x.trim().parse().map_err(|_| ())?, y.trim().parse().map_err(|_| ())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        crate::press::testing::read_back([
            Press::Window,
            Press::Close,
            Press::Yes,
            Press::No,
            Press::Small,
            Press::Large,
            Press::Help,
            Press::Turn,
            Press::Kinds,
            Press::Kind(3),
            Press::Group(7),
            Press::Member(2),
            Press::Service,
            Press::Share,
            Press::Pick(Place(133, 20)),
            Press::Find,
            Press::Airship,
            Press::Files,
            Press::Icons,
            Press::Icon(8),
            Press::Route,
            Press::Fold,
            Press::Go("prontera,150,180,0,101,0,0".to_owned()),
            Press::Chat("prontera2q2U".to_owned()),
            Press::Field("prt_fild08".to_owned()),
            Press::Mark(Place(-1, 4)),
        ]);
    }
}
