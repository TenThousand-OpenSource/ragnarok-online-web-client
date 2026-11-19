//! The merchant's cart as it is drawn behind its owner: which sprite a kind of cart is, where it stands as its
//! owner walks, which way it is seen and which picture of its roll shows (drawn by `web/world/cart.rs`).
//!
//! The client makes the cart an effect of its own that follows its owner (`CGameActor::SendMsg`,
//! `AM_MAKE_CART`: a `CMsgEffect` of the kind `MET_CART`, whose sprite is named in
//! `CGameMode::SpawnMsgEffect`), not a part of the body: it is pulled as on a rope one cell long
//! (`CMsgEffect::FollowOwner`), looks at its owner, and its wheels turn by the way it has gone. Which cart it
//! is, is the first value of the state `EFST_ON_PUSH_CART` alone: the client never looks at the bits of the
//! old effect state, nor at the owner's level (the server does, when a look is asked for).

/// The carts the client has a sprite for, by kind (the rows of `effectIds` in `AM_MAKE_CART`; the names are
/// `CartTypeName`'s): 0 is the Super Novice's own, which a Super Novice has whatever the kind.
const SPRITES: [&str; 14] = ["슈노손수레", "손수레", "손수레1", "손수레2", "손수레3", "손수레4", "손수레5", "손수레6", "손수레7", "손수레8", "선물상자카트", "포링싣은카트", "포링카트", "마도카트"];

/// How long the rope is, in cells (the client's 5.0 of the world's measure, a cell being 5).
pub const ROPE: f32 = 1.0;

// Whose cart it is: the client's `IsSuperNoviceClass` (the Super Novice, its baby, the Expanded Super Novice and
// its baby) and `IsBabyJob` (the ranges of its `JT_…_B`).
use crate::jobs::{baby, super_novice};

/// The sprite of a cart of that kind behind one of that job, under `data/sprite/`; none for a kind the client
/// has no cart for. The carts from the sixth on have a smaller sprite of their own for a baby (`…베이비`,
/// `…_미니`). For the first five and the Super Novice's the client draws the grown one's pictures by a baby's
/// own table of where they stand (`베이비손수레.act` over `손수레.spr`); this client reads a sprite's two files
/// by one name, so a baby pulls the grown one's cart there.
pub fn sprite(kind: u8, job: u16) -> Option<String> {
    let kind = if super_novice(job) { 0 } else { usize::from(kind) };
    let name = *SPRITES.get(kind)?;
    let small = match kind {
        6..=9 if baby(job) => "베이비",
        10..=13 if baby(job) => "_미니",
        _ => "",
    };
    Some(format!("이팩트/{name}{small}"))
}

/// The sprite of a kind of cart as the window that offers the looks shows it (`UISelectCartWnd::AddCart`), and
/// where its button stands, 100 square: five to a row 70 apart, the rows 82 apart from 28 down, each kind
/// moved a little as the client moves it. `place` is its place among those offered.
pub fn offered(kind: u8, place: usize) -> Option<(String, (i32, i32))> {
    let name = *SPRITES.get(usize::from(kind)).filter(|_| kind > 0)?;
    let (x, y) = ((place % 5) as i32 * 70, (place / 5) as i32 * 82 + 28);
    let (dx, dy) = match kind {
        1 | 2 | 10 => (0, -13),
        4 => (18, -10),
        5 => (15, -5),
        6..=9 => (13, 0),
        11 => (16, -13),
        12 => (0, -23),
        13 => (4, -10),
        _ => (0, 0),
    };
    Some((format!("이팩트/{name}"), (x + dx, y + dy)))
}

/// The looks the skill Change Cart offers a character of that level (`UISelectCartWnd::OnCreate`): one more
/// kind past each of the levels, and the thirteenth past 100. (The server lets the same through:
/// `clif_parse_ChangeCart`.)
pub fn by_level(level: u32) -> Vec<u8> {
    const PAST: [u32; 9] = [0, 40, 65, 80, 90, 100, 110, 120, 130];
    let mut kinds: Vec<u8> = PAST.iter().zip(1u8..).filter(|(past, _)| level > **past).map(|(_, kind)| kind).collect();
    if level > 100 {
        kinds.push(13);
    }
    kinds
}

/// The place so far behind one who stands at `owner` and looks the game's way `facing` (0 south, then by
/// eighths through west), in cells: where what follows an owner is made (`AM_SET_MASTER_POINTER`: a cart, a
/// falcon).
pub fn behind(owner: (f32, f32), facing: u8, away: f32) -> (f32, f32) {
    let turn = (f32::from(facing) * 45.0).to_radians();
    // One who looks south has it to the north.
    (owner.0 + turn.sin() * away, owner.1 + turn.cos() * away)
}

/// A cart being pulled: where it is, in cells, and how far it has rolled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pulled {
    pub at: (f32, f32),
    pub rolled: f32,
}

