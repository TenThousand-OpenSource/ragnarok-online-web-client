//! The marks over NPCs that have something for the character: a quest to give, one to take back.

use super::*;

/// Sets the mark over an NPC, or takes it away: which action of the marks' sprite it is
/// (`crate::quests::mark_action`). The marks are those of the map the character is on now: the first one set
/// on another map drops the last map's (the client's `minimapMarkers`, cleared as a map is entered).
pub fn quest_mark(npc: u32, action: Option<usize>) {
    STATE.with_borrow_mut(|state| {
        if state.quest_marks.0 != state.entered {
            state.quest_marks = (state.entered, HashMap::new());
        }
        match action {
            Some(action) => state.quest_marks.1.insert(npc, action),
            None => state.quest_marks.1.remove(&npc),
        };
    });
}

/// Which visit to a map this is, counted from the start: what was told of one map is not of the next.
pub fn visit() -> u32 {
    STATE.with_borrow(|state| state.entered)
}

/// The marks of this frame, each a picture standing in the world as the sprites do (`frame.rs` adds them to
/// those).
/// The client makes a mark an effect that follows its NPC (`CNpc::UpdateMinimapMarker`, `CMsgEffect` of kinds
/// 28 to 30 and 34 to 36): 20 above the NPC's feet and 3 to the west of them, the pictures of its action one
/// after another, over and over, each for as long as the sprite says; its own size, nothing see-through.
pub(super) fn marks(state: &mut State, time: f64, pictures: &mut Vec<Stood>) {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return };
    if state.quest_marks.0 != state.entered {
        return;
    }
    for (npc, action) in &state.quest_marks.1 {
        let Some(place) = scene.place_of(Some(*npc)) else { continue };
        // Until the sprite is read its first picture is asked for, which has it read.
        let picture = figure::timing(crate::quests::MARKS, *action).map_or(0, |(count, each)| (time / each.max(1.0)) as usize % count.max(1));
        let key = SpriteKey::Creature(crate::quests::MARKS.to_owned(), *action, picture);
        if gpu.has_sprite(&key) {
            let feet = on_ground(&scene.gat, place);
            pictures.push((key, [feet[0] - 3.0, feet[1] + 20.0, feet[2]], 1.0, crate::body::Quad::PLAIN));
        }
    }
}
