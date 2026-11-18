# roweb — instructions for agents

Read `ARCHITECTURE.md` before changing anything here, and follow its section **"The rules every change
follows"**. They are not suggestions: a change that breaks one is not done.

The short of it:

- **Find the home first.** Every thing belongs to a system (pets, items, skills, guide, settings…) and a layer
  (`net/` → `web/world*` → `web/app*`, `web/hud*`; the gateway's `tables/`). Put code in that system's file of
  that layer. A new system gets its own files, under the same name in every layer. Never add to a file just
  because it is open.
- **Keep files under about 500 lines.** If your change pushes one over, split it by responsibility in the same
  change.
- **Keep functions under about 100 lines** (a plain `match` that hands work out is the exception), and
  **write nothing twice**: look for the helper first (`ARCHITECTURE.md`, rule 11, names the common ones).
- **The original Windows client is the reference.** Take behaviour, sizes and texts from the decompiled client
  (`../rodecomp`) and the client's own files (`../Feb-Client-CLEAN`), and say in the comment where from. Do not
  invent interfaces or behaviour; when something was not found in the client, say so plainly — in the comment
  and to the user.
- **Game data comes through the gateway** (`gateway/src/tables/<kind>.rs` ↔ `client/src/web/data/<kind>.rs`),
  never typed into the code.
- **Test what can be tested, look at the rest in the game**, and report honestly which is which.
- **Keep `ARCHITECTURE.md` true**: add new files to its directory map.

Before finishing: `cargo test`, `./build-web.sh` (no warnings), and check the largest files:
`find client/src gateway/src -name '*.rs' | xargs wc -l | sort -rn | head`.

Reply to the user in Russian. Commit only when asked.
