//! The bridge against a real server. These need the local rAthena of `server/docker-compose.yml` and are skipped
//! by default: `cargo test -p roweb-gateway -- --ignored`.
//!
//! Against the real rAthena of server/docker-compose.yml (PACKETVER 20260219), with the second seeded account so a
//! player logged in as the first one is not disturbed (a login kicks whoever is online on the same account).

mod common;

use common::*;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

const LOGIN_PORT: u16 = 6900;
const CHAR_PORT: u16 = 6121;
const MAP_PORT: u16 = 5121;
const ACCOUNT_ID: u32 = 2_000_001;

/// Reads one variable-length packet (`u16 id, u16 length, ...`) whole.
async fn read_packet(socket: &mut Socket) -> Vec<u8> {
    let mut packet = next_bytes(socket).await;
    while packet.len() < 4 || packet.len() < usize::from(u16_at(&packet, 2)) {
        packet.extend(next_bytes(socket).await);
    }
    packet
}

#[tokio::test]
#[ignore = "needs the local rAthena"]
async fn live_rathena_refuses_a_wrong_password() {
    let gateway = gateway("127.0.0.1", vec![LOGIN_PORT]).await;
    let mut login = open(&gateway, LOGIN_PORT).await;
    login.send(Message::Binary(login_request("tester2", "wrong-password").into())).await.unwrap();

    // AC_REFUSE_LOGIN (0x083e): `u16 id, u32 error, char[20] unblock_time`; error 1 is "incorrect password".
    let reply = read_exactly(&mut login, 26).await;
    assert_eq!(u16_at(&reply, 0), 0x083e, "{reply:02x?}");
    assert_eq!(u32_at(&reply, 2), 1);
}

/// The whole way in, as the client goes: login server, then the char server it names, over two bridged sockets.
#[tokio::test]
#[ignore = "needs the local rAthena"]
async fn live_rathena_login_and_char_server() {
    let gateway = gateway("127.0.0.1", vec![LOGIN_PORT, CHAR_PORT, MAP_PORT]).await;

    let mut login = open(&gateway, LOGIN_PORT).await;
    login.send(Message::Binary(login_request("tester2", "tester2").into())).await.unwrap();
    // AC_ACCEPT_LOGIN (0x0ac4): `u16 id, u16 length, u32 login_id1, u32 AID, u32 login_id2, u32 last_ip,
    // char[26] last_login, u8 sex, char[17] token`, then 160 bytes per char server (`u32 ip, u16 port, char[20] name, ...`).
    let accept = read_packet(&mut login).await;
    assert_eq!(u16_at(&accept, 0), 0x0ac4, "{:02x?}", &accept[..accept.len().min(32)]);
    assert_eq!(accept.len(), usize::from(u16_at(&accept, 2)));
    let (login_id1, account_id, login_id2, sex) = (u32_at(&accept, 4), u32_at(&accept, 8), u32_at(&accept, 12), accept[46]);
    assert_eq!(account_id, ACCOUNT_ID);
    let servers = &accept[64..];
    assert_eq!(servers.len() % 160, 0, "char server list of {} bytes", servers.len());
    assert!(!servers.is_empty(), "the login server lists no char server (is char-server connected?)");
    assert_eq!(servers[..4], [127, 0, 0, 1]);
    let char_port = u16_at(servers, 4);
    assert_eq!(char_port, CHAR_PORT);
    drop(login);

    // CH_ENTER (0x0065): `u16 id, u32 AID, u32 login_id1, u32 login_id2, u16 unused, u8 sex`.
    let mut char_server = open(&gateway, char_port).await;
    let mut enter = vec![0u8; 17];
    enter[..2].copy_from_slice(&0x0065u16.to_le_bytes());
    enter[2..6].copy_from_slice(&account_id.to_le_bytes());
    enter[6..10].copy_from_slice(&login_id1.to_le_bytes());
    enter[10..14].copy_from_slice(&login_id2.to_le_bytes());
    enter[16] = sex;
    char_server.send(Message::Binary(enter.into())).await.unwrap();

    // The char server answers with the bare account ID, then its packets (slot info first: HC_ACCEPT_ENTER2, 0x082d).
    let mut reply = Vec::new();
    while reply.len() < 6 {
        reply.extend(next_bytes(&mut char_server).await);
    }
    assert_eq!(u32_at(&reply, 0), ACCOUNT_ID, "{:02x?}", &reply[..reply.len().min(32)]);
    assert_eq!(u16_at(&reply, 4), 0x082d, "{:02x?}", &reply[..reply.len().min(32)]);
}
