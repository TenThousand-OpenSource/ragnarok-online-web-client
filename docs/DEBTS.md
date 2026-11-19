# Technical debt

What in roweb is not as the Windows client has it, what was never seen working, and what was knowingly left.
Keep it true: an item is removed when it is fixed or seen; a new deviation is added here, with its file, in the
change that makes it. Paths are under `roweb/`; content of later game versions that is simply absent is in
`BACKLOG.md`, not here.

Parts: 1 our own deviations · 2 made by reading, not seen in the game · 3 not found in the decompiled client ·
4 bugs known and left · 5 refactoring left undone · 6 what a browser does not allow · 7 small things of the
classic game not made yet.

---

## 1. Deviations from the Windows client that are this project's own choice

### Way in
- The windows are HTML laid over a canvas, not drawn on one surface (`client/src/web/view.rs`, `web/hud.rs`).
  Text fields are real input elements.
- Registration and password recovery windows have no original (`client/src/web/view/accounts.rs form`); the
  gateway makes the account and mails the reset link (`gateway/src/accounts.rs`, `accounts/recover.rs`:
  `POST /api/register`, `/api/recover`, `/api/reset`). Passwords are kept as rAthena's unsalted MD5.
- Login: the ID and password start at the left of their fields; the client centres them
  (`web/view/accounts.rs login_window`).
- Character creation: on a classic server the Doram panel is shown and cannot be pressed; the client has no
  such switch (`web/view/creation.rs creation_races`).
- The windows are laid out for 1024 by 768 and enlarged together on a larger view; a view narrower than 1024
  is covered by a note (`web/app/screen.rs render`, `MIN_WIDTH`).
- Opening the server to others: an invitation word asked before anything is served, kept in a cookie for a
  month (`public.sh`, `gateway/src/public.rs`, `--invite-file`, `--behind-proxy`).

### Movement
- `/farwalk` and the far-walk journey: a press farther than the server walks in one go is walked in stretches.
  In the client such a press does nothing. ON by default by the owner's decision (a place in sight is to be
  walked to when the way round is there); `/farwalk` switches it off (`web/app/commands.rs`,
  `web/world/pursuit.rs journey`, `map/paths.rs goal_towards`, `web/world/hooks.rs Options::far_walk`).
- One who walks does not sit down: the Sit key does nothing until the character stands
  (`web/app/keys.rs interface_key`).
- The server's clock is a line that does not run round after 2^32 ms (`server_time.rs`).

### Fight
- The monster being fought keeps its turning mark while the pointer is on another, which gets a second one;
  the client has one mark (`web/world/overlays.rs pointed`).
- Chasing: reach is counted in cells as the server does, not straight across as the client does
  (`web/world/pursuit.rs chase`).
- A right press is a menu and not a camera turn when the pointer moved 4 pixels or less; the client's measure
  was not found (`web/world/input/pointer.rs`).
- The screen always shakes when an effect asks: the client's option is not asked first (`quake.rs` header;
  the world's `Options::quake` exists in `web/world/hooks.rs` — the header may be stale, to be checked).

### Skills and effects
- Effects the client draws "over everything" are drawn as the other effects are (`web/world/prims/struck.rs`,
  `blasts.rs`, `sprites.rs`, `missiles.rs`; `prim/tei/dust.rs`, `fins.rs`, `flames.rs`, `orbit.rs`, `plates.rs`,
  `icons.rs`).
- Parts that in the client move by the machine's speed or are drawn by lot at every drawing move a tick at a
  time here (`web/world/prims/heals.rs`, `motes.rs`, `hits.rs`, `bolts.rs`; `prim/tei/fins.rs`, `scans.rs`).
- Flat primitives are measured in pixels of a sprite, not of the screen (`web/world/prims.rs` header).
- Sounds of coded effects are only started, all equally loud; the client places them
  (`web/world/prims/sounds.rs`, `played.rs`).
- The aura of the highest level is held to once a frame; three differences from the client are listed in
  `web/world/aura.rs` (not begun anew, ended with effects off, ended on one who shows no aura).
- Water Screw's sprite (`EF_SPR_LIGHTPRINT3`, 941): the client writes no colour; drawn in the sprite's own
  (`web/world/prims/prints.rs`, `prim/fourfold.rs`).
- Graffiti's words stand over the unit, not on the ground (`web/world/written.rs`).
- The cooldown shade is a cone the page turns, not a fan of triangles (`web/hud/cooldowns.rs`); every wait
  the server tells of is kept, not only those of the hotkey bar (`web/app/incoming/cooldowns.rs`).
- A cast circle turns a step each 16 ms frame: this project's reading of the client's tick
  (`web/world/scope.rs drawn`).
- A figure's parts are painted into one picture, not drawn as quads of their own (`web/figure/compose.rs`).
- The falcon and the mount of the player's own character are taken from the states told at login
  (`web/app/incoming/statuses.rs rides`).

