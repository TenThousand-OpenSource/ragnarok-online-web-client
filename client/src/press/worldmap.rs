//! The presses of the World Map (the client's `UIRoMapWnd`, Ctrl+~) and of the large window of the map one is
//! on (`UIMiniMapWnd`, the minimap's "mini" button).

presses! {
    /// What can be pressed here.
    Press {
        /// The World Map opens, or closes if it was open.
        Window = "worldmap",
        /// The box that names the world shown: its list opens and closes.
        Worlds = "worldmap-worlds",
        /// A world of that list, by its place in the client's table.
        World(usize) = "worldmap-world",
        /// The box "Monster(Tab)": the maps' names and levels are written on them, or no more.
        Labels = "worldmap-labels",
        /// The large window of the map one is on opens, or closes if it was open.
        Large = "map-large",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Window, Press::Worlds, Press::World(3), Press::Labels, Press::Large];
        assert_eq!(all.len(), Press::NAMES.len(), "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
