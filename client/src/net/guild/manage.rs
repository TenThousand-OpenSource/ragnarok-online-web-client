//! What the master of a guild does with it: the positions and who has which, the emblem, the guilds allied and
//! opposed, and the packets that answer. The layouts are rAthena's for PACKETVER 20260219
//! (`clif_parse_GuildChangePositionInfo`, `clif_parse_GuildChangeMemberPosition`, `clif_parse_GuildChangeEmblem`,
//! `clif_parse_GuildRequestAlliance`, `clif_parse_GuildOpposition`, `clif_parse_GuildDelAlliance` in `clif.cpp`).

use super::{Position, Writer, long, text_at};

/// ZC_REQ_ALLY_GUILD (0x0171, 30 bytes): `u32 account, char guild[24]`: another guild's master asks for an
/// alliance.
pub const ZC_REQ_ALLY_GUILD: u16 = 0x0171;
/// ZC_ACK_REQ_ALLY_GUILD (0x0173, 3 bytes): `u8 answer`: what became of an alliance asked for (`ally`).
pub const ZC_ACK_REQ_ALLY_GUILD: u16 = 0x0173;
/// ZC_ACK_REQ_HOSTILE_GUILD (0x0181, 3 bytes): `u8 result`: what became of opposing a guild (`oppose`).
pub const ZC_ACK_REQ_HOSTILE_GUILD: u16 = 0x0181;
/// ZC_DELETE_RELATED_GUILD (0x0184, 10 bytes): `u32 guild, u32 relation`: a guild is allied or opposed no more.
pub const ZC_DELETE_RELATED_GUILD: u16 = 0x0184;
/// ZC_ADD_RELATED_GUILD (0x0185, 34 bytes): `u32 relation, u32 guild, char name[24]`: a guild is allied (0) or
/// opposed (1) now. (rAthena tells the whole list anew with ZC_MYGUILD_BASIC_INFO and never sends this.)
pub const ZC_ADD_RELATED_GUILD: u16 = 0x0185;

/// What ZC_ACK_REQ_ALLY_GUILD answers (the client's `REQALLYGUILD_*`).
pub mod ally {
    pub const ALREADY: u8 = 0;
    pub const REJECTED: u8 = 1;
    pub const ACCEPTED: u8 = 2;
    /// The other guild has too many alliances.
    pub const THEIRS_FULL: u8 = 3;
    pub const OURS_FULL: u8 = 4;
    pub const DISABLED: u8 = 5;
}

/// What ZC_ACK_REQ_HOSTILE_GUILD answers (the client's `RESULT_HOSTILE_*`).
pub mod oppose {
    pub const DONE: u8 = 0;
    pub const FULL: u8 = 1;
    pub const ALREADY: u8 = 2;
    pub const DISABLED: u8 = 3;
}

/// ZC_REQ_ALLY_GUILD: the account of the master who asks, and the guild's name.
pub fn ally_asked(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 2)?, text_at(packet, 6, 24)?))
}

/// ZC_DELETE_RELATED_GUILD: the guild, and whether it was opposed.
pub fn unrelated(packet: &[u8]) -> Option<(u32, bool)> {
    Some((long(packet, 2)?, long(packet, 6)? != 0))
}

/// ZC_ADD_RELATED_GUILD: the guild, its name, and whether it is opposed.
pub fn related_added(packet: &[u8]) -> Option<(u32, String, bool)> {
    Some((long(packet, 6)?, text_at(packet, 10, 24)?, long(packet, 2)? != 0))
}

/// CZ_REQ_ALLY_GUILD (0x0170, 14 bytes): `u32 account, u32 own_account, u32 own_character`: asks the master in
/// sight for an alliance.
pub fn ask_ally(account: u32, own_account: u32, own_character: u32) -> Vec<u8> {
    Writer::new(0x0170).u32(account).u32(own_account).u32(own_character).done()
}

/// CZ_ALLY_GUILD (0x0172, 10 bytes): `u32 account, u32 answer`: the alliance that master asked for is taken (1)
/// or not (0).
pub fn answer_ally(account: u32, accept: bool) -> Vec<u8> {
    Writer::new(0x0172).u32(account).u32(u32::from(accept)).done()
}

/// CZ_REQ_HOSTILE_GUILD (0x0180, 6 bytes): `u32 account`: the guild of that player in sight is opposed.
pub fn oppose(account: u32) -> Vec<u8> {
    Writer::new(0x0180).u32(account).done()
}

