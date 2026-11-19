//! The pets' entries of the list of windows: the pet window, what it asks, the menu on the pet, the eggs to
//! hatch, the slot machine of taming.

use super::*;

/// The slot machine of taming: 196 by 256 in the middle of the screen; a press on it starts the reels.
pub(in crate::web) static TAMING: Window = Window {
    id: "taming",
    open: |app| app.pets.taming.is_some(),
    frame: |_, view| {
        format!(
            r#"class="a" style="{};cursor:pointer;pointer-events:auto" {}"#,
            at((view.0 as i32 - 196) / 2, (view.1 as i32 - 256) / 2, 196, 256),
            pressed(crate::press::pets::Press::TamingShoot)
        )
    },
    draw: |_| r#"<img id="taming-picture" alt="" draggable="false" style="position:absolute;left:0;top:0">"#.to_owned(),
    order: None,
    layer: Layer::Top,
    drawn: None,
};

/// The list of eggs to hatch: 200 by 200, where the client opens it.
pub(in crate::web) static EGGS: Window = Window::pick("eggs", |app| app.pets.eggs.is_some(), App::egg_window);

/// The pet window, where the client opens it: 185 across its 640-wide stage, 60 above 400 of 480 down.
pub(in crate::web) static PET: Window = Window::usual(
    "pet",
    |app| app.pets.open && app.pets.pet.is_some(),
    |_, view| framed(format!("left:{}px;top:{}px;width:280px", (view.0 as i32 - 640) / 2 + 185, view.1 as i32 * 400 / 480 - 60)),
    App::pet_window,
    switched_off!(pets.open),
);

/// What the pet window asks before it acts, in the middle of the screen.
pub(in crate::web) static PET_ASKS: Window = Window {
    id: "pet-asks",
    open: |app| app.pets.open && app.pets.pet.is_some(),
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:280px", (view.0 as i32 - 280) / 2, (view.1 as i32 - 120) / 2)),
    draw: App::pet_question,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};

/// The menu of the pet, opened with the right button on it, where the pointer was.
pub(in crate::web) static PET_MENU: Window =
    Window::menu("pet-menu", |app| app.pets.menu_at.is_some() && app.pets.pet.is_some(), |app, _| menu_frame(app.pets.menu_at), App::pet_menu);
