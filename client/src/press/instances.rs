//! The presses of the instances' window (the client's `UIMemorialDunWnd`).

presses! {
    /// What can be pressed here.
    Press {
        /// The key of the window (`HK_MEMORIALWND_ONOFF`, Alt+B): it shows and hides while the party has an
        /// instance.
        Window = "instance",
        /// The cross of the window, and Escape: it is put away.
        Close = "instance-close",
        /// "Destroy instance", the leader's button.
        Destroy = "instance-destroy",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        crate::press::testing::read_back([Press::Window, Press::Close, Press::Destroy]);
    }
}
