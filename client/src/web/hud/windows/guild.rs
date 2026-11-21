//! The guild's entries of the list of windows.

use super::*;
use crate::web::app::GuildAsk;
use crate::web::hud::guild::{HEIGHT, WIDTH};

/// The guild window, where the client first opens it (`GuildWndInfo::Reset`: 0, 34).
pub(in crate::web) static GUILD: Window = Window::usual(
    "guild",
    |app| app.guild.open,
    |_, _| framed(format!("left:0;top:34px;width:{}px;height:{}px", WIDTH, HEIGHT)),
    App::guild_window,
    |app| app.pressed(crate::press::guild::Press::Window),
);

/// What the guild asks: an invitation in the client's message box, 200 of 480 down the middle of the screen;
/// the reason of leaving or expelling in its own box, 120 of 480 down; whether to go to a castle in the very
/// middle of the screen (`UIListBoxAgitMenu::SendMsg` puts the box there); what the server told of a castle
/// where the client's message boxes open, 185 across and 180 down its stage of 640 by 480
/// (`UIMessageBox::Configure`).
pub(in crate::web) static GUILD_ASKS: Window = Window {
    id: "guild-asks",
    open: |app| app.guild.asks.is_some(),
    frame: |app, view| {
        let (width, height) = app.guild_ask_size();
        match app.guild.asks {
            Some(GuildAsk::Castle(_)) => framed(centred(view, width, height)),
            Some(GuildAsk::Invested(_)) =>
                framed(format!("left:{}px;top:{}px;width:{width}px", (view.0 as i32 - 640) / 2 + 185, (view.1 as i32 - 480) / 2 + 180)),
            _ => middle(view, width, if height == 120 { 200 } else { 120 }),
        }
    },
    draw: App::guild_question,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};

/// The menu on a member of the guild, opened with the right button in the guild window, where the pointer was.
pub(in crate::web) static GUILD_MENU: Window =
    Window::menu("guild-menu", |app| app.guild.menu.is_some(), |app, _| menu_frame(app.guild.menu.map(|(at, _)| at)), App::guild_menu);
