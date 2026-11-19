//! A deal between two players: asking for one, answering, putting things and zeny in, closing one's side, the
//! trade and calling it off (the client's `CUIExchangeUI`, `UIExchangeAcceptWnd` and the `MM_*EXCHANGE_ITEM`
//! arms of `CGameMode::SendMsg`).

use super::*;
use crate::net::exchange;

impl App {
    /// A press of this system (`crate::press::exchange`).
    pub(super) fn exchange_pressed(&mut self, press: crate::press::exchange::Press) {
        use crate::press::exchange::Press;
        match press {
            Press::Accept | Press::Refuse => {
                if self.exchange.asked.take().is_some() {
                    self.send_on_map(&exchange::reply(if press == Press::Accept { exchange::answer::ACCEPT } else { exchange::answer::CANCEL }));
                }
                self.redraw(&[&wnd::EXCHANGE_ASKS]);
            }
            Press::Lock => self.exchange_lock(),
            // The trade, once both sides are closed (`Conclude`): the button is pressed once.
            Press::Trade => {
                if self.exchange.closed == (true, true) && !self.exchange.trading {
                    self.exchange.trading = true;
                    // With the box "Screenshot Trade" ticked a picture of the screen is made first, named for
                    // the partner (`CUIExchangeUI`'s `Conclude`: `MM_SCREENSHOT` with line 1832 of the message
                    // table, "[Trade_%s]", before `MM_EXEC_EXCHANGE_ITEM`): the trade is asked for once the
                    // picture is made (`screenshot_saved`), so the window is still in it.
                    if self.exchange.screenshot {
                        self.exchange.picturing = true;
                        self.screenshot(&Self::words(1832, "[Trade_%s]").replace("%s", &self.exchange.partner.clone()));
                    } else {
                        self.send_on_map(&exchange::trade());
                    }
                    self.redraw(&[&wnd::EXCHANGE]);
                }
            }
            Press::Cancel => {
                if self.exchange.open && !self.exchange.cancelling {
                    self.exchange.cancelling = true;
                    self.send_on_map(&exchange::cancel());
                    self.redraw(&[&wnd::EXCHANGE]);
                }
            }
            // The box "Screenshot Trade": a picture of the screen is made as Trade is pressed.
            Press::Screenshot => {
                self.exchange.screenshot = !self.exchange.screenshot;
                self.redraw(&[&wnd::EXCHANGE]);
            }
            // The box that asks how many is a form, and its OK sends it: the press itself does nothing.
            Press::CountOk => {}
        }
    }

    /// The window's OK (`SubmitZeny`): the zeny typed goes into the deal and the player's side is closed. What
    /// is typed must be a number; more than the character has is all it has.
    fn exchange_lock(&mut self) {
        if !self.exchange.open || self.exchange.locking || self.exchange.closed.0 {
            return;
        }
        let has = self.hero.params.get(&packets::parameter::ZENY).copied().unwrap_or(0).clamp(0, i64::from(u32::MAX)) as u32;
        match exchange::zeny_typed(&field("exchange-zeny"), has) {
            Some(zeny) => {
                (self.exchange.locking, self.exchange.adding) = (true, Some((0, zeny)));
                self.send_on_map(&exchange::add(0, zeny));
                self.send_on_map(&exchange::conclude());
                self.redraw(&[&wnd::EXCHANGE]);
            }
            None => {
                self.message = Some(Self::words(602, "Only the numbers (0~9) are available."));
                self.render();
            }
        }
    }

    /// That player is asked for a deal (`MENUID_DEAL`, `MM_REQ_EXCHANGE_ITEM`): not while an NPC is being talked
    /// to, which the chat says in white.
    pub(in crate::web::app) fn exchange_request(&mut self, who: u32) {
        if self.dialog.is_some() {
            return self.chat_line(Self::words(1923, "Please finish the task in progress (NPC conversation, manufacturing ...) and try again."), WHITE);
        }
        if let Some(name) = world::player_name(who) {
            self.exchange.partner = name;
            self.send_on_map(&exchange::request(who));
        }
    }

    /// What was typed into the chat, when it is this system's: `/deal name` asks the player of that name in
    /// sight for a deal (the client's `TT_REQ_EXCHANGE_ITEM`). `false` for anything else.
    pub(in crate::web::app) fn exchange_typed(&mut self, words: &str) -> bool {
        let Some(name) = words.strip_prefix("/deal ").map(str::trim).filter(|name| !name.is_empty()) else { return false };
        if let Some(who) = world::player_named(name) {
            self.exchange_request(who);
        }
        true
    }

    /// A thing was dragged out of its window and let go over the deal's window (`DragAndDrop`): an item of the
    /// inventory goes in, one of it at once, more after its count is asked for; nothing goes into a side that
    /// is closed.
    pub(in crate::web::app) fn exchange_offered(&mut self, dragged: Drag) {
        let Drag::Item(id) = dragged else { return };
        if !self.exchange.open || self.exchange.closed.0 || self.exchange.locking {
            return;
        }
        let mine = &self.exchange.mine;
        let Some(item) = self.bag.items.iter().find(|item| item.id == id && item.worn == 0 && !mine.iter().any(|put| put.index == item.index)) else { return };
        let (index, amount) = (item.index, item.amount);
        if amount <= 1 {
            self.exchange_put(index, 1);
        } else {
            self.exchange.counting = Some((index, amount));
            self.redraw(&[&wnd::EXCHANGE_COUNT]);
        }
    }

    /// The count asked for was given. As the window that asks how many to drop: a number that is none, or more
    /// than there is, puts nothing in.
    pub(in crate::web::app) fn exchange_counted(&mut self) {
        if let Some((index, most)) = self.exchange.counting.take() {
            if let Some(count) = Self::counted(&wnd::EXCHANGE_COUNT, most) {
                self.exchange_put(index, u32::from(count));
            }
        }
        self.redraw(&[&wnd::EXCHANGE_COUNT]);
    }

    /// So many of the thing carried there go into the deal (`MM_ADD_EXCHANGE_ITEM`); one thing at a time, the
    /// next once the server has answered for the last.
    fn exchange_put(&mut self, index: u16, amount: u32) {
        if self.exchange.adding.is_none() {
            self.exchange.adding = Some((index, amount));
            self.send_on_map(&exchange::add(index, amount));
        }
    }
}
