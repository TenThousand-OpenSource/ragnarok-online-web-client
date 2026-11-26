//! Fighting other players: what the map server tells of the map's kind and of the standing on a PVP map. What
//! the client does with each is its `CGameMode::ReceiveMapProperty`, the `ID_ZC_NOTIFY_MAPPROPERTY` and
//! `ID_ZC_NOTIFY_MAPPROPERTY2` arms of its packet handler, and `CGameMode::ReceiveEffectNumber` (its name in the
//! decompilation for the handler of ZC_NOTIFY_RANKING).

use super::*;
use crate::net::pvp;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn pvp_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            pvp::ZC_MAPPROPERTY_R2 => {
                if let Some((_, zone)) = pvp::zone(packet) {
                    self.zoned(zone);
                }
            }
            pvp::ZC_NOTIFY_MAPPROPERTY => {
                if let Some((kind, zone)) = pvp::old_zone(packet) {
                    // A server where players fight everywhere says so (`MSI_PKZONE`, in the client's 0x9B9BFF).
                    if kind == pvp::property::PK_SERVER {
                        self.chat_line(Self::words(855, "You are in a PK area. Please beware of sudden attack."), "#ff9b9b");
                    }
                    self.zoned(zone);
                }
            }
            pvp::ZC_NOTIFY_MAPPROPERTY2 => {
                if let Some(zone) = pvp::typed(packet, self.pvp.zone) {
                    self.zoned(zone);
                }
            }
            pvp::ZC_NOTIFY_RANKING => {
                if let Some((who, rank, total)) = pvp::ranking(packet) {
                    self.ranked(who, rank, total);
                }
            }
            _ => return false,
        }
        true
    }

    /// The map the character is on is of this kind now: the world goes by it for whom a press fights, and the
    /// standing of the map before means nothing here.
    fn zoned(&mut self, zone: crate::pvp::Zone) {
        self.pvp.zone = zone;
        (self.pvp.rank, self.pvp.total) = (0, 0);
        world::zone(zone);
        self.sides_told();
        self.rank_shown();
    }

    /// Tells the world who the player's friends are: the party's accounts, the player's guild and its allies.
    /// Called when any of them changes.
    pub(in crate::web::app) fn sides_told(&self) {
        let party = self.party.members.iter().map(|member| member.account).collect();
        world::sides(party, self.guild.id, self.guild.allies());
    }

    /// ZC_NOTIFY_RANKING. Only the player's own standing is kept: the client draws that one in the corner of
    /// the screen, tells in the chat of coming into the first ten and of falling out of them (in its 0xFFFF,
    /// yellow), and plays `effect\number_change.wav` when the rank changed. It does nothing of it off a PVP map.
    ///
    /// Not done: over anyone of the first ten the client plays its effect `EF_TOPRANK` with their rank, and
    /// while the player is the first of several it counts the time at the top of the screen
    /// (`DrawTimeAttackTimer`).
    fn ranked(&mut self, who: u32, rank: i32, total: i32) {
        if !self.pvp.zone.ranked() || who != self.own_account() {
            return;
        }
        let before = self.pvp.rank;
        if before != rank {
            if let Some(line) = crate::pvp::rank_message(before, rank) {
                let name = self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default();
                let words = Self::words(line, if line == 582 {
                    "Congratulations! %s's rank has gone up to %d."
                } else {
                    "What a pity! %s's rank has gone down to %d."
                });
                self.chat_line(words.replacen("%s", &name, 1).replacen("%d", &rank.to_string(), 1), YELLOW);
            }
            sound::effect("effect/number_change.wav", 1.0);
        }
        (self.pvp.rank, self.pvp.total) = (rank, total);
        self.rank_shown();
    }

    /// Writes the standing in the corner of the screen anew. Its digits are pictures of the client's sprite
    /// `이팩트\RankFont`, read the first time there is something to write.
    pub(in crate::web::app) fn rank_shown(&mut self) {
        // (The score of a battleground is written in the same digits, into the same corner.)
        let line = self.rank_line() + &self.score_board();
        // Whether there is a standing to write is asked of the rule, not of the line: the line is made of the
        // pictures, and has none until they are read (asking the line never read them: seen in the game,
        // 2026-11-26, a PVP map with no standing in its corner).
        let ranked = self.pvp.zone.ranked() && !crate::pvp::rank_line(self.pvp.rank, self.pvp.total).is_empty();
        let due = ranked || self.battleground.score.is_some();
        if due && self.pvp.digits.is_empty() && !self.pvp.reading {
            self.pvp.reading = true;
            wasm_bindgen_futures::spawn_local(async {
                let mut digits = Vec::new();
                for action in 0..=crate::pvp::STROKE {
                    match figure::frame_picture("이팩트/RankFont", action, 0).await {
                        Ok(picture) => digits.push(picture),
                        Err(_) => return,
                    }
                }
                dispatch(Action::PvpRead(digits));
            });
        }
        if let Some(shown) = element("pvp-rank") {
            shown.set_inner_html(&line);
        }
    }

    /// The digits of the standing were read (the pictures of `RankFont`, action by action).
    pub(in crate::web::app) fn pvp_digits_read(&mut self, digits: Vec<(String, (usize, usize), (f32, f32))>) {
        self.pvp.digits = digits;
        self.rank_shown();
    }
}
