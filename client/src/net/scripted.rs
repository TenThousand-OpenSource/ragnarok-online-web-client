//! What an NPC's script has the client show and play beside its words: a picture over the screen (`cutin`), a
//! mark on the minimap (`viewpoint`), a bar that fills over someone (`progressbar`), a sound and a piece of
//! music (`soundeffect`, `playBGM`), words over a head (`showscript`), an effect taken away. The layouts are
//! rAthena's for PACKETVER 20260219 (`clif_cutin`, `clif_viewpoint`, `clif_progressbar`, `clif_soundeffect`,
//! `clif_playBGM`, `clif_showscript` in `src/map/clif.cpp`); what the client does with each is in
//! `CGameMode`'s handlers (`GameModePackets.cpp`, `GameModeSounds.cpp`, `GameModeMiniMap.cpp`).

use super::bytes::{long, text, text_at};

/// ZC_SHOW_IMAGE (0x0145, 19 bytes): `char name[16], u8 kind`; ZC_SHOW_IMAGE2 (0x01b3, 67 bytes): `char
/// name[64], u8 kind`, the one rAthena sends at this PACKETVER.
pub const ZC_SHOW_IMAGE: u16 = 0x0145;
pub const ZC_SHOW_IMAGE2: u16 = 0x01b3;
/// ZC_COMPASS (0x0144, 23 bytes): `u32 npc, u32 kind, u32 x, u32 y, u8 id, u32 colour`.
pub const ZC_COMPASS: u16 = 0x0144;
/// ZC_PROGRESS (0x02f0, 10 bytes): `u32 colour, u32 seconds`: a bar over the player's own character.
pub const ZC_PROGRESS: u16 = 0x02f0;
/// ZC_PROGRESS_CANCEL (0x02f2, 2 bytes): that bar goes.
pub const ZC_PROGRESS_CANCEL: u16 = 0x02f2;
/// ZC_PROGRESS_ACTOR (0x09d1, 14 bytes): `u32 who, u32 colour, u32 seconds`: a bar over someone else.
pub const ZC_PROGRESS_ACTOR: u16 = 0x09d1;
/// ZC_SOUND (0x01d3, 35 bytes): `char name[24], u8 act, u32 term, u32 who`.
pub const ZC_SOUND: u16 = 0x01d3;
/// ZC_PLAY_NPC_BGM (0x0b8c at this PACKETVER): `u16 length, u8 kind, char name[]`. (The older 0x07fe, a name
/// of 24 bytes alone, is not sent at this PACKETVER and has no length in the table.)
pub const ZC_PLAY_NPC_BGM: u16 = 0x0b8c;
/// ZC_SHOWSCRIPT (0x08b3): `u16 length, u32 who, char words[]`.
pub const ZC_SHOWSCRIPT: u16 = 0x08b3;
/// ZC_CLEAR_EFFECT (0x0b0d, 10 bytes): `u32 who, u32 effect`: an effect on someone ends.
pub const ZC_CLEAR_EFFECT: u16 = 0x0b0d;

/// Where the client puts a picture a script shows (`CGameMode::UpdateMiniMapTooltip`, the kinds 0 to 4 of
/// its `UIIllustWnd`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stand {
    /// On the bottom edge, its middle 200 from the left (`Move(200 - w / 2, H - h)`).
    Left,
    /// On the bottom edge, in the middle (`Move(W / 2 - w / 2, H - h)`).
    Middle,
    /// On the bottom edge, its middle 200 from the right (`Move(W - 200 - w / 2, H - h)`).
    Right,
    /// In the very middle of the screen, as a window with a title bar and a cross
    /// (`Move(W / 2 - w / 2, H / 2 - h / 2)`, its height with the title bar's).
    Titled,
    /// In the very middle of the screen, with a cross and no title bar.
    Centre,
}

/// What a ZC_SHOW_IMAGE says.
#[derive(Debug, PartialEq)]
pub enum Cutin {
    /// That picture, standing so. The name is as the client makes it: in small letters, with `.bmp` after it
    /// unless it names a `.bmp` or a `.png` already (`MM_SHOW_IMAGE`).
    Show(String, Stand),
    /// The picture shown goes (kind 255).
    Away,
}

