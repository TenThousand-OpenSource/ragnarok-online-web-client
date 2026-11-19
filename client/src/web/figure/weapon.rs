//! The weapon a character holds: which sprite it is, and its picture by itself.
//!
//! The client draws a character a layer at a time, and some effects have it treat the weapon's layer by
//! itself: added to what is behind it, or drawn twice (`CPc::ApplyLayerEffects`, its layer 5;
//! `crate::body`). A character here is one picture, so for those effects the weapon has a picture of its own:
//! the same frame, of the same size and with the feet at the same place, holding the weapon's layers alone.
//! The client draws a weapon, and the trail of light it leaves (a sprite of its own beside the weapon's, its
//! name ending in `_검광`: the client's layer 6), only in the actions of standing ready and striking
//! (`IsReadyOrAttackAction`), and so it is here, in the character's picture and in this one alike.

use super::*;

thread_local! {
    /// The pictures of weapons alone, by the figure whose weapon it is: nothing for one that is being made,
    /// or whose weapon has no sprite.
    static ALONE: RefCell<HashMap<Figure, Option<Rc<Canvas>>>> = RefCell::new(HashMap::new());
}

/// The weapon in a figure's hand as it is in the frame the figure is in: its sprite's pictures and that frame
/// of it. The weapon has a sprite of its own, with a picture for each of the body's: one made for that very
/// weapon if the client has it, else the one of its kind (a dagger, a sword). `trail` asks for the weapon's
/// trail in its place (`GetWeaponEffectResourceName`). Nothing for a figure that holds nothing, in an action
/// that shows no weapon, or for a weapon without a sprite.
pub(super) async fn part(figure: &Figure, trail: bool) -> Option<(Sprite, Frame)> {
    if figure.weapon == 0 || !crate::pose::armed(figure.action) {
        return None;
    }
    let ending = if trail { "_검광" } else { "" };
    let sex = if figure.sex == 1 { "남" } else { "여" };
    let race = if data::is_doram(figure.job) { "도람족" } else { "인간족" };
    let job = data::body_sprite(figure.job);
    let base = format!("data/sprite/{race}/{job}/{job}_{sex}");
    let (pictures, actions) = match sprite(&format!("{base}_{}{ending}", figure.weapon)).await {
        Ok(weapon) => weapon,
        Err(_) => {
            let kind = data::about(figure.weapon).and_then(|(_, facts)| facts.into_iter().find(|(key, _)| key == "SubType")).and_then(|(_, kind)| data::weapon_sprite(&kind))?;
            sprite(&format!("{base}{kind}{ending}")).await.ok()?
        }
    };
    let part = frame(&actions, figure.action, figure.frame).ok()?;
    Some((pictures, part))
}

/// The picture of a figure's weapon alone, if it has been made. If not, making it is started, and a later
/// call has it.
pub(super) fn alone(figure: &Figure) -> Option<Rc<Canvas>> {
    if let Some(known) = ALONE.with_borrow(|alone| alone.get(figure).cloned()) {
        return known;
    }
    ALONE.with_borrow_mut(|alone| alone.insert(figure.clone(), None));
    let figure = figure.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Some((pictures, part)) = part(&figure, false).await {
            let mut canvas = Canvas::new(WIDTH, HEIGHT);
            canvas.draw(FEET, &pictures, None, &part);
            ALONE.with_borrow_mut(|alone| alone.insert(figure, Some(Rc::new(canvas))));
        }
    });
    None
}

/// Forgets the pictures made so far (`figure::forget`).
pub(super) fn forget() {
    ALONE.with_borrow_mut(HashMap::clear);
}
