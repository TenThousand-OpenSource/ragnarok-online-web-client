# Architecture

A map of the project for someone opening it for the first time. Read this page, then the module comment
(`//!`) at the top of the file you are about to change: every file says what it is for.

## The three programs

```
browser (client, WebAssembly) ── HTTP ──▶ gateway ──▶ the game client's files (GRF archives, Lua tables)
                               ── WebSocket ──▶ gateway ── TCP ──▶ rAthena (login, char, map servers)
```

- **`client/`** — the game itself: Rust compiled to WebAssembly. Draws the world on a canvas with wgpu and the
  windows as HTML laid over it.
- **`gateway/`** — a small web server. It serves the client, reads game files out of the player's own game
  folder on demand, answers a few table lookups, and bridges WebSockets to the game servers' TCP ports.
- **`server/`** — a Docker setup of an ordinary rAthena, for development. The game's rules live there, not here.

Nothing of the game's art or data is in the repository: the gateway reads it from a kRO client folder.

## The client in four layers

Each layer knows only the ones under it.

| Layer | Where | What it is | Needs a browser |
|---|---|---|---|
| 4. App | `client/src/web/app*`, `hud*`, `view.rs` | State of the session, what each packet / click / key does, the windows | yes |
| 3. World | `client/src/web/world*`, `figure.rs`, `sound.rs`, `data.rs` | The map in 3D, everyone on it, effects, the mouse over it | yes |
| 2. Protocol | `client/src/net/` | Packets: reading the servers' and writing the client's | no |
| 1. Formats | `client/src/{map,model,sprite,effect,bmp}.rs` | The game's file formats, parsed into plain structures | no |

Layers 1 and 2 are plain Rust with unit tests (`cargo test -p roweb-client`); everything that touches the page
or the GPU is under `web/` and only builds for `wasm32`.

## Directory map

```
client/src/
  lib.rs            the crate's modules
  i18n.rs           every text the client itself says, in English and Russian
  bmp.rs            BMP pictures (magenta is see-through)
  sprite.rs         .spr / .act: sprites and their animations
  map.rs            .gat / .gnd: walkable cells, ground mesh, path finding
  model.rs          .rsw / .rsm: what stands on a map
  effect.rs         .str: the client's effect files, frame by frame
  hotkeys.rs        the client's keys: key numbers, how a key is written, what a Mac lacks
  talk.rs           PetTalkTable.xml: what pets say
  net/
    mod.rs          framing a byte stream into packets (lengths from packet_lengths*.txt)
    packets.rs      login and character servers, the basics of the map server
    units.rs        everyone on the map: appearing, walking, striking, loot on the ground
    items.rs        inventory, equipment, skills known, hotkey bar
    skills.rs       casting and landing of skills
    pets.rs         the player's pet: its state, feeding, the commands given to it
    emotions.rs     the emotions shown over heads
    npc.rs          NPC dialogs, whispers, restart
    trade.rs        NPC shops and storage
  web/
    mod.rs          entry point, fetching files from the gateway
    socket.rs       the WebSocket
    data.rs         tables fetched from the gateway: names of items, skills, jobs, sprites
    figure.rs       sprites put together into pictures (a character = body + head + headgear + weapon)
    sound.rs        music and sound effects
    view.rs         the windows before the game: login, server and character select, creation
    app.rs          the App: its state and types          ── children in app/ ──
      wiring.rs       page events (click, key, drag, timer) -> Action
      actions.rs      what each Action does (the one place everything arrives at)
      clicks.rs       what each `data-click` does; split by kind in clicks/, as incoming/ is
      keys.rs         the keyboard
      hotkeys_set.rs  which key does what for this player; setting keys (kept in the browser)
      account.rs      login, registration, recovery
      connection.rs   login -> char -> map server, and back
      incoming.rs     packets from the servers; the map server's are split by kind in incoming/
        travel.rs  actors.rs  combat.rs  talk.rs  status.rs  inventory.rs  shops.rs  pets.rs
      game.rs         using items and skills, hotkeys, chat lines, entering a map
      screen.rs       which screen shows; painting characters into windows
      page.rs         DOM helpers, storage, fade and loading screen, dragging windows
    hud.rs          the in-game windows: shared pieces         ── children in hud/ ──
      basic.rs  inventory.rs  equipment.rs  skills.rs  status.rs  chat.rs  dialogs.rs  trade.rs  pets.rs
    world.rs        the world: its state and types             ── children in world/ ──
      session.rs      entering/leaving a map, the player's own character      (called by the app)
      actors.rs       everyone else and the loot                                (called by the app)
      combat.rs       blows, casts, effects                                     (called by the app)
      frame.rs        one frame: move everyone, advance effects, draw, overlays
      input.rs        the mouse over the world
      gpu.rs          pipelines, textures, the draw call
      loading.rs      reading a map's files
      camera.rs  mover.rs  effects.rs  shaders.rs  math.rs
gateway/src/
  main.rs           routes and start-up
  grf.rs            reading files out of GRF archives
  tables.rs         the client's Lua tables (items, skills, jobs) and the translations
  images.rs         game pictures as PNG
  bridge.rs         WebSocket <-> TCP, only to the configured server ports
  accounts.rs       registration and password recovery
  limiter.rs        limits on attempts
  mapcache.rs       rAthena's map cache
translations/
  GLOSSARY.md       terms, so that translators agree
  src/              English sources of the translated batches
  ru/               Russian: items-*.json, skills-*.json  ({"<id>": {"name", "description"}})
tools/
  packet-lengths    builds the packet length table from rAthena's sources
  split_module.py   moves functions of a file that grew too large into child modules
```