### Items
- A card in a slot is shown in the item's own window, not in a second one (`web/hud/inventory.rs slots_row`).
- "Kept from…" under a description uses the browser's tooltip, not the client's round one
  (`web/hud/inventory.rs kept_from`).
- Forging: the inventory keeps showing a thing put into a slot until the server takes it
  (`web/app/clicks/choosing.rs forge_dropped`).
- The skill window's figure shows its first frame; the client's looks about (`web/app/screen.rs`).
- Equipment, costume page: the "off" button is not drawn; the client draws it switched off
  (`web/hud/equipment.rs`).

### Shops, storage, cart, vending
- The storage stands where this client has always had it, not at the client's 360, 200
  (`web/hud/windows/storage.rs`).
- Opening a stall: what is wrong is said in the chat in red, not in message boxes
  (`web/app/clicks/vending.rs stall_opened`).
- Long lists are scrolled themselves; the client has a 15-wide bar (`web/hud/lists.rs list_of`).

### Chat
- `/packets` and its window: a developer's list of the map server's packets nobody reads; the client has
  none (`web/app/commands.rs`, `web/hud/unread.rs`, `net/unread.rs`, `press/unread.rs`).
- Whisper blocks (`/ex name`) last one visit; the client keeps them with the account (`web/app/commands.rs
  whisper_from`).
- The whisper window is of one size; the client's can be made larger (`web/hud/whispers.rs`).
- A coloured broadcast with a letter height of nought is drawn at 12; the client would draw nothing
  (`net/broadcast.rs coloured`).
- A chat room's title is not checked against a list of bad words (`chat_rooms.rs wrong`).
- A chat room's password is measured in bytes of UTF-8, which is what this client sends; the client
  measures the bytes of its own code page (`UIEditCtrl`: 8 at most, 4 at least). So two Cyrillic letters pass
  for four here, and four Korean ones do not fit (`chat_rooms.rs password`, `wrong`).
- An indexed picture of a sprite whose size its run-length code cannot fill (more than 255 pixels to a coded
  byte) is refused as a broken file; what the client does with one was not looked for (`sprite/spr.rs`).

### Party, guild
- The party's side window stands where it was opened; the client keeps it beside the party window as that
  is dragged (`web/hud/windows/party.rs`).
- Guild rights: decided from the member's position where rAthena does not send ZC_UPDATE_GDID
  (`web/app/incoming/guild/members.rs guild_rights_known`).
- A guild's and a party's name are asked as a player comes into sight, not when the pointer first comes
  over them (`web/app/incoming/guild.rs guild_looked_up`).

### Friends, mail
- A letter opens by its picture too, and its title is pressed along its whole width
  (`web/hud/mail/list.rs mail_row`).
- The mailbox's cross and icon close everything, as its Escape does (`web/app/clicks/mail.rs mail_toggled`).
- A letter read twice shows its text once; the client would add it again (`web/app/incoming/mail.rs`).

### Companions
- The AI scripts run in a Lua interpreter written in Rust (`lunita`, Lua 5.1 mode), not the client's Lua
  (`ai.rs`). Its own bounds: 400 steps of 4096 fuel a call, 32 MB, strings of 1 MB (`STEPS`, `FUEL`,
  `MEMORY`).
- A script has no disk and no system clock: files are those the gateway gave (`ReadFile`, `ai/host.rs`),
  written ones live while the script runs, `os.time` is the game's tick, `os.date` is empty
  (`ai/prelude.lua`; the scripts come from `gateway/src/tables/ai.rs`).
- A pet's lines are found by its sprite's name in small letters (`web/data/pets.rs pet_line`).
- The right-button menu exists for the pet and for other players only (`web/app/actions/taming.rs unit_menu`).

