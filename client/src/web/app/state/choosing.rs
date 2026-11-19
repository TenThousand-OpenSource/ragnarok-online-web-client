//! What the app keeps of the lists a skill or a card opens to pick one thing of.

use crate::net::choosing::Listed;

/// A list that is open, by what it lists; the client has a window to each, of three shapes (`Choice::shape`).
pub(in crate::web) enum Choice {
    /// The places of Teleport or Warp Portal (`UIChooseWarpWnd`): the skill, and the places as the server
    /// names them, the client's own "cancel" after them.
    Warp(u16, Vec<String>),
    /// What arrows can be made of (`UIMakingArrowListWnd`, its kind 2): items by number.
    Arrows(Vec<u32>),
    /// What can be cooked or mixed (the same window with the kind the server told).
    Mix(u16, Vec<u32>),
    /// The spells of Auto Spell (`UISpellListWnd`): skills by number.
    Spells(Vec<u32>),
    /// The equipment a card can be set into (`UIItemCompositionWnd`): where the card is carried, and where each
    /// such thing is.
    Compound(u16, Vec<u16>),
    /// What can be forged or brewed (`UIMakeTargetListWnd`): items by number.
    Make(Vec<u32>),
    /// The thing picked of those, shown with what it needs (`UIMakeTargetProcessWnd`), and what was put into
    /// its three slots (items by number).
    Forge(u32, Vec<u32>),
    /// The broken things of someone, to repair one (`UIItemRepairWnd`).
    Repair(Vec<Listed>),
    /// The weapons carried that can be refined (`UIWeaponRefineWnd`).
    Refine(Vec<Listed>),
}

/// The three shapes of the client's windows for these lists.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Shape {
    /// 280 by 120 in a system box: a list of lines (`UIChooseWarpWnd`).
    Places,
    /// 200 by 200: rows with a picture and a name (`UIItemIdentifyWnd` and those made of it).
    Pictured,
    /// 280 by 150 under a title bar: a list of lines, or a thing being forged (`UIMakeTargetListWnd`…).
    Lines,
}

impl Choice {
    pub fn shape(&self) -> Shape {
        match self {
            Choice::Warp(..) => Shape::Places,
            Choice::Arrows(_) | Choice::Mix(..) | Choice::Spells(_) | Choice::Compound(..) => Shape::Pictured,
            Choice::Make(_) | Choice::Forge(..) | Choice::Repair(_) | Choice::Refine(_) => Shape::Lines,
        }
    }

    /// How many rows it has to pick from.
    pub fn rows(&self) -> usize {
        match self {
            Choice::Warp(_, places) => places.len(),
            Choice::Arrows(items) | Choice::Mix(_, items) | Choice::Spells(items) | Choice::Make(items) => items.len(),
            Choice::Compound(_, places) => places.len(),
            Choice::Forge(..) => 0,
            Choice::Repair(things) | Choice::Refine(things) => things.len(),
        }
    }
}

/// The list that is open, and the row picked in it.
#[derive(Default)]
pub(in crate::web) struct Choosing {
    pub open: Option<Choice>,
    pub picked: Option<usize>,
    /// Where the card is carried that was last double-clicked: the one the server's list of equipment is
    /// for (the client's `cardItemIndex`).
    pub card: u16,
}
