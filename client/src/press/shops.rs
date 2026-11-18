//! The presses of an NPC's shop and of the Kafra's storage: the question whether to buy or to sell, the OK and
//! Cancel of the lists, and the storage's tabs and its "close".

presses! {
    /// What can be pressed here.
    Press {
        /// A line of the shop's list is picked: which, counted from 0.
        Pick(usize) = "shop-pick",
        /// The shop's question: to buy.
        Buy = "shop-buy",
        /// The shop's question: to sell.
        Sell = "shop-sell",
        /// Cancel: the question or the list is left.
        Cancel = "shop-cancel",
        /// The button under what was put aside: it is bought or sold.
        Ok = "shop-ok",
        /// A tab of the storage is opened: which, counted from 0.
        StorageTab(usize) = "storage-tab",
        /// The storage is closed.
        StorageClose = "storage-close",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Pick(2), Press::Buy, Press::Sell, Press::Cancel, Press::Ok, Press::StorageTab(5), Press::StorageClose] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
