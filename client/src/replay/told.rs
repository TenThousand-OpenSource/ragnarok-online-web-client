//! What the client's windows were told, as the Windows client writes it among the recorded packets
//! (`CC_REPLAYPACKET`, 0x08ae) and does it again when the recording plays (`CGameMode::ProcessUiMessage`): a
//! button of a window (`UIWindow::RecordResponse`, kind 900: the window's number, the command's number, three
//! values), a thing let go over a window (`RecordDrop`, 901), what stood in a window's fields as its OK was
//! pressed (`RecordEditTexts`, 902), a tab or a box of the quest window (`RecordValues`, 913).
//!
//! Here is which press of this client's page (`crate::press`) is which of those records, both ways. A press of
//! a recorded window that the client has no record for is written as this project's own kind
//! (`sections::press_record`, 990); a record nothing here knows is passed by.

use crate::press::{choosing, exchange, friends, guild, inventory, party, pets, quests, shops, talk, Press};

use super::sections::{press_record, ui, CC_REPLAYPACKET};

/// The kinds of the records (`ReplayPacketRecorder.cpp`: the markers 0x038408AE, 0x038508AE, 0x038608AE and
/// 0x039108AE).
pub mod kind {
    pub const RESPONSE: u16 = 900;
    pub const DROP: u16 = 901;
    pub const TEXTS: u16 = 902;
    pub const VALUES: u16 = 913;
}

/// The client's numbers of the windows (`UI/Window/WindowId.h`).
pub mod window {
    pub const COUNT: u32 = 15;
    pub const SAY: u32 = 16;
    pub const CHOOSE: u32 = 17;
    pub const SHOP_LIST: u32 = 22;
    pub const PURCHASE: u32 = 23;
    pub const SELL: u32 = 24;
    pub const CHOOSE_SELL_BUY: u32 = 25;
    pub const EXCHANGE_ACCEPT: u32 = 32;
    pub const JOIN_PARTY: u32 = 35;
    pub const CHOOSE_WARP: u32 = 39;
    pub const NPC_NUMBER: u32 = 56;
    pub const JOIN_GUILD: u32 = 70;
    pub const IDENTIFY: u32 = 73;
    pub const COMPOSITION: u32 = 74;
    pub const MAKE_LIST: u32 = 79;
    pub const MAKE_PROCESS: u32 = 80;
    pub const PET_EGGS: u32 = 90;
    pub const MAKING_ARROWS: u32 = 94;
    pub const SPELLS: u32 = 99;
    pub const NPC_WORDS: u32 = 100;
    pub const REPAIR: u32 = 107;
    pub const JOIN_FRIEND: u32 = 109;
    pub const REFINE: u32 = 111;
    pub const QUEST_UI: u32 = 10008;
    pub const EXCHANGE: u32 = 10011;
}

/// The client's numbers of the commands (`UI/Window/UIEnums.h`), and of the two messages of its modes the say
/// window answers with (`ModeMessage.h`: `MM_REQ_NEXT_SCRIPT`, `MM_CLOSE_DIALOG`).
pub mod command {
    pub const OK: u32 = 184;
    pub const CANCEL: u32 = 185;
    pub const CLOSE: u32 = 201;
    pub const CHECK: u32 = 213;
    pub const NEXT: u32 = 218;
    pub const SELL: u32 = 223;
    pub const BUY: u32 = 224;
    /// What the say window's button stands at once "next" was pressed (`UM_DISABLENEXTBUTTON`): nothing.
    pub const NOTHING: u32 = 600;
    pub const REQ_NEXT_SCRIPT: u32 = 22;
    pub const CLOSE_DIALOG: u32 = 89;
    /// The second value of the count box's record (`UIItemDropCntWnd::SendMsg`: `RecordResponse(184, 100, …)`).
    pub const COUNTED: u32 = 100;
    /// `UM_TAB`, and the message the quest window's box is recorded under (`UIWindow::RecordWindowFlag`).
    pub const TAB: u32 = 22;
    pub const FLAG: u32 = 6;
}

