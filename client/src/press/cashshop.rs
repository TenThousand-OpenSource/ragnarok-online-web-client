//! The presses of the cash shop (the client's `UICashShopWnd`).

use super::Step;
use crate::cashshop::Turn;

impl Turn {
    pub const ALL: [Turn; 4] = [Turn::First, Turn::Back, Turn::On, Turn::Last];

    /// How the arrow is written on the page.
    pub fn text(self) -> &'static str {
        match self {
            Turn::First => "first",
            Turn::Back => "back",
            Turn::On => "on",
            Turn::Last => "last",
        }
    }
}

worded!(Turn);

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon and the key: the server is asked for the window, and its answer opens it, or
        /// closes it if it was open.
        Window = "cashshop",
        /// The cross of the window, and Escape: it closes.
        Close = "cashshop-close",
        /// A tab, by the list it shows.
        Tab(usize) = "cashshop-tab",
        /// The four arrows under the things.
        Page(Turn) = "cashshop-page",
        /// "Buy" on a thing of the page, by its place there: one of it goes into the list of what is bought.
        Add(usize) = "cashshop-add",
        /// The plus and the minus of a kind in that list, by the item: one more, one less.
        More(u32) = "cashshop-more",
        Less(u32) = "cashshop-less",
        /// The small cross of a kind in that list: it is taken out.
        Take(u32) = "cashshop-take",
        /// The list of what is bought moved by the wheel's notch or its bar's arrows.
        Scroll(Step) = "cashshop-scroll",
        /// The button by the line that is searched with.
        Search = "cashshop-search",
        /// "Refresh": the sale for one account is asked for again.
        Refresh = "cashshop-refresh",
        /// "Charging": the page where cash is bought.
        Charge = "cashshop-charge",
        /// "Purchase": what is in the list is bought.
        Buy = "cashshop-buy",
        /// OK and Cancel of what the shop asks or tells in a message box.
        Yes = "cashshop-yes",
        No = "cashshop-no",
        /// A banner of the strip under the title: its page is opened in the browser.
        Banner = "cashshop-banner",
        /// One of the small buttons that pick a banner, by its place.
        BannerPicked(usize) = "cashshop-banner-picked",
        /// The moment the next banner is due, with the count of banners shown when it was set: a banner picked
        /// since makes it late, and it does nothing.
        BannerDue(u32) = "cashshop-banner-due",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        crate::press::testing::read_back(
            [
                Press::Window,
                Press::Close,
                Press::Tab(9),
                Press::Add(8),
                Press::More(501),
                Press::Less(501),
                Press::Take(501),
                Press::Scroll(Step::More),
                Press::Scroll(Step::Less),
                Press::Search,
                Press::Refresh,
                Press::Charge,
                Press::Buy,
                Press::Yes,
                Press::No,
                Press::Banner,
                Press::BannerPicked(3),
                Press::BannerDue(12),
            ]
            .into_iter()
            .chain(Turn::ALL.map(Press::Page))
        );
    }
}
