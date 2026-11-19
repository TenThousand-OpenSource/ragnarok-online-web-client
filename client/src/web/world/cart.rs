//! The carts merchants pull: drawn behind each owner as the client's effect that follows one (`CMsgEffect` of
//! the kind `MET_CART`, made by `AM_MAKE_CART`; the rules are `crate::carts`).

use super::*;
use crate::carts::Pulled;

/// The carts in the world.
#[derive(Default)]
pub(super) struct Carts {
    /// The kind of cart the player's own character pushes, which is kept from map to map.
    own: Option<u8>,
    /// The kinds the others in sight push, of the visit to a map they were told on (`State::entered`).
    visit: u32,
    others: HashMap<u32, u8>,
    /// Where each cart is and how far it has rolled, once its owner was seen (none is the player's own).
    pulled: HashMap<Option<u32>, Pulled>,
}

/// Someone pushes a cart of that kind (none named: the player's own character), or none any more.
pub fn carted(owner: Option<u32>, kind: Option<u8>) {
    STATE.with_borrow_mut(|state| {
        let carts = &mut state.carts;
        if carts.visit != state.entered {
            (carts.visit, carts.others) = (state.entered, HashMap::new());
            carts.pulled.clear();
        }
        match (owner, kind) {
            (None, kind) => carts.own = kind,
            (Some(owner), Some(kind)) => {
                carts.others.insert(owner, kind);
            }
            (Some(owner), None) => {
                carts.others.remove(&owner);
            }
        }
        if kind.is_none() {
            carts.pulled.remove(&owner);
        }
    });
}

/// The carts of this frame, each a picture standing in the world as the sprites do (`frame.rs` adds them to
/// those, and they are sorted by where they stand as the client sorts its own: a cart behind its owner is
/// drawn before it, one in front after). Each is pulled along by where its owner is now, looks at its owner,
/// and shows the picture its roll has come to. The client takes the cart away while its owner hides (the
/// bits 2, 4 and 0x40 of the effect state), and so does this; a cart whose owner went out of sight goes with
/// it.
pub(super) fn drawn(state: &mut State, pictures: &mut Vec<Stood>) {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return };
    let carts = &mut state.carts;
    if carts.visit != state.entered {
        (carts.visit, carts.others) = (state.entered, HashMap::new());
        carts.pulled.clear();
    }
    // (Those shown while the map was still being read are put on it later: their carts wait for them.)
    if state.reading.is_none() && state.early.is_empty() {
        carts.others.retain(|owner, _| scene.units.contains_key(owner));
    }
    let yaw = scene.camera.yaw;
    let owners: Vec<(Option<u32>, u8)> = carts.own.map(|kind| (None, kind)).into_iter().chain(carts.others.iter().map(|(owner, kind)| (Some(*owner), *kind))).collect();
    carts.pulled.retain(|owner, _| owners.iter().any(|(pulls, _)| pulls == owner));
    for (owner, kind) in owners {
        let mover = match owner {
            None => &scene.player,
            Some(id) => match scene.units.get(&id) {
                Some(mover) => mover,
                None => continue,
            },
        };
        if !matches!(mover.look, Look::Character(_)) || mover.state & (2 | 4 | 0x40) != 0 {
            carts.pulled.remove(&owner);
            continue;
        }
        let Some(sprite) = crate::carts::sprite(kind, mover.job) else { continue };
        let cart = carts.pulled.entry(owner).or_insert_with(|| Pulled::behind(mover.at, mover.facing));
        cart.follow(mover.at);
        let action = cart.action(mover.at, yaw);
        // Until the sprite is read its first picture is asked for, which has it read.
        let picture = figure::timing(&sprite, action).map_or(0, |(count, _)| cart.picture(count));
        let key = SpriteKey::Creature(sprite, action, picture);
        if gpu.has_sprite(&key) {
            pictures.push((key, on_ground(&scene.gat, cart.at), 1.0, crate::body::Quad::PLAIN));
        }
    }
}
