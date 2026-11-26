//! The presses of the bank (the client's `UIBank_NewWnd`).

use super::Step;

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon and the key: the balance is asked of the server, whose answer opens the window, or
        /// closes it if it was open.
        Window = "bank",
        /// The cross of the window, and Escape: it closes.
        Close = "bank-close",
        /// "Deposit" and "Withdraw": the sum typed goes that way.
        Deposit = "bank-deposit",
        Withdraw = "bank-withdraw",
        /// The two arrows by the field: a zeny more, a zeny less.
        One(Step) = "bank-one",
        /// "+ 100,000", "+ 1,000,000" and "+ 10,000,000": so much more.
        Add(u32) = "bank-add",
        /// "Max": the word for all there is stands in the field.
        Max = "bank-max",
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
            Press::Deposit,
            Press::Withdraw,
            Press::One(Step::More),
            Press::One(Step::Less),
            Press::Add(100_000),
            Press::Max,
        ]);
    }
}
