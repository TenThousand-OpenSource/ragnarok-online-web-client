//! The lists a skill or a card opens to pick one thing of: picking a row, OK and Cancel, and what each list
//! then sends (the `UM_COMMAND` arms of `UIChooseWarpWnd`, `UIMakingArrowListWnd`, `UISpellListWnd`,
//! `UIItemCompositionWnd`, `UIMakeTargetListWnd`, `UIMakeTargetProcessWnd`, `UIItemRepairWnd` and
//! `UIWeaponRefineWnd`); a double click on a card, which asks for its list; things dragged into a forging.

use super::*;
use crate::net::choosing;

impl App {
    /// A press of this system (`crate::press::choosing`).
    pub(super) fn choosing_pressed(&mut self, press: crate::press::choosing::Press) {
        use crate::press::choosing::Press;
        match press {
            Press::Pick(row) => {
                if self.choosing.open.as_ref().is_some_and(|choice| row < choice.rows()) {
                    self.choosing.picked = Some(row);
                    self.redraw_choice();
                }
            }
            Press::Ok => self.choice_made(),
            Press::Cancel => self.choice_dropped(),
            // What was put into a slot of the forging comes out (the client has it dragged out:
            // `MM_BEGIN_DRAG_FROM_METALPROCESSWND`).
            Press::Material(slot) => {
                if let Some(Choice::Forge(_, put)) = &mut self.choosing.open {
                    if slot < put.len() {
                        put.remove(slot);
                        self.redraw_choice();
                    }
                }
            }
        }
    }

    fn redraw_choice(&self) {
        self.redraw(&[&wnd::CHOOSE_PLACE, &wnd::CHOOSE_PICTURED, &wnd::CHOOSE_LINE]);
    }

    /// A double click on a row: it is picked and confirmed (`UIItemIdentifyWnd::OnLBtnDblClk`; a list box
    /// sends its window OK).
    pub(in crate::web::app) fn choice_used(&mut self, row: usize) {
        self.choosing_pressed(crate::press::choosing::Press::Pick(row));
        self.choice_made();
    }

    /// OK: the picked row is sent as its list sends it, and the list closes. With no row picked nothing
    /// happens, but an empty list of lines closes as if cancelled, as the client's does.
    fn choice_made(&mut self) {
        let Some(choice) = self.choosing.open.take() else { return };
        let Some(row) = self.choosing.picked.filter(|row| *row < choice.rows()) else {
            // A thing being forged has no rows: OK asks for it with what was put in.
            return match choice {
                Choice::Forge(make, put) => {
                    self.send_on_map(&choosing::make(make, crate::choosing::put_in(&put)));
                    self.redraw_choice();
                }
                Choice::Repair(_) | Choice::Refine(_) | Choice::Make(_) if choice.rows() == 0 => {
                    self.choosing.open = Some(choice);
                    self.choice_dropped();
                }
                other => self.choosing.open = Some(other),
            };
        };
        match choice {
            // The client's last row is its own "cancel", sent as a place is.
            Choice::Warp(skill, places) => self.send_on_map(&choosing::warp(skill, &places[row])),
            Choice::Arrows(items) => self.send_on_map(&choosing::make_arrows(items[row])),
            Choice::Mix(kind, items) => self.send_on_map(&choosing::cook(kind, items[row])),
            Choice::Spells(skills) => self.send_on_map(&choosing::pick_spell(skills[row])),
            Choice::Compound(card, places) => self.send_on_map(&choosing::compound(card, places[row])),
            // The thing picked is shown with what it needs, in the same place (`WID_MAKETARGETPROCESSWND`).
            Choice::Make(items) => {
                self.choosing.open = Some(Choice::Forge(items[row], Vec::new()));
                self.choosing.picked = None;
            }
            Choice::Repair(things) => self.send_on_map(&choosing::repair(things.get(row))),
            Choice::Refine(things) => self.send_on_map(&choosing::refine(Some(things[row].index))),
            Choice::Forge(..) => {}
        }
        self.redraw_choice();
    }

    /// Cancel: the server is told that nothing was picked, each list in its own way, and the list closes.
    fn choice_dropped(&mut self) {
        let Some(choice) = self.choosing.open.take() else { return };
        self.choosing.picked = None;
        match choice {
            Choice::Warp(skill, _) => self.send_on_map(&choosing::warp(skill, choosing::CANCEL)),
            // -1 for the item (`MM_ARROW_FROM_LIST`, `MM_ITEM_FROM_LIST`).
            Choice::Arrows(_) => self.send_on_map(&choosing::make_arrows(u32::MAX)),
            Choice::Mix(kind, _) => self.send_on_map(&choosing::cook(kind, u32::MAX)),
            Choice::Spells(_) => self.send_on_map(&choosing::pick_spell(0)),
            // -1 for both places (`MM_REQ_ITEMCOMPOSITION`).
            Choice::Compound(..) => self.send_on_map(&choosing::compound(u16::MAX, u16::MAX)),
            Choice::Make(_) | Choice::Forge(..) => self.send_on_map(&choosing::make(0, [0; 3])),
            Choice::Repair(_) => self.send_on_map(&choosing::repair(None)),
            Choice::Refine(_) => self.send_on_map(&choosing::refine(None)),
        }
        self.redraw_choice();
    }

    /// A double click on a thing carried that is a card (the server's kind 6) asks what it can be set into,
    /// and the card is remembered for the answer (`cardItemIndex`). False for anything else.
    pub(in crate::web::app) fn card_used(&mut self, index: u16) -> bool {
        if !self.bag.items.iter().any(|item| item.index == index && item.kind == 6) {
            return false;
        }
        self.choosing.card = index;
        self.send_on_map(&choosing::ask_compound(index));
        true
    }

    /// A thing dragged out of the inventory and let go over the window of a forging goes into one of its
    /// three slots, if it is one that may (`crate::choosing::goes_in`). The client takes one of it out of the
    /// inventory's window meanwhile; here the inventory shows it until the server takes it.
    pub(in crate::web::app) fn forge_dropped(&mut self, dragged: Drag) {
        let Drag::Item(item) = dragged else { return };
        if let Some(Choice::Forge(make, put)) = &mut self.choosing.open {
            // No more of a thing goes in than is carried.
            let carried = self.bag.items
                .iter()
                .filter(|carried| carried.id == item && carried.worn == 0)
                .map(|carried| usize::from(carried.amount))
                .sum::<usize>();
            if crate::choosing::goes_in(*make, put, item) && put.iter().filter(|put| **put == item).count() < carried {
                put.push(item);
                self.redraw_choice();
            }
        }
    }
}
