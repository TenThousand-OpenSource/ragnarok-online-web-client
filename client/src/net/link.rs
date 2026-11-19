//! The connection itself, as packets: the two that tell a server and the client that the other is still there.
//! The client's watch on a connection gone silent, and why a server ends one, are `crate::link`.

/// CZ_PING_LIVE (0x0b1c, 2 bytes): the client is still there. The server answers with ZC_PING_LIVE.
pub const CZ_PING_LIVE: u16 = 0x0b1c;
/// ZC_PING_LIVE (0x0b1d, 2 bytes): the server is still there, in answer or by itself. The client does nothing
/// with it: that it came is enough.
pub const ZC_PING_LIVE: u16 = 0x0b1d;

pub fn ping_live() -> Vec<u8> {
    super::bytes::Writer::new(CZ_PING_LIVE).done()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_client_says_it_is_there() {
        assert_eq!(ping_live(), vec![0x1c, 0x0b]);
    }
}
