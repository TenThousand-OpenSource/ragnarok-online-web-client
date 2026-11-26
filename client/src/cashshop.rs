//! The cash shop as rules (the client's `UICashShopWnd` with its `UICashShopWnd_Cart`): its tabs and pages,
//! the list of what is to be bought and what the client says when it will not take more, what a search
//! finds, what the buying costs and what the server's answer says. The lines are numbers of the client's
//! message table (`msgstringtable.txt`).

/// How many lists the client keeps (`CSession::inventoryCategories`): the nine tabs of the server (New, Hot,
/// Limited, Rental, Permanent, Scrolls, Consumables, Other, Sale), the sale for one account, and what a
/// search found.
pub const LISTS: usize = 11;
/// The list of the sale for one account (`CASHSHOP_TAB_ACCOUNT_LIMITED`), whose page is laid out larger.
pub const LIMITED: usize = 9;
/// The list of what a search found.
pub const FOUND: usize = 10;
/// The lists that have a tab to press, in the order of the tabs (`UICashShopWnd::OnCreate`: every list but
/// the eighth and the search's).
pub const TABS: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 9];
/// How many kinds of thing the list of what is bought holds, and how many of one kind
/// (`UICashShopWnd_Cart::SendMsg`, `UM_ADDITEM`).
pub const KINDS: usize = 8;
pub const MOST: i32 = 100;
/// How many kinds the list shows at once (`UICashShopWnd_Cart::m_visibleCount`).
pub const SHOWN: usize = 6;

/// The line of the message table a tab tells when pointed at (`OnCreate`'s table of tooltips).
pub fn tab_line(tab: usize) -> usize {
    match tab {
        9 => 3656,
        10 => 3790,
        tab => 3290 + tab,
    }
}

/// A thing the shop offers: the item, its price in cash points, and the list of the server it came in.
#[derive(Debug, Clone, PartialEq)]
pub struct Good {
    pub item: u32,
    pub price: u32,
    pub tab: u16,
}

impl Good {
    /// The list as it is named when the thing is bought (`CGameMode::ReceiveTypedItemList`: the eighth list,
    /// the sale, is written as 0xFFFF).
    pub fn bought_as(&self) -> u16 {
        if self.tab == 8 { 0xFFFF } else { self.tab }
    }
}

/// A kind of thing in the list of what is bought, and how many of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub good: Good,
    pub amount: i32,
}

/// How many things a page shows under a tab (`UM_REFRESHITEM`): six of the larger ones under the sale for
/// one account, nine elsewhere.
pub fn page_size(tab: usize) -> usize {
    if tab == LIMITED { 6 } else { 9 }
}

/// How many pages so many things make.
pub fn pages(things: usize, page_size: usize) -> usize {
    things.div_ceil(page_size.max(1))
}

/// A turn of the pages (`UICashShopWnd::SendMsg`: the four arrows).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    First,
    Back,
    On,
    Last,
}

/// The page after an arrow was pressed. (With no pages the client's "on" and "last" come to the page before
/// the first, and it lays out nothing; here the first page stays, which shows the same nothing.)
pub fn turned(page: usize, pages: usize, turn: Turn) -> usize {
    let last = pages.saturating_sub(1);
    match turn {
        Turn::First => 0,
        Turn::Back => page.saturating_sub(1),
        Turn::On => (page + 1).min(last),
        Turn::Last => last,
    }
}

/// What the client says in a message box when the list of what is bought does not take what was asked, as a
/// line of the message table and the number written into it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Said {
    /// Nothing was asked for (line 3664).
    Nothing,
    /// The list holds its eight kinds (line 3312).
    Full,
    /// More than a hundred of a kind: the hundred are kept (line 1704).
    TooMany,
}

impl Said {
    pub fn line(self) -> (usize, Option<i32>) {
        match self {
            Said::Nothing => (3664, None),
            Said::Full => (3312, Some(KINDS as i32)),
            Said::TooMany => (1704, Some(MOST)),
        }
    }
}

/// So many of a thing are put into the list of what is bought, or taken out of it when the number is under
/// nothing (`UICashShopWnd_Cart::SendMsg`, `UM_ADDITEM`): a kind already there has so many more, never under
/// one and never over a hundred; a new kind is added while there are under eight. Gives what the client says
/// when it does not do all of it.
pub fn put(list: &mut Vec<Line>, good: &Good, amount: i32) -> Option<Said> {
    if amount == 0 {
        return Some(Said::Nothing);
    }
    if let Some(line) = list.iter_mut().find(|line| line.good.item == good.item) {
        let total = line.amount + amount;
        line.amount = total.clamp(1, MOST);
        return (total > MOST).then_some(Said::TooMany);
    }
    if list.len() >= KINDS {
        return Some(Said::Full);
    }
    list.push(Line { good: good.clone(), amount });
    None
}

