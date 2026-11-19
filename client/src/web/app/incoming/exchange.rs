//! A deal between two players: what the map server tells of it. What the chat says, and in which colour, is
//! the client's (`ReceiveExchangeRequestAck2`, `ReceiveExchangeItemAddAck` and the `ID_ZC_*_EXCHANGE_ITEM` arms
//! of its packet handler).

use super::*;
use crate::net::exchange;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn exchange_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Someone asks for a deal. With the client's `/notrade` on it is turned down at once; otherwise
            // the question is asked in a box of its own (`UIExchangeAcceptWnd`).
            exchange::ZC_REQ_EXCHANGE_ITEM => {
                if self.option("TT_NOTRADE_ON_OFF") {
                    self.send_on_map(&exchange::reply(exchange::answer::CANCEL));
                } else if let Some((name, level)) = exchange::asked(packet) {
                    (self.exchange.partner, self.exchange.asked) = (name.clone(), Some((name, level)));
                    self.redraw(&[&wnd::EXCHANGE_ASKS]);
                }
            }
            exchange::ZC_ACK_EXCHANGE_ITEM => self.exchange_answered(packet),
            // The other side put something in: zeny (all of it so far), or a thing.
            exchange::ZC_ADD_EXCHANGE_ITEM => {
                match exchange::put(packet) {
                    Some(exchange::Put::Zeny(zeny)) => self.exchange.zeny.1 = zeny,
                    Some(exchange::Put::Item { id, kind, amount, identified, refine, cards }) => {
                        self.exchange.theirs.push(Item {
                            index: 0,
                            id,
                            kind,
                            amount: amount.min(u32::from(u16::MAX)) as u16,
                            wearable: 0,
                            worn: 0,
                            refine,
                            identified,
                            cards
                        });
                        data::want_items(std::iter::once(id), || dispatch(Action::ItemsKnown));
                    }
                    None => {}
                }
                self.redraw(&[&wnd::EXCHANGE]);
            }
            exchange::ZC_ACK_ADD_EXCHANGE_ITEM => self.exchange_added(packet),
            // A side is closed: nothing more goes into it, and with both closed the trade can be made.
            exchange::ZC_CONCLUDE_EXCHANGE_ITEM => {
                match exchange::concluded(packet) {
                    Some(0) => self.exchange.closed.0 = true,
                    Some(1) => self.exchange.closed.1 = true,
                    _ => {}
                }
                self.redraw(&[&wnd::EXCHANGE]);
            }
            exchange::ZC_CANCEL_EXCHANGE_ITEM => {
                self.exchange = Exchange::default();
                self.chat_line(Self::words(74, "The deal has been canceled."), RED);
                self.render();
            }
            exchange::ZC_EXEC_EXCHANGE_ITEM => {
                self.exchange = Exchange::default();
                match exchange::executed(packet) {
                    Some(0) => self.chat_line(Self::words(75, "The deal has successfully completed."), CYAN),
                    _ => self.chat_line(Self::words(76, "The deal has failed."), RED),
                }
                self.render();
            }
            // What the player put in is the player's again: the client gives its own count of the things back
            // (`ResetSlots`). rAthena does not send this; it gives the things back as things gained.
            exchange::ZC_EXCHANGEITEM_UNDO => {}
            _ => return false,
        }
        true
    }

    /// ZC_ACK_EXCHANGE_ITEM: the other agreed and the deal's window opens, or why there is no deal.
    fn exchange_answered(&mut self, packet: &[u8]) {
        use exchange::answer::*;
        let Some((answer, level)) = exchange::answered(packet) else { return };
        let words = match answer {
            ACCEPT => {
                let partner = std::mem::take(&mut self.exchange.partner);
                self.exchange = Exchange { partner, level, open: true, ..Exchange::default() };
                return self.render();
            }
            CANCEL => Self::words(69, "The deal has been rejected."),
            TOO_FAR => Self::words(70, "You are too far away from the person to trade."),
            NO_SUCH_CHARACTER => Self::words(71, "The Character is not currently online or does not exist."),
            FAILED => Self::words(72, "The person is in another deal."),
            BUSY => Self::words(1708, "[%s] is currently busy and cannot accept the request.").replace("%s", &self.exchange.partner),
            _ => return,
        };
        self.chat_line(words, RED);
    }

    /// ZC_ACK_ADD_EXCHANGE_ITEM: what the player put in was taken (it shows on the player's side), or why not.
    fn exchange_added(&mut self, packet: &[u8]) {
        use exchange::added::*;
        let (Some((index, result)), adding) = (exchange::add_answered(packet), self.exchange.adding.take()) else { return };
        match (result, adding) {
            (DONE, Some((_, amount))) if index == 0 => self.exchange.zeny.0 = amount,
            // It shows on the player's side and is out of the inventory from now on, as in the client. The
            // server says nothing of it leaving: if the deal is made it is the other's, and if the deal is
            // called off the server gives it back as a thing gained (rAthena's `trade_tradecancel`).
            (DONE, Some((_, amount))) => {
                let amount = amount.min(u32::from(u16::MAX)) as u16;
                if let Some(item) = self.bag.at_mut(index) {
                    self.exchange.mine.push(Item { amount, ..item.clone() });
                    item.amount = item.amount.saturating_sub(amount);
                }
                self.bag.items.retain(|item| item.amount > 0);
                self.refresh_items();
            }
            (OVERWEIGHT, _) => self.chat_line(Self::words(73, "You cannot trade because this character will exceed his weight limit."), RED),
            (TOO_MANY, _) => self.chat_line(
                Self::words(1938, "Relative character that has possession of the items cannot trade because amount is exceeded.."),
                RED
            ),
            (TOO_MANY_OF_IT, _) => self.chat_line(
                Self::words(1939, "Relative character that has possession of the item amount which exceeds makes it impossible to trade."),
                RED
            ),
            _ => {}
        }
        self.redraw(&[&wnd::EXCHANGE]);
    }
}
