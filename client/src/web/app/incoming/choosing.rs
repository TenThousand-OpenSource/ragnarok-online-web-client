//! What the server says of the lists a skill or a card opens to pick one thing of, and of how each choice
//! ended: the list is kept and its window opened; a result is said in the chat as the client's handler says
//! it and shown on the character (the world's `None`: the player's own). It overhears every packet to ask the server for the names of the makers of
//! things that a window looked up in vain (`data::makers_wanted`, the client's `CPlayerNameDictionary`).

use super::*;
use crate::net::choosing;
use crate::choosing::effect;

/// The colours of these lines only (the client's `0xBBGGRR` turned): a refining that failed (`0xCDCD00`),
/// one that could not be tried (`0xC8C8FF`).
const REFINE_FAILED: &str = "#00cdcd";
const REFINE_REFUSED: &str = "#ffc8c8";

impl App {
    /// Asks the server for the names of the makers of things that a window looked up in vain
    /// (`data::makers_wanted`): done as any packet arrives, whosever it is.
    fn ask_names_wanted(&mut self) {
        for maker in data::makers_wanted() {
            self.send_on_map(&crate::net::actors::ask_name(maker));
        }
    }

    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn choosing_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        self.ask_names_wanted();
        match id {
            // The client's own last row "cancel" follows the places (`case ID_ZC_WARPLIST`).
            choosing::ZC_WARPLIST => {
                if let Some((skill, mut places)) = choosing::warps(packet) {
                    places.push(choosing::CANCEL.to_owned());
                    self.choose(Choice::Warp(skill, places), Some(0));
                }
            }
            choosing::ZC_ACK_REMEMBER_WARPPOINT => self.warp_remembered(packet),
            choosing::ZC_MAKINGARROW_LIST => self.choose_items(Choice::Arrows(choosing::arrow_sources(packet))),
            choosing::ZC_MAKINGITEM_LIST => {
                if let Some((kind, items)) = choosing::cookable(packet) {
                    self.choose_items(Choice::Mix(kind, items));
                }
            }
            // A list with no spell in it opens nothing (`case ID_ZC_AUTOSPELLLIST`).
            choosing::ZC_AUTOSPELLLIST => {
                let spells = choosing::spells(packet);
                if !spells.is_empty() {
                    self.choose(Choice::Spells(spells), None);
                }
            }
            // An empty list is answered with the client's box "You can't create items yet." (line 424).
            choosing::ZC_MAKABLEITEMLIST => match choosing::makable(packet) {
                items if items.is_empty() => self.nothing_to_choose(),
                items => self.choose_items(Choice::Make(items)),
            },
            choosing::ZC_REPAIRITEMLIST => match choosing::repairable(packet) {
                things if things.is_empty() => self.nothing_to_choose(),
                things => self.choose_items(Choice::Repair(things)),
            },
            choosing::ZC_NOTIFY_WEAPONITEMLIST => match choosing::refinable(packet) {
                things if things.is_empty() => self.nothing_to_choose(),
                things => self.choose_items(Choice::Refine(things)),
            },
            // The card is the one that was double-clicked (`cardItemIndex`); things no longer carried are
            // left out, as the client leaves them.
            choosing::ZC_ITEMCOMPOSITION_LIST => {
                let places: Vec<u16> = choosing::compoundable(packet)
                    .into_iter()
                    .filter(|place| self.bag.items.iter().any(|item| item.index == *place))
                    .collect();
                self.choose(Choice::Compound(self.choosing.card, places), None);
            }
            choosing::ZC_ACK_ITEMCOMPOSITION => self.card_set(packet),
            // The client shows the effect and says nothing (`CGameMode::ReceiveItemEffectResult` makes the
            // line "Successfully created %s." and does not send it to the chat).
            choosing::ZC_ACK_REQMAKINGITEM => {
                if let Some(shown) = choosing::made(packet).and_then(|(result, _)| crate::choosing::made_effect(result)) {
                    world::burst(shown, None, None, 0.0);
                }
            }
            choosing::ZC_ACK_ITEMREPAIR => match choosing::repaired(packet) {
                Some(true) => self.say_filled(said::GENERAL, 813, &[], CYAN),
                Some(false) => self.say_filled(said::GENERAL, 814, &[], RED),
                None => {}
            },
            choosing::ZC_ACK_WEAPONREFINE => {
                if let Some((result, item)) = choosing::refined(packet) {
                    let name = data::item_title(&crate::item_names::Named { id: item, identified: true, ..Default::default() }, false);
                    let said = [(911, CYAN), (912, REFINE_FAILED), (913, REFINE_REFUSED), (914, REFINE_REFUSED)];
                    if let Some((line, colour)) = said.get(result as usize) {
                        self.say_filled(said::GENERAL, *line, &[&name], colour);
                    }
                }
            }
            choosing::ZC_ACK_ITEMREFINING => self.item_refined(packet),
            crate::net::actors::ZC_ACK_REQNAME_BYGID => {
                if let Some((character, Some(name))) = crate::net::actors::character_name(packet) {
                    data::maker_named(character, name);
                    self.refresh_items();
                }
            }
            _ => return false,
        }
        true
    }

    /// Opens a list, with the row picked in it at first: a list box of the client's has its first line
    /// picked (`UIListBox`), a list of rows with pictures none (`UIItemIdentifyWnd`).
    fn choose(&mut self, choice: Choice, picked: Option<usize>) {
        self.choosing.open = Some(choice);
        self.choosing.picked = picked;
        self.redraw(&[&wnd::CHOOSE_PLACE, &wnd::CHOOSE_PICTURED, &wnd::CHOOSE_LINE]);
    }

    /// Opens a list of items: their names are asked of the gateway, and the list is drawn anew when they
    /// are there.
    fn choose_items(&mut self, choice: Choice) {
        let items: Vec<u32> = match &choice {
            Choice::Arrows(items) | Choice::Mix(_, items) | Choice::Make(items) => items.clone(),
            Choice::Repair(things) | Choice::Refine(things) => things.iter().map(|thing| thing.id).collect(),
            _ => Vec::new(),
        };
        data::want_items(items.into_iter(), || dispatch(Action::ItemsKnown));
        let picked = (choice.shape() == Shape::Lines).then_some(0);
        self.choose(choice, picked);
    }

    /// The client's box for a list that came empty (`MSI_CANT_MAKE_ITEM`).
    fn nothing_to_choose(&mut self) {
        self.message = Some(Self::words(424, "You can't create items yet."));
        self.render();
    }

    /// ZC_ACK_REMEMBER_WARPPOINT: what `/memo` came to (`case ID_ZC_ACK_REMEMBER_WARPPOINT`): the place is
    /// kept (line 217, in the client's 0xFFFF00), the skill's level is too low (214), the skill is not
    /// known (216), the place cannot be kept (2535), these in red.
    fn warp_remembered(&mut self, packet: &[u8]) {
        match choosing::remembered(packet) {
            Some(0) => self.say_filled(said::GENERAL, 217, &[], CYAN),
            Some(1) => self.say_filled(said::GENERAL, 214, &[], RED),
            Some(2) => self.say_filled(said::GENERAL, 216, &[], RED),
            Some(3) => self.say_filled(said::GENERAL, 2535, &[], RED),
            _ => {}
        }
    }

    /// ZC_ACK_ITEMCOMPOSITION (`CGameMode::ReceiveCardCompositionResult`): the card goes into the first
    /// empty slot of the thing and one of it leaves the bag, both done here as the client does them itself;
    /// the chat says which way it went (lines 493 and 494).
    fn card_set(&mut self, packet: &[u8]) {
        let Some((equipment, card, done)) = choosing::compounded(packet) else { return };
        if !done {
            return self.say_filled(said::GENERAL, 494, &[], RED);
        }
        self.say_filled(said::GENERAL, 493, &[], CYAN);
        let Some(id) = self.bag.at(card).map(|item| item.id) else { return };
        let Some(thing) = self.bag.at_mut(equipment) else { return };
        crate::choosing::set_card(&mut thing.cards, id);
        if let Some(item) = self.bag.at_mut(card) {
            item.amount = item.amount.saturating_sub(1);
        }
        self.bag.items.retain(|item| item.amount > 0);
        self.refresh_items();
    }

    /// ZC_ACK_ITEMREFINING (`CGameMode::ReceiveRefineResult`, with no refining window open): the thing is
    /// refined as far as the server says, at once; the effect of a refining that went well or did not shows
    /// on the character, and the chat says it (lines 498, 499 and 1537, all in the client's 0xFFFF00).
    fn item_refined(&mut self, packet: &[u8]) {
        let Some((result, index, refine)) = choosing::item_refined(packet) else { return };
        if let Some(item) = self.bag.at_mut(index) {
            item.refine = refine;
        }
        let said = [(effect::REFINE_OK, 498), (effect::REFINE_FAIL, 499), (effect::REFINE_FAIL, 1537)];
        if let Some((shown, line)) = said.get(usize::from(result)) {
            world::burst(*shown, None, None, 0.0);
            self.say_filled(said::GENERAL, *line, &[], CYAN);
        }
        self.refresh_items();
    }
}
