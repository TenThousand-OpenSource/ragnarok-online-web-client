//! The entries of the list of windows for what a fight opens.

use super::*;
use crate::web::hud::fight::SENSE;

/// The Monster Info window of Sense, where the client opens it: in the corner of the screen
/// (`WID_MONSTERINFOWND`: `Move(0, 0)`), 400 by 200. Escape closes it as its OK does.
pub(in crate::web) static SENSE_WINDOW: Window = Window::usual(
    "sense",
    |app| app.fight.sensed.is_some(),
    |_, _| framed(at(0, 0, SENSE.0, SENSE.1)),
    App::sense_window,
    |app| app.pressed(crate::press::fight::Press::SenseOk),
);

/// The question about a Star Gladiator's place, in the client's message box (`WID_STARPLACEACCEPTWND`).
pub(in crate::web) static STAR_ASKS: Window = Window::question("star-asks", |app| app.fight.star.is_some(), App::star_question);