/// CZ_REQ_DELETE_RELATED_GUILD (0x0183, 10 bytes): `u32 guild, u32 relation`: an alliance (0) or an opposition
/// (1) is ended.
pub fn unrelate(guild: u32, opposed: bool) -> Vec<u8> {
    Writer::new(0x0183).u32(guild).u32(u32::from(opposed)).done()
}

/// CZ_REG_CHANGE_GUILD_POSITIONINFO (0x0161): `u16 length`, then `u32 position, u32 mode, u32 ranking, u32 tax,
/// char name[24]` each, as the client writes them (`MM_REQ_CHANGE_GUILD_POSITIONINFO`: the ranking is the
/// position's own number, the name 23 letters at most).
pub fn change_positions(positions: &[Position]) -> Vec<u8> {
    let mut packet = Writer::sized(0x0161);
    for position in positions {
        let mode = u32::from(position.invites) | u32::from(position.expels) << 4 | u32::from(position.storage) << 8;
        packet = packet.u32(position.id).u32(mode).u32(position.id).u32(position.tax).field_whole_chars::<24>(&position.name);
    }
    packet.done()
}

/// CZ_REQ_CHANGE_MEMBERPOS (0x0155): `u16 length`, then `u32 account, u32 character, u32 position` each. The
/// first position (0) hands the guild to that member (rAthena's `guild_gm_change`).
pub fn move_members(moves: &[(u32, u32, u32)]) -> Vec<u8> {
    let mut packet = Writer::sized(0x0155);
    for (account, character, position) in moves {
        packet = packet.u32(*account).u32(*character).u32(*position);
    }
    packet.done()
}

/// Why a file is not taken as an emblem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmblemFault {
    /// Not a BMP of 24 by 24 that the server reads, or too large packed (the client's line 3587).
    Unfit,
    /// More of it is see-through than the server lets through (rAthena's `emblem_transparency_limit`).
    Transparent,
}

/// The part of an emblem that may be see-through, in hundredths, as rAthena has it unless its
/// `inter_athena.conf` says otherwise (`emblem_transparency_limit: 80`).
pub const TRANSPARENCY_LIMIT: usize = 80;

/// How rAthena counts the see-through part of an emblem (`clif_validate_emblem`, written as it is there: from
/// where the pixels begin it reads four bytes at every byte and counts a place where three such in a row are
/// magenta followed by magenta). `true` when the count is past the limit.
fn too_transparent(bitmap: &[u8], pixels_at: usize, limit: usize) -> bool {
    let at = |index: usize| u32::from_le_bytes(std::array::from_fn(|byte| bitmap.get(index + byte).copied().unwrap_or(0)));
    let (mut count, mut seen) = (1usize, [0u32; 3]);
    for index in pixels_at..bitmap.len().saturating_sub(1) {
        seen[index % 3] = at(index);
        if index % 3 == 2 && seen == [0xFFFF_00FF, 0x00FF_FF00, 0xFF00_FFFF] {
            count += 1;
        }
    }
    count * 300 / (bitmap.len() - pixels_at).max(1) > limit
}

/// A file picked as the guild's emblem, packed for CZ_REGISTER_GUILD_EMBLEM_IMG. The client takes a picture of
/// 24 by 24 (`UIGuildInfoManageWnd`, `UM_COMBOBOX_SELECTED`); rAthena takes a BMP whose header tells its own
/// size, no more than 1800 bytes (so 8 or 24 bits a pixel), and not too see-through (`clif_validate_emblem`); the
/// packet holds 2048 bytes in all (`MM_SEND_GUILD_EMBLEM`).
pub fn emblem_packed(file: &[u8]) -> Result<Vec<u8>, EmblemFault> {
    let picture = crate::bmp::decode(file).map_err(|_| EmblemFault::Unfit)?;
    let pixels_at = long(file, 10).ok_or(EmblemFault::Unfit)? as usize;
    if (picture.width, picture.height) != (24, 24) || file.len() > 1800 || long(file, 2) != Some(file.len() as u32) || pixels_at >= file.len() {
        return Err(EmblemFault::Unfit);
    }
    if too_transparent(file, pixels_at, TRANSPARENCY_LIMIT) {
        return Err(EmblemFault::Transparent);
    }
    let packed = miniz_oxide::deflate::compress_to_vec_zlib(file, 6);
    if packed.len() + 4 > 0x800 { Err(EmblemFault::Unfit) } else { Ok(packed) }
}

