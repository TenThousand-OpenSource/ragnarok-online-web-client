//! The entries of the list of windows for what a fight opens.

use super::*;
use crate::web::hud::fight::SENSE;

/// The Monster Info window of Sense, where the client opens it: in the corner of the screen
/// (`WID_MONSTERINFOWND`: `Move(0, 0)`), 400 by 200. Escape closes it as its OK does.
pub(in crate::web) static SENSE_WINDOW: Window = Window {
    id: "sense",
    open: |app| app.fight.sensed.is_some(),
    frame: |_, _| framed(format!("left:0px;top:0px;width:{}px;height:{}px", SENSE.0, SENSE.1)),
    draw: App::sense_window,
    order: ordered(|app| app.fight.sensed.is_some(), |app| app.pressed(crate::press::fight::Press::SenseOk)),
    layer: Layer::Usual,
    drawn: None,
};

/// The question about a Star Gladiator's place, in the client's message box (`WID_STARPLACEACCEPTWND`).
pub(in crate::web) static STAR_ASKS: Window = Window::question("star-asks", |app| app.fight.star.is_some(), App::star_question);
