//! The bank: asking for the balance, which is how the client opens its window, putting zeny in and taking it
//! out, and the server's answer to each. The layouts are rAthena's (`clif_bank_open`, `clif_bank_deposit`,
//! `clif_bank_withdraw` in `src/map/clif.cpp`); the client's readers are `CGameMode::ReceiveBankCheck`,
//! `ReceiveBankDepositAck` and `ReceiveBankWithdrawAck`. The client of this version never sends
//! CZ_REQ_OPEN_BANKING nor CZ_REQ_CLOSE_BANKING, and takes their answers for nothing.

use super::bytes::{Writer, quad, int, short};

/// ZC_BANKING_CHECK (0x09a6, 12 bytes): `i64 balance`, `i16 reason`.
pub const ZC_BANKING_CHECK: u16 = 0x09a6;
/// ZC_ACK_BANKING_DEPOSIT (0x09a8, 16 bytes): `i16 reason`, `i64 balance`, `i32 zeny`.
pub const ZC_ACK_BANKING_DEPOSIT: u16 = 0x09a8;
/// ZC_ACK_BANKING_WITHDRAW (0x09aa, 16 bytes): the same.
pub const ZC_ACK_BANKING_WITHDRAW: u16 = 0x09aa;
/// ZC_ACK_OPEN_BANKING (0x09b7) and ZC_ACK_CLOSE_BANKING (0x09b9), 4 bytes each: the client reads neither.
pub const ZC_ACK_OPEN_BANKING: u16 = 0x09b7;
pub const ZC_ACK_CLOSE_BANKING: u16 = 0x09b9;

/// What the server answers to a deposit or a withdrawal.
#[derive(Debug, PartialEq)]
pub struct Moved {
    /// 0 when it was done; otherwise why not (`crate::bank::deposit_refused`, `withdrawal_refused`).
    pub reason: i16,
    /// What the bank holds now, and what the character carries.
    pub balance: i64,
    pub zeny: i32,
}

/// The reason and the balance of a ZC_BANKING_CHECK.
pub fn checked(packet: &[u8]) -> Option<(i16, i64)> {
    Some((short(packet, 10)?, quad(packet, 2)?))
}

/// A ZC_ACK_BANKING_DEPOSIT or a ZC_ACK_BANKING_WITHDRAW.
pub fn moved(packet: &[u8]) -> Option<Moved> {
    Some(Moved { reason: short(packet, 2)?, balance: quad(packet, 4)?, zeny: int(packet, 12)? })
}

/// CZ_REQ_BANKING_CHECK (0x09ab, 6 bytes): `u32 account`. The menu's icon and the key send it; the answer
/// opens the window, or closes it if it was open.
pub fn check(account: u32) -> Vec<u8> {
    Writer::new(0x09ab).u32(account).done()
}

/// CZ_REQ_BANKING_DEPOSIT (0x09a7, 10 bytes): `u32 account`, `i32 zeny`.
pub fn deposit(account: u32, zeny: i32) -> Vec<u8> {
    Writer::new(0x09a7).u32(account).i32(zeny).done()
}

/// CZ_REQ_BANKING_WITHDRAW (0x09a9, 10 bytes): `u32 account`, `i32 zeny`.
pub fn withdraw(account: u32, zeny: i32) -> Vec<u8> {
    Writer::new(0x09a9).u32(account).i32(zeny).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing;

    #[test]
    fn reads_the_balance_asked_for() {
        let packet = testing::packet(ZC_BANKING_CHECK, &[&5_000_000_000i64.to_le_bytes(), &0i16.to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_BANKING_CHECK), Some(packet.len() as i32));
        assert_eq!(checked(&packet), Some((0, 5_000_000_000)));
        assert_eq!(checked(&packet[..11]), None);
    }

    #[test]
    fn reads_what_a_deposit_and_a_withdrawal_came_to() {
        let packet = testing::packet(ZC_ACK_BANKING_DEPOSIT, &[&2i16.to_le_bytes(), &1500i64.to_le_bytes(), &250i32.to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_ACK_BANKING_DEPOSIT), Some(packet.len() as i32));
        assert_eq!(crate::net::packet_length(ZC_ACK_BANKING_WITHDRAW), Some(packet.len() as i32));
        assert_eq!(moved(&packet), Some(Moved { reason: 2, balance: 1500, zeny: 250 }));
        assert_eq!(moved(&packet[..15]), None);
    }

    #[test]
    fn what_is_sent_is_as_long_as_the_server_expects() {
        testing::assert_lengths(&[check(2_000_001), deposit(2_000_001, 1), withdraw(2_000_001, 1)]);
    }
}
