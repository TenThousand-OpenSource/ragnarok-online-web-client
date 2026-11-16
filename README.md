# ragnarok-online-web-client

A Ragnarok Online client that runs in the browser, written from scratch in Rust and compiled to WebAssembly.
Nothing to install for the player: open the page, log in, play. It talks to an ordinary
[rAthena](https://github.com/rathena/rathena) server.

It is work in progress. What is there today:

- Login, account registration and password recovery by e-mail, with limits on attempts.
- Character select, creation (human and Doram) and deletion, in the original client's windows, with the
  characters drawn from the game's sprites.
- The game world in 3D (WebGPU, WebGL2 fallback): a map's ground with its baked light, its models (buildings,
  trees), water; the original client's camera; walking by clicking, in step with the server; changing maps
  through portals.
- In-game windows of the original client: Basic Information, Status, Inventory, the skill list, the hotkey bar,
  chat, and the minimap.
- English and Russian.

Not there yet: other players, NPCs and monsters on the map, combat, equipment and the other windows, sound
effects in the world.

## No game data in this repository

The client draws the game from **your own** copy of the game's files (a kRO client folder with its `data.grf`).
None of Gravity's art, maps or music is in this repository, and none should be added to it.

## How it is put together

```
browser ── HTTP ──▶ gateway ──▶ GRF archives of your game client   (files, on demand)
        ── WebSocket ─▶ gateway ── TCP ──▶ rAthena login / char / map servers
```

- `client/` — the game client: Rust → WebAssembly. Protocol, file formats and path finding are plain Rust with
  unit tests; `client/src/web/` is what needs a browser.
- `gateway/` — the one native process beside rAthena. It serves game files out of the GRF archives, bridges
  WebSockets to the game servers' TCP ports (browsers cannot open TCP), keeps password guessing and connection
  floods in check, and runs the account service (registration, password recovery).
- `server/` — a Docker setup of rAthena and MariaDB for local development.
- `tools/packet-lengths/` — makes the client's table of packet lengths from rAthena's headers.

Players reach rAthena from the gateway's address, so rAthena's own per-address protections are switched off in
`server/conf/` and the gateway does that work with the players' real addresses. Do not expose rAthena's ports
to the outside: only the gateway should reach them.

## Running it locally

You need Rust (stable) with the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` of the version in
`Cargo.lock`, Docker, a checkout of rAthena, and a kRO client folder. By default these are looked for beside
this folder as `../rathena` and `../Feb-Client-CLEAN`; the gateway's `--client-dir` and `--item-database`
options, and the paths in `server/docker-compose.yml`, say otherwise.

```sh
# 1. The server's map cache, made from your client's maps, so the server's walls are the ones the client draws.
cargo build --release -p roweb-gateway
./target/release/roweb-gateway mapcache server/mapcache/map_cache.dat

# 2. rAthena (PACKETVER 20260219) and MariaDB.
docker compose -f server/docker-compose.yml up -d --build

# 3. The browser client, into client/dist.
./build-web.sh

# 4. The gateway. Then open http://localhost:8080
./target/release/roweb-gateway
```

`server/conf/accounts.sql` seeds two accounts for development (`tester`, a game master, and `tester2`). They
are for a server on your own machine only.

E-mails of the account service go to a development mailbox at http://127.0.0.1:8025; point the gateway's
`--smtp-url` at a real server for anything else.

## Tests

```sh
cargo test                                   # client and gateway
cargo test -p roweb-gateway -- --ignored     # against the running rAthena, database and mailbox
```

## Sources this builds on

- Protocol facts come from [rAthena](https://github.com/rathena/rathena)'s source.
- File formats and many layout numbers were checked against [korangar](https://github.com/vE5li/korangar) (MIT)
  and [rustnarok-client](https://github.com/kindraywind/rustnarok-client) (MIT OR Apache-2.0);
  `tools/packet-lengths/` is taken from the latter, with its licences beside it.
- Where the original client's behaviour had to be matched, it was read from a decompilation of that client.

Ragnarok Online is a trademark of Gravity Co., Ltd. This project is not affiliated with or endorsed by Gravity.