/// ZC_SHOW_IMAGE and ZC_SHOW_IMAGE2. The kinds 5 to 9 set no picture in the client and the kinds from 10 on
/// are its `UIMultiIllustWnd`, pictures over the whole screen that later content uses: neither is read here.
pub fn cutin(id: u16, packet: &[u8]) -> Option<Cutin> {
    let size = if id == ZC_SHOW_IMAGE { 16 } else { 64 };
    let name = text_at(packet, 2, size)?.to_lowercase();
    let stand = match *packet.get(2 + size)? {
        0 => Stand::Left,
        1 => Stand::Middle,
        2 => Stand::Right,
        3 => Stand::Titled,
        4 => Stand::Centre,
        255 => return Some(Cutin::Away),
        _ => return None,
    };
    let name = if name.contains(".bmp") || name.contains(".png") { name } else { format!("{name}.bmp") };
    Some(Cutin::Show(name, stand))
}

/// A mark a script puts on the minimap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compass {
    /// What tells the marks apart: a later one of the same number takes the earlier one's place.
    pub id: u8,
    /// 2 takes the mark away; 0 shows it for a while (`COMPASS_SHOWN_FOR`); anything else leaves it.
    pub kind: u32,
    pub cell: (u32, u32),
    /// Red, green and blue (the packet has them as 0x00RRGGBB; the client draws the mark opaque).
    pub colour: [u8; 3],
}

/// How long a mark of kind 0 is shown, in milliseconds (`CGameMode::DrawMiniMap`: 15000).
pub const COMPASS_SHOWN_FOR: f64 = 15_000.0;

/// ZC_COMPASS.
pub fn compass(packet: &[u8]) -> Option<Compass> {
    let colour = long(packet, 19)?;
    Some(Compass { id: *packet.get(18)?, kind: long(packet, 6)?, cell: (long(packet, 10)?, long(packet, 14)?), colour: [(colour >> 16) as u8, (colour >> 8) as u8, colour as u8] })
}

/// ZC_PROGRESS: for how many milliseconds the bar fills. (The colour is in the packet and the client does
/// nothing with it: its gauge has colours of its own, `UIGage::UIGage`.)
pub fn progress(packet: &[u8]) -> Option<u32> {
    Some(long(packet, 6)?.saturating_mul(1000))
}

/// ZC_PROGRESS_ACTOR: over whom, and for how many milliseconds.
pub fn progress_of(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 10)?.saturating_mul(1000)))
}

/// CZ_PROGRESS (0x02f1, 2 bytes): the bar over the player's character is full, or was taken away. The script
/// goes on only once the server has this (`clif_parse_progressbar`).
pub fn progress_done() -> Vec<u8> {
    0x02f1u16.to_le_bytes().to_vec()
}

/// What a ZC_SOUND asks for.
#[derive(Debug, PartialEq)]
pub struct Sound {
    /// The file, under the client's `wav` folder.
    pub name: String,
    /// 0 plays it once, 1 plays it again and again, 2 ends that.
    pub act: u8,
    /// How long between two plays of one that repeats, in milliseconds.
    pub term: u32,
    /// From whom it sounds.
    pub who: u32,
}

/// ZC_SOUND.
pub fn sound(packet: &[u8]) -> Option<Sound> {
    Some(Sound { name: text_at(packet, 2, 24)?, act: *packet.get(26)?, term: long(packet, 27)?, who: long(packet, 31)? })
}

/// ZC_PLAY_NPC_BGM (0x0b8c): the kind (0 plays the piece over and over, 1 plays it once, 2 goes back to the
/// map's music: `CGameMode::PlayBgm`) and the piece's name, a file of `bgm\` without its `.mp3`.
pub fn music(packet: &[u8]) -> Option<(u8, String)> {
    Some((*packet.get(4)?, text(packet.get(5..)?)))
}

/// ZC_SHOWSCRIPT: who says it, and the words.
pub fn shown_words(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 4)?, text(packet.get(8..)?)))
}

