# roweb — instructions for agents

Read `ARCHITECTURE.md` before changing anything here, and follow its section **"The rules every change
follows"**. They are not suggestions: a change that breaks one is not done.

The short of it:

- **Find the home first.** Every thing belongs to a system (pets, shops, skills, guide, settings…) and a layer:
  the formats (`client/src/{map,model,sprite,effect,bmp}.rs`), the rules that need no page
  (`client/src/<system>.rs`, each with its tests), the packets (`client/src/net/<system>.rs`), the tables
  (`gateway/src/tables/<kind>.rs` ↔ `client/src/web/data/<kind>.rs`), the world (`web/world*`), the app
  (`web/app/{incoming,clicks,state}/<system>.rs`), the windows (`web/hud/<system>.rs`), what is pressed
  (`client/src/press/<system>.rs`). Put code in that system's file of that layer. A new system gets its own
  files, under the same name in every layer. Never add to a file just because it is open.
- **A rule that needs no page is not written under `web/`**: it goes into the system's native module with a
  test, and the window or the app calls it.
- **Keep files under about 500 lines.** If your change pushes one over, split it by responsibility in the same
  change.
- **Keep functions under about 100 lines** (a plain `match` that hands work out is the exception), and
  **write nothing twice**: look for the helper first (`ARCHITECTURE.md`, rule 11, names the common ones).
- **The original Windows client is the reference.** Take behaviour, sizes and texts from the decompiled client
  (`../rodecomp`) and the client's own files (`../Feb-Client-CLEAN`), and say in the comment where from. Do not
  invent interfaces or behaviour; when something was not found in the client, or is decided here against it,
  say so plainly — in the comment, to the user, and in `docs/DEBTS.md`.
- **Classic first.** The classic (pre-renewal) game is finished before anything else. What only Renewal or
  later content uses is written down in `BACKLOG.md` and is not built until the owner says so.
- **Name nothing on the page by hand.** A button is a value of `crate::press` drawn with `pressed(..)`; a
  window is an entry of `hud/windows.rs`. See rule 12 and the How-tos "…add or change a window" and "…add a
  button, a tab, a row that is pressed".
- **Game data comes from the client's files through the gateway**, each kind with a file in
  `gateway/src/tables/` and its mirror in `client/src/web/data/`. None is typed into the code or committed,
  except small tables the client has compiled into it, which are written out with a note of their origin.
- **A packet the client sends is held by the golden test** (`client/src/net/golden.rs` against
  `client/src/net/golden.txt`). A new builder gets its line there. `golden.txt` is written anew only when a
  packet is changed on purpose (`ROWEB_GOLDEN=write cargo test -p roweb-client golden`), and the change in it is
  looked at; never to make a failing test pass.
- **Test what can be tested, look at the rest in the game**, and report honestly which is which.
- **Never restart or stop the owner's gateway or game server without saying so first**: the gateway on port
  8080 (`target/release/roweb-gateway`) and the Docker server are the owner's running game. Build a gateway of
  your own in another target folder (`CARGO_TARGET_DIR`) to try its commands; `./build-web.sh` (and so
  `./check.sh`) replaces `client/dist`, which that gateway serves to the next page opened.
- **Keep the documents true**, in the same change:
  - `ARCHITECTURE.md` — its directory map is written from the tree: give a new file its `//!` and run
    `python3 tools/directory_map.py --write`; a new kind of thing goes to "How to…".
  - `docs/DONE.md` — what is implemented, by system: its files, where in the Windows client it was taken from,
    and its state (seen in the game / tests only / by reading only). Change a state only to what is true.
  - `docs/DEBTS.md` — deviations from the Windows client that are this project's own, what was not found in
    the client, bugs known and left, refactoring left, what a browser does not allow.
  - `BACKLOG.md` — the Renewal pool.

Two chat commands are this project's own, for whoever works on it: `/packets` opens the list of the packets
the servers sent that no reader took (look at it after trying something new), and `/farwalk` switches walking
in stretches to a place pressed farther away than one request of the client reaches.

Before finishing, and before any commit: `./check.sh`. It runs the tests, the build for the browser (no
warnings) and `tools/limits.py` (the sizes of files and functions, names of the page written by hand, the
directory map), and must end with "all checks passed".

Reply to the user in Russian. Commit only when asked.
