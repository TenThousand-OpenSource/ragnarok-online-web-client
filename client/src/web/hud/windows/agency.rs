//! The entries of the list of windows for the Adventurer's Agency.

use super::*;
use crate::press::agency::Press;

/// The board (`WID_ADVENPARTYBOARDWND`): 480 by 340, where the client first puts it, 200 across and 200 down
/// (`AdvenPartyBoardWndInfo::Reset`).
pub(in crate::web) static AGENCY: Window = Window::usual(
    "agency",
    |app| app.agency.open,
    |_, _| framed("left:200px;top:200px;width:480px;height:340px".to_owned()),
    App::agency_board,
    |app| app.pressed(Press::Close),
);

/// The window a notice is written in (`WID_REGISTERPARTYWND`) and the one it is read in
/// (`WID_APPLYFORPARTYWND`): each 220 by 190 at the same first place (`RegisterPartyWndInfo::Reset`,
/// `ApplyForPartyWndInfo::Reset`); one closes as the other opens.
pub(in crate::web) static AGENCY_NOTICE: Window = Window::usual(
    "agency-notice",
    |app| app.agency.writing.is_some(),
    |_, _| framed("left:200px;top:200px;width:220px;height:190px".to_owned()),
    App::agency_notice,
    |app| app.pressed(Press::NoticeClose),
);
pub(in crate::web) static AGENCY_READ: Window = Window::usual(
    "agency-read",
    |app| app.agency.reading.is_some(),
    |_, _| framed("left:200px;top:200px;width:220px;height:190px".to_owned()),
    App::agency_read,
    |app| app.pressed(Press::ReadClose),
);

/// The question to a leader (`WID_REQUESTJOINPARTYWND`): 220 by 160 in the middle of the screen; Escape
/// refuses, as closing it does in the client.
pub(in crate::web) static AGENCY_ASKS: Window = Window::usual(
    "agency-asks",
    |app| app.agency.asking.is_some(),
    |_, view| framed(centred(view, 220, 160)),
    App::agency_question,
    |app| app.pressed(Press::Refuse),
);

/// Every window of the agency: drawn anew together when what they show changes.
pub(in crate::web) static AGENCY_WINDOWS: [&Window; 4] = [&AGENCY, &AGENCY_NOTICE, &AGENCY_READ, &AGENCY_ASKS];
