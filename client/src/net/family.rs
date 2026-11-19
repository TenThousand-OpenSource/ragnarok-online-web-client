//! The family: what the server asks and tells of a wedding, a divorce and an adoption. The layouts are
//! rAthena's for PACKETVER 20260219 (`clif_wedding_effect`, `clif_divorced`, `clif_Adopt_request`,
//! `clif_Adopt_reply` in `src/map/clif.cpp`); what the client does with each is `CGameMode`'s
//! (`GameModePackets.cpp`, `GameModeNotices.cpp`) and its `UIBabyAcceptWnd`.

use super::bytes::{Writer, long, text_at};

/// ZC_CONGRATULATION (0x01ea, 6 bytes): `u32 who`: the wedding's confetti falls on that one.
pub const ZC_CONGRATULATION: u16 = 0x01ea;
/// ZC_COUPLENAME (0x01e6, 26 bytes): `char name[24]`: what the player's partner is called.
pub const ZC_COUPLENAME: u16 = 0x01e6;
/// ZC_DIVORCE (0x0205, 26 bytes): `char name[24]`: the marriage with that one is over.
pub const ZC_DIVORCE: u16 = 0x0205;
/// ZC_REQ_BABY (0x01f6, 34 bytes): `u32 account, u32 character, char name[24]`: that one asks to adopt the
/// player's character.
pub const ZC_REQ_BABY: u16 = 0x01f6;
/// ZC_BABYMSG (0x0216, 6 bytes): `u32 why`: why an adoption cannot be.
pub const ZC_BABYMSG: u16 = 0x0216;

/// The effect of ZC_CONGRATULATION (`EF_COLORPAPER`).
pub const CONFETTI: u32 = 347;

/// ZC_CONGRATULATION: on whom.
pub fn congratulated(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// ZC_COUPLENAME and ZC_DIVORCE: the name.
pub fn partner(packet: &[u8]) -> Option<String> {
    text_at(packet, 2, 24)
}

/// Who asks to adopt: the account, the character, and the name.
#[derive(Debug, Clone, PartialEq)]
pub struct Parent {
    pub account: u32,
    pub character: u32,
    pub name: String,
}

/// ZC_REQ_BABY.
pub fn adoption_asked(packet: &[u8]) -> Option<Parent> {
    Some(Parent { account: long(packet, 2)?, character: long(packet, 6)?, name: text_at(packet, 10, 24)? })
}

/// ZC_BABYMSG: the line of the client's message table for it, and the words until the table is read
/// (`ReceiveBabyMessage`: 0, 1 and 2).
pub fn adoption_refused(packet: &[u8]) -> Option<(usize, &'static str)> {
    crate::family::adoption_refusal(long(packet, 2)?)
}

/// CZ_JOIN_BABY (0x01f7, 14 bytes): `u32 account, u32 character, u32 answer`: the answer to the one who asked
/// (`MM_JOIN_BABY`): 1 to be adopted, 0 not to be.
pub fn adoption_answer(parent: &Parent, accepted: bool) -> Vec<u8> {
    Writer::new(0x01f7).u32(parent.account).u32(parent.character).u32(u32::from(accepted)).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::divorce_line;
    use crate::net::bytes::name24;

    #[test]
    fn reads_who_asks_to_adopt_and_answers() {
        let mut packet = vec![0xf6, 0x01, 1, 0, 0, 0, 2, 0, 0, 0];
        packet.extend(name24("Mother"));
        assert_eq!(crate::net::packet_length(ZC_REQ_BABY), Some(packet.len() as i32));
        let parent = adoption_asked(&packet).unwrap();
        assert_eq!(parent, Parent { account: 1, character: 2, name: "Mother".to_owned() });
        assert_eq!(adoption_answer(&parent, true), [0xf7, 0x01, 1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(crate::net::packet_length(0x01f7), Some(14));
    }

    #[test]
    fn reads_the_notices() {
        let mut packet = vec![0x05, 0x02];
        packet.extend(name24("Partner"));
        assert_eq!(partner(&packet).as_deref(), Some("Partner"));
        assert_eq!(divorce_line("%s has divorced %s.", "Hero", "Partner"), "Hero has divorced Partner.");
        assert_eq!(congratulated(&[0xea, 0x01, 7, 0, 0, 0]), Some(7));
        assert_eq!([0u8, 1, 2, 3].map(|why| adoption_refused(&[0x16, 0x02, why, 0, 0, 0]).map(|said| said.0)), [Some(899), Some(900), Some(916), None]);
        for (id, length) in [(ZC_CONGRATULATION, 6), (ZC_COUPLENAME, 26), (ZC_DIVORCE, 26), (ZC_BABYMSG, 6)] {
            assert_eq!(crate::net::packet_length(id), Some(length), "{id:#06x}");
        }
    }
}