/// What the count box is asked for (the states of `UIItemDropCntWnd` this client's shops and deals use).
pub mod asked {
    pub const BOUGHT: u32 = 1;
    pub const SOLD: u32 = 2;
    pub const NOT_BOUGHT: u32 = 3;
    pub const NOT_SOLD: u32 = 4;
    pub const DEALT: u32 = 5;
}

/// Where a thing was dragged from (`ModeDragState::source`): the inventory, a shop's list, the basket of what
/// is bought and that of what is sold.
pub mod from {
    pub const INVENTORY: i32 = 1;
    pub const SHOP_LIST: i32 = 3;
    pub const PURCHASE: i32 = 4;
    pub const SELL: i32 = 5;
}

/// The part of a talk with an NPC that is being answered: each a window of the client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Talk {
    /// What the NPC said, with its button (`UISayDialogWnd`).
    Says,
    /// Its menu, and the row picked (`UIChooseWnd`).
    Menu(usize),
    /// The field a number is typed into (`UINpcEditDialogWnd`), and the one for words
    /// (`UINpcTextEditDialogWnd`).
    Number,
    Words,
}

/// Which of a shop's windows stands open.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shop {
    Choose,
    Buy,
    Sell,
}

/// A list to pick one thing of: each a window of the client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum List {
    Warp,
    Arrows,
    Spells,
    Compound,
    Make,
    Forge,
    Repair,
    Refine,
    Identify,
    Eggs,
}

/// What stands open as a press is written or a record is read, as far as the records need it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Open {
    pub talk: Option<Talk>,
    /// What stands in the field of the window the press is of: the talk's, or the deal's zeny.
    pub typed: String,
    pub shop: Option<Shop>,
    /// The shop's box "Toggle Item Amount" is ticked.
    pub whole_stacks: bool,
    /// The list open (of `crate::press::choosing`), and the row picked in it.
    pub list: Option<(List, Option<usize>)>,
    /// The rows picked in the magnifier's list and in the list of eggs.
    pub identify: Option<usize>,
    pub egg: usize,
    /// What the boxes of the party and of the guild ask is an invitation.
    pub party_invited: bool,
    pub guild_invited: bool,
    /// The quest window's box: the list on the screen shows.
    pub quests_shown: bool,
}

/// A thing let go over a window (`UIWindow::RecordDrop`), as far as this client has it: the window, where it
/// was dragged from, the thing's place where it is carried (`ModeDragState::id`: -1 in a shop's list of what
/// is bought), the item's number (the client's `name`, written as text), how many, and the item's kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dropped {
    pub onto: u32,
    pub from: i32,
    pub id: u32,
    pub item: u32,
    pub amount: i32,
    pub kind: u32,
}

/// What a record says was done.
#[derive(Debug, Clone, PartialEq)]
pub enum Deed {
    /// Presses of this client's page, to be done in their order.
    Presses(Vec<Press>),
    /// The texts of a window's fields, as they stood (`UIWindow::RestoreEditTexts`).
    Typed { window: u32, texts: Vec<String> },
    /// The count box's OK: what was asked (`asked`), and how many were typed.
    Counted { asked: u32, amount: u32 },
    Dropped(Dropped),
}

/// `UIWindow::RecordResponse`: 26 bytes, the window, the command and three values after the six.
pub fn response(window: u32, command: u32, values: [u32; 3]) -> Vec<u8> {
    let numbers = [window, command, values[0], values[1], values[2]];
    ui(kind::RESPONSE, &numbers.iter().flat_map(|number| number.to_le_bytes()).collect::<Vec<u8>>())
}

/// `UIWindow::RecordEditTexts`: the window, how many fields, and each field's length and text.
pub fn texts(window: u32, texts: &[&str]) -> Vec<u8> {
    let mut body = [window.to_le_bytes(), (texts.len() as u32).to_le_bytes()].concat();
    for text in texts {
        // (The client's record is a block of 0x1000 bytes: a text is cut to what is sure to fit.)
        let text = &text.as_bytes()[..text.len().min(1000)];
        body.extend((text.len() as u32).to_le_bytes());
        body.extend(text);
    }
    ui(kind::TEXTS, &body)
}

