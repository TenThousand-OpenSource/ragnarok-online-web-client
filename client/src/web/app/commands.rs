//! The client's own commands: typed into the chat after a slash, or run from the Game Settings window.

use super::*;

/// The skills of help the client lets go on a monster only with Shift held, unless its No Shift option is on
/// (its `IsOptionalSelfTargetSkill`, those of the first jobs): Heal, Increase and Decrease Agility, Blessing,
/// Cure, Detoxify, Resurrection, Impositio Manus, Suffragium, Aspersio, Slow Poison, Status Recovery, Kyrie
/// Eleison, Turn Undead, Lex Aeterna.
const HELPING: [u16; 15] = [28, 29, 30, 34, 35, 53, 54, 66, 67, 68, 71, 72, 73, 77, 78];

impl App {
    /// Whether the skill needs Shift held to go on a monster, as the player's options stand.
    pub(super) fn guarded(&self, skill: u16) -> bool {
        HELPING.contains(&skill) && !self.option("TT_NOSHIFT_ON_OFF")
    }

    /// Does one of the client's commands; `false` for words that are none of them (they are said, then).
    pub(super) fn command(&mut self, words: &str) -> bool {
        let Some(socket) = &self.socket else { return false };
        match words {
            // One who walks does not sit down (as with the key).
            "/sit" if world::walking() => {}
            "/sit" | "/stand" => socket.send(&units::sit(words == "/sit")),
            // Where the character is, as the client writes it: the map's full name, its own, and the cell
            // (`ShowPlayerLocation`).
            "/where" => {
                let Some(game) = &self.game else { return true };
                let map = game.map.trim_end_matches(".gat");
                let line = match data::map_sign(map) {
                    Some(sign) => format!("{}({map}) : {}, {}", sign.name, game.x, game.y),
                    None => format!("{map} : {}, {}", game.x, game.y),
                };
                self.chat_line(line, "#ffff64");
            }
            // CZ_REMEMBER_WARPPOINT (0x011d): this place is kept for Warp Portal.
            "/memo" => socket.send(&0x011du16.to_le_bytes()),
            // CZ_DORIDORI (0x01e7): the head shakes; the server gives a novice's SP back faster for it.
            "/doridori" => socket.send(&0x01e7u16.to_le_bytes()),
            // The character turns an eighth of the way round, clockwise or the other way.
            "/bangbang" | "/bingbing" => {
                if let Some(direction) = world::turn(words == "/bangbang") {
                    socket.send(&units::face(direction));
                }
            }
            _ => match data::emotion_named(words).and_then(|emotion| u8::try_from(emotion).ok()) {
                // An emotion by its name (`/ho`, `/delight`): the client's list of them.
                Some(emotion) => socket.send(&emotions::show(emotion)),
                None => return false,
            },
        }
        true
    }

    /// The command behind one of the "run" rows of the Game Settings window, by the client's name for the row.
    pub(in crate::web) fn command_of(option: &str) -> Option<&'static str> {
        Some(match option {
            "TT_EX_MAP_POS" => "/where",
            "TT_EX_REMEMBER_WARPPOINT" => "/memo",
            "TT_EX_DORIDORI" => "/doridori",
            "TT_EX_BANGBANG" => "/bangbang",
            "TT_EX_BINGBING" => "/bingbing",
            _ => return None,
        })
    }
}
