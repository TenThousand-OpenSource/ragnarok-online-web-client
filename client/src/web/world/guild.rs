//! The guilds of the players in sight: which guild each is of, what the app has learnt of their names, and the
//! emblems read so far. The name under a player the pointer is on is written from these (the client's
//! `UINameBalloonText::SetName` and `SetIcon`).

use super::*;

/// What the world knows of it.
#[derive(Default)]
pub(super) struct Guilds {
    /// By player: the names of the party and of the guild, and the position in the guild (ZC_ACK_REQNAMEALL).
    cards: HashMap<u32, (String, String, String)>,
    /// The emblems as pictures for the page, by guild and version.
    emblems: HashMap<(u32, u32), String>,
    /// The same emblems as pixels, for the flags they are drawn on (`granny.rs`).
    pixels: HashMap<(u32, u32), bmp::Image>,
    /// The player's own character, which is not among those in sight: its ID, its name, and its guild with
    /// the version of the emblem.
    own: Option<(u32, String, (u32, u32))>,
}

/// The guild a player in sight is of and the version of its emblem; none for anyone who is not a player.
pub fn player_guild(id: u32) -> Option<(u32, u32)> {
    STATE.with_borrow(|state| {
        let unit = state.scene.as_ref()?.units.get(&id)?;
        matches!(unit.look, Look::Character(_)).then_some(unit.guild)
    })
}

/// Someone in sight is of another guild now, or of none, or their guild has another emblem.
pub fn guild_changed(id: u32, guild: u32, emblem: u32) {
    STATE.with_borrow_mut(|state| {
        if let Some(unit) = state.scene.as_mut().and_then(|scene| scene.units.get_mut(&id)) {
            unit.guild = (guild, emblem);
        }
        if guild == 0 {
            if let Some(card) = state.guilds.cards.get_mut(&id) {
                (card.1, card.2) = (String::new(), String::new());
            }
        }
    });
}

/// A guild is no more (the player's own, broken up): none of those in sight is of it from now, and the
/// flags of a castle it held are bare at once. This project's own: the Windows client keeps a flag's emblem
/// (`CGameActor::SetGuildInfo` takes no guild of 0, and `C3dGrannyGameActor` keeps the texture it made) until
/// the flag comes into sight anew, and the server sends nothing for the flags of a castle left without an
/// owner.
pub fn guild_gone(guild: u32) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = state.scene.as_mut().filter(|_| guild != 0) else { return };
        for unit in scene.units.values_mut().filter(|unit| unit.guild.0 == guild) {
            unit.guild = (0, 0);
        }
    });
}

/// The player's own character: its ID, what it is called, and the guild it is of with the version of its emblem
/// (0 and 0 for none). Kept across maps.
pub fn own_guild(id: u32, name: &str, guild: u32, emblem: u32) {
    STATE.with_borrow_mut(|state| {
        state.guilds.own = Some((id, name.to_owned(), (guild, emblem)));
        if guild == 0 {
            if let Some(card) = state.guilds.cards.get_mut(&id) {
                (card.1, card.2) = (String::new(), String::new());
            }
        }
    });
}

/// The server told what a player's party and guild are called, and the player's position in the guild.
pub fn player_card(id: u32, party: String, guild: String, position: String) {
    STATE.with_borrow_mut(|state| state.guilds.cards.insert(id, (party, guild, position)));
}

/// The position a player has in their guild changed.
pub fn player_position(id: u32, position: String) {
    STATE.with_borrow_mut(|state| {
        if let Some(card) = state.guilds.cards.get_mut(&id) {
            card.2 = position;
        }
    });
}

/// An emblem was read: its picture for the page.
pub fn emblem_known(guild: u32, version: u32, picture: String) {
    STATE.with_borrow_mut(|state| state.guilds.emblems.insert((guild, version), picture));
}

/// An emblem was read: its pixels, for the guild's flags.
pub fn emblem_pixels(guild: u32, version: u32, pixels: bmp::Image) {
    STATE.with_borrow_mut(|state| state.guilds.pixels.insert((guild, version), pixels));
}

impl Guilds {
    /// A guild's emblem of a version as pixels, once it was read.
    pub(super) fn emblem(&self, guild: (u32, u32)) -> Option<&bmp::Image> {
        self.pixels.get(&guild).filter(|_| guild.0 != 0)
    }

    /// What is written under a player the pointer is on: the name, with the party's in brackets after it, and
    /// under it the guild's with the position in brackets (`UINameBalloonText::SetName`, as it writes them
    /// for the Korean service: "%s (%s)" both). The name alone for anyone nothing more is known of.
    pub(super) fn label(&self, id: u32, name: &str) -> String {
        let Some((party, guild, position)) = self.cards.get(&id) else { return name.to_owned() };
        // (What others named has stars for a word that is not let be written: `crate::manner`.)
        let first = if party.is_empty() { name.to_owned() } else { format!("{name} ({})", data::starred(party)) };
        if guild.is_empty() { first } else { format!("{first}\n{} ({})", data::starred(guild), data::starred(position)) }
    }

    /// The player's own ID and what is written under the player's own character, once the app has told who
    /// that is.
    pub(super) fn own_label(&self) -> Option<(u32, String)> {
        let (id, name, _) = self.own.as_ref().filter(|(_, name, _)| !name.is_empty())?;
        Some((*id, self.label(*id, name)))
    }

    /// Lays the name's label out for what `label` wrote into it, after `name_shown` placed it: two lines when
    /// there are two, and the emblem of the guild of the one named (someone in sight, or the player's own
    /// character), 24 pixels each way, 4 from the label's left edge and half way down it, the label 25 wider
    /// for it (`UINameBalloonText::SetIcon`, `Paint`).
    pub(super) fn decorate(&self, id: u32, unit: Option<&Mover>, emblems: bool) {
        let guild = match (unit, &self.own) {
            (Some(unit), _) if matches!(unit.look, Look::Character(_)) => unit.guild,
            (None, Some((own, _, guild))) if *own == id => *guild,
            _ => (0, 0),
        };
        let Some(label) = crate::web::by_id("unit-name").and_then(|label| label.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
        let style = label.style();
        let _ = style.set_property("white-space", "pre");
        let _ = style.set_property("text-align", "center");
        // The emblem shows when the player has it so (`/emblem`).
        if let Some(picture) = self.emblems.get(&guild).filter(|picture| emblems && guild.0 != 0 && !picture.is_empty()) {
            let _ = style.set_property("background", &format!("url('{picture}') 4px 50% / 24px 24px no-repeat"));
            let _ = style.set_property("padding-left", "29px");
            let _ = style.set_property("min-height", "24px");
        }
    }
}
