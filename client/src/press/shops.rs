//! The presses of an NPC's shop: the question whether to buy or to sell, and what is pressed in its two windows,
//! the list and the basket.

presses! {
    /// What can be pressed here.
    Press {
        /// The shop's question: to buy.
        Buy = "shop-buy",
        /// The shop's question: to sell.
        Sell = "shop-sell",
        /// Cancel: the question or the shop is left.
        Cancel = "shop-cancel",
        /// The button under the basket: what is in it is bought or sold.
        Ok = "shop-ok",
        /// The list's box "Toggle Item Amount": whether a stack goes across whole without the question.
        WholeStacks = "shop-whole-stacks",
        /// The OK of the box that asks how many. The box is a form and its OK sends it, so the press does nothing.
        CountOk = "shop-count-ok",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Buy, Press::Sell, Press::Cancel, Press::Ok, Press::WholeStacks, Press::CountOk];
        assert_eq!(all.len(), Press::NAMES.len());
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
