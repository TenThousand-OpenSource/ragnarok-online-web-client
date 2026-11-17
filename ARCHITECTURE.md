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
  model.rs          .rsw / .rsm: what stands on a map (its matrices in model/matrix.rs)
  effect.rs         .str: the client's effect files, frame by frame
  effect_tables.rs  tables the client has compiled into it: which effect a skill or an item shows, which file
                    and sound an effect is, how the body moves for a skill (the files in effect_tables/, written
                    by tools/client-tables/skill_effects.py; effects_missing.txt lists what is not drawn yet)
  hotkeys.rs        the client's keys: key numbers, how a key is written, what a Mac lacks
  talk.rs           PetTalkTable.xml: what pets say
  net/
    mod.rs          framing a byte stream into packets (lengths from packet_lengths*.txt)
    packets.rs      the way in: what the packets share, and the login server   ── children in packets/ ──
      chars.rs        the character server            zone.rs   the basics of the map server
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
    data.rs         the client's tables fetched from the gateway: what is shared, and `load`  ── children in data/ ──
      jobs.rs  items.rs  skills.rs  pets.rs  interface.rs  maps.rs  guide.rs
                      each kind of table: what it keeps, how it is read (`read`), how it is asked (`data::…`)
    figure.rs       sprites put together into pictures (a character = body + head + headgear + weapon)
    sound.rs        music and sound effects
    view.rs         the page outside the game's own windows: shared pieces    ── children in view/ ──
      accounts.rs     logging in, registering, recovering          characters.rs   server and character select, creation
      game.rs         the game's screen: the minimap and the character's place
    app.rs          the App: its state and types          ── children in app/ ──
      wiring.rs       page events (click, key, timer) -> Action
      dragging.rs     taking a thing out of a window and letting it go, as the client does (its sprite under the pointer)
      actions.rs      which code sees to each Action (the one place everything arrives at); the longer ones in actions/
        forms.rs  bag.rs  link.rs  taming.rs  travel.rs
      clicks.rs       what each `data-click` does; split by kind in clicks/, as incoming/ is
      keys.rs         the keyboard
      hotkeys_set.rs  which key does what for this player; setting keys (kept in the browser)
      account.rs      login, registration, recovery
      connection.rs   login -> char -> map server, and back
      incoming.rs     packets from the servers; the map server's are split by kind in incoming/
        travel.rs  actors.rs  combat.rs  talk.rs  status.rs  inventory.rs  shops.rs  pets.rs
      clicks/         account  talk  status  inventory  skills  chat  shops  pets  hotkeys  settings  guide
      settings.rs     the Game Settings: the player's options, kept in the browser, and what they switch
      commands.rs     what is typed after a slash
      game.rs         using items and skills, hotkeys, chat lines, entering a map
      screen.rs       which screen shows; painting characters into windows
      page.rs         DOM helpers, storage, fade and loading screen, dragging windows
    hud.rs          the in-game windows: shared pieces         ── children in hud/ ──
      basic.rs  inventory.rs  equipment.rs  skills.rs  status.rs  chat.rs  dialogs.rs  trade.rs  pets.rs
      hotkeys_set.rs  settings.rs  guide.rs
    world.rs        the world: its state and types             ── children in world/ ──
      session.rs      entering/leaving a map, the player's own character      (called by the app)
      actors.rs       everyone else and the loot                                (called by the app)
      combat.rs       blows, casts, effects                                     (called by the app)
      frame.rs        one frame: move everyone, advance effects, draw
      overlays.rs     what is laid over the picture: names, bars, numbers of blows, the pointer
      pursuit.rs      what the character does by itself: walk on, fetch a thing, chase and strike, walk into reach
      pick.rs         what the pointer is on: someone, or a thing on the ground
      input.rs        the mouse over the world
      gpu.rs          pipelines, textures, the draw call
      loading.rs      reading a map's files
      camera.rs  mover.rs  effects.rs  shaders.rs  math.rs
      prims.rs        the effects the client draws by code: its primitives (CEffectPrim), kind by kind, and
                      the effects made of them (CRagEffect), as far as they are ported
