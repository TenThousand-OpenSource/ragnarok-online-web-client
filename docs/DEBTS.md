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

### Fog
- The fog goes out with the camera beyond 400 (`scenery/fog.rs Fog::seen_from`, used in `web/world/gpu/land.rs`):
  by the owner's decision. In the client the fog's start and end stay where the table puts them, and with
  Zoom Out at 730 a map such as gef_fild13 is all fog. Not compared with the running client.

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
- A Talkie Box and Graffiti ask for their words as soon as the place is pressed; the client asks when its
  character has come within the skill's reach (`AM_REQ_GROUND_SKILL`), for it walks there itself
  (`web/app/clicks/skills.rs skill_aimed`). The words go as UTF-8, cut at a whole letter within 20 bytes; the
  client cuts at a byte (`net/skills.rs use_on_ground_with_words`).
- The sounds the client's handlers of a skill start (`effect_tables/skill_sounds.txt`: the songs, Berserk,
  Gospel…), the monk's combos' and the fireworks' (682 to 686) the client plays whole, wherever in sight the
  skill is used (`PlayWave(name, 0, 0, 0, …)`); here each is heard from where the one it is of stands
  (`world::sound_on`, `world::ground_skill`), as every other sound of the world.
- The cast circles 12 and 54 to 59 are written twice: a cast draws them as plain geometry
  (`web/world/effects.rs cast_rings`), an effect by its number (`world::burst`, what an NPC's script sends)
  as the client's primitives (`web/world/prims/walls.rs`). The cast was left as it is until the new ones
  are seen.
- A map's emitters (`web/world/scenery/emitters.rs`): a frame is taken as 16 ms though the emitter's figures
  mean 1/60 s; the client's fog is not put on the particles; the blends 7 and 8 are taken as one and zero.
- With the client's option for fewer effects (`TT_MIN_EFFECT_ON_OFF`) taken as off, Meteor Storm, Lord of
  Vermilion, Storm Gust and Quagmire show their full effects only (`skill_units.rs ground_effect`).
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
- **For the owner to decide.** `skillinfolist.lub` has `SpAmount` (SP at each level) and `_NeedSkillList` /
  `NeedSkillList` (what a skill asks for) for every skill, and the Windows client shows from them the SP of
  a skill at the level chosen in the skill window, on the hotkey bar and in its tip
  (`UINewSkillListWnd.cpp:345-362`, `CSkillInfo::UseSpAmount` through `GetLevelUseSpAmount`) and the skills
  asked for in the tree (`c_AddNeedSkillList`, `Source/Script/LuaScript.cpp:900`). In this client's files the two are Renewal's wherever the games differ:
  of the 492 skills up to 700 with an `SpAmount`, 417 are the same in rAthena's `db/pre-re` and `db/re`, 44
  are `db/re`'s alone (Brandish Spear 24 where the classic server takes 12, Chain Combo 5 to 9 for 11 to 15,
  the songs), 6 `db/pre-re`'s alone, 25 neither's; of the requirements 403 are the same, 19 `db/re`'s alone
  (Full Strip asks for Strip Weapon 5 alone), 1 `db/pre-re`'s. Not read here: this client says "Sp : n" with the
  server's figure for the level known (ZC_SKILLINFO_LIST) whatever level is chosen, and shows no
  requirements; the tree's places come from `skilltreeview.lub`. To decide: the client's table as it is (as
  the Windows client would show on this server, wrong for some seventy skills), or the classic figures from
  the server's database through the gateway (`db/pre-re/skill_db.yml`, `skill_tree.yml`, as the Adventure
  Guide's are read), or as it is now.

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
- Refining at the smith NPCs does nothing on the owner's server as it is set: rAthena's `feature.refineui` is
  on by default (`rathena/conf/battle/feature.conf:85`), so `npc/merchants/refine.txt:527` calls `refineui()`
  and the server sends ZC_OPEN_REFINING_UI (0x0aa0), a window of a later client that is in `BACKLOG.md` and is
  not made. For a classic server the owner would add `feature.refineui: off` to
  `roweb/server/conf/battle_conf.txt`: the same NPCs then talk the old way (the script's own branch: a menu of
  what is worn, the price, the chance, `successrefitem` / `failedrefitem`), which this client reads. Not
  changed here: the server's settings are the owner's.
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
- The words that are not let be written (`manner.rs`, the client's `manner.txt`) are asked in the places
  `docs/DONE.md` lists under Chat. Left of what the client does with them: what is typed for an NPC is not
  turned red in its line (`UINpcTextEditDialogWnd::SendMsg`); the guild's own name, the names of its
  positions and of the guilds allied and opposed are kept as the server sent them, for they are sent back
  and compared (`ReceiveGuildInfo`, `ReceiveGuildPositionNames`, `ReceiveGuildRelationList`,
  `ReceiveAddRelatedGuild` put stars in them); a new character's name with such a word is refused in this
  client's own words, not line 20; a stall's title and a guild's name are refused in the chat where the
  client has a message box (as the other refusals of those two are here); a mercenary's name under the
  pointer has the stars a homunculus's has (`web/world/names.rs`: the two are not told apart there).
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
- The guild's castles (`web/hud/guild/castles.rs`): the line picked in the list has the band
  `hud/choosing.rs` gives a `UIListBox` (the client's is cell 3, 2 of the skin's colours, which was not
  settled), and the row of the menu under the pointer the colour of every menu's row here (the client's is
  cell 6, 2); past twelve castles the list scrolls with no bar drawn; the two boxes are not dragged and
  Enter and Escape do not answer them.
