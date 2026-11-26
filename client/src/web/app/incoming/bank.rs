//! The bank's answers (`crate::net::bank`): the balance, which opens the window, and what a deposit and a
//! withdrawal came to.

use super::*;
use crate::net::bank;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn bank_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // `CGameMode::ReceiveBankCheck`: the balance is kept, and the window closes if it was open and
            // opens if it was not. A reason is told in the chat instead.
            bank::ZC_BANKING_CHECK => {
                let Some((reason, balance)) = bank::checked(packet) else { return true };
                match crate::bank::check_refused(reason) {
                    Some(line) => self.say_filled(said::GENERAL, line, &[], RED),
                    None => {
                        self.bank.balance = balance;
                        self.bank.open = !self.bank.open;
                        (self.bank.typed, self.bank.status) = (String::new(), None);
                        // Drawn where it stands, so that its field has the keyboard as it opens.
                        self.redraw(&[&wnd::BANK]);
                    }
                }
            }
            // `ReceiveBankDepositAck`, `ReceiveBankWithdrawAck`: what the bank holds and what is carried, as
            // the server has them now; or the reason, in the chat.
            bank::ZC_ACK_BANKING_DEPOSIT | bank::ZC_ACK_BANKING_WITHDRAW => {
                let Some(moved) = bank::moved(packet) else { return true };
                let refused = if id == bank::ZC_ACK_BANKING_DEPOSIT {
                    crate::bank::deposit_refused(moved.reason)
                } else {
                    crate::bank::withdrawal_refused(moved.reason)
                };
                match refused {
                    Some(line) => self.say_filled(said::GENERAL, line, &[], RED),
                    None => {
                        self.bank.balance = moved.balance;
                        self.hero.params.insert(packets::parameter::ZENY, i64::from(moved.zeny));
                        self.refresh_values();
                        self.redraw(&[&wnd::BANK]);
                    }
                }
            }
            // The client takes these two for nothing (`CGameMode::ProcessPacket`); it never asks for them.
            bank::ZC_ACK_OPEN_BANKING | bank::ZC_ACK_CLOSE_BANKING => {}
            _ => return false,
        }
        true
    }
}
