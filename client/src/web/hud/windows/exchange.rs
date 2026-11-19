//! The entries of the list of windows for a deal between two players: its window, the question whether to
//! deal, how many of a thing go in.

use super::*;

/// The window of a deal between two players, where the client opens it (`CUIExchangeUI::OnInit`): 100 across
/// its 640-wide stage, 100 of 480 down. Escape calls the deal off, as its Cancel does.
pub(in crate::web) static EXCHANGE: Window = Window::usual(
    "exchange",
    |app| app.exchange.open,
    |_, view| staged(view, 100, 100, 560, 377),
    App::exchange_window,
    |app| app.pressed(crate::press::exchange::Press::Cancel),
);

/// The question whether to deal, in the client's message box: 200 of 480 down the middle of the screen.
pub(in crate::web) static EXCHANGE_ASKS: Window = Window::question("exchange-asks", |app| app.exchange.asked.is_some(), App::exchange_question);

/// How many of a thing go into the deal.
pub(in crate::web) static EXCHANGE_COUNT: Window = Window::count("exchange-count", |app| app.exchange.counting.is_some(), App::exchange_count_window);
