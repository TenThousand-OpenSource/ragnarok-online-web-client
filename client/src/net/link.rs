//! The connection itself: the client's watch on a connection gone silent, and why a server ends one.
//! The watch is `CPingLiveSchedule::Process`, run every five seconds while the client is in the game; the
//! reasons are `CGameMode::ReceiveNotifyBan` and `CLoginMode::ReceiveNotifyBan`.

/// CZ_PING_LIVE (0x0b1c, 2 bytes): the client is still there. The server answers with ZC_PING_LIVE.
pub const CZ_PING_LIVE: u16 = 0x0b1c;
/// ZC_PING_LIVE (0x0b1d, 2 bytes): the server is still there, in answer or by itself. The client does nothing
/// with it: that it came is enough.
pub const ZC_PING_LIVE: u16 = 0x0b1d;

/// How often the watch looks, and the two waits it goes by, in milliseconds.
pub const WATCH_EVERY: i32 = 5_000;
const SILENT_TOO_LONG: f64 = 60_000.0;
const SAY_SOMETHING: f64 = 10_000.0;

pub fn ping_live() -> Vec<u8> {
    super::bytes::Writer::new(CZ_PING_LIVE).done()
}

/// What the watch does when it looks.
#[derive(Debug, PartialEq)]
pub enum Watch {
    /// Nothing came for more than a minute: the connection is closed and the player told.
    Close,
    /// Nothing was sent for more than ten seconds: CZ_PING_LIVE is.
    Ping,
    Wait,
}

/// The watch at a moment, by when something last came and was last sent (all in milliseconds).
pub fn watch(now: f64, heard_at: f64, sent_at: f64) -> Watch {
    if heard_at + SILENT_TOO_LONG < now {
        Watch::Close
    } else if sent_at + SAY_SOMETHING < now {
        Watch::Ping
    } else {
        Watch::Wait
    }
}

/// Why a server ended the connection (SC_NOTIFY_BAN), by the reason's number: the line of the client's
/// message table that tells it, and whether the number follows the line in brackets. In the game
/// (`CGameMode::ReceiveNotifyBan`) it does, and a number the client does not know is told with the line of
/// number 0; number 15 has a line only on the Taiwanese service, and elsewhere the line for a reason not
/// known (`MSI_RESULT_FAIL_UNKNOWN`) stands alone. On the way in (`CLoginMode::ReceiveNotifyBan`) every line
/// stands alone, and more numbers have a line of their own.
pub fn ban_line(reason: u8, in_game: bool) -> (usize, bool) {
    let line = match (reason, in_game) {
        (0, _) => 3,
        (1, _) => 4,
        (2, _) => 5,
        (3, _) => 241,
        (4, _) => 264,
        (5, _) => 305,
        (9, _) => 529,
        (10, _) => 530,
        (110, _) => 1589,
        (113, _) => 2021,
        (15, true) => return (1816, false),
        (115, true) => 2347,
        (6, false) => 438,
        (7, false) => 439,
        (8, false) => 440,
        (11, false) => 575,
        (12, false) => 576,
        (13, false) => 577,
        (14, false) => 578,
        (16, false) => 606,
        (17, false) => 607,
        (18, false) => 678,
        (100, false) => 1123,
        (101, false) => 1178,
        (102, false) => 1179,
        (103, false) => 1309,
        (104, false) => 1310,
        (105, false) => 1311,
        (108, false) => 1582,
        (109, false) => 1583,
        (114, false) => 2344,
        (116, false) => 3635,
        _ => 3,
    };
    (line, in_game)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_watch_closes_a_silent_connection_and_speaks_on_a_quiet_one() {
        assert_eq!(watch(100_000.0, 39_999.0, 99_000.0), Watch::Close);
        assert_eq!(watch(100_000.0, 40_000.0, 89_999.0), Watch::Ping);
        assert_eq!(watch(100_000.0, 40_000.0, 90_000.0), Watch::Wait);
        // A silent connection is closed whatever was sent.
        assert_eq!(watch(100_000.0, 0.0, 0.0), Watch::Close);
        assert_eq!(ping_live(), vec![0x1c, 0x0b]);
    }

    #[test]
    fn a_reason_has_its_line() {
        assert_eq!((ban_line(1, true), ban_line(2, true), ban_line(3, true)), ((4, true), (5, true), (241, true)));
        // The game tells of a reason it does not know with the first line; the way in has more lines.
        assert_eq!((ban_line(8, true), ban_line(8, false)), ((3, true), (440, false)));
        assert_eq!((ban_line(115, true), ban_line(115, false)), ((2347, true), (3, false)));
        assert_eq!((ban_line(15, true), ban_line(200, false)), ((1816, false), (3, false)));
    }
}
