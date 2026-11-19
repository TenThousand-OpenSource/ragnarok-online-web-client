# ragnarok-online-web-client

A Ragnarok Online client that runs in the browser, written from scratch in Rust and compiled to WebAssembly.
Nothing to install for the player: open the page, log in, play. It talks to an ordinary
[rAthena](https://github.com/rathena/rathena) server.

It is work in progress. What the client has today (each line is a system with its own files, under the same
name in `client/src/net/`, `client/src/web/app/`, `client/src/web/hud/` and `client/src/web/world/`):

- **The way in.** Login, account registration and password recovery by e-mail, with limits on attempts; the
  list of servers; character select, creation (human; the Doram where the server plays by Renewal's rules) and
  deletion, in the original client's windows, with the characters drawn from the game's sprites.
- **The world** in 3D (WebGPU, WebGL2 fallback): a map's ground with its baked light, its models, water, fog,
  the sky with its clouds and stars, the weather a map has (snow, sakura), the darkening of a night, the
  effects and sounds a map places; the original client's camera and mouse pointer; portals between maps.
- **Movement by the server's clock.** Walking by clicking, everyone's walk shown by the server's time and not
  by the moment its packet came (`client/src/server_time.rs`), so that it holds under lag.
- **Everyone on the map:** NPCs, monsters, other players, loot; job and mount sprites, carts, a hunter's
  falcon, a monk's spheres, the aura of the highest level, emblems and names, chat-room and shop signs.
- **Fights and skills:** attacking with chase, bows with arrows in flight, skills aimed with the pointer, cast
  bars, cooldowns, damage numbers, death and resurrection; the client's own effects for skills, blows, states
  and ailments; what skills leave on the ground (walls, songs, traps as models); the PVP standing.
- **The character's windows:** Basic Information with the menu icons, Status, Equipment (and looking at
  another player's), the skill tree with a skill's description, the hotkey bar, the list of states (buffs).
- **Items:** the inventory with its tabs and favourites, descriptions, cards and their illustrations, books
  that are read, identifying, repairing and refining lists, forging and brewing, arrow making, the lists a
  skill opens (Teleport, Auto Spell and the like).
- **NPCs and trade:** dialogs with choices, typed answers and pictures; NPC shops; the Kafra's storage and a
  guild's; the merchant's cart; vending (one's own shop and buying from another's).
- **Deals between players:** the exchange window with items and zeny.
- **Chat:** tabs and tabs taken out into windows, whisper windows, chat rooms, emotions with their Shortcut
  List, the server's announcements, links in text, and the client's chat commands
  (`client/src/commands.txt`; one this client does nothing for yet says so in the chat).
- **People:** party, guild (members, positions, emblem, notice, alliances, skills), friends, mail (RODEX),
  marriage and adoption, the right-click menu on a player.
- **Companions:** pets (taming, the egg, the pet window), the homunculus and the mercenary, both moved by the
  client's own AI scripts (`AI\AI.lua`, `AI\AI_M.lua` and the player's `USER_AI`) run in the browser.
- **Finding one's way:** the quest journal, the minimap with its marks, the large map window, the World Map,
  the Tipbox, the Adventure Guide.
- **Settings:** Game Settings, Shortcut Settings (every key of the client can be set), screenshots; windows
  are moved and dock to each other; options, keys, sound levels and the camera are kept by the browser.
- **Sound:** the map's music and the game's sound effects, as loud as their distance leaves them.
- **English and Russian** (the interface; items and skills are translated in batches, see `translations/`).

How far this can be trusted: most of it was made by reading the decompiled Windows client and has unit tests
for its rules, but only a part has been looked at in the running game so far. `docs/DONE.md` says for each
system what state it is in, `docs/DEBTS.md` lists the known deviations from the Windows client and the known
bugs, and [BACKLOG.md](BACKLOG.md) holds what waits on purpose: the project finishes the classic
(pre-renewal) game first, and what only Renewal or later content uses is not built until then.

Not there yet (from `BACKLOG.md`): the cash shop and what is bought with cash; achievements and titles; the
bank; navigation; replay; the Adventurer's Agency; the battlegrounds queue, the instances window and clans;
buying stores and shop search; the refine, enchant and item reform windows; the models of War of Emperium
(guardians, the Emperium); the newer formats of maps and models; and everything of the third and fourth jobs
(their effects, stances, auras, mounts).

The project has two chat commands of its own, which the Windows client does not have:

- `/packets` opens the developer's list of the packets of the map server that this client did not read, by
  name and count (`client/src/net/unread.rs`).
- `/farwalk` switches "far walk" off or on (on at first): with it a press on a place farther than the server
  walks in one go is walked to in stretches of 14 cells; without it such a press does nothing, as in the
  Windows client.

**New to the code? Start with [ARCHITECTURE.md](ARCHITECTURE.md)**; then `docs/DONE.md` and `docs/DEBTS.md`
for what is finished and what is owed.

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
- `tools/` — what the tables and the checks are made with:
  - `tools/packet-lengths/` makes the client's table of packet lengths from rAthena's headers;
  - `tools/packet-names.py` writes the packets' names by number from rAthena's headers (for `/packets`);
  - `tools/client-tables/` holds the generators that write out tables compiled into the Windows client, read
    from its decompilation: `commands.py` (chat commands), `skill_effects.py` (skills and effects),
    `statuses.py` (states and their icons), `ground_scopes.py` (the circle of a skill cast on a place),
    `job_bases.py` and `job_mounts.py` (which job a mounted or re-dressed job is, and is drawn as); and two
    for the server's scripts, `classic_overlay.py` and `placement_check.py` (see "Running it locally");
  - `tools/limits.py` checks what a script can of `ARCHITECTURE.md`'s rules: the sizes of files and
    functions, and that nothing on the page is named by hand;
  - `tools/directory_map.py` writes the directory map of `ARCHITECTURE.md` from the tree (`--write`);
    `limits.py` refuses a map that is not the tree's.
  - `tools/split_module.py` moves functions of one large Rust file into child modules by a plan.
- `check.sh` — everything a change must pass: the tests, the build for the browser with no warnings, and
  `tools/limits.py`.

What the gateway answers (`gateway/src/main.rs`, `gateway/src/routes.rs`):

| Address | What it is |
|---|---|
| `GET /assets/<client path>` | a file out of the client's GRF archives, as it is |
| `GET /image/<client path>` | a picture out of the GRF as a PNG the browser can show |
| `GET /tables.json` | the client's tables for the browser, all that is given at once |
| `GET /tables/items/<ids>`, `/tables/skills/<ids>` | names, pictures and descriptions, asked for by number |
| `GET /tables/ru/items/<ids>`, `/tables/ru/skills/<ids>`, `/tables/ru/skill-names.json` | the same in Russian |
| `GET /tables/items.ko/<ids>` | the Korean names and descriptions of items |
| `GET /tables/ai.json`, `/tables/ai-user.json` | the client's AI scripts, and those with the player's own |
| `GET /bgm/<file>` | the client's loose BGM folder |
| `GET /ws/<port>` | a WebSocket bridged to a TCP connection to the game server |
| `POST /api/register`, `/api/recover`, `/api/reset` | accounts and lost passwords |
| everything else | the built web client (`client/dist`) |

Every packet the client writes is held by a golden test: `client/src/net/golden.rs` builds each one with fixed
arguments and compares it byte for byte with `client/src/net/golden.txt`.

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

`server/conf/accounts.sql` seeds two accounts for development: `tester` (group 99, a game master) and `tester2`
(group 0, an ordinary player). They are for a server on your own machine only.

### The gateway's options

All are optional (`gateway/src/config.rs`); an argument it does not know stops it.

| Option | Default | What it sets |
|---|---|---|
| `--client-dir <folder>` | `../Feb-Client-CLEAN` | the game client's folder with its GRF archives |
| `--web-dir <folder>` | `client/dist` | the built browser client |
| `--listen <address:port>` | `127.0.0.1:8080` | where the gateway listens |
| `--server-host <host>` | `127.0.0.1` | where rAthena runs |
| `--server-ports <a,b,c>` | `6900,6121,5121` | the ports bridged; the first is the login server's |
| `--item-database <folder>` | `../rathena/db/pre-re`, or `../rathena/db/re` when `server/.env` says Renewal | rAthena's item database, for the items' names |
| `--compress on\|off` | `on` | gzip or Brotli, as the browser takes; `off` where a proxy or a CDN in front compresses |
| `--cache-client on\|off` | `on` | whether the client's files, named by their build, may be kept by the browser |
| `--invite-file <file>` | none | the file with the invitation word (at least 8 characters) a browser must give before anything is served |
| `--behind-proxy on\|off` | `off` | a proxy or tunnel stands in front: a visitor's address is the one it reports |
| `--lag <ms>`, `--jitter <ms>` | none | debug: delays everything between a browser and the game server, each way |
| `--database-url <url>` | the database of `server/docker-compose.yml` | the account service's database |
| `--smtp-url <url>` | `smtp://127.0.0.1:1025` | where the account service sends its e-mails (`smtps://user:password@host` for a real server) |
| `--mail-from <address>` | `Ragnarok 2026 <no-reply@localhost>` | whom those e-mails are from |
| `--public-url <url>` | `http://localhost:8080` | where players reach the gateway: the links to reset a password start with it |

To see walking and fighting as a player far away has them: `--lag 150 --jitter 100` delays each way by 150
milliseconds and a random part of up to 100 more, keeping the order as a TCP connection would. It delays
everyone who plays through that gateway, so the gateway refuses to start with `--lag` or `--jitter` together
with `--invite-file` or `--behind-proxy on` (which `public.sh` gives).

### The gateway's debug commands

Given a command, the gateway opens the client's archives, does that one thing, prints and leaves; it serves
nothing. (Of several, the last counts.)

| Command | What it prints or writes |
|---|---|
| `stats` | how many files the archives hold, and how many of each kind of storing |
| `ls <words>` | the files of the archives whose path has those words |
| `lua <file>,<file>` | runs datainfo files of the client and lists the tables they leave |
| `dump <file>,<file> <global>` | runs those files and prints that table of theirs as JSON |
| `items <id>,<id>` | what the item table says of those items |
| `tables` | what the browser client is given of the client's tables (`/tables.json`) |
| `chat-tabs` | the client's table of the chat window's tabs |
| `mapcache <file>` | writes the server's map cache from the client's maps |

### The server's switches

The server plays by the classic (pre-renewal) rules unless told otherwise. `./rules.sh renewal` rebuilds and
restarts it for Renewal (third and fourth jobs, their monsters, items, skills and NPCs), and `./rules.sh classic`
goes back; restart the gateway after either. The choice is kept in `server/.env`, which the gateway reads too for
the item database that goes with the rules. Each set of rules has its own database of characters.

rAthena's classic scripts place the NPCs and portals of Izlude, Alberta and Morroc by their old maps, and the
client has the new ones. `server/npc/roweb_classic.txt` moves them, and `roweb_classic_warps.sed` puts right the
cells the scripts themselves send players to; both are written by `tools/client-tables/classic_overlay.py
../rathena server/npc/roweb_classic.txt server/npc/roweb_classic_warps.sed`. Under the Renewal rules both do
nothing. `tools/client-tables/placement_check.py` lists who stands in a wall on any map.

`server/conf/` holds what differs from rAthena's defaults; `server/docker-compose.yml` puts each file into
rAthena's `conf/import`, which is read over rAthena's own:

- `groups.yml` — gives every player (group 0) the command `@autotrade` (`@at`), which rAthena gives to group 1
  and above only: a merchant with a shop open types it, is disconnected, and the character stays selling.
  The file says how to switch it off.
- `battle_conf.txt` — battle settings: a pet fights beside its master, the line about the server's version is
  not shown, and the others its comments explain one by one.
- `login_conf.txt` — accounts are made through the gateway only, passwords are stored as MD5, and the ban by
  address is off (the gateway limits attempts by the players' real addresses).
- `packet_conf.txt` — rAthena's count of connections per address is off, for the same reason.
- `char_conf.txt`, `map_conf.txt`, `inter_conf.txt` — the addresses of the servers and the database inside
  Docker, the server's name, no PIN code, the message of the day (`motd.txt`), and the script of the classic
  overlay.
- `accounts.sql` — the two test accounts (see "Running it locally"), put into the database on its first
  start.

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
ROWEB_GOLDEN=write cargo test -p roweb-client golden   # only after a packet was changed on purpose
```

The last one writes `client/src/net/golden.txt` anew; look at what changed in it.

## Sources this builds on

- Protocol facts come from [rAthena](https://github.com/rathena/rathena)'s source.
- File formats and many layout numbers were checked against [korangar](https://github.com/vE5li/korangar) (MIT)
  and [rustnarok-client](https://github.com/kindraywind/rustnarok-client) (MIT OR Apache-2.0);
  `tools/packet-lengths/` is taken from the latter, with its licences beside it.
- Where the original client's behaviour had to be matched, it was read from a decompilation of that client.

Ragnarok Online is a trademark of Gravity Co., Ltd. This project is not affiliated with or endorsed by Gravity.