## How things flow

**Something happens on the page.** `app/wiring.rs` turns the browser's event into an `Action` and calls
`App::handle` (`app/actions.rs`). Nothing else listens to the page, except the world's own mouse handling in
`world/input.rs`, which reports back through `world::Hooks` — again as Actions.

**The server says something.** `socket.rs` → `Action::Socket` → `App::received` (`app/connection.rs`) frames the
bytes with `net::mod` and hands each packet to `App::packet` (`app/incoming.rs`). The map server's packets go
through the eight `*_packet` functions in `app/incoming/`; each returns whether the packet was its own. A
handler does three things, in this order: parse (`net::*`), update the App's state, and tell the world
(`world::*`) and the windows (`refresh_*` or `render`).

**The world.** `world.rs` holds one `State` (thread-local): the GPU, the `Scene` of the current map, and what is
going on in it. The app only *tells* it things through the public functions in `world/session.rs`,
`world/actors.rs` and `world/combat.rs`; it never reaches into the state. `world/frame.rs` runs once per
animation frame.

**The windows.** They are HTML strings built by `hud.rs` and `hud/*.rs` from the App's state and from the
original client's bitmaps, placed at the original's coordinates. `App::render` writes the whole page; the
`refresh_*` functions rewrite one part (items, values, chat, a dialog) so that typing and scrolling survive.
Elements carry `data-click`, `data-use` (double click), `data-info` / `data-skill` (right click), `data-drag`.

**One system, one name.** A system of the game (pets, shops, skills…) is the same word in every layer:
`net/pets.rs` (its packets), `app/incoming/pets.rs` (what the server says of it), `app/clicks/pets.rs` (what its
buttons do), `hud/pets.rs` (its window), and `App::pets` (what is remembered of it). What belongs to no system
(the connection, the account, the screen shown) is a plain field of `App`.

## How to…

**…handle a new packet from the map server**
1. Its length must be in `client/src/net/packet_lengths.txt` (generated) or `packet_lengths_more.txt` (by hand).
2. Add a parser with a unit test in the fitting `client/src/net/*.rs`.
3. Add an arm to the fitting `client/src/web/app/incoming/*.rs`.
4. If the world must show it, add or call a function in `world/{session,actors,combat}.rs`.

**…send a packet**: a builder function next to the parsers in `net/`, called from `app/clicks/*.rs`, `app/keys.rs`
or `app/actions.rs` as `socket.send(&…)`.

**…add or change a window**: the HTML in `hud/<window>.rs`; its place in `App::hud` (`hud.rs`); its state as a
field of its system's group in `App` (`app.rs`: `Hero`, `Bag`, `Skills`, `Chat`, `Deal`, `Pets`; a new system
gets a group of its own); what its buttons do in `app/clicks/<system>.rs`; its texts through `i18n::tr`.

**…add an effect**: a `.str` effect needs no code — `world::effect("name.str", on_whom)`. An effect the original
draws by code goes to `world/effects.rs` (geometry) and is fed to the GPU in `world/frame.rs`.

**…translate**: interface texts are in `client/src/i18n.rs`. Items and skills are JSON files in
`translations/ru/`; the gateway reads every `items-*.json` and `skills-*.json` at start.

## Conventions

- **The original client is the reference.** Sizes, coordinates, timings, file names and behaviour are taken from
  the original client (its decompilation and its data), and the comment says from where (`UIItemWnd`,
  `CArrowEffect`…). What was *not* found there and had to be decided here says so too.
- Comments say what a thing is and why, in plain words; packet layouts are written out where they are parsed.
- Types live in the parent module (`app.rs`, `world.rs`, `hud.rs`), behaviour in the children; a child starts
  with `use super::*;`.
- A file that passes ~500 lines is split by responsibility (`tools/split_module.py` does the moving).
- No game data is committed.

## Build, run, test

```
cargo test -p roweb-client                       # formats and protocol
./build-web.sh                                   # client -> client/dist (wasm + page)
cargo run --release -p roweb-gateway             # serves http://localhost:8080
docker compose -f server/docker-compose.yml up   # a local rAthena
```

Test accounts for the local server are in `server/conf/accounts.sql`.

## Known debts

- `world/frame.rs` is one long function (move, simulate, draw, overlays): to be cut into those four steps.
- `app/actions.rs` is a long `match`; it can be split by kind the way `incoming/` and `clicks/` are.
- Windows are rebuilt as strings; there is no component model, by choice so far.