/// A kind is taken out of the list (`UM_DECITEM`).
pub fn taken(list: &mut Vec<Line>, item: u32) {
    list.retain(|line| line.good.item != item);
}

/// What everything in the list costs (`UM_SET_CASHSHOP_TOTAL_PRICE`).
pub fn total(list: &[Line]) -> i64 {
    list.iter().map(|line| i64::from(line.good.price) * i64::from(line.amount)).sum()
}

/// The kind the list shows first after the wheel or a new kind moved it by so many (`UM_SCROLLVERT`): it
/// stays where no more kinds are than show, and never goes past the last six.
pub fn scrolled(top: usize, by: i32, kinds: usize) -> usize {
    let most = kinds.saturating_sub(SHOWN);
    (top as i64 + i64::from(by)).clamp(0, most as i64) as usize
}

/// What a search finds (`UICashShopWnd::SendMsg`, `ID_SEARCH`): the things of every list but the two sales
/// and the search's own whose name has the words in it, as they are written, each item once. `name` gives an
/// item's name.
pub fn found(lists: &[Vec<Good>], sought: &str, name: impl Fn(u32) -> String) -> Vec<Good> {
    let mut found: Vec<Good> = Vec::new();
    for (at, list) in lists.iter().enumerate() {
        if matches!(at, 8 | LIMITED | FOUND) {
            continue;
        }
        for good in list {
            if name(good.item).contains(sought) && !found.iter().any(|had| had.item == good.item) {
                found.push(good.clone());
            }
        }
    }
    found
}

/// What pressing "Purchase" comes to (`UICashShopWnd::SendMsg`, `ID_BUY`).
#[derive(Debug, Clone, PartialEq)]
pub enum Buying {
    /// The list is empty: nothing happens.
    Nothing,
    /// The client refuses, with that line in a message box: more free points are to be used than the things
    /// cost (1860), or the cash and the free points to be used do not come to the price (1254).
    Refused(usize),
    /// The client asks whether it is meant, with that line of the table and the numbers written into it: the
    /// price alone (3771), or what is paid in cash and what in free points (3772).
    Asked(usize, Vec<i64>),
}

/// "Purchase" was pressed with that list, that much cash, and so many free points typed in to be used.
pub fn buying(list: &[Line], cash: i64, points: i64) -> Buying {
    if list.is_empty() {
        return Buying::Nothing;
    }
    let price = total(list);
    if price < points {
        return Buying::Refused(1860);
    }
    if price > cash + points {
        return Buying::Refused(1254);
    }
    if points > 0 { Buying::Asked(3772, vec![price - points, points]) } else { Buying::Asked(3771, vec![price]) }
}

/// What the server's answer to a purchase does (`CGameMode::ReceiveRequestResult`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Answer {
    /// The points are as the answer says now and the list of what is bought is emptied.
    pub done: bool,
    /// A line of the message table said in the chat, in red.
    pub line: Option<usize>,
    /// The answers 303 and 304 are shown in a message box titled "Error" as well, with their number before
    /// the line.
    pub boxed: bool,
}