/// ZC_CLEAR_EFFECT: on whom, and which effect.
pub fn cleared_effect(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(name: &str, kind: u8) -> Vec<u8> {
        let mut packet = vec![0xb3, 0x01];
        packet.extend(name.bytes().chain(std::iter::repeat(0)).take(64));
        packet.push(kind);
        packet
    }

    #[test]
    fn reads_a_picture_to_show_and_names_its_file_as_the_client_does() {
        assert_eq!(cutin(ZC_SHOW_IMAGE2, &image("Kafra_01", 2)), Some(Cutin::Show("kafra_01.bmp".to_owned(), Stand::Right)));
        assert_eq!(cutin(ZC_SHOW_IMAGE2, &image("ein_soldier.bmp", 0)), Some(Cutin::Show("ein_soldier.bmp".to_owned(), Stand::Left)));
        assert_eq!(cutin(ZC_SHOW_IMAGE2, &image("", 255)), Some(Cutin::Away));
        assert_eq!(cutin(ZC_SHOW_IMAGE2, &image("x", 12)), None);
        let mut old = vec![0x45, 0x01];
        old.extend(b"verkana\0\0\0\0\0\0\0\0\0");
        old.push(4);
        assert_eq!(cutin(ZC_SHOW_IMAGE, &old), Some(Cutin::Show("verkana.bmp".to_owned(), Stand::Centre)));
        for packet in [image("a", 0), old] {
            assert_eq!(crate::net::packet_length(u16::from_le_bytes([packet[0], packet[1]])), Some(packet.len() as i32));
        }
    }

    #[test]
    fn reads_a_mark_for_the_minimap() {
        let mut packet = vec![0x44, 0x01];
        for value in [7u32, 1, 120, 85] {
            packet.extend(value.to_le_bytes());
        }
        packet.push(3);
        packet.extend(0x00ff8000u32.to_le_bytes());
        assert_eq!(crate::net::packet_length(ZC_COMPASS), Some(packet.len() as i32));
        assert_eq!(compass(&packet), Some(Compass { id: 3, kind: 1, cell: (120, 85), colour: [255, 128, 0] }));
        assert_eq!(compass(&packet[..20]), None);
    }

    #[test]
    fn reads_the_bars_and_answers_a_full_one() {
        let mut own = vec![0xf0, 0x02];
        own.extend(0x00ff00u32.to_le_bytes());
        own.extend(5u32.to_le_bytes());
        assert_eq!(progress(&own), Some(5000));
        let mut other = vec![0xd1, 0x09];
        for value in [110_000_001u32, 0xffffff, 3] {
            other.extend(value.to_le_bytes());
        }
        assert_eq!(progress_of(&other), Some((110_000_001, 3000)));
        assert_eq!(progress_done(), [0xf1, 0x02]);
        for (id, length) in [(ZC_PROGRESS, own.len()), (ZC_PROGRESS_ACTOR, other.len()), (0x02f1, 2), (ZC_PROGRESS_CANCEL, 2)] {
            assert_eq!(crate::net::packet_length(id), Some(length as i32), "{id:#06x}");
        }
    }

    #[test]
    fn reads_a_sound_a_piece_of_music_and_words_over_a_head() {
        let mut packet = vec![0xd3, 0x01];
        packet.extend(b"effect\\ef_bash.wav\0\0\0\0\0\0");
        packet.push(1);
        packet.extend(2000u32.to_le_bytes());
        packet.extend(9u32.to_le_bytes());
        assert_eq!(crate::net::packet_length(ZC_SOUND), Some(packet.len() as i32));
        assert_eq!(sound(&packet), Some(Sound { name: "effect\\ef_bash.wav".to_owned(), act: 1, term: 2000, who: 9 }));
        assert_eq!(music(&[0x8c, 0x0b, 8, 0, 1, b'0', b'8', 0]), Some((1, "08".to_owned())));
        let mut words = vec![0xb3, 0x08, 12, 0, 9, 0, 0, 0];
        words.extend(b"Hey\0");
        assert_eq!(shown_words(&words), Some((9, "Hey".to_owned())));
        assert_eq!(cleared_effect(&[0x0d, 0x0b, 9, 0, 0, 0, 91, 1, 0, 0]), Some((9, 347)));
    }
}
