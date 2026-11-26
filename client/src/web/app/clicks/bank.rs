//! The bank: its window asked for, zeny put in and taken out, the buttons that write a sum into its field
//! (`crate::bank`, the client's `UIBank_NewWnd::SendMsg`).

use super::*;
use crate::bank::Side;

impl App {
    /// The key of the bank (`HK_BANK` of `UIWindowMgr`'s keys): the window closes if it is open; otherwise the
    /// balance is asked for, as the menu's icon asks. The key is used.
    pub(in crate::web::app) fn bank_key(&mut self) -> bool {
        if self.bank.open {
            self.pressed(crate::press::bank::Press::Close);
        } else {
            self.pressed(crate::press::bank::Press::Window);
        }
        true
    }

    /// A press of this system (`crate::press::bank`).
    pub(super) fn bank_pressed(&mut self, press: crate::press::bank::Press) {
        use crate::press::Step;
        use crate::press::bank::Press;
        match press {
            // `UIMenuIconWnd::SendMsg`, `MENUICON_BANK`: the balance is asked for whether the window is open
            // or not; it is the answer that opens the window or closes it (`ReceiveBankCheck`).
            Press::Window => self.send_on_map(&net::bank::check(self.own_account())),
            Press::Close => {
                self.bank.open = false;
                self.render();
            }
            Press::Deposit => self.bank_moved(Side::Deposit),
            Press::Withdraw => self.bank_moved(Side::Withdraw),
            Press::One(step) => self.bank_stepped(if step == Step::More { 1 } else { -1 }),
            Press::Add(sum) => self.bank_stepped(i64::from(sum)),
            // "Max": its word stands in the field, which has the keyboard.
            Press::Max => self.bank_typed(Self::words(crate::bank::MAX_WORD, "Max")),
        }
    }

    /// The field shows that, and has the keyboard (`SetText`, `SetFocus` of the window's `m_amountEdit`).
    fn bank_typed(&mut self, text: String) {
        self.bank.typed = text;
        self.redraw(&[&wnd::BANK]);
        // A field that had the keyboard is drawn anew with what was typed into it: what the button wrote
        // goes in after that.
        set_field(&wnd::BANK.field(), &self.bank.typed);
    }

    /// A button added so much to what is typed (`UIBank_NewWnd::AdjustAmount`).
    fn bank_stepped(&mut self, step: i64) {
        self.bank_typed(crate::bank::stepped(&field(&wnd::BANK.field()), step).to_string());
    }

    /// Deposit or Withdraw was pressed: the sum typed is sent, or the client says why not, in the chat and at
    /// the window's foot. Either way the field is emptied (`UIBank_NewWnd::TransactionAmount`). The client
    /// first refuses while a window of Lapine, of reforming, of enchanting or of runes is open
    /// (`CGameMode::IsMenuWindowOpen`): this client has none of them.
    fn bank_moved(&mut self, side: Side) {
        let typed = field(&wnd::BANK.field());
        let max = Self::words(crate::bank::MAX_WORD, "Max");
        match crate::bank::sum(&typed, &max, side, self.hero.zeny(), self.bank.balance) {
            Ok(Some(sum)) => {
                self.bank.status = None;
                let account = self.own_account();
                self.send_on_map(&if side == Side::Deposit { net::bank::deposit(account, sum as i32) } else { net::bank::withdraw(account, sum as i32) });
            }
            Ok(None) => {}
            Err(told) => {
                self.say_filled(said::GENERAL, told.chat, &[], RED);
                self.bank.status = Some(told.status);
            }
        }
        self.bank_typed(String::new());
    }
}