impl Pulled {
    /// A cart as it is made: a rope's length behind its owner, who looks the game's way `facing` (0 south,
    /// then by eighths through west) (`AM_SET_MASTER_POINTER`).
    pub fn behind(owner: (f32, f32), facing: u8) -> Self {
        Pulled { at: behind(owner, facing, ROPE), rolled: 0.0 }
    }

    /// One frame of `CMsgEffect::FollowOwner`: while the owner is farther than the rope is long, the cart is
    /// drawn to the rope's end on the line between them, and rolls by as much as it moved.
    pub fn follow(&mut self, owner: (f32, f32)) {
        let (dx, dy) = (self.at.0 - owner.0, self.at.1 - owner.1);
        let away = (dx * dx + dy * dy).sqrt();
        if away > ROPE {
            let to = (owner.0 + dx / away * ROPE, owner.1 + dy / away * ROPE);
            self.rolled += ((to.0 - self.at.0).powi(2) + (to.1 - self.at.1).powi(2)).sqrt();
            self.at = to;
        }
    }

    /// Which of the sprite's eight actions shows to a camera turned by `yaw` degrees: the cart looks at its
    /// owner (`Face`, `Get8Dir2`), and its actions are numbered as the game's directions are.
    pub fn action(&self, owner: (f32, f32), yaw: f32) -> usize {
        let facing = (self.at.0 - owner.0).atan2(self.at.1 - owner.1).to_degrees();
        ((facing - yaw).rem_euclid(360.0) + 22.5) as usize / 45 % 8
    }

    /// Which picture of the action shows: `travelDistance * 0.23 * 4 / speed` of the world's measure, the
    /// speed being the 4 every cart's sprite has for a picture, among the action's `pictures`.
    pub fn picture(&self, pictures: usize) -> usize {
        (self.rolled * crate::map::CELL * 0.23) as usize % pictures.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_of_cart_has_its_sprite() {
        assert_eq!(sprite(1, 5).as_deref(), Some("이팩트/손수레"));
        assert_eq!(sprite(5, 10).as_deref(), Some("이팩트/손수레4"));
        assert_eq!(sprite(6, 4008).as_deref(), Some("이팩트/손수레5"));
        assert_eq!(sprite(13, 18).as_deref(), Some("이팩트/마도카트"));
        assert_eq!(sprite(14, 5), None);
    }

    #[test]
    fn a_super_novice_has_its_own_and_a_baby_a_small_one() {
        assert_eq!(sprite(3, 23).as_deref(), Some("이팩트/슈노손수레"));
        assert_eq!(sprite(6, 4028).as_deref(), Some("이팩트/손수레5베이비"));
        assert_eq!(sprite(10, 4028).as_deref(), Some("이팩트/선물상자카트_미니"));
        // The first five have no sprite of a baby's own.
        assert_eq!(sprite(2, 4028).as_deref(), Some("이팩트/손수레1"));
    }

    #[test]
    fn the_looks_offered_follow_the_level() {
        assert_eq!(by_level(1), [1]);
        assert_eq!(by_level(40), [1]);
        assert_eq!(by_level(41), [1, 2]);
        assert_eq!(by_level(99), [1, 2, 3, 4, 5]);
        assert_eq!(by_level(101), [1, 2, 3, 4, 5, 6, 13]);
        assert_eq!(offered(4, 6), Some(("이팩트/손수레3".to_owned(), (88, 100))));
        assert_eq!(offered(0, 0), None);
    }

    #[test]
    fn a_cart_is_made_behind_its_owner_and_pulled_on_a_rope() {
        let mut cart = Pulled::behind((10.0, 10.0), 0);
        assert!((cart.at.0 - 10.0).abs() < 1e-5 && (cart.at.1 - 11.0).abs() < 1e-5);
        // The owner walks three cells south: the cart follows a cell behind and has rolled three.
        cart.follow((10.0, 7.0));
        assert!((cart.at.1 - 8.0).abs() < 1e-5 && (cart.rolled - 3.0).abs() < 1e-5);
        // Within the rope's length it stays where it is.
        cart.follow((10.3, 7.5));
        assert!((cart.at.1 - 8.0).abs() < 1e-5 && (cart.rolled - 3.0).abs() < 1e-5);
        // Three cells are 15 of the world's measure: the fourth picture of six.
        assert_eq!(cart.picture(6), 3);
    }

    #[test]
    fn a_cart_looks_at_its_owner() {
        // North of its owner it looks south; east of it, west; and a camera turned a quarter sees it turned.
        let north = Pulled { at: (10.0, 11.0), rolled: 0.0 };
        assert_eq!(north.action((10.0, 10.0), 0.0), 0);
        let east = Pulled { at: (11.0, 10.0), rolled: 0.0 };
        assert_eq!(east.action((10.0, 10.0), 0.0), 2);
        assert_eq!(east.action((10.0, 10.0), 90.0), 0);
        let south = Pulled { at: (10.0, 9.0), rolled: 0.0 };
        assert_eq!(south.action((10.0, 10.0), 0.0), 4);
    }
}
