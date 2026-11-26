//! The attendance check: asking for its window, the server's opening of it, claiming today's reward and the
//! answer. The layouts are rAthena's (`clif_parse_open_ui`, `clif_ui_open`, `clif_parse_attendance_request`,
//! `clif_attendence_response` in `src/map/clif.cpp`); the client's side is `UIMenuIconWnd::SendMsg`
//! (`MENUICON_ATTENDANCE`), `CGameMode::ReceiveServiceRequest`, `UICheckAttendanceWnd::SendMsg` and
//! `CGameMode::ReceiveWindow328Result`.

use super::bytes::{Writer, int};

/// ZC_UI_OPEN (0x0ae2, 7 bytes): `u8 window`, `i32 value`. The server opens a window of the client's; the
/// attendance check is window 7, and its value is ten times the days attended, with 1 added once today was.
pub const ZC_UI_OPEN: u16 = 0x0ae2;
/// The window of ZC_UI_OPEN that is the attendance check (the client's `OUT_UI_ATTENDANCE`).
pub const OUT_ATTENDANCE: u8 = 7;
/// ZC_ACK_CHECK_ATTENDANCE (0x0af0, 10 bytes): `i32 result`, `i32 days`.
pub const ZC_ACK_CHECK_ATTENDANCE: u16 = 0x0af0;

/// The window and the value of a ZC_UI_OPEN.
pub fn opened(packet: &[u8]) -> Option<(u8, i32)> {
    Some((*packet.get(2)?, int(packet, 3)?))
}

/// The result (0 is done, 1 failed) and the days attended of a ZC_ACK_CHECK_ATTENDANCE.
pub fn answered(packet: &[u8]) -> Option<(i32, i32)> {
    Some((int(packet, 2)?, int(packet, 6)?))
}

/// CZ_UI_OPEN (0x0a68, 3 bytes): `u8 window`, here 5, the attendance check (the client's `IN_UI_ATTENDANCE`):
/// the menu's icon asks the server to open the window.
pub fn open() -> Vec<u8> {
    Writer::new(0x0a68).u8(5).done()
}

/// CZ_REQ_CHECK_ATTENDANCE (0x0aef, 2 bytes): today's reward is claimed.
pub fn claim() -> Vec<u8> {
    Writer::new(0x0aef).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing;

    #[test]
    fn reads_a_window_the_server_opens() {
        let packet = testing::packet(ZC_UI_OPEN, &[&[OUT_ATTENDANCE], &31i32.to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_UI_OPEN), Some(packet.len() as i32));
        assert_eq!(opened(&packet), Some((7, 31)));
        assert_eq!(opened(&packet[..6]), None);
    }

    #[test]
    fn reads_the_answer_to_a_claim() {
        let packet = testing::packet(ZC_ACK_CHECK_ATTENDANCE, &[&0i32.to_le_bytes(), &4i32.to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_ACK_CHECK_ATTENDANCE), Some(packet.len() as i32));
        assert_eq!(answered(&packet), Some((0, 4)));
        assert_eq!(answered(&packet[..9]), None);
    }

    #[test]
    fn what_is_sent_is_as_long_as_the_server_expects() {
        testing::assert_lengths(&[open(), claim()]);
    }
}
