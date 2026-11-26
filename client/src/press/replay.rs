//! The presses of the replays: the menu's icon and the window a session is recorded with (the client's
//! `UIReplayRECControlWnd`), the list of recordings before the game (`UISelectReplayDataWnd`), and the
//! window that plays one (`UIReplayControlWnd`).

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon: the recorder opens, or closes if it was open.
        Window = "replay",
        /// The recorder: to begin, to stop, to open its options and to close them, to put it away.
        Start = "replay-start",
        Stop = "replay-stop",
        Options = "replay-options",
        OptionsClose = "replay-options-close",
        End = "replay-end",
        /// A box of what is recorded, by its place among the options; the box "User Defined File Name" and
        /// the box "Repeated File Check".
        Option(usize) = "replay-option",
        NameAsked = "replay-name-asked",
        Checked = "replay-checked",
        /// The name is typed: the recording begins.
        NameOk = "replay-name-ok",
        /// The button of the login screen: the list of recordings opens.
        List = "replay-list",
        /// A recording of the list; OK plays the one picked; the button that goes back to the login.
        Row(usize) = "replay-row",
        Play = "replay-play",
        Back = "replay-back",
        /// Of this project's own, the browser having no folder of replays: the recording picked is saved as a
        /// file, a file is taken into the list, the recording picked is thrown away.
        Save = "replay-save",
        Load = "replay-load",
        Delete = "replay-delete",
        /// The player: to go on or to hold, from the beginning again, out of it, faster and slower.
        Hold = "replay-hold",
        Again = "replay-again",
        Out = "replay-out",
        Faster = "replay-faster",
        Slower = "replay-slower",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let recorder = [Press::Window, Press::Start, Press::Stop, Press::Options, Press::OptionsClose, Press::End, Press::Option(9)];
        let names = [Press::NameAsked, Press::Checked, Press::NameOk];
        let list = [Press::List, Press::Row(2), Press::Play, Press::Back, Press::Save, Press::Load, Press::Delete];
        let player = [Press::Hold, Press::Again, Press::Out, Press::Faster, Press::Slower];
        crate::press::testing::read_back(recorder.into_iter().chain(names).chain(list).chain(player));
    }
}
