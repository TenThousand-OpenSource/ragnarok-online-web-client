//! Fighting other players as rules: what holds on the map the character is on and who on it is fought (the
//! client's `CGameMode::HandleActorPick`), the standing's line in the corner of the screen
//! (`CGameMode::DrawMiniMapCoordinates`) and what the chat says when the standing changes
//! (`CGameMode::ReceiveEffectNumber`). The packets that tell of them are read in `net/pvp.rs`.

/// What holds on the map the character is on. The flags are those of ZC_MAPPROPERTY_R2, each named for what the
/// client does with it (the numbers are the client's own for them, in `CMapPropertyMgr`).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Zone {
    /// Bit 0 (the client's 7): everyone who is not of the player's party is fought.
    pub party_foes: bool,
    /// Bit 1 (8): everyone who is not of the player's guild is fought.
    pub guild_foes: bool,
    /// Bit 2 (9): a siege: guilds fight, and a guild's emblem shows over its members.
    pub siege: bool,
    /// Bit 3 (15): effects are drawn the simple way, as with `/mineffect`.
    pub simple_effects: bool,
    /// Bit 4 (16): the client's `m_isSiegeZone`: a press on another player fights them.
    pub fights_on_press: bool,
    /// Bit 5 (17): the client's `m_isPvpZone`: the standing is counted and shown.
    pub counted: bool,
    /// Bit 6 (18): no party is made or changed here.
    pub no_party: bool,
    /// Bit 7 (19): a battleground: teams fight.
    pub battlefield: bool,
    /// Bit 8 (20): costumes are not shown.
    pub no_costumes: bool,
    /// The older packets' word for it (the client's `isPKZone`): a map, or a server, of free fighting.
    pub pk: bool,
}

impl Zone {
    /// Whether another player is fought here as a monster is: the pointer over them is the sword, and a press
    /// attacks. `party`, `guild` and `ally` tell whether they are of the player's party, of the player's guild,
    /// of a guild allied to it. The client's rule for a player under the pointer with no skill aimed and Shift
    /// not held (`CGameMode::HandleActorPick`).
    ///
    /// On a battleground the client goes by the two teams (`playerTeam`, `battleTeam`), which this client does
    /// not know: nobody is fought there.
    pub fn fights(&self, party: bool, guild: bool, ally: bool) -> bool {
        if self.battlefield {
            return false;
        }
        if self.siege {
            return !(guild || ally);
        }
        let by_side = if self.party_foes { !party } else { self.guild_foes && !guild };
        by_side || ((self.pk || self.fights_on_press) && !party)
    }

    /// Whether the standing is shown (the client draws it when `isPKZone` or `m_isPvpZone` is set).
    pub fn ranked(&self) -> bool {
        self.pk || self.counted
    }
}

/// The action of `RankFont` that is the stroke between the rank and the total.
pub const STROKE: usize = 10;

/// The "rank / total" line the client draws in the corner of the screen on a PVP map
/// (`CGameMode::DrawMiniMapCoordinates`): each piece is an action of the sprite `이팩트\RankFont` (a digit, or
/// `STROKE`) and where its origin stands, from the right edge and from the bottom of the screen. Nothing while
/// either number is not above zero. The total is written up to 9999, as the client does; the rank too (the
/// client does not bound it, and writes nonsense past that).
pub fn rank_line(rank: i32, total: i32) -> Vec<(usize, i32, i32)> {
    if rank <= 0 || total <= 0 {
        return Vec::new();
    }
    // The digits of a number, the last first.
    let digits = |number: i32| {
        let mut left = number.min(9999);
        let mut digits = vec![(left % 10) as usize];
        while left >= 10 {
            left /= 10;
            digits.push((left % 10) as usize);
        }
        digits
    };
    let mut line = Vec::new();
    let total = digits(total);
    let last = -80 + total.len() as i32 * 25;
    line.extend(total.into_iter().enumerate().map(|(index, digit)| (digit, last - index as i32 * 25, -30)));
    line.push((STROKE, -80, -40));
    line.extend(digits(rank).into_iter().enumerate().map(|(index, digit)| (digit, -105 - index as i32 * 25, -50)));
    line
}

/// What the chat says when the player's rank changed from `before` to `now`, by the client's rule
/// (`CGameMode::ReceiveEffectNumber`): the number of its message (582 for coming into the first ten, 583 for
/// falling out of them), or none.
pub fn rank_message(before: i32, now: i32) -> Option<usize> {
    match (before > 10, now > 10) {
        (true, false) => Some(582),
        (false, true) if before != now => Some(583),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_is_laid_out_as_the_client_does() {
        // Rank 3 of 12: the total's digits end 30 from the right, the stroke stands at 80, the rank at 105.
        assert_eq!(rank_line(3, 12), vec![(2, -30, -30), (1, -55, -30), (STROKE, -80, -40), (3, -105, -50)]);
        assert_eq!(rank_line(10, 7), vec![(7, -55, -30), (STROKE, -80, -40), (0, -105, -50), (1, -130, -50)]);
        assert!(rank_line(0, 5).is_empty());
        assert!(rank_line(-1, 5).is_empty());
        assert!(rank_line(1, 0).is_empty());
        assert_eq!(rank_line(1, 123_456).len(), 6);
    }

    #[test]
    fn the_chat_tells_of_the_first_ten() {
        assert_eq!(rank_message(11, 10), Some(582));
        assert_eq!(rank_message(10, 11), Some(583));
        // The first standing told is counted from none, as the client counts it: one who starts within the
        // ten is told nothing, one who starts outside them is told of being out.
        assert_eq!(rank_message(0, 4), None);
        assert_eq!(rank_message(0, 14), Some(583));
        assert_eq!(rank_message(4, 4), None);
        assert_eq!(rank_message(12, 14), None);
    }
}
