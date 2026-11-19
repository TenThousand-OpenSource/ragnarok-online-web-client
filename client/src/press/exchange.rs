//! The presses of a deal between two players: the question whether to deal, the deal's window with its OK,
//! Trade and Cancel, and how many of a thing go into it.

presses! {
    /// What can be pressed here.
    Press {
        /// Someone asks for a deal: OK and Cancel.
        Accept = "exchange-accept",
        Refuse = "exchange-refuse",
        /// The window's OK: the zeny typed goes in and the player's side is closed.
        Lock = "exchange-lock",
        /// Trade, once both sides are closed.
        Trade = "exchange-trade",
        /// Cancel: the deal is called off.
        Cancel = "exchange-cancel",
        /// The box that has a screenshot taken of the deal.
        Screenshot = "exchange-screenshot",
        /// How many of a thing go in: OK.
        CountOk = "exchange-count-ok",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Accept, Press::Refuse, Press::Lock, Press::Trade, Press::Cancel, Press::Screenshot, Press::CountOk];
        for name in Press::NAMES {
            assert!(all.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
