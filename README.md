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
- Everyone on the map: NPCs with their dialogs, shops and storage, monsters, other players, loot.
- Fights: attacking with chase, bows with arrows in flight, skills aimed with the pointer with their cast bar and
  effects, damage numbers, death and resurrection, taming a monster and hatching its egg.
- In-game windows of the original client: Basic Information, Status, Inventory, Equipment, the skill tree, item
  and skill descriptions, the hotkey bar, chat, the minimap.
- Sound: the map's music and the game's sound effects.
- English and Russian (the interface; items and skills are translated in batches, see `translations/`).

Not there yet: parties and guilds, trading between players, pets' own windows, second-job skills' effects, the
world map and quests.

**New to the code? Start with [ARCHITECTURE.md](ARCHITECTURE.md).**

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

# 2. rAthena (PACKETVER 20260219, the classic pre-renewal rules) and MariaDB.
docker compose -f server/docker-compose.yml up -d --build

# 3. The browser client, into client/dist.
./build-web.sh

# 4. The gateway. Then open http://localhost:8080
./target/release/roweb-gateway
```

`server/conf/accounts.sql` seeds two accounts for development (`tester`, a game master, and `tester2`). They
are for a server on your own machine only.

What the gateway sends is compressed (gzip or Brotli, as the browser takes) and the client's own files, named by
their build, are kept by the browser. Both can be switched off: `--compress off` where a proxy or a CDN in front
does the compressing, `--cache-client off` to have the client asked for anew each time.

To see walking and fighting as a player far away has them, the gateway can hold back everything that goes
between the browser and the game server: `--lag 150 --jitter 100` delays each way by 150 milliseconds and a
random part of up to 100 more, keeping the order as a TCP connection would. It is a debug option, off unless
given, and delays everyone who plays through that gateway: never give it to the gateway others play through
(it does not start with `--invite-file` or `--behind-proxy`, which `public.sh` gives).

The server plays by the classic (pre-renewal) rules unless told otherwise. `./rules.sh renewal` rebuilds and
restarts it for Renewal (third and fourth jobs, their monsters, items, skills and NPCs), and `./rules.sh classic`
goes back; restart the gateway after either. The choice is kept in `server/.env`, which the gateway reads too for
the item database that goes with the rules. Each set of rules has its own database of characters.

rAthena's classic scripts place the NPCs and portals of Izlude, Alberta and Morroc by their old maps, and the
client has the new ones. `server/npc/roweb_classic.txt` moves them, and `roweb_classic_warps.sed` puts right the
cells the scripts themselves send players to; both are written by `tools/client-tables/classic_overlay.py
../rathena server/npc/roweb_classic.txt server/npc/roweb_classic_warps.sed`. Under the Renewal rules both do
nothing. `tools/client-tables/placement_check.py` lists who stands in a
wall on any map.

E-mails of the account service go to a development mailbox at http://127.0.0.1:8025; point the gateway's
`--smtp-url` at a real server for anything else.

## Working on the client

```sh
./dev.sh           # builds the client again at every save
```

Open the page as `http://localhost:8080/?dev` and it loads itself anew when a new build is there. That is a new
page, so the game is left and logged into again: a change is not put into a running game, whose state lives in
the program being replaced. The gateway is not watched; start it again after changing it.

## Letting friends in

```sh
./public.sh        # the gateway behind an invitation word, and a Cloudflare quick tunnel to it
```

It prints an address on `trycloudflare.com` that lasts until it is stopped, and is new every time; `cloudflared`
must be installed. Nothing is served to a browser that has not given the invitation word
(`server/public/invite.txt`, made on the first run and not committed): not the page, not the client's files, not
the bridge to the game server. Friends make their own accounts on the login page.

Before the first time, change the passwords of the test accounts of `server/conf/accounts.sql` in the database:
they are written in this repository. This is for a handful of invited people. The game's files are Gravity's;
serving them to the public is not what this is for.

## Tests, and before a commit

```sh
./check.sh                                   # all of it: tests, the browser build with no warnings, sizes
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