/// `UIWindow::RecordValues`: 30 bytes, the window, four values and the message after the six.
pub fn values(window: u32, message: u32, values: [u32; 4]) -> Vec<u8> {
    let numbers = [window, values[0], values[1], values[2], values[3], message];
    ui(kind::VALUES, &numbers.iter().flat_map(|number| number.to_le_bytes()).collect::<Vec<u8>>())
}

/// `UIWindow::RecordDrop`: the client's `DropRecord` of 0x191 bytes. What this client keeps nothing of is
/// nought: where on the window, the prices, the slot, the refining, the picture's name, the cards, the options.
pub fn drop_record(dropped: &Dropped) -> Vec<u8> {
    let mut body = vec![0u8; 0x191 - 6];
    let mut put = |at: usize, bytes: &[u8]| body[at - 6..at - 6 + bytes.len()].copy_from_slice(bytes);
    put(0x06, &dropped.onto.to_le_bytes());
    put(0x12, &dropped.from.to_le_bytes());
    put(0x16, &dropped.id.to_le_bytes());
    put(0x22, &dropped.amount.to_le_bytes());
    put(0x2a, &dropped.kind.to_le_bytes());
    // (An item drawn in a list is one that is identified.)
    put(0x32, &[1]);
    put(0x97, dropped.item.to_string().as_bytes());
    ui(kind::DROP, &body)
}

/// The count box's OK (`UIItemDropCntWnd`): its field as the button records it, then the command with the
/// number typed and what was asked.
pub fn counted(asked: u32, typed: &str) -> Vec<Vec<u8>> {
    // (`atoi`: the digits a text begins with.)
    let digits: String = typed.trim().chars().take_while(char::is_ascii_digit).collect();
    let amount = digits.parse::<u32>().unwrap_or(0);
    vec![texts(window::COUNT, &[typed]), response(window::COUNT, command::OK, [command::COUNTED, amount, asked])]
}

/// The window of a shop a row is let go over: the basket of what is bought or sold, or the list.
pub fn shop_window(basket: bool, selling: bool) -> u32 {
    match (basket, selling) {
        (true, false) => window::PURCHASE,
        (true, true) => window::SELL,
        (false, _) => window::SHOP_LIST,
    }
}

/// What the count box is asked for in a shop (`UIItemPurchaseWnd::DragAndDrop`, `UIItemSellWnd::DragAndDrop`,
/// `UIItemShopWnd::DragAndDrop`): how many go into the basket, or back out of it.
pub fn shop_asked(into: bool, buying: bool) -> u32 {
    match (into, buying) {
        (true, true) => asked::BOUGHT,
        (true, false) => asked::SOLD,
        (false, true) => asked::NOT_BOUGHT,
        (false, false) => asked::NOT_SOLD,
    }
}

/// The window of a list, and which of the three values holds the row picked in it: the first in a window
/// whose list is a `UIListBox` (its double click sends the row there; the button leaves 0), the second in
/// `UIItemCompositionWnd`, `UIItemIdentifyWnd` and `UIPetEggListWnd` (`m_selected`), the third in
/// `UIMakingArrowListWnd`, `UIMakeTargetListWnd`, `UIItemRepairWnd` and `UIWeaponRefineWnd`. `UISpellListWnd`
/// records no row: this client writes it into the first value, which that window does not read. None for the
/// forging, which has no rows (`UIMakeTargetProcessWnd`).
fn of_list(list: List) -> (u32, Option<usize>) {
    match list {
        List::Warp => (window::CHOOSE_WARP, Some(0)),
        List::Spells => (window::SPELLS, Some(0)),
        List::Compound => (window::COMPOSITION, Some(1)),
        List::Identify => (window::IDENTIFY, Some(1)),
        List::Eggs => (window::PET_EGGS, Some(1)),
        List::Arrows => (window::MAKING_ARROWS, Some(2)),
        List::Make => (window::MAKE_LIST, Some(2)),
        List::Repair => (window::REPAIR, Some(2)),
        List::Refine => (window::REFINE, Some(2)),
        List::Forge => (window::MAKE_PROCESS, None),
    }
}