- The guilds allied and opposed (`web/app/clicks/guild.rs guild_related_pressed`): the right button on a row
  no guild is in puts away a menu left open by an earlier pick; the client leaves that menu open, and its
  row then does nothing. The client's row is hard to hit by design (87 of the list's 168 pixels across take
  the button, 80 are marked, nothing shows under the pointer): kept as it is.
- After a change of map the castles are forgotten, as in the client (`CGameMode::InitializeState`), and are
  told again only when the guild window's first tab is opened anew: whether the client's window, left open
  across maps, asks again by itself was not found (`web/app/clicks/guild/castles.rs guild_castles_moved`).

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
- A map that `data\mapNameTable.txt` names keeps the sign `mapInfo_true.lub` gives it. By the decompiled client
  it would show none: `CSession::LoadMapNameTable` (`Source/Contents/Map/MapNameTable.cpp:21`) calls
  `AddMapDisplayName(map, name, false)` for every line after `mapInfo_true.lub` was run (`CMapInfoMgr::Create`
  in `CSession`'s constructor), and the `false` is "the sign is shown". With the owner's files that would take
  the sign off 790 maps (Geffen, Morroc, Payon… ; Prontera is not in the file). Only the name is written over
  (`gateway/src/tables/maps.rs renamed`) until this was looked at in the Windows client: the owner decides.
- A map's files are asked of the gateway by the name without the number of its copy (`maps::original`); the
  client finds a copy only by the line `resnametable.txt` has for the map (`1@tower.gat#1@tower.gat#`), so a
  copy of a map with no such line would not load there and does here.
- The weather of `etcinfo.txt` and the marks of `Towninfo.lub` are looked up by a copy's name as the map's own
  is in the world (`web/world/loading.rs`), and by the server's name on the minimap; the client's rule for
  those two was not looked for.
- The eight PVP rooms `pvp_y_1-5` … `pvp_y_8-5` do not load: `resnametable.txt` has their `.rsw`, `.gnd` and
  minimap (Morroc's) and no line for their `.gat`, the archives have no such file, and the client has no other
  table of names (`ResourceMgr::Resolve`, `Source/Resource/ResourceAliases.cpp:17`, reads that one file). So the
  Windows client cannot open them either with these files; no line was made up here. (rAthena's PVP NPCs do
  not warp there: the rooms are only in `conf/maps_athena.conf` and the map flags.)
- A boss's mark is left out when the boss is outside the part of the minimap shown (`web/hud/marks.rs`).

### Options, hotkeys
- The Hotkey Guide writes the player's own keys beside its eighteen things (`App::bound`); the client asks its
  Lua table (`GetHotKey`), and whether that holds what the player has set was not followed
  (`web/hud/hotkeys/guide.rs guide_list`). On a Mac the guide still shows the client's keyboard.
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

### Granny models
- The curves of an animation are played as plain B-splines with their ends held (`model/granny/pose.rs`);
  Granny wraps the ends of a looped one round, and its own sampling was not read out of the DLL.
- A model turns to a new facing at once; the client turns it 6 degrees a frame (`C3dGrannyGameActor::
  OnProcess`; `web/world/granny.rs`).
- The light of the models other than the flag leaves out the wind's direction, which the client adds to the
  eye's place before making it one long (`shown::light`); for the flag the sign of "down" in the client's
  view was taken as given by its y pointing down, and was not compared with the Windows client.
- A model struck down fades as every corpse here does (255 steps of 24 ms); the client's `CGrannyCorpse3d`
  fades from its tick 355 to 600. Which actions the client asks for once and which over and over was not
  followed through: a blow, a flinch and a fall play once here (`shown::Action::looped`).
- The texture packing is not proven the same to the last bit as the DLL's (nothing to run the DLL with): see
  the head of `model/granny/bink/wavelet.rs`. S3TC textures and raw ones without alpha are not read: no file
  of the client has them.
- An emblem on a flag is drawn smoothed (the usual sampler); which filter the client draws it with was not
  found, nor what its texture of 32 by 32 holds beside the 24 pixels of the emblem (taken as nothing).
- `dragon_5.gr2` (job 1181) and `Hugeling90_6.gr2` (1284) are named in the table of jobs and are not among
  the client's files: those two are not drawn at all (the Windows client would stop with an error).

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

### The names of sprite files by the client's tables (`dress.rs`, `creatures.rs`)
- Never seen in the game: a reborn job's weapon and shield, the clothes' dye by the client's palette names
  (and that colour 1 is the sprite's own for all but the first reborn jobs, which always wear palette 1:
  `dress::clothes_colour`), a mercenary, an NPC that wears a monster's sprite, a disguise and
  `setnpcdisplay` (`web/world/actors.rs reshaped`). The files were all asked of the gateway
  (`tools/sprite_files.py`).

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
- The box of a Talkie Box's and of Graffiti's words, and the packet sent from it (`web/hud/skills.rs
  skill_words_window`, `net/skills.rs`): never used in the game.
- Every coded effect of `client/src/web/world/prims/*.rs` (40 files) and `prim/**`, except the few seen:
  bow attack, Double Strafe, Arrow Shower, Improve Concentration, Blitz Beat, Detect, Ankle Snare, Land Mine,
  Skid Trap, Claymore, the level-up and job-change effects, Fire Wall.
- Effects played from files (`effect.rs`, `effect/**`, `web/world/launch.rs`) beyond those skills.- Everything the audit of the classic game added (`docs/DONE.md`, section 4 "What the audit … found" and
  section 18 "Effects of classic maps"): the sounds of 44 effects and of 82 classic skills, about 95 effects
  drawn by code, the units of the later classic jobs, the monk's combos, the emitters of Prontera, Morocc
  and Comodo, the invisible NPCs as places of effects. Look first at: the sounds of Sanctuary and of a
  Bard's song, Meteor Storm, Heaven's Drive, an NPC's `specialeffect` 4, 54 and 71, the waterfalls of
  Yggdrasil, the ghosts of Niflheim, the dust of Morocc, a potion's effect 7 (now `Exit`).

- States on a body (`ailments.rs`, `state_effects/**`, `body/**`, `web/world/ailments.rs`), spirit spheres
  (`spirits.rs`), ground units other than the traps seen (`skill_units.rs`, `web/world/skill_units.rs`),
  cast circles (`scope.rs`), cooldown shades (`cooldowns.rs`), the aura (`aura.rs`, `web/world/aura.rs`).

### Fight
- PVP and its ranks (`pvp.rs`, `net/pvp.rs`, `web/hud/pvp.rs`, `web/world/pvp.rs`); melee and combos
  (`blow/**`, `web/world/combat.rs`) beyond the bow.
- The action a monster is shown in for a skill (`monster_skills.rs`, `web/world/mover.rs shows`): which
  action of a sprite the table's number is was taken from `SetAction(action * 8 - 8)` and not compared with
  the Windows client; an action that goes round ends here when the monster walks or begins a motion, where
  the client ends it at the monster's next state whatever that is.

### Items
- Cards and making: set card, forge, mend, identify, warps list (`choosing.rs`, `net/choosing/**`,
  `web/hud/choosing.rs`); viewing another's equipment (`net/viewequip.rs`, `web/hud/viewequip.rs`).

### Shops, storage, cart, vending
- The panel over the basket of what is bought (`web/hud/shops.rs shop_compare_window`): beside an NPC's shop
  the client's code never tells it of an item to compare, so it shows what is worn twice; whether the
  Windows client of this version really does only that was not looked at. Its place is not kept between
  visits (the client keeps it in the registry). The marks are drawn 8 by 8 where GDI's region leaves the last
  row and column out.
- NPC shops (`shops.rs`, `web/hud/shops.rs`), storage (`storage.rs`, `web/hud/storage.rs`), cart (`cart.rs`,
  `web/hud/cart.rs`, `web/world/cart.rs`), vending both sides (`vending.rs`, `web/hud/vending.rs`,
  `vending/buying.rs`).

### Chat
- The words that are not let be written (`manner.rs`): every place they are asked in. With the client's
  file the stars fall inside ordinary words ("class", "Assassin") and into @commands: as the Windows client
  does it, by reading.
- The away message: `/am`, its box, the answer whispered back (`web/app/clicks/whispers.rs`). Escape with the
  box open switches the message off and leaves the box, as the client's code reads (`web/app/keys.rs`): OK
  pressed after that says the message is on while it is off. Not seen in either client.
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

### Granny models
- The guardians' other actions, a model's life bar and its fading when
  struck down (`web/world/granny.rs`, `web/world/gpu/granny.rs`).

### Options, hotkeys, sound, gateway
- The Hotkey Guide (`web/hud/hotkeys/guide.rs`): its texts' places over the picture, the keys lit while a
  button is held (`.held` of `client/web/index.html`, in a `<button>`), and its button to the Shortcut
  Settings, which opens them on the second tab ("Hotkey Bar 2": the client's `SelectTab(1)`). The buttons
  that tell of a level gained and the marks on the menu's icons (`web/hud/status.rs`, `web/hud/basic.rs`).
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
- Whether Escape closes the box of a Talkie Box's words: `IsOrderFixed` names Graffiti's box and not the
  Talkie Box's, but `CloseWindow` has no arm for `WID_TALKBOXTRAPINPUTWND`; here neither box is closed by
  Escape (`web/hud/windows/skills.rs SKILL_WORDS`).
- Whether sight-state effects count loop passes or ticks: `state_effects/kept.rs`; the same for a body's
  passes: `web/world/ailments.rs bodies`.
- What `renderFlag & 0x200` does to a played sprite: `web/world/prims/played.rs`.
- The colour of Water Screw's parts (the client's function writes none): `prim/fourfold.rs`,
  `web/world/prims/prints.rs`.
- What the client reads past the 360th entry of its cosine table: `prim/tei/beams.rs fan`.
- Where the client takes its own falcon and mount from at login: `web/app/incoming/statuses.rs rides`.

- The planes of an area a skill warns of (`web/world/scope.rs warned`): how `CParticle` blends them by
  default was not found (they are drawn see-through, adding no light), nor which way up the picture lies, nor
  how long a tick of `CWarningPlaneEffect::OnProcess` is (taken as the circles' 16 ms). `GetSkillScale` is
  compiled Lua: it was read by the names it holds (`SKILL_INFO_LIST`, `SkillScale`, `x`, `y`), not run.
- A ranking told by numbers (`web/app/incoming/commands.rs ranking_looked`): `CShowRankingScheduler::Process`
  as decompiled compares names with an empty global (`g_namelessName`) in a way that cannot be followed to the
  letter; what is done here is its plain sense (wait up to three seconds for the names, say "Fetching rank..."
  at each look, then "Nameless" for the unknown).

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
- An alliance refused: rAthena answers the one who asked with 3 of ZC_ACK_REQ_ALLY_GUILD
  (`guild_reply_reqalliance`), which is the client's "The other guild has too many Alliances." (line of
  `REQALLYGUILD_OVERSIZE`), not its line for a refusal (1): the chat says what the client would say to that
  answer. Seen in the game.
- Whether the client's menu on another player has the master's two rows (alliance, opposition) for a player
  of no guild was not looked up: here it has them (`web/hud/player_menu.rs`).
- ZC_CHANGE_AGIT_OWNER (0x0b2b, `CGameMode::ReceiveGuildAgitOwnershipChange`: a castle is added to the
  guild's list or taken out of it) is not read: rAthena has no such packet and its length is not in our
  tables.
- Going to a castle of the guild's on the classic server: rAthena drops CZ_REQ_MOVE_GUILD_AGIT without an
  answer unless the castle has `WarpEnabled: true` (with `WarpX`, `WarpY`) in `castle_db.yml`
  (`clif_parse_guild_castle_teleport_request`); `db/re/castle_db.yml` has it for every castle,
  `db/pre-re/castle_db.yml` for none, so on the classic server the box is answered and nothing happens. It
  is no `battle_conf` setting; the owner decides whether `db/import/castle_db.yml` gets those lines. The
  server takes the zeny its own database names (`WarpCost`, `WarpCostSiege`: 100 and 100000 unless set), the
  box names the client's table's. rAthena never sends result 2 of ZC_REQ_ACK_MOVE_GUILD_AGIT.

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
- Shadows and a character hidden behind a tree: not compared by eye.
- Fog (`scenery/fog.rs`): its strength is the client's sum, checked in the client's program itself
  (`DepthAlpha` at 0x55D190: `(1/rhw − 10) / 1490` between the table's two figures, which nothing scales by
  the camera; the view is a plain look-at in world units) and held by tests. So with Zoom Out (the camera at
  730, on by default as the fog is) `gef_fild13` (fog from 308 to 680.5) is one flat blue and `prt_fild08`
  (382.5 to 904) two thirds fog at the character's feet, by the client's sums as here; at the 400 without
  Zoom Out they are a quarter and next to nothing. Whether the Windows client's screen really looks so was
  not seen (no Windows client was run): if it does not, the cause is outside what was read (Direct3D's
  handling of the fog of corners already placed), not in these sums. Known and left, against the client:
  - sprites (characters, monsters, items) and effects are not fogged; the client fogs every corner it queues
    (`RenderQueues::Submit` → `ApplyDepthAlpha`) and switches the fog off only for its last lists
    (`CRendererDX9::DrawQueues`), so there a character fades into the fog with the ground;
  - the colour the screen is cleared with is not mixed with the fog's by the colour's first byte on a map
    whose sky asks for it (`CRendererDX9::ClearWorld`, `IsBackGroundFog`);
  - the view here goes to 5000, the client's ends at 1500 (`ApplyFog`'s projection, `WorldViewSpatial.cpp`).
- The mark of the cell under the pointer (`web/world/frame/marks.rs`) lies on the cell's own four corners as
  the client's (`GroundGeometry::DrawGrid`, `CellRectangle`), but lifted 0.4 over them where the client draws
  it nearer by 3.05e-5 of the depth; its colour (the client's 0x6600FFAA over `grid.tga`) was not compared.
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
  (`guild/members.rs`, `%y.%m.%d`: not moved into the native `guild.rs`). `quests.rs quest_lines` still
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
- The guild's result→line tables are inside `web/app/incoming/guild.rs` (not moved into the native `guild.rs`).
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

### Characters and those who are no players
- The gate that has the client not even look for a shield's sprite on some jobs (`CSession::HasJobShieldSprite`:
  a mage or an archer shows a guard only, a job on a bird no shield of its own number) is not ported: a
  shield is shown wherever its file is there (`web/figure/compose.rs shield`). `tools/sprite_files.py` lists
  the files that are not.
- The weapon is found as before the tables were ported: by the item's own number first, then by its kind
  (`web/figure/weapon.rs`); the client's `CPc::SetSprWeapon2` and `IsJobWeaponTypeAllowed` (which weapon a job
  shows at all, the Star Gladiator's light) are not ported. Only the job's folder and name are the client's
  (`dress::weapon`).
- The job a character's costume is of (the packets' `body`, the client's `bodyStyle`, which `GetSpriteJob`
  names every part by) is taken to be the job itself: a classic server sends no other (`dress.rs`).
- An NPC that wears a player's job is drawn as the client's `CCharacterNpc` draws it (`web/world/actors.rs
  figure_of`), but the rest of the world takes whoever is drawn as a character for a player (its name, the
  pointer over it, a press on it). No classic script places one (`tools/sprite_files.py` walks them).
- A pet's actions with its accessory on are looked for among the monsters (`몬스터/…`) whatever the pet's
  own folder: every pet of the classic database is a monster.

Small things of the classic client that the code says it does not have. They are no Renewal content, so they
are not in `BACKLOG.md`. Whether each is classic was decided by which job or window owns it, not by reading
the decompiled client.

### Skills and effects
- Effects of the classic game still not drawn: `EF_POISON` 335 (`POISON(0)`, primitive kind 72), `EF_ANGEL2`
  338 (`SuperAngel`, kind 85; the item 12315), `EF_THROWITEM` 298 (`ThrowItem`, kind 53), `EF_ACIDDEMON` 537
  (kind 118; nothing in the client was found to launch it), `EF_RO2YEAR` 545 (its sprite is not in this
  client's files), the units of Evil Land (199, `EF_EVILLAND` 1555), the effects 650 and 651 the maps
  ice_dun03 and ice_dun04 place; the poses the monk's combos give the striker (`SetFrameSequence`); the falcon
  sent to the place of Detect. `effect_tables/effects_missing.txt` lists what the tables of skills and of
  items reach.
- Left of what was ported (said in the comments): `EF_ENHANCE` 9's yellow ring has no width, as the client's
  figures give it; `EF_NAPALMBEAT` 32 bursts every tick of 15, as the decompiled test reads (three, if the
  test was "every fifth tick"); the dome of 438 keeps the way its actor faced as it began; the ghosts are
  drawn among the other sprites of effects (`renderFlag 0x200` puts them in another queue of the client's);
  `Prim::fade` sets the speed of fading every tick where the client sets it once; Remove Trap's sparks end
  with the trap's unit, which the server takes away at once; only the lights 0 and 1 of the kind 29 and the
  branch 2 of `RenderGroundFog` are ported; the emitters turned by their own angles (`billboard_off`) and
  the wind, animated emitters, magic floors of the `effecttool` files, which no classic map uses, are in
  `BACKLOG.md`; `EF_DUSTSTORM`'s emitter (`CGameMode::ReceiveNotifyEffect`) is not made.
- A script's effect 92 (Meteor Storm through `specialeffect`) plays its file without the shake of the screen:
  an effect that is files is not also run by code, but for what stands on the ground (`launch::burst`).
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

### Chat, party, guild (the classic gaps)
- `EF_TOPRANK` over each of the first ten on a PVP map, with its rank (`CGameMode::ReceiveEffectNumber`,
  `Source/Client/Mode/GameModePackets.cpp:11481`; the effect is coded, `CRagEffect` at `RagEffect.cpp:23895`,
  kept until taken away and told its number with `AM_SETCOUNT`), and the time counted at the top of the screen
  while the player is first (`DrawTimeAttackTimer`) — `client/src/web/app/incoming/pvp.rs ranked`. It needs a
  primitive of `web/world/prims/` and an effect that stays on someone until it is taken off.
- The mark over one who may not talk (`CPc::SetHonor` for ZC_PAR_CHANGE_USER with a value under nought,
  `Source/Contents/Actors/Player.cpp:288`: the cash emotion `ChatProhibit`) —
  `client/src/web/app/incoming/notices.rs`. ZC_ACK_PVPPOINT has no reader in the client and stays dropped.
- The font of words over heads (`ZC_NOTIFY_FONT`, and the `font` of the packets that show someone:
  `UITransBalloonText::m_fontMode`) is not kept: the balloons are written in one font —
  `client/src/web/app/incoming/actors.rs`.
- The lines of ZC_ACK_GIVE_MANNER_POINT 3 and 4 are written with their `%d` unfilled, as the client writes
  them (`client/src/notices.rs manner_answer`).

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