/// CZ_REGISTER_GUILD_EMBLEM_IMG (0x0153): `u16 length`, then the emblem: a BMP packed with zlib.
pub fn set_emblem(packed: &[u8]) -> Vec<u8> {
    Writer::sized(0x0153).bytes(packed).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::field;

    /// A BMP of 24 by 24 at 24 bits, every pixel of one colour (blue first).
    fn bitmap(pixel: [u8; 3]) -> Vec<u8> {
        let mut file = vec![0u8; 54];
        file[..2].copy_from_slice(b"BM");
        file[2..6].copy_from_slice(&1782u32.to_le_bytes());
        file[10..14].copy_from_slice(&54u32.to_le_bytes());
        file[14..18].copy_from_slice(&40u32.to_le_bytes());
        file[18..22].copy_from_slice(&24u32.to_le_bytes());
        file[22..26].copy_from_slice(&24u32.to_le_bytes());
        file[26] = 1;
        file[28] = 24;
        file.extend(pixel.iter().cycle().take(24 * 24 * 3));
        file
    }

    #[test]
    fn the_relations_are_read_and_written() {
        let asked = [&[0x71, 0x01][..], &2_000_005u32.to_le_bytes(), &field("Owls", 24)].concat();
        assert_eq!(ally_asked(&asked), Some((2_000_005, "Owls".to_owned())));
        assert_eq!(unrelated(&[0x84, 0x01, 7, 0, 0, 0, 1, 0, 0, 0]), Some((7, true)));
        let added = [&[0x85, 0x01][..], &[0, 0, 0, 0, 9, 0, 0, 0], &field("Owls", 24)].concat();
        assert_eq!(added.len(), 34);
        assert_eq!(related_added(&added), Some((9, "Owls".to_owned(), false)));
        assert_eq!(ask_ally(5, 6, 7), [0x70, 0x01, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0]);
        assert_eq!(answer_ally(5, true), [0x72, 0x01, 5, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(oppose(5), [0x80, 0x01, 5, 0, 0, 0]);
        assert_eq!(unrelate(9, true), [0x83, 0x01, 9, 0, 0, 0, 1, 0, 0, 0]);
    }

    #[test]
    fn the_positions_and_the_members_positions_are_written() {
        let packet = change_positions(&[Position { id: 3, name: "Guard".to_owned(), invites: true, expels: true, storage: true, tax: 25 }]);
        assert_eq!(packet.len(), 44);
        assert_eq!(&packet[..20], [0x61, 0x01, 44, 0, 3, 0, 0, 0, 0x11, 0x01, 0, 0, 3, 0, 0, 0, 25, 0, 0, 0]);
        assert_eq!(&packet[20..26], b"Guard\0");
        assert_eq!(move_members(&[(5, 6, 2)]), [0x55, 0x01, 16, 0, 5, 0, 0, 0, 6, 0, 0, 0, 2, 0, 0, 0]);
    }

    #[test]
    fn an_emblem_is_packed_as_the_server_unpacks_it() {
        let file = bitmap([0, 0, 255]);
        let packed = emblem_packed(&file).expect("a red square is an emblem");
        assert_eq!(miniz_oxide::inflate::decompress_to_vec_zlib(&packed).ok(), Some(file));
        assert!(super::super::emblem_picture(&packed).is_some());
        let packet = set_emblem(&packed);
        assert_eq!(usize::from(u16::from_le_bytes([packet[2], packet[3]])), packet.len());
    }

    #[test]
    fn what_is_no_emblem_is_refused() {
        assert_eq!(emblem_packed(b"GIF89a"), Err(EmblemFault::Unfit));
        // Of another size.
        let mut wide = bitmap([0, 0, 255]);
        wide[18] = 25;
        assert_eq!(emblem_packed(&wide), Err(EmblemFault::Unfit));
        // A header that tells another length than the file has.
        let mut short = bitmap([0, 0, 255]);
        short[2] = 0;
        assert_eq!(emblem_packed(&short), Err(EmblemFault::Unfit));
        // All magenta: all see-through.
        assert_eq!(emblem_packed(&bitmap([255, 0, 255])), Err(EmblemFault::Transparent));
    }
}
