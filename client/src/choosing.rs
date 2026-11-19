//! The lists a skill or a card opens, as rules: what may be put into a forging beside what it needs, the
//! effect the client shows for how a making or a refining ended, and the card set into a thing's first empty
//! slot. The packets of them are in `net::choosing`.

/// The stones of the four elements (Flame Heart, Mystic Frozen, Rough Wind, Great Nature) and the Star Crumb:
/// what a smith may put into a weapon being forged beside what it needs (`ITEM_INFO::IsElementalStone`,
/// `IsForgeMaterial`).
pub const STAR_CRUMB: u32 = 1000;

pub fn elemental_stone(item: u32) -> bool {
    matches!(item, 994..=997)
}

/// Whether anything more may be put into the making of that thing: not into the stones, the iron, the steel
/// and the star crumb a smith makes (`UIMakeTargetProcessWnd::Paint` draws no slots for those).
pub fn takes_more(make: u32) -> bool {
    !matches!(make, 994..=1000)
}

/// Whether a thing dragged onto the window of a forging goes into one of its three slots
/// (`UIMakeTargetProcessWnd::DragAndDrop`): a star crumb or a stone of an element, while a slot is free, and
/// no second stone of an element.
pub fn goes_in(make: u32, put: &[u32], item: u32) -> bool {
    takes_more(make) && put.len() < 3 && (item == STAR_CRUMB || elemental_stone(item)) && !(elemental_stone(item) && put.iter().copied().any(elemental_stone))
}

/// What goes to the server of what was put in: three numbers, noughts for the empty slots.
pub fn put_in(put: &[u32]) -> [u32; 3] {
    [0, 1, 2].map(|slot| put.get(slot).copied().unwrap_or(0))
}

/// The client's effects of a refining or a making that went well or did not, on the one who did it
/// (`EffectTypes.h`: `EF_REFINEOK`, `EF_REFINEFAIL`, `EF_PHARMACY_OK`, `EF_PHARMACY_FAIL`).
pub mod effect {
    pub const REFINE_OK: u32 = 154;
    pub const REFINE_FAIL: u32 = 155;
    pub const PHARMACY_OK: u32 = 305;
    pub const PHARMACY_FAIL: u32 = 306;
}

/// The effect the client shows for the result of a making (`CGameMode::ReceiveItemEffectResult`); the results
/// 4 to 7 belong to later content.
pub fn made_effect(result: u16) -> Option<u32> {
    match result {
        0 => Some(effect::REFINE_OK),
        1 => Some(effect::REFINE_FAIL),
        2 => Some(effect::PHARMACY_OK),
        3 => Some(effect::PHARMACY_FAIL),
        _ => None,
    }
}

/// The card set into the first empty slot of a thing, as the client does it itself when the server says the
/// compounding went well (`CGameMode::ReceiveCardCompositionResult`). False when no slot is empty.
pub fn set_card(cards: &mut [u32; 4], card: u32) -> bool {
    match cards.iter_mut().find(|slot| **slot == 0) {
        Some(slot) => {
            *slot = card;
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_forging_takes_star_crumbs_and_one_stone() {
        assert!(goes_in(1101, &[], 994));
        assert!(goes_in(1101, &[994], STAR_CRUMB));
        assert!(!goes_in(1101, &[994], 995), "a second stone of an element");
        assert!(!goes_in(1101, &[1000, 1000, 1000], 994), "no slot is free");
        assert!(!goes_in(1101, &[], 998), "iron is no such material");
        assert!(!goes_in(999, &[], 1000), "nothing more goes into steel");
        assert_eq!(put_in(&[994, 1000]), [994, 1000, 0]);
    }

    #[test]
    fn an_outcome_has_the_clients_effect() {
        assert_eq!([0, 1].map(made_effect), [Some(effect::REFINE_OK), Some(effect::REFINE_FAIL)]);
        assert_eq!([2, 3, 4].map(made_effect), [Some(effect::PHARMACY_OK), Some(effect::PHARMACY_FAIL), None]);
    }

    #[test]
    fn a_card_goes_into_the_first_empty_slot() {
        let mut cards = [4001, 0, 0, 0];
        assert!(set_card(&mut cards, 4002));
        assert_eq!(cards, [4001, 4002, 0, 0]);
        assert!(!set_card(&mut [1, 2, 3, 4], 5));
    }
}
