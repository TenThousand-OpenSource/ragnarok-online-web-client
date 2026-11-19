//! The presses of a merchant's shop: the window a shop is made in, one's own shop while it is open, the sign
//! over a vendor's head, a vendor's list and the basket of what is bought from it.

presses! {
    /// What can be pressed here.
    Press {
        /// The shop being made: its OK (the form it stands in is sent) and its Cancel.
        StallOk = "stall-ok",
        StallCancel = "stall-cancel",
        /// What the making of a shop asks: OK and Cancel.
        Yes = "vending-yes",
        No = "vending-no",
        /// One's own shop is closed.
        MineClose = "vending-mine-close",
        /// The sign over a vendor's head, by the vendor's account.
        Sign(u32) = "vending-sign",
        /// The basket: what is in it is to be bought; the vendor is left.
        Buy = "vending-buy",
        Leave = "vending-leave",
        /// The window that asks whether to buy for that sum: Buy and Cancel.
        Confirm = "vending-confirm",
        Refuse = "vending-refuse",
        /// How many of a thing go into the shop or the basket: OK.
        CountOk = "vending-count-ok",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [
            Press::StallOk, Press::StallCancel, Press::Yes, Press::No, Press::MineClose, Press::Sign(2_000_001),
            Press::Buy, Press::Leave, Press::Confirm, Press::Refuse, Press::CountOk
        ];
        crate::press::testing::read_back(all);
    }
}
