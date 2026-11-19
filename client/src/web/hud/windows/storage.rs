//! The entries of the list of windows for the storage: the storage itself, the eight windows that show one of
//! its tabs apart, the one that lists what a search found, and the box that asks how many go in or out.

use super::*;

/// The storage. The client first opens it at 360, 200 (`ItemStoreFindListInfo::Reset`); here it stands where
/// this client has always had it, with the inventory beside it.
pub(in crate::web) static STORAGE: Window = Window::usual(
    "storage",
    |app| app.storage.open.is_some(),
    |_, _| framed(at(225, 80, 280, 458)),
    App::storage_window,
    |app| app.pressed(crate::press::storage::Press::Close),
);

/// A window that shows one tab of the storage apart: 140 by 145, where the client first puts each
/// (`ItemStoreFindSlotInfo::Reset`).
macro_rules! apart {
    ($name:ident, $id:literal, $tab:literal, $x:literal, $y:literal) => {
        pub(in crate::web) static $name: Window = Window::usual(
            $id,
            |app| app.storage.open.is_some() && app.storage.apart & (1 << $tab) != 0,
            |_, _| framed(at($x, $y, 140, 145)),
            |app| app.storage_apart_window($tab),
            |app| app.pressed(crate::press::storage::Press::Apart($tab)),
        );
    };
}

apart!(STORAGE_APART_0, "storage-apart-0", 0, 210, 50);
apart!(STORAGE_APART_1, "storage-apart-1", 1, 60, 50);
apart!(STORAGE_APART_2, "storage-apart-2", 2, 210, 205);
apart!(STORAGE_APART_3, "storage-apart-3", 3, 60, 205);
apart!(STORAGE_APART_4, "storage-apart-4", 4, 210, 360);
apart!(STORAGE_APART_5, "storage-apart-5", 5, 60, 360);
apart!(STORAGE_APART_6, "storage-apart-6", 6, 210, 515);
apart!(STORAGE_APART_7, "storage-apart-7", 7, 60, 515);

/// What a search of the storage found (`ItemStoreFindWndInfo::Reset`: 510, 360, 280 by 312).
pub(in crate::web) static STORAGE_FOUND: Window = Window::usual(
    "storage-found",
    |app| app.storage.open.is_some() && app.storage.found.is_some(),
    |_, _| framed(at(510, 360, 280, 312)),
    App::storage_found_window,
    |app| app.pressed(crate::press::storage::Press::FoundClose),
);

/// How many of a thing go into the storage or out of it.
pub(in crate::web) static STORAGE_COUNT: Window = Window::count("storage-count", |app| app.storage.counting.is_some(), App::storage_count_window);

/// Every window that shows the storage's things: drawn anew together when they change.
pub(in crate::web) static STORAGE_WINDOWS: [&Window; 10] =
    [
        &STORAGE, &STORAGE_APART_0, &STORAGE_APART_1, &STORAGE_APART_2, &STORAGE_APART_3, &STORAGE_APART_4,
        &STORAGE_APART_5, &STORAGE_APART_6, &STORAGE_APART_7, &STORAGE_FOUND
    ];
