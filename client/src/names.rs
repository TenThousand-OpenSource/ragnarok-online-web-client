//! The name under someone the pointer is on: its colour, what is written, and the mark beside a boss's.
//!
//! From the client's `CGameMode::ActorNameColor` and `CGameMode::UpdateNameBalloon` (the colours, which it
//! writes blue first) and `UINameBalloonText::SetName` (the title, the pictures of a boss and of an MVP).

/// The colours of a name: an NPC's (the client's 0xF7B895), a monster's (0xC3C3FF), a monster's that is more
/// than ten levels above the player (0xFF) or sixteen or more below (0x9B9B9B), and anyone else's.
pub const NPC: &str = "#95b8f7";
pub const MONSTER: &str = "#ffc3c3";
pub const STRONGER: &str = "#ff0000";
pub const WEAKER: &str = "#9b9b9b";
pub const PLAIN: &str = "#ffffff";

/// What the server told of someone beside the name: the level and whether it is a boss (0 not, 1 a boss, 2 an
/// MVP: the packet that shows a unit), and the title and group of a monster or an NPC (ZC_ACK_REQNAMEALL_NPC).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Card {
    pub level: u16,
    pub boss: u8,
    pub title: String,
    pub group: u32,
}

/// Who a name is of, as far as its colour goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Who {
    /// An NPC (the client's `IsAllyActor`).
    Npc,
    /// A monster that is fought.
    Monster,
    /// Anyone else: a player, a pet, a homunculus. A pet's job is a monster's, and its name a monster's colour.
    Other,
}

/// The colour of a name. `job` is what the one named is shown as, `level` its level (0: not told) and
/// `own_level` the player's.
pub fn colour(who: Who, job: u16, level: u16, own_level: u16) -> &'static str {
    let plain = match who {
        Who::Npc => NPC,
        _ if (1001..3999).contains(&job) || (20021..=31999).contains(&job) => MONSTER,
        _ => PLAIN,
    };
    if who != Who::Monster || level == 0 {
        return plain;
    }
    match i32::from(own_level) - i32::from(level) {
        ..-10 => STRONGER,
        16.. => WEAKER,
        _ => plain,
    }
}

/// The name as it is written: with its title in brackets before it, if it has one (`"[%s] %s"`).
pub fn written(name: &str, title: &str) -> String {
    if title.is_empty() { name.to_owned() } else { format!("[{title}] {name}") }
}

/// The picture that stands beside the name, among the client's pictures of its windows: the group's, if the
/// server named one (`group\group_%d.bmp`); else a boss's or an MVP's mark (`montype_boss.bmp`,
/// `montype_mvp.bmp`) for a monster that is one.
pub fn mark(card: &Card, is_monster: bool) -> Option<String> {
    match (card.group, card.boss) {
        (group, _) if group != 0 => Some(format!("group/group_{group}.bmp")),
        (_, 1) if is_monster => Some("montype_boss.bmp".to_owned()),
        (_, 2) if is_monster => Some("montype_mvp.bmp".to_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_npc_is_pale_blue_a_monster_pink_a_player_white() {
        assert_eq!(colour(Who::Npc, 97, 0, 50), NPC);
        assert_eq!(colour(Who::Monster, 1002, 1, 10), MONSTER);
        assert_eq!(colour(Who::Other, 7, 99, 10), PLAIN);
        // A pet is drawn as a monster and named in a monster's colour, whatever its level.
        assert_eq!(colour(Who::Other, 1002, 99, 10), MONSTER);
    }

    #[test]
    fn a_monster_far_above_is_red_and_one_far_below_grey() {
        assert_eq!(colour(Who::Monster, 1002, 61, 50), STRONGER);
        assert_eq!(colour(Who::Monster, 1002, 60, 50), MONSTER);
        assert_eq!(colour(Who::Monster, 1002, 34, 50), WEAKER);
        assert_eq!(colour(Who::Monster, 1002, 35, 50), MONSTER);
        // A level the server did not tell changes nothing.
        assert_eq!(colour(Who::Monster, 1002, 0, 50), MONSTER);
    }

    #[test]
    fn a_title_stands_in_brackets_before_the_name() {
        assert_eq!(written("Poring", ""), "Poring");
        assert_eq!(written("Poring", "Angry"), "[Angry] Poring");
    }

    #[test]
    fn the_mark_beside_a_name() {
        assert_eq!(mark(&Card { boss: 1, ..Card::default() }, true).as_deref(), Some("montype_boss.bmp"));
        assert_eq!(mark(&Card { boss: 2, ..Card::default() }, true).as_deref(), Some("montype_mvp.bmp"));
        assert_eq!(mark(&Card { boss: 2, group: 3, ..Card::default() }, true).as_deref(), Some("group/group_3.bmp"));
        assert_eq!(mark(&Card { boss: 1, ..Card::default() }, false), None);
        assert_eq!(mark(&Card::default(), true), None);
    }
}