### NPC scripts, books
- A book's lines are measured with the browser's font (`web/app/clicks/books.rs`); pages 0 and "before the
  first" show the first (`books.rs turned`, `aloud_from`); lines lose their carriage return (`books.rs
  wrapped`).
- A bookmark is kept in the browser by character name and book number (`web/app/clicks/books.rs`).
- "No such book" is this client's own words; the client's are Korean and in no table
  (`web/app/clicks/books.rs`).
- Links to a place, a quest or a web address in an NPC's text are shown and not pressed (`web/hud/talk.rs
  linked`).

### Quests, guide
- Only the quests rAthena's `quest_db.yml` knows are given to the page (`gateway/src/tables/quests.rs`).
- Adventure Guide, first page: a press on a row opens that page; in the client it does nothing
  (`web/hud/guide.rs guide_intro`).
- Hunt progress is said in the chat only, not also in the middle of the screen
  (`web/app/incoming/quests.rs quest_hunted`).

### Maps, minimap
- A boss's mark is left out when the boss is outside the part of the minimap shown (`web/hud/marks.rs`).

### Options, hotkeys
- On a Mac, Sit is Option+X and the screenshot Option+P; Alt is written "Option" (`hotkeys.rs mac_instead`,
  `written`; `web/app/hotkeys.rs`).
- Keys, options, chat tabs, emotions, camera, quest list and skill levels are kept in the browser's storage,
  not in the client's `SaveData` files (`kept.rs`, `web/app/page.rs stored`, `web/app/settings.rs`).
- The Game Settings' tab names and "off" are this client's words (`web/hud/settings.rs settings_window`).
- The window of unread packets stands where it was decided here (`web/hud/windows/unread.rs`).

### World
- All text is drawn by the browser: sizes from `font.rs` (`cell`, `pixels`), face and smoothing from the
  page (`--face` in `client/web/index.html`: Arial, "Liberation Sans").
- Screenshots are PNG files handed to the browser to save; the client writes `.jpg`. The game's pointer does
  not come out (`screenshot.rs file_name`, `web/app/screenshot.rs`).
- Dragging uses the page's own code, not the browser's drag (`web/app/dragging.rs`).

### Gateway
- The Russian layer: window texts (`i18n.rs tr`, `i18n/phrases.rs`), item and skill texts from
  `translations/ru` served under `/tables/ru/…` (`gateway/src/tables/items.rs`, `gateway/src/routes.rs`).
- `--lag` and `--jitter`: a debug delay on the bridge, refused together with `--invite-file` or
  `--behind-proxy` (`gateway/src/config.rs`, `bridge/lag.rs`).
- Names the game-settings file uses and the client never defines stand for themselves
  (`gateway/src/tables/interface.rs`).

---

## 2. Made by reading the decompiled client and NOT yet seen in the game

Rule: everything that is not in the list of what was seen — the list is in `docs/DONE.md` — is in this state:
written from `../rodecomp`, covered by tests where it is pure, never looked at in a browser.

### Never looked at after the refactoring
- The HTML tour of every window (the windows package, `client/src/web/hud/**`, `web/view/**`): the strings
  were compared by script only.
- A session with `/packets` open (the app package, `web/app/**`).
- A cast, a bolt, a ground skill, a warp portal, a walk and a fight (the world package, `web/world/**`,
  `prim/**`).
- After the bugs of part 4 were mended (tests only, none looked at in a browser): the key written in a
  hotkey slot's tooltip (`web/hud/skills.rs hotkeys`); which lines a chat tab lists and `/savechat` writes
  (`web/app/state/chat.rs Chat::listed`); what `/aura2`, `/monsterhp`, `/quake` and the other options whose
  lines stand past 2076 say when switched (`commands.txt`); a pet's Korean lines (`web/data/pets.rs`); the
  tables read through the safe `data::list`/`row` (`web/data/*.rs`: the world map, the cards, the guide's
  tabs, the towns' people, the keys); a coloured announcement whose six letters are no plain number
  (`net/broadcast.rs hex`); leaving refused (`net/packets/zone.rs leave_refused`); the Russian labels of an
  item's description out of `translations/ru/labels.json` (`gateway/src/tables/items.rs`).

### Skills and effects
- Every coded effect of `client/src/web/world/prims/*.rs` (40 files) and `prim/**`, except the few seen:
  bow attack, Double Strafe, Arrow Shower, Improve Concentration, Blitz Beat, Detect, Ankle Snare, Land Mine,
  Skid Trap, Claymore, the level-up and job-change effects, Fire Wall.
- Effects played from files (`effect.rs`, `effect/**`, `web/world/launch.rs`) beyond those skills.
- States on a body (`ailments.rs`, `state_effects/**`, `body/**`, `web/world/ailments.rs`), spirit spheres
  (`spirits.rs`), ground units other than the traps seen (`skill_units.rs`, `web/world/skill_units.rs`),
  cast circles (`scope.rs`), cooldown shades (`cooldowns.rs`), the aura (`aura.rs`, `web/world/aura.rs`).

### Fight
- PVP and its ranks (`pvp.rs`, `net/pvp.rs`, `web/hud/pvp.rs`, `web/world/pvp.rs`); melee and combos
  (`blow/**`, `web/world/combat.rs`) beyond the bow.

### Items
- Cards and making: set card, forge, mend, identify, warps list (`choosing.rs`, `net/choosing/**`,
  `web/hud/choosing.rs`); viewing another's equipment (`net/viewequip.rs`, `web/hud/viewequip.rs`).

### Shops, storage, cart, vending
- NPC shops (`shops.rs`, `web/hud/shops.rs`), storage (`storage.rs`, `web/hud/storage.rs`), cart (`cart.rs`,
  `web/hud/cart.rs`, `web/world/cart.rs`), vending both sides (`vending.rs`, `web/hud/vending.rs`,
  `vending/buying.rs`).

### Chat
- Chat rooms (`chat_rooms.rs`, `web/hud/chat_rooms.rs`), whisper windows (`whispers.rs`), chat tabs
  (`chat_tabs.rs`), emotions and the Shortcut List (`web/hud/emotions.rs`), broadcasts (`net/broadcast.rs`).

### Party, friends, mail, family
- Party (`party.rs`, `net/party.rs`, `web/hud/party.rs`, `party/side.rs`); friends and the messenger
  (`friends.rs`, `web/hud/friends.rs`, `messenger.rs`); mail beyond its icon (`mail.rs`, `net/mail.rs`,
  `web/hud/mail/**`); family (`family.rs`, `web/hud/family.rs`).

### Companions
- Pets, homunculus, mercenary (`pets.rs`, `homunculus.rs`, `mercenary.rs` and their `net/` and `web/hud/`
  files); their AI (`ai.rs`, `ai/**`, `web/app/ai.rs`, `web/world/ai.rs`), tested natively against the
  client's own scripts only.

### NPC scripts, books, quests, guide
- Books (`books.rs`, `web/hud/books.rs`); scripted pictures, marks and cut-ins (`net/scripted.rs`,
  `web/hud/scripted.rs`); the Adventure Guide (`guide.rs`, `web/hud/guide.rs`, `guide/pages.rs`); the Tipbox
  beyond its first opening (`web/hud/tips.rs`).

### Maps, world
- The World Map (`web/hud/worldmap.rs`); weather (`web/world/prims/weather.rs`), sky, fog and night
  (`scenery/dark.rs`, `fog.rs`, `views.rs`, `web/world/scenery.rs`); map effects and sounds
  (`scenery/makers.rs`, `web/world/scenery/placed.rs`).

### Options, hotkeys, sound, gateway
- The Shortcut Settings window (`web/hud/hotkeys.rs`), most rows of Game Settings (`web/hud/settings.rs`),
  screenshots (`web/app/screenshot.rs`), sounds by distance (`earshot.rs`, `web/sound.rs`).
- `/tables/ai.json` and the other routes were not asked of a gateway built after the refactoring (its
  one-shot commands were compared byte for byte instead).

---

## 3. Not found in the decompiled client

### Way in
- Where the client writes when a character waiting to be deleted can be: `web/view/characters.rs`.

### Movement, fight
- Whether the client sends the player on to the same cell when it stands in a chase: `web/world/pursuit.rs`.
- The client's own measure between a press and a drag of the right button: `web/world/input/pointer.rs`.
- Which way round a turned picture is turned (clockwise here): `web/world/gpu/draw.rs`.

### Skills and effects
- Whether sight-state effects count loop passes or ticks: `state_effects/kept.rs`; the same for a body's
  passes: `web/world/ailments.rs bodies`.
- What `renderFlag & 0x200` does to a played sprite: `web/world/prims/played.rs`.
- The colour of Water Screw's parts (the client's function writes none): `prim/fourfold.rs`,
  `web/world/prims/prints.rs`.
- What the client reads past the 360th entry of its cosine table: `prim/tei/beams.rs fan`.
- Where the client takes its own falcon and mount from at login: `web/app/incoming/statuses.rs rides`.

### Items, cart
- Where in its button the cart-look window draws the cart: `web/hud/cart.rs cart_looks_window`.

### Exchange, chat, mail
- The "code of the character" the client writes after a name (`CSession::ShortcutFilename`); left out in
  `web/hud/exchange.rs`, `whispers.rs sent` / `heard`, `web/hud/whispers.rs`, `web/hud/mail/letter.rs`,
  `web/hud/player_menu.rs`.
- That `TT_SKILLFAIL_ON_OFF` and `TT_LOGINOUT_ON_OFF` start switched on: `web/app/commands/beside.rs BESIDE`.

### Party, guild
- What the chat says when a party gets another leader (nothing is said): `web/app/incoming/party.rs`.
- How the client counts a guild member's `expRatio`: `web/hud/guild/members.rs guild_member_row`.

### Player menu
- The two states (`StateFlag01`, `StateFlag02`) in which the client leaves out the first two rows; the words
  of line 4067 (past the end of the message table): `web/hud/player_menu.rs`.

### Books, quests
- That Escape closes the book's window: `web/hud/windows/books.rs`.
- Whether the client falls back to `bg_quest01.bmp` / `bg_quest02.bmp` when the `.png` is missing:
  `web/hud/quests/pane.rs`.

---

## 4. Bugs known and left

### Noticed during the refactoring (behaviour had to stay as it was)
- `/tables/items.ko/{ids}` (`gateway/src/routes.rs`) has no reader in `client/src`: no defect, a route nobody
  asks; whether it goes is the owner's to say (the translations' sources may be made with it).
- `/fog` and `/emblem` say nothing when they switch: their lines (`MSI_GAME_SETTINGS_FOG_ON`/`_OFF`,
  `MSI_GAME_SETTINGS_EMBLEM_ON`/`_OFF`, the client's 4223 to 4226) are past the end of `msgstringtable.txt`,
  which has 4022 lines. The client of this version fills its table from `data\msgstringtable.csv` by the
  lines' names (`ImportMessageTable`, `InsertMessageRow`), where they are ("Fog On", "Guild Emblem On"…); the
  gateway reads the `.txt` only (`gateway/src/tables.rs messages`). The two files agree line for line up to
  4021, but for the wording of some fifty lines.
- rodecomp's `Source/Resource/MSGSTRINGID.h` is misnumbered: it lacks the names the client misspells
  (`MIS_PUT_GUILD_NAME_IN`, `MIS_GUILD_TIP`, `MIS_GUILD_SYSTEM` at 2077 to 2079, and
  `MIS_SHORTCUT_SKILL_TOOLTIP_HOTKEY_SP_PERCENT` at 4108), so every number in it from 2077 on is three short,
  and from 4108 on four. The client's numbers are the places in `messageNames`
  (`Source/Contents/Tables/MessageTable.cpp`), which `msgstringtable.txt` follows; rodecomp's own code that
  names a line by that header names the wrong one (`LookupMessage(MSI_AURA_EFFECT_ON)` where RODEX is closed
  for refining is line 2991, "You cannot use RODEX while refining…"). Take a line's number from the `.txt`
  or from `messageNames`, never from that header (`tools/client-tables/commands.py` does so now; the numbers
  written by hand in `client/src` were compared with the `.txt` by script and agree).
- At start the gateway prints `attempt to index global 'io'` for `System/mapInfo_true_EN.lub` and
  `System/Towninfo_EN.lub` (`gateway/src/tables/maps.rs` through `tables.rs beside`; `state` gives Lua no
  `io`). The error is in each file's last lines (an `io.open` of an optional `SystemEN/mapinfo_C.lub`,
  `SystemEN/Towninfo_C.lub`), after the tables are filled. Nothing is lost: neither file is in the client's
  folder, and the lines are the translation's own, not the client's. Only the two lines printed are left.
- It also prints `attempt to index global 'JobSkillTab'` for `skillinfoz\skilltreeview.lub`
  (`gateway/src/tables/skills.rs`). Lost by it: the eleven calls of `JobSkillTab.ChangeSkillTabName(job,
  "1st", "2nd", …)` that end the file and name the skill window's tabs for some jobs (the client's
  `JobSkillTab_GetTabName`, `UINewSkillListWnd`); the places of the skills stand before them and are read
  whole. The gateway gives no such names to the browser.

### Noticed during the second round of refactoring (left as they are, some pinned by tests)
- `scenery/views.rs held_turn`, `kept`: `rem_euclid(360.0)` of a tiny negative turn gives exactly 360;
  `held_turn` panics if a map's table has `rotationFrom` over `rotationTo` (tilt and distance are guarded).
- `scenery/views.rs kept` takes "x,10,45,300": what is no number is skipped, not refused.
- `bars::member` divides by `most` (both callers filter nought out); `bars::monster` does not cap a life over
  its most; `pose::direction_of` underflows for a facing over 12 (facings are 0 to 7 wherever read).
- `shops::kept` takes the first entry of a row the basket has twice, while the basket's window lists both.
- Gateway: `skilltreeview.lub` fails at its line 1603 (`JobSkillTab` is nil), so the tree is what the file
  built before; `System/mapInfo_true_EN.lub` and `Towninfo_EN.lub` fail on `io`, and the file without `_EN`
  is never tried after that, because the first file there is the one taken.

### Found in the game by the owner and the lead, not explained or not compared
- The readied arrow's name lies over the figure's face in the Equipment window; the places are the client's
  (`client/src/web/hud/equipment.rs equipment_ammunition`). To compare with a video.
- A trap cannot be pointed at or pressed, and a hidden trap shows to its setter only, not to the party
  (`client/src/web/world/skill_units/traps.rs`). Trap models: size not compared with the client.
- Flasher's flash (effect 99, `skill_units/traps.rs sprung_with`) is reported as not drawn; not re-checked.
- A monster's life bar empties before the arrow lands (the server tells life when damage is counted).
- "White washed-out figure right after being struck": seen once, cause not found.
- Shadows, fog strength (`scenery/fog.rs`) and a character hidden behind a tree: not compared by eye.
- `@autotrade` is given to every player by `server/conf/groups.yml`; it is switched there.

---

## 5. Refactoring left undone on purpose

### Risky: decide with the owner first (the plan's section 7)
- Merging `inventory::count_typed` and `shops::count_asked`: they differ on "12a".
- One state and window for the seven count boxes: changes ids, forms and four payload types at once.
- `name24` / `Writer::text` / `field_whole_chars` (`net/bytes.rs`) into one: only two stop at a whole letter.
- One queue for sounds; a place for placeless sounds: changes what is heard.
- Cast rings, lock-on and portal rings of `web/world/effects.rs` as primitives: looks may differ.
- Merging launchers in `web/world/prims/*.rs`: no test; small numeric differences change effects.
- `under_a_round` twins and `% 360` against `rem_euclid` in `prim/tei`: differ for negatives.
- `map::direction` against `route::facing`; `Gat` nearest-free against `ai/host.rs`: results differ.
- Hud near-copies with other colours or 1-px offsets, the tabs, the drop-downs, `title=` → `data-tip`.
- `PET_ASKS`, `GUILD_ASKS` (`web/hud/windows/pets.rs`, `guild.rs`) as one kind of question window.
- `render()` → `redraw(&[..])` everywhere: changes what is redrawn while typing or scrolling.
- An explicit list of overheard packets; folding `Action` variants; `press/things.rs` per system; one
  scheduler for timers: each changes the order events are handled in.
- Raw `data::message` → `App::words` where `tr` is skipped: changes Russian texts.
- The style sheet as a `.css` file: `build-web.sh`, `dev.sh` and `tools/limits.py` change together.
- Renaming `web/world/prims/*.rs` and `prim/tei/*.rs` by look: large churn, a generator reads the tree.
- Deleting packet constants and parsers nothing calls yet; pet and homunculus intimacy words as one table.

### Native rules and protocol (`client/src/net/**`, native modules)
- Rules still inside parsers of `net/`: the balloon's `SHOWN_FOR`, `SHOWN_AT_ONCE`, `LETTERS`, `LINE`
  (`net/broadcast.rs`: the parsers fill the announcement with them); `net/notices.rs overweight` (only over
  nought) and `damaged` (drops the place 0). `Boss::marked` / `says` (`marks.rs`) and `Experience::lines` /
  `colour` (`notices.rs`) are native rules written as methods of types that stay in `net/`, where the parsers
  build them.
- The 16 ms and 24 ms ticks have several homes: `effect/carried.rs TICK_MS`, `falcon.rs TICK_MS`,
  `pose.rs PASS_MS`, `blow.rs FRAME_MS`, `web/world.rs CAST_FRAME_MS` / `EFFECT_FRAME_MS`.
- Kept though nothing but tests names them: `map/gat.rs Gat::block`, `net/inventory/hotkeys.rs hotkeys_bar`,
  `net/notices.rs user_value` / `MANNER` / `ZC_ACK_PVPPOINT`, `net/packets.rs PARAMETER_SPEED`,
  `ai.rs command::NONE`, `whispers.rs answer::ABSENT`, `net/guild.rs menu::INFO` / `EXPELLED`,
  `chat_rooms.rs kind::ARENA`.
- `.gat` trimmed by hand beside `maps::bare`: `web/hud/worldmap.rs` (five places), `web/app/commands.rs`;
  also `scenery.rs`. `net/packets/chars.rs` is another rule and stays: it takes one `.gat` off, `maps::bare`
  takes off `.rsw` too and a repeated ending.
- `net/golden.rs` calls four renamed files (`units`, `items`, `trade`, `npc`) and four moved builders
  (`restart`, `whisper`, `config::set`, `ask_name`) by their old names, so that `golden.txt` stays as written.

### Gateway and tables (`gateway/src/**`, `client/src/web/data/**`)
- The debug commands of the gateway (`main.rs`: `items`, `chat-tabs`, `dump`, `lua`) read the client's files
  on their own, not through `tables::Sources`: each is one purpose and not on the way of `for_browser`.
- `js_sys::Math::random` picks in `web/data/pets.rs` and `song_talk.rs`; rAthena's YAML read by hand in
  `tables/items.rs`, `quests.rs`, `guide.rs`.
- Four changes of the refactoring that are not strictly the old behaviour, none seen on real data: the
  debug commands' parsing (`config.rs enum Command`), `items.rs with_english` would run a bytecode file, a
  tip's first page in the guide, the `commands` state.

### Windows (`client/src/web/hud/**`, `web/view/**`)
- Pure rules still under `web/hud`: the experience bar's hundredths under the pointer (`basic.rs
  experience`); the vendor's basket total (`vending/buying.rs`); the date of a guild's member
  (`guild/members.rs`, `%y.%m.%d`: there is no native `guild.rs` to take it). `quests.rs quest_lines` still
  gathers its rows in the window (each needs the tables and `App`); its rules are `quests::boxes` and
  `put_away_last`.
- Alike and kept apart: `status::attack_speed` (i64) and `homunculus::attack_speed` (i32) are one formula of
  two windows of the client; `basic::weight_share` is not `inventory::weight::percent` (a most of nought is 0
  in the first and counted as 1 in the second).
- Fields of the page named by hand (`field("room-title")`, `field("mail-sought")`… in `web/app/**`) instead
  of `Window::field` (`web/hud/windows.rs`).
- `web/view.rs` still has its own `states_ab` and `button`.
- The minimap's pieces (`minimap_marks`, `minimap_spot`, `minimap_place`) are in `web/view/game.rs`; there
  is no `hud/minimap.rs`.
- Fields and buttons left off the shared helpers because the HTML would differ: `guild-reason`,
  `party-invitee`, `mail-title`, the chat room's password, `hud/tips.rs page_html`, `hud/choosing.rs
  line_list`.
- `hud/chat.rs refresh_chat` writes the DOM itself; `hud/quests.rs` writes `z-index:10` inline; the mail's
  letter count and fee follow typing by inline script in `hud/mail/letter.rs`.

### App (`client/src/web/app/**`)
- The guild's result→line tables are inside `web/app/incoming/guild.rs` (no native `guild.rs`).
- `map_sign` writes HTML in `web/app/actions/travel.rs`.
- Drop targets named by element id in `web/app/dragging.rs` (`#exchange`, `#choose-line`, `#mail-write`,
  `#inventory`).
- `App::ai` is of type `Scripts` (`web/app.rs`, `web/app/state/ai.rs`): `Ai` is the interpreter's type.
- Readers in another system's file, left because moving one changes the order of readers: overweight in
  `incoming/notices.rs`, `ZC_WHISPER_LIST` in `incoming/commands.rs`, `ZC_USER_COUNT` in `incoming/status.rs`
  and `incoming/commands.rs`.
- No shared `App::ask_count` (the seven count boxes differ) and no `App::toggle` of a window.
- Catch-all arms left: `clicks/chat_rooms.rs room_menu_pressed`, `clicks/guild/manage.rs
  guild_right_pressed`, `clicks/storage.rs storage_stowed`.
- `keys.rs game_key` is one long guarded match; `clicks/skills.rs` has the bare `skill == 93`; `FADE_MS` and
  `LOADING_PICTURES` stay in `web/app.rs` though only `app/page.rs` uses them.

### World and primitives (`client/src/web/world/**`, `prim/**`)
- Pure rules still under `web/world`: `pursuit.rs`; in `camera.rs` the vector arithmetic of `Camera::view`
  and `View::ray` / `project` / `pick` (tied to the types of the world and to `Gat`) and `shaken` (draws a
  lot of the browser); `overlays/pointer.rs` (`DISTANCE / distance * height / 480.0`). `pose.rs` keeps only
  what writes to a `Mover`.
- `web/world/loading.rs`, `land.rs` and `launch.rs` keep their own Korean decoders instead of `web::korean`:
  they are the same with a decoder and differ without one, where `web::korean` reads the bytes as UTF-8:
  `loading.rs` fails the map, `land.rs` shows no model, `launch.rs` plays no effect.
- No shared `ticks_due`: each catch-up site has its own bound. Cells are turned into world places by hand in
  `camera.rs`, `scenery.rs`, `frame/marks.rs`, `frame/effects.rs`.
- Action numbers are literals where `pose` has names (characters' and monsters' are mixed).
- `State` (`web/world/state.rs`) is not grouped by system.
- Near-copies in `prim/tei` (`waits`, `over_once_gone`, `to_screen` in `rect.rs`) and the `PICTURES` /
  `fn sounds` tables of `web/world/prims/*.rs` are not merged.
- "Not ported…" headers in `prim/horn.rs`, `particle.rs`, `slots.rs`, `walls.rs`, `plate.rs` were not each
  read against the code again.

### Written twice and left (`python3 tools/duplicates.py`: 11 groups, 93 lines, of 21 groups and 218)
- `scenery/makers.rs heard_as_it_was` (and its last lines against `earshot.rs`), `prim/tei/max_level.rs
  swirls`, `prim/tei/forest.rs light`: a test's own copy of the rule, kept whole to hold the rule to it.
- `net/notices/tests.rs a_line_takes_what_it_asks_for` is the test of `numbers.rs` once more: no test's name
  was to be lost.
- `web/world/shaders.rs`: two programs of WGSL share lines of their text; a text is not divided here.
- `web/world/prims/*.rs`: `known` / `thrown` / `run` begin alike in every file of effects (the table and the
  rule differ), `blows.rs cloud_streaks` / `cloud_spread` draw their lots in another order: see "Risky".
- Alike in shape only (`--shape`, `--loose`): the parsers of `net/` (each another layout through the same
  helpers), `cart_counted` / `storage_counted` (the seven count boxes, "Risky"), `guild_oppose_answered` /
  `guild_invite_answered` (two tables of the client), `Window::question` / `pick` (another width, in a `const
  fn`), the homunculus's and the mercenary's windows and presses (two systems with rules of their own).
- Lines over 160 characters left (4): Lua, JavaScript and tests' tables inside a text
  (`web/app/screenshot.rs`, `ai/tests.rs`, `gateway/src/tables/quests.rs`), which are the text's content.

---

## 6. What does not work in a browser at all

Only what the code or its comments say. Keys the browser never hands to a page (Ctrl+Tab for the minimap,
Ctrl+T for the mercenary's stand-by, Alt+F4) are bound in the client's table (`client/src/hotkeys.rs`), and
the code has no note on them: whether they arrive was not checked.

### Options, hotkeys
- F11 (the client: windows put away and brought back) is left to the browser's full screen: `web/app/page.rs
  browsers_own` passes it by, and `web/app/keys.rs interface_key` has no arm for `interface::WINDOWS`.
- In the game the page keeps for itself the other function keys, Alt with a key, Tab, the scrolling keys and
  some Ctrl page commands; typing, copying and reloading stay the browser's (`browsers_own`).
- A key pressed while something else is being handled is left to the browser (`web/app/wiring.rs wire_keys`).
- A MacBook lacks Insert, Home, End, Page Up, Page Down, Print Screen, Scroll Lock, Pause and the number pad
  (`hotkeys.rs missing_on_mac`); such a key cannot be set there (`web/app/hotkeys.rs assign_key`).
- The Game Settings' graphics page is empty: the browser sets the graphics (`web/hud/settings.rs`).

### World
- The pointer is a picture the page shows (`web/world/overlays/pointer.rs`, `cursor.rs`); the system's
  pointer cannot be moved or snapped by a page, and nothing here does.
- A screenshot is put together from the canvas and a copy of the windows; in a browser that will not read
  back such a canvas the windows are left out, and a fetched font is replaced by the browser's own
  (`web/app/screenshot.rs`).
- WebGPU where the browser has it, WebGL2 otherwise (`web/world/gpu.rs`).

### Sound
- Nothing plays before the first press on the page: the music starts with it, and is asked for again twice a
  second until then (`web/sound.rs`, `web/app/wiring.rs`).
- A tab in the background falls silent (`web/app/screen.rs tune`).

### Companions, files
- A page reads no file of the player's disk: AI scripts and their notes go through the gateway and memory
  (`ai/prelude.lua`, `ai/host.rs reads`); what the client keeps in `SaveData` is in the browser's storage,
  which may be switched off — then nothing is remembered (`web/app/page.rs stored`).

## 7. Of the classic game, named in the code's comments as not made yet

Small things of the classic client that the code says it does not have. They are no Renewal content, so they
are not in `BACKLOG.md`. Whether each is classic was decided by which job or window owns it, not by reading
the decompiled client.

### Skills and effects
- The copies of `EFST_NJ_BUNSINJYUTSU` (617, the Ninja's) are not drawn; `EFST_DEVIL1` does not darken the own
  screen (`EF_BLACKDEVIL`, the `EF_DEVIL`s); nor are the effects of `EFST_PROPERTYUNDEAD` (655, 656),
  `EFST_SMA_READY` (546), `EFST_RG_CCONFINE_M`/`_S` (604), `EFST_WZ_SIGHTBLASTER` (601), `EFST_DA_CONTRACT`
  (586), `EFST_WIDEWEB` (404) — `client/src/state_effects.rs`.
- `EF_BURNING`, `EF_BOTTOM_BASILICA2`, the Angelus effect, the `ST_SLEEP` state, `EF_BLACKBODY` —
  `client/src/ailments.rs`; body flags 0x80000 and 0x100000 — `client/src/body.rs`.
- The sparks of `AlphaDownRipples` (`client/src/web/world/effects.rs`); the smoke of `AlphaCenterRise`
  (`client/src/web/world/prims/bursts.rs`).
- `CSkillInfo::UsesLevel` tables are not read: every used skill has the level arrows —
  `client/src/web/hud/skill_list.rs`.

### Chat, party, guild
- `/setcamera` (`TT_SET_CAMERA`) and `/viewpointvalue` (`TT_VIEW_CAMERAINFO_ON_OFF`) are in
  `client/src/commands.txt` and do nothing.
- Chat-room menu rows (look at a character, write to a file) — `client/src/web/hud/chat_rooms.rs`.
- Letters to party members, the small windows of single members, Lock and Sort —
  `client/src/web/app/clicks/party.rs`.
- `UIGuildHelperWnd` (the guild key with no guild opens nothing) and `WID_CREATEGUILDWND_MODE_1` (the window
  that disbands) — `client/src/web/app/clicks/guild.rs`.

### Items, exchange, companions
- The second `PriceColor` (a ground behind the zeny) — `client/src/web/hud/exchange.rs`.
- An egg's description — `client/src/web/app/incoming/pets.rs`.
- The other settings of `ZC_CONFIG` — `client/src/web/app/incoming/viewequip.rs`.

### Quests, maps, tips
- The menu on a quest under the right button — `client/src/web/hud/quests.rs`.
- The monster's picture in Monster Info — `client/src/web/hud/fight.rs`.
- World Map search and the Kafra's arrows; the large map's boxes for the marks of quests, towns and the guild,
  and its notes — `client/src/web/hud/worldmap.rs`.
- The Tipbox: a tip's picture and the button that links a tip into the chat — `client/src/web/hud/tips.rs`.
- Fireworks weather (`pokjuk` of `etcinfo.txt`) — `client/src/web/world/scenery.rs`.
