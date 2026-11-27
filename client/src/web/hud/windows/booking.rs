//! The entries of the list of windows for the old party booking.

use super::*;
use crate::press::booking::Press;

/// Where the client first opens these windows: 510 across and 360 down (`SeekPartyListWndInfo::Reset` and
/// its like). (Kept on the screen here: the client's own first place lies partly off a screen 1024 wide, and
/// it puts a window back there whenever the place kept is off the screen.)
fn first_place(view: (f64, f64), width: i32, height: i32) -> String {
    let (left, top) = ((view.0 as i32 - width).clamp(0, 510), (view.1 as i32 - height).clamp(0, 360));
    framed(format!("left:{left}px;top:{top}px;width:{width}px;height:{height}px"))
}

/// The list of ads (`WID_SEEKPARTYLISTWND`): 550 by 350, and 17 high when made small.
pub(in crate::web) static BOOKING_LIST: Window = Window::usual(
    "booking",
    |app| app.booking.list,
    |app, view| first_place(view, 550, if app.booking.small { 17 } else { 350 }),
    App::booking_list,
    |app| app.pressed(Press::Close),
);

/// Its help (`WID_PARTYBOOKINGHELPWND`): 320 by 199, 200 across the client's stage and 200 down it.
pub(in crate::web) static BOOKING_HELP: Window = Window::usual(
    "booking-help",
    |app| app.booking.help,
    |_, view| staged(view, 200, 200, 320, 199),
    App::booking_help,
    |app| app.pressed(Press::HelpClose),
);

/// The window an ad is registered in (`WID_SEEKPARTYMBWND`): 360 by 350.
pub(in crate::web) static BOOKING_REGISTER: Window = Window::usual(
    "booking-register",
    |app| app.booking.register,
    |app, view| first_place(view, 360, if app.booking.register_small { 17 } else { 350 }),
    App::booking_register,
    |app| app.pressed(Press::RegisterClose),
);

/// The window of one's own ad (`WID_SEEKPARTYWND`): 152 by 199, and 44 high when made small. Escape does not
/// close it: only taking the ad away does.
pub(in crate::web) static BOOKING_OWN: Window = Window {
    id: "booking-own",
    open: |app| app.booking.own.is_some(),
    frame: |app, view| first_place(view, 152, if app.booking.own.as_ref().is_some_and(|own| own.small) { 44 } else { 199 }),
    draw: App::booking_own,
    order: None,
    layer: Layer::Usual,
    drawn: None,
};

/// The window the jobs of one's ad are changed in (`WID_JOBLISTWND`): 152 by 199.
pub(in crate::web) static BOOKING_JOBS: Window = Window::usual(
    "booking-jobs",
    |app| app.booking.changing.is_some(),
    |_, view| first_place(view, 152, 199),
    App::booking_jobs,
    |app| app.pressed(Press::JobsClose),
);

/// Every window of the booking: drawn anew together when what they show changes.
pub(in crate::web) static BOOKING_WINDOWS: [&Window; 5] = [&BOOKING_LIST, &BOOKING_HELP, &BOOKING_REGISTER, &BOOKING_OWN, &BOOKING_JOBS];
