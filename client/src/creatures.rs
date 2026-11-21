//! What stands on the map for an object that is no player, by its job alone: a character's body, a model, or
//! one sprite, and the folder that sprite is in.
//!
//! The client does not look at the kind of object the packet names for this, only at the job. Whoever the
//! packet does not call a player (`CGameMode::SpawnMovingActor` and its like: object type 0 is a `CPc`) is
//! made by `CGameMode::NewActor(job)`: a `CCharacterNpc` for a player's job (`IsPlayerJob`), a
//! `C3dGrannyGameActor` for a job whose name ends in `.gr2`, a `CNpc` for the rest; and a `CNpc` finds its
//! sprite by `CNpc::SetSprJob(job)`, which the changes of its look call again with the new job
//! (`CGameActor::SetMonsterTransformJob`, ZC_NPCSPRITE_CHANGE). So an NPC the server shows as a monster is
//! found among the monsters, a homunculus in `homun\`, and a mercenary, whose name has its sex before it
//! (`여\활용병`), among the characters' bodies.

/// Whether a job is a player's (the client's `IsPlayerJob`: up to `JT_SUMMER2`, and from the High Novice up
/// to, not reaching, the mercenaries at 6000).
pub fn player(job: u16) -> bool {
    job <= 30 || (4001..6000).contains(&job)
}

/// Whether a job is an NPC's (`CSession::IsNpcJob`: from the portal at 45 up to the monsters at 1000, and the
/// later ones from 10001 to 19998).
pub fn npc(job: u16) -> bool {
    (45..1000).contains(&job) || (10001..=19998).contains(&job)
}

/// The folder of `data\sprite` that has the sprite of a job that is no player's (`CNpc::SetSprJob`, its
/// ranges in its order): the NPCs', the characters' bodies for the mercenaries (6017 to 6046), the homunculi's
/// (the rest of 6001 to 6052), the monsters' for all else.
pub fn folder(job: u16) -> &'static str {
    match job {
        job if npc(job) => "npc",
        6017..=6046 => "인간족/몸통",
        6001..=6052 => "homun",
        _ => "몬스터",
    }
}

/// The hidden NPC (`JT_HIDDEN_NPC`): what a script puts where something is to be pressed or to happen, with
/// nothing to see (a plate under a monster of the museum, a sign, a door).
pub const HIDDEN_NPC: u16 = 111;

/// What is drawn for an object that is no player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A character put together of body and head, as the packet dresses it (`CCharacterNpc`).
    Character,
    /// A model of the client's `model\3dmob` (`C3dGrannyGameActor`): the file's name.
    Model(String),
    /// One sprite: its path under `data/sprite`, without an ending.
    Sprite(String),
    /// Nothing, yet it is there to be pressed (`CNpc::Render`, its `JT_HIDDEN_NPC`: no sprite and no shadow
    /// are drawn, and `SetRenderInfo` is given a rectangle at its feet as wide as the size the server tells
    /// with it, doubled by `CNpc::SetState`, and half as high; rAthena tells a size of none for all but
    /// players, so the rectangle is the place of its feet, which `CGameActor::SetRenderInfo` makes 40 pixels
    /// each way as it does anyone's). `IsVisibleJob` is false for it.
    Hidden,
}

/// What is drawn for a job that no player object has, `name` being what the client's table of jobs calls it
/// (`GetJobName`: the gateway's `sprites`); nothing for a job the table does not have.
pub fn shape(job: u16, name: Option<&str>) -> Option<Shape> {
    if player(job) {
        return Some(Shape::Character);
    }
    if job == HIDDEN_NPC {
        return Some(Shape::Hidden);
    }
    let name = name?;
    if name.contains(".gr2") {
        return Some(Shape::Model(name.to_owned()));
    }
    Some(Shape::Sprite(sprite(job, name)))
}

/// The sprite of a job that is one sprite, under `data/sprite` and without an ending: its folder, then the
/// name the client's table of jobs has for it (`CNpc::SetSprJob`).
pub fn sprite(job: u16, name: &str) -> String {
    format!("{}/{}", folder(job), name.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite(job: u16, name: &str) -> Option<String> {
        match shape(job, Some(name))? {
            Shape::Sprite(path) => Some(path),
            _ => None,
        }
    }

    #[test]
    fn one_that_is_a_sprite_stays_one_under_any_job() {
        // `CNpc::SetSprJob` with a player's job looks among the monsters.
        assert_eq!(super::sprite(1, "x"), "몬스터/x");
        assert_eq!(super::sprite(1002, "PORING"), "몬스터/PORING");
        assert_eq!(super::sprite(46, "1_ETC_01"), "npc/1_ETC_01");
    }

    #[test]
    fn the_folder_is_the_jobs_not_the_objects() {
        // A Kafra; a Poring, and an Alice whatever kind of object wears it; a later NPC.
        assert_eq!(sprite(114, "4_F_KAFRA6").as_deref(), Some("npc/4_F_KAFRA6"));
        assert_eq!(sprite(1002, "PORING").as_deref(), Some("몬스터/PORING"));
        assert_eq!(sprite(1275, "ALICE").as_deref(), Some("몬스터/ALICE"));
        assert_eq!(sprite(10001, "4_TOWER_14").as_deref(), Some("npc/4_TOWER_14"));
        assert_eq!((folder(999), folder(1000), folder(19998), folder(19999), folder(20021)), ("npc", "몬스터", "npc", "몬스터", "몬스터"));
    }

    #[test]
    fn homunculi_and_mercenaries_have_folders_of_their_own() {
        assert_eq!(sprite(6001, "LIF").as_deref(), Some("homun/LIF"));
        assert_eq!(sprite(6016, "VANILMIRTH_H2").as_deref(), Some("homun/VANILMIRTH_H2"));
        assert_eq!(sprite(6048, "MER_EIRA").as_deref(), Some("homun/MER_EIRA"));
        // A mercenary's name has its sex's folder before it.
        assert_eq!(sprite(6017, "여\\활용병").as_deref(), Some("인간족/몸통/여/활용병"));
        assert_eq!(sprite(6046, "남\\검용병").as_deref(), Some("인간족/몸통/남/검용병"));
        assert_eq!((folder(6000), folder(6047), folder(6052), folder(6053)), ("몬스터", "homun", "homun", "몬스터"));
    }

    #[test]
    fn a_players_job_is_a_character_and_a_model_a_model() {
        assert_eq!(shape(1, None), Some(Shape::Character));
        assert_eq!(shape(4008, Some("anything")), Some(Shape::Character));
        assert!(player(0) && player(30) && !player(31) && !player(45) && player(4001) && player(5999) && !player(6000) && !player(1002));
        assert_eq!(shape(722, Some("Guildflag90_1.gr2")), Some(Shape::Model("Guildflag90_1.gr2".to_owned())));
        // A job the client's table does not name is not drawn.
        assert_eq!(shape(1002, None), None);
    }

    #[test]
    fn the_hidden_npc_is_there_with_nothing_to_draw() {
        // Whatever the table calls it; the jobs beside it are sprites as any.
        assert_eq!(shape(HIDDEN_NPC, Some("1_ETC_01")), Some(Shape::Hidden));
        assert_eq!(shape(HIDDEN_NPC, None), Some(Shape::Hidden));
        assert_eq!(sprite(110, "x").as_deref(), Some("npc/x"));
        assert_eq!(sprite(112, "x").as_deref(), Some("npc/x"));
    }
}