gateway/src/
  main.rs           routes and start-up
  grf.rs            reading files out of GRF archives
  tables.rs         the client's Lua tables: running them, and everything the browser is given at once
    tables/           one file per kind: items  skills  interface  maps  guide  (+ job_codes.json)
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
  client-tables     writes out tables compiled into the original client (skill_effects.py: skills and their
                    effects; job_bases.py: the plain job of each mounted job, data/job_bases.txt), the classic server's scripts put right for the client's maps (classic_overlay.py
                    writes server/npc/roweb_classic*), and a check that everyone the scripts place stands where the
                    client's map lets one stand (placement_check.py)
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

## The rules every change follows

These hold for everyone who writes here, people and agents alike. A change that breaks one is not done.

1. **Find the home before writing.** A thing belongs to a *system* (pets, items, skills, the guide, the
   settings…) and to a *layer*. Its code goes into that system's file of that layer; if the system is new, it
   gets its own files under the same name in every layer it touches (see "One system, one name"). Nothing is
   added to a file because it happens to be open.
2. **A layer knows only the layers under it.** `net/` and the formats never touch the page. The world never
   reaches into the app: it reports through `world::Hooks`. The app never reaches into the world's state: it
   calls the public functions of `world/{session,actors,combat}.rs`. Windows (`hud/`) only read the App's state.
3. **Parents hold what is shared, children hold the behaviour.** `app.rs`, `world.rs`, `hud.rs`, `data.rs` and
   the gateway's `tables.rs` have the types, the shared helpers and the list of children; a child starts with
   `use super::*;` and is about one thing.
4. **No file grows past about 500 lines.** One that does is cut by responsibility *in the same change* — not
   later.
5. **One state, owned in one place.** What the game remembers is a field of its system's group in `App`
   (`app.rs`), or of the world's `State`. No second copy, no hidden global except the tables in `data/`.
6. **The original client is the reference, and the comment says where from.** Behaviour, sizes and texts come
   from the client's code and files; the comment names the function or file (`UIItemIdentifyWnd`,
   `Guide_Monster.lub`). What was not found there, or was decided here against it, says so in the comment.
7. **Data comes from the client's files through the gateway**, each kind with a file in `gateway/src/tables/`
   and its mirror in `client/src/web/data/`. No game data is typed into the code or committed, except small
   tables the client has compiled into it, which are written out with a note of their origin.
8. **What can be tested without a browser is.** A parser or a rule in `net/`, the formats or the gateway gets a
   unit test with the change. What only shows on the page is looked at in the game before it is called done,
   and the report says what was and was not looked at.
9. **A change leaves the documents true.** A new file is added to the directory map below; a new kind of thing
   to "How to…".

Before finishing: `cargo test`, `./build-web.sh` with no warnings, the largest files checked
(`find client/src gateway/src -name '*.rs' | xargs wc -l | sort -rn | head`).

## How to…

**…add a table from the client's files**: read it in `gateway/src/tables/<kind>.rs` and add it to
`tables::for_browser`; on the client, keep and read it in `web/data/<kind>.rs` (`read`, called from
`data::load`) and bump the number in `data::load`'s URL so browsers fetch the tables anew.

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
- A file that passes ~500 lines is split by responsibility (`tools/split_module.py` does the moving). The
  tests of a file that has many are in `<file>/tests.rs`.
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

- `world/frame.rs`'s `frame` is still one function of some 350 lines (moving everyone and building what is
  drawn share too many values to part cheaply).
- `app.rs` and `world.rs` hold all the types of their layer and stand at the limit of 500 lines.
- The upper layers (windows, clicks, the world) have no tests: they are checked by hand in the game.
- Clicks are matched by text (`data-click="…"`), not by a type the compiler checks.
- Windows are rebuilt as strings; there is no component model, by choice so far.