const LISTS: [List; 10] = [
    List::Warp,
    List::Spells,
    List::Compound,
    List::Identify,
    List::Eggs,
    List::Arrows,
    List::Make,
    List::Repair,
    List::Refine,
    List::Forge,
];

/// A list's OK or Cancel: the row picked where its window records it, -1 for none.
fn list_answered(list: List, picked: Option<usize>, ok: bool) -> Vec<u8> {
    let (window, place) = of_list(list);
    let mut values = [0; 3];
    if let Some(place) = place {
        // (A `UIListBox` window's button sends 0, not the row: with no row picked it is 0 there.)
        values[place] = picked.map_or(if place == 0 { 0 } else { u32::MAX }, |row| row as u32);
    }
    response(window, if ok { command::OK } else { command::CANCEL }, values)
}

/// The records a press is written as, in their order: the client's own where it has one, else this project's
/// kind for a press of a window the server opens (`playing::press_recorded`), else none.
pub fn records_of(press: &Press, open: &Open) -> Vec<Vec<u8>> {
    let own = || {
        let (what, with) = press.name();
        if super::playing::press_recorded(press) { vec![press_record(what, &with)] } else { Vec::new() }
    };
    let answer = |window: u32, yes: bool| vec![response(window, if yes { command::OK } else { command::CANCEL }, [0; 3])];
    match press {
        Press::Talk(press) => talk_records(press, open),
        Press::Shops(press) => shop_records(press, open),
        // A row only picked is not recorded (`UIListBox::OnLBtnDown`): the OK carries it.
        Press::Choosing(choosing::Press::Pick(_)) | Press::Inventory(inventory::Press::IdentifyPick(_)) => Vec::new(),
        Press::Pets(pets::Press::EggPick(_)) => Vec::new(),
        Press::Choosing(choosing::Press::Ok | choosing::Press::Cancel) => match open.list {
            Some((list, picked)) => vec![list_answered(list, picked, *press == Press::Choosing(choosing::Press::Ok))],
            None => Vec::new(),
        },
        Press::Inventory(inventory::Press::IdentifyOk) => vec![list_answered(List::Identify, open.identify, true)],
        Press::Inventory(inventory::Press::IdentifyCancel) => vec![list_answered(List::Identify, open.identify, false)],
        Press::Pets(pets::Press::EggOk) => vec![list_answered(List::Eggs, Some(open.egg), true)],
        Press::Pets(pets::Press::EggCancel) => vec![list_answered(List::Eggs, Some(open.egg), false)],
        // `UIExchangeAcceptWnd`, `UIJoinPartyAcceptWnd`, `UIJoinGuildAcceptWnd`, `UIJoinFriendAcceptWnd`.
        Press::Exchange(exchange::Press::Accept) => answer(window::EXCHANGE_ACCEPT, true),
        Press::Exchange(exchange::Press::Refuse) => answer(window::EXCHANGE_ACCEPT, false),
        // The deal's window records none of its buttons (`CUIExchangeUI`), but what stands in its field goes
        // into the recording with its OK (the window is one of those `RecordEditTexts` writes).
        Press::Exchange(exchange::Press::Lock) => [vec![texts(window::EXCHANGE, &[&open.typed])], own()].concat(),
        Press::Party(party::Press::Yes | party::Press::No) if open.party_invited => {
            answer(window::JOIN_PARTY, *press == Press::Party(party::Press::Yes))
        }
        Press::Guild(guild::Press::Yes | guild::Press::No) if open.guild_invited => {
            answer(window::JOIN_GUILD, *press == Press::Guild(guild::Press::Yes))
        }
        Press::Friends(friends::Press::Yes) => answer(window::JOIN_FRIEND, true),
        Press::Friends(friends::Press::No) => answer(window::JOIN_FRIEND, false),
        // `CUIRenewQuestUI`: a tab (`OnProcessTabClick` and the three others: 1, 2, 4, 8 down the side), and
        // the box that shows the list on the screen, as it stands after the press.
        Press::Quests(quests::Press::Tab(tab)) if *tab < 4 => vec![values(window::QUEST_UI, command::TAB, [1 << tab, 0, 0, 0])],
        Press::Quests(quests::Press::Display) => {
            vec![values(window::QUEST_UI, command::FLAG, [command::CHECK, u32::from(!open.quests_shown), 0, 0])]
        }
        _ => own(),
    }
}

