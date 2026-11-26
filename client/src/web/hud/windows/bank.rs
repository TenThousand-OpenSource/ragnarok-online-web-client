//! The entry of the list of windows for the bank.

use super::*;

/// The bank (`UIBank_NewWnd`): 280 by 153, in the middle of the screen (`UIWindowMgr::EnsureWindow`,
/// `WID_BANK_NEWWND`: half the view less 140 across, half less 77 down). Its field has the keyboard as the
/// window opens and after each button that writes into it (`SetFocus` of `m_amountEdit`). Escape closes it as
/// its cross does: that the client's Escape does so was not found in its code and is taken from what its
/// other framed windows do.
pub(in crate::web) static BANK: Window = Window {
    id: "bank",
    open: |app| app.bank.open,
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:280px;height:153px", view.0 as i32 / 2 - 140, view.1 as i32 / 2 - 77)),
    draw: App::bank_window,
    order: ordered(|app| app.bank.open, |app| app.pressed(crate::press::bank::Press::Close)),
    layer: Layer::Usual,
    drawn: Some(typed_into),
};