/// By the answer's result.
pub fn answer(result: i16) -> Answer {
    let (done, line, boxed) = match result {
        0 => (true, None, false),
        8 => (true, Some(1814), false),
        9 => (false, Some(1944), false),
        10 => (false, Some(1945), false),
        11 => (false, Some(1946), false),
        12 => (false, Some(1947), false),
        303 => (false, Some(3940), true),
        304 => (false, Some(3941), true),
        _ => (false, Some(57), false),
    };
    Answer { done, line, boxed }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good(item: u32, price: u32) -> Good {
        Good { item, price, tab: 1 }
    }

    #[test]
    fn a_page_is_nine_things_and_six_under_the_sale_for_one_account() {
        assert_eq!((page_size(0), page_size(LIMITED), page_size(FOUND)), (9, 6, 9));
        assert_eq!((pages(0, 9), pages(1, 9), pages(9, 9), pages(10, 9), pages(13, 6)), (0, 1, 1, 2, 3));
    }

    #[test]
    fn the_arrows_turn_the_pages_and_stop_at_the_ends() {
        assert_eq!(turned(0, 3, Turn::On), 1);
        assert_eq!(turned(2, 3, Turn::On), 2);
        assert_eq!(turned(2, 3, Turn::Back), 1);
        assert_eq!(turned(0, 3, Turn::Back), 0);
        assert_eq!(turned(1, 3, Turn::First), 0);
        assert_eq!(turned(0, 3, Turn::Last), 2);
        assert_eq!((turned(0, 0, Turn::On), turned(0, 0, Turn::Last)), (0, 0));
    }

    #[test]
    fn the_list_takes_eight_kinds_and_a_hundred_of_each() {
        let mut list = Vec::new();
        assert_eq!(put(&mut list, &good(501, 10), 0), Some(Said::Nothing));
        assert_eq!(put(&mut list, &good(501, 10), 1), None);
        assert_eq!(put(&mut list, &good(501, 10), 1), None);
        assert_eq!(list, [Line { good: good(501, 10), amount: 2 }]);
        // One less, and never under one.
        assert_eq!(put(&mut list, &good(501, 10), -1), None);
        assert_eq!(put(&mut list, &good(501, 10), -1), None);
        assert_eq!(list[0].amount, 1);
        assert_eq!(put(&mut list, &good(501, 10), 150), Some(Said::TooMany));
        assert_eq!(list[0].amount, 100);
        for item in 502..509 {
            assert_eq!(put(&mut list, &good(item, 1), 1), None);
        }
        assert_eq!(put(&mut list, &good(600, 1), 1), Some(Said::Full));
        assert_eq!(list.len(), 8);
        assert_eq!(total(&list), 1007);
        taken(&mut list, 501);
        assert_eq!((list.len(), total(&list)), (7, 7));
        assert_eq!((Said::Full.line(), Said::TooMany.line(), Said::Nothing.line()), ((3312, Some(8)), (1704, Some(100)), (3664, None)));
    }

    #[test]
    fn the_list_shows_six_kinds_and_is_moved_within_them() {
        assert_eq!(scrolled(0, 1, 6), 0);
        assert_eq!(scrolled(0, 1, 8), 1);
        assert_eq!(scrolled(1, 1, 8), 2);
        assert_eq!(scrolled(2, 1, 8), 2);
        assert_eq!(scrolled(2, -1, 8), 1);
        assert_eq!(scrolled(0, -1, 8), 0);
        assert_eq!(scrolled(2, 0, 6), 0);
    }

    #[test]
    fn a_search_looks_through_the_tabs_but_not_the_sales_and_finds_an_item_once() {
        let mut lists = vec![Vec::new(); LISTS];
        lists[0] = vec![good(501, 10), good(502, 20)];
        lists[1] = vec![good(501, 10)];
        lists[8] = vec![good(503, 5)];
        lists[LIMITED] = vec![good(504, 5)];
        let name = |item: u32| ["Red Potion", "Orange Potion", "Yellow Potion", "White Potion"][item as usize - 501].to_owned();
        assert_eq!(found(&lists, "Potion", name), [good(501, 10), good(502, 20)]);
        assert_eq!(found(&lists, "Orange", name), [good(502, 20)]);
        // As it is written: capitals count.
        assert_eq!(found(&lists, "potion", name), []);
    }

    #[test]
    fn a_purchase_is_asked_about_when_it_can_be_paid() {
        let list = vec![Line { good: good(501, 10), amount: 3 }];
        assert_eq!(buying(&[], 100, 0), Buying::Nothing);
        assert_eq!(buying(&list, 100, 0), Buying::Asked(3771, vec![30]));
        assert_eq!(buying(&list, 100, 5), Buying::Asked(3772, vec![25, 5]));
        assert_eq!(buying(&list, 100, 31), Buying::Refused(1860));
        assert_eq!(buying(&list, 29, 0), Buying::Refused(1254));
        assert_eq!(buying(&list, 20, 10), Buying::Asked(3772, vec![20, 10]));
    }

    #[test]
    fn the_server_s_answers_have_their_lines() {
        assert_eq!(answer(0), Answer { done: true, line: None, boxed: false });
        assert_eq!(answer(8), Answer { done: true, line: Some(1814), boxed: false });
        assert_eq!(answer(2), Answer { done: false, line: Some(57), boxed: false });
        assert_eq!(answer(11).line, Some(1946));
        assert!(answer(303).boxed);
        assert_eq!(good(1, 1).bought_as(), 1);
        assert_eq!(Good { item: 1, price: 1, tab: 8 }.bought_as(), 0xFFFF);
        assert_eq!((tab_line(0), tab_line(7), tab_line(9)), (3290, 3297, 3656));
    }
}