/// A talk's presses (`UISayDialogWnd::SendMsg`, `UIChooseWnd::SendMsg`, `UINpcEditDialogWnd::SendMsg`,
/// `UINpcTextEditDialogWnd::SendMsg`).
fn talk_records(press: &talk::Press, open: &Open) -> Vec<Vec<u8>> {
    use talk::Press;
    match (press, open.talk) {
        // Both buttons of the say window carry the number 218 (`UM_ENABLENEXTBUTTON`, `UM_ENABLECLOSEBUTTON`):
        // the third value, the message the window answers with, tells them apart.
        (Press::Next, _) => vec![response(window::SAY, command::NEXT, [0, 0, command::REQ_NEXT_SCRIPT])],
        (Press::Close, _) => vec![response(window::SAY, command::NEXT, [0, 0, command::CLOSE_DIALOG])],
        // The menu's OK with the row, as a double click on the row sends it.
        (Press::Ok, Some(Talk::Menu(row))) => vec![response(window::CHOOSE, command::OK, [row as u32, 0, 0])],
        // A field's OK: the field as the button records it (`UIButton::OnLBtnUp`), then the command.
        (Press::Ok, Some(Talk::Number)) => {
            vec![texts(window::NPC_NUMBER, &[&open.typed]), response(window::NPC_NUMBER, command::OK, [0; 3])]
        }
        (Press::Ok, Some(Talk::Words)) => {
            vec![texts(window::NPC_WORDS, &[&open.typed]), response(window::NPC_WORDS, command::OK, [0; 3])]
        }
        (Press::Cancel, _) => vec![response(window::CHOOSE, command::CANCEL, [0; 3])],
        // A row only picked, a link to an item, an OK with nothing to answer: nothing is recorded.
        (Press::Pick(_) | Press::Item(_) | Press::Ok, _) => Vec::new(),
    }
}

/// A shop's presses (`UIChooseSellBuyWnd::SendMsg`, `UIItemPurchaseWnd::SendMsg`, `UIItemSellWnd::SendMsg`,
/// and the box of `UIItemShopWnd::SendMsg`, recorded with what it becomes).
fn shop_records(press: &shops::Press, open: &Open) -> Vec<Vec<u8>> {
    use shops::Press;
    let basket = |shop| if shop == Shop::Sell { window::SELL } else { window::PURCHASE };
    match (press, open.shop) {
        (Press::Buy, Some(Shop::Choose)) => vec![response(window::CHOOSE_SELL_BUY, command::BUY, [0; 3])],
        (Press::Sell, Some(Shop::Choose)) => vec![response(window::CHOOSE_SELL_BUY, command::SELL, [0; 3])],
        (Press::Cancel, Some(Shop::Choose)) => vec![response(window::CHOOSE_SELL_BUY, command::CANCEL, [0; 3])],
        (Press::Cancel, Some(shop)) => vec![response(basket(shop), command::CANCEL, [0; 3])],
        (Press::Ok, Some(shop)) if shop != Shop::Choose => vec![response(basket(shop), command::OK, [0; 3])],
        (Press::WholeStacks, Some(Shop::Sell)) => {
            vec![response(window::SHOP_LIST, command::CHECK, [u32::from(!open.whole_stacks), 0, 0])]
        }
        // (The count box's OK is a form's: `counted`.)
        _ => Vec::new(),
    }
}

mod read;
pub use read::deed;

#[cfg(test)]
mod tests;
