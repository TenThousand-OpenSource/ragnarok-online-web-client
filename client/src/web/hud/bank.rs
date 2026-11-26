//! The bank's window (the client's `UIBank_NewWnd`).

use super::*;
use crate::press::Step;
use crate::press::bank::Press;

/// One of the window's buttons: three pictures named `bank\<name>_out`, `_over`, `_press`
/// (`UIFrameWnd::AddBitmapButton`), telling what it does when pointed at.
fn button(x: i32, y: i32, size: (i32, i32), name: &str, press: Press, tells: &str) -> String {
    pictured_by(Convention::OutOverPress, x, y, size, &format!("bank/{name}"), press, &format!(r#" data-tip="{}""#, escape(tells)))
}

impl App {
    /// The bank (`UIBank_NewWnd`, 280 by 153; `OnCreate` and `Paint`): the title "Zeny Storage" (line 2771 of
    /// the message table) with the cross; `bank\bg_bank.bmp` under it; "In Bank:" (2772) at (30, 29) and
    /// "On Hand:" (2773) at (30, 46), each with its sum and " z" after it, both sums beginning 13 after where
    /// the first label ends; the field of ten letters at (78, 90), 99 by 17, with the two arrows at 177 that
    /// add and take a zeny (89 and 99 down); Deposit at (209, 77) and Withdraw at (209, 102); at 132 down
    /// "Max" (45), "+ 10,000,000" (93), "+ 1,000,000" (141) and "+ 100,000" (189); and at 112 down, in red
    /// and in the middle, the short line of why a sum was not sent.
    pub(in crate::web) fn bank_window(&self) -> String {
        let mut window = title_bar(280, &Self::words(2771, "Zeny Storage"));
        window.push_str(&cross(280, Press::Close));
        window.push_str(&art("bank/bg_bank.bmp", 0, 17, 280, 136));
        // The client measures the first label and writes both sums 43 from the window's edge past its width.
        let label = Self::words(2772, "In Bank:");
        let sums = [(29, label.clone(), self.bank.balance), (46, Self::words(2773, "On Hand:"), self.hero.zeny())];
        for (y, words, sum) in sums {
            window.push_str(&text(30, y, BLACK, &words));
            window.push_str(&[
                format!(r#"<div class="a t" style="left:30px;top:{y}px;white-space:nowrap;color:{BLACK}">"#),
                format!(r#"<span style="visibility:hidden">{}</span>"#, escape(&label)),
                format!(r#"<span style="margin-left:13px;{}">{} z</span></div>"#, SMALL.style(), crate::numbers::grouped(sum))
            ].concat());
        }
        window.push_str(&field(&windows::BANK.field(), (78, 90, 99, 17), &self.bank.typed, 10, "background:#f7f7f7;border:1px solid #c6c6c6"));
        window.push_str(&button(177, 89, (18, 10), "btn_upper", Press::One(Step::More), &Self::words(2776, "+ 1z")));
        window.push_str(&button(177, 99, (18, 9), "btn_buttom", Press::One(Step::Less), &Self::words(2777, "- 1z")));
        window.push_str(&button(209, 77, (42, 20), "btn_deposit", Press::Deposit, &Self::words(2774, "Deposit")));
        window.push_str(&button(209, 102, (42, 20), "btn_withdraw", Press::Withdraw, &Self::words(2775, "Withdraw")));
        // The files are named a tenth of what the buttons add (`OnCreate`: "+ 100,000" is `btn_10mil`).
        window.push_str(&button(189, 132, (46, 16), "btn_10mil", Press::Add(100_000), "+ 100,000"));
        window.push_str(&button(141, 132, (46, 16), "btn_100mil", Press::Add(1_000_000), "+ 1,000,000"));
        window.push_str(&button(93, 132, (46, 16), "btn_1000mil", Press::Add(10_000_000), "+ 10,000,000"));
        window.push_str(&button(45, 132, (46, 16), "btn_max", Press::Max, &Self::words(crate::bank::MAX_WORD, "Max")));
        if let Some(line) = self.bank.status {
            window.push_str(&format!(
                r#"<div class="a t" style="left:0;top:112px;width:280px;text-align:center;pointer-events:none;color:{RED};{}">{}</div>"#,
                SMALL.style(),
                escape(&Self::words(line, ""))
            ));
        }
        window
    }
}
