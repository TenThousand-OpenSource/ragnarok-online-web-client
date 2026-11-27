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
- Going round what is in the way to attack, to use a skill and to pick up — part of the far walk, ON by
  default by the owner's decision ("it must find the way"), off with `/farwalk`. The client has none of it:
  `CPlayer::UpdatePendingAction` (`Source/Contents/Actors/Player.cpp`) measures the distance alone, asks for
  the blow or the skill when it is within reach, and when it is not asks for the cell `range` cells from the
  target on the straight line to it (`CGameMode::FindWalkableCellToward`; a step towards a thing,
  `StepTowardWalkableCell`), with no search and no look at the line of sight; rAthena never walks a player and
  answers a blow or a skill asked across a wall with nothing (`unit_attack_timer_sub`, `unit_skilluse_id2`;
  ZC_ATTACK_FAILURE_FOR_DISTANCE only for the distance), so the client's character stands behind a pillar.
  Here in reach is what the server lets through (`map/sight.rs Gat::reaches`, `sees`: rAthena's
  `battle_check_range`, `path_search_long`), the character walks to the nearest cell from which it is
  (`Gat::way_into_reach`, in stretches when far; `web/world/scene.rs step_into_reach`,
  `web/world/pursuit.rs chase`/`close_in`, `reach`, `fetch`), and the fight, the skill or the thing is let be
  when there is no such cell within 20 cells of the two. Known and left: the server's extra cell of reach at
  a walking target is not counted.
- An order given while the character walks (a monster, a skill's target or a thing pressed) ends the walk
  before it is asked for: a walk to the cell the character is on and the request go together, the walk
  first (`map/sight.rs order`, `Order::Halt`; `web/world/pursuit.rs`, `web/world/frame.rs tell`). The client
  only asks (`CPlayer::UpdatePendingAction`). rAthena keeps a blow or a skill asked of one who walks and is
  not a cell nearer than the reach needs until the walk is over (`unit_attack`, `unit_skilluse_id2`:
  `stepaction`) — seen: asked at 209,204 in a walk to 202,204, the character walked on, stopped, and no blow
  came — and calls a blow off when a walk is asked for after it (`unit_walktoxy`: `unit_stop_attack`).
  Also this project's own here: a monster pressed again while fought is asked for again no sooner than the
  client's 1200 ms, and a press on another monster does not wait those out.
- One who walks does not sit down: the Sit key does nothing until the character stands
  (`web/app/keys.rs interface_key`).
- The server's clock is a line that does not run round after 2^32 ms (`server_time.rs`).

### Fight
- The monster being fought keeps its turning mark while the pointer is on another, which gets a second one;
  the client has one mark (`web/world/overlays.rs pointed`).
- Chasing: reach is judged as the server does (`Gat::reaches`: straight across less a tenth of a cell, in
  whole cells, and a clear line), not by the client's own measure (`web/world/pursuit.rs chase`); with
  `/farwalk` off, by cells along the longer side as before.
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
- The aura of the highest level is held to once a frame (`web/world/aura.rs` lists the differences): a job set
  anew does not begin a kept aura again; the client's `hasLevelAura` is not kept (one who came into sight while
  `/aura2` hid the auras, or invisible, shows the aura here as soon as it may, in the client only after coming
  into sight anew); a script's effect 200/976/977 on one who shows no aura is ended; the client's range of 130
  for effects is not held.
- The blue lights of the aura (976) are drawn 25 nearer the eye like every effect here; the client holds them
  at their own depth (`CParticle::Render`). The squares, the bands and the baby lights (200, 977) are at their
  own depth and under the bodies, as in the client. On a map whose RSW sets the renderer's `twoPass` the
  client's sprites write depth and would hide lights behind them: not done.
- What `/aura` and `/aura2` say is chosen by the row's tick (`data::option_says`); the client's code chooses by
  the value it keeps, which for the five options of `IsServiceOption` is the row's reverse
  (`NotifyServiceOption`, `GameSettings::ToggleOption`). By the code hiding auras would say "Aura effect is ON".
  Not settled without the client itself; left as it was.
- How the client's switches start is read from `CmdOnOffList` of `System\LuaFiles514\OptionInfo.lub` since
  2026-11-27 (`commands.rs starts`, `web/app/settings.rs option_starts`). Left: "reset" in the Game Settings
  gives that start too, where the client's gives the `Default` of its table; the rows were not compared one by
  one in the game with the Windows client; the list's values that are no switches of a command (`Trilinear`,
  `MouseExclusive`, the skin, `OptionInfoList`: the volumes, the camera's start) are not read.
- Effect files the client draws primitives beside, which are played here as the file alone: Aspersio (86: the
  drops that rise round one, `CRagEffect::Aspersio`), `EF_GLASSWALL` (13), Cart Revolution (170), the berserk
  potion (220). Their sounds are played (`effect_sounds.txt`). (Storm Gust's spikes are drawn since
  2026-11-27, `web/world/prims/grounds.rs`.)
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
- **The owner's decision, 2026-11-21: the classic figures.** A skill's SP at each level and the skills it asks
  for are read from the server's database through the gateway (`db/pre-re/skill_db.yml` `Requires.SpCost`,
  `db/pre-re/skill_tree.yml`, the jobs' numbers from `src/common/mmo.hpp`; `db/re` when the server plays
  Renewal: `gateway/src/tables/skill_needs.rs`, `web/data/skill_needs.rs`), not from the client's
  `skillinfolist.lub` (`SpAmount`, `_NeedSkillList` / `NeedSkillList`), which the Windows client asks
  (`CSkillInfo::UseSpAmount` through `GetLevelUseSpAmount`; `c_AddNeedSkillList`) and whose figures are
  Renewal's wherever the games differ (some seventy skills: Brandish Spear 24 where the classic server takes
  12). They stand where the client uses its table: "Sp : n" of the level a skill is used at in the list of a
  homunculus's or mercenary's skills (`UISkillListWnd`), "… Use Lv n (Sp : n)" when a skill of the hotkey bar
  is pointed at (`SkillTooltipFlat`), and in the tree the marks of what a skill asks for with the levels
  asked (`UINewSkillListWnd::UpdateRequiredSkills`, `DrawSkillTree`). With a gateway that finds no database
  the server's own figure for the level known is said and nothing is marked. This project's own in it:
  - A press on a skill plans a level and Apply sends what is planned (2026-11-27), but not in the client's
    order: a press on a skill whose needs are lacking plans the levels of what it asks for first and then its
    own, as far as the points go, and also of what those ask for in turn (`crate::skills::raises`); the
    client's `IncreaseSkillLevel` plans the skill and only what it asks for directly, in the order skill,
    needs, and sends them so. A skill the server has not listed is planned from the table (points, the job
    level asked, needs lacking). The client's refusal of a skill that cannot be planned (`canPlanLevel`, its
    line 3531 in the chat) is not said: such a press does nothing. Planned levels stay when the window is
    closed. A double press on a skill that can be raised plans two levels and uses the skill, as two presses.
  - The list of rows (`ID_MINI`): always 400 high, the corner that pulls it shorter (`m_btnDetail`, steps of
    32) is not made, and whether it is the list or the tree is not kept between visits (the client's
    `Simplicity_SkillList` of `OptionInfo.lua`); a planned level is written in the row's colour, not in red;
    a name ends 148 from its start as in the companions' list, the window being wider. A tree of more than
    six rows is not scrolled (`UpdateScrollBar` for the tree): no classic job was found with one.
  - The marks show while the skill's picture is pointed at (the page's style sheet) and stay while its
    description is open (the client's `m_showDetail` after the right button); the tab of another page that has
    a skill asked for is coloured only in the second case, not while pointing. The pink lies over the
    pictures; the client tints the picture (`DrawAuxiliary(2, …)`, not read).
  - The " (Sp : n)" the description window adds to the skill's name is not written (`UISkillDescribeWnd`,
    `UM_SKILLDESCRIBE`; where that text is drawn was not found).
  - Not read: what `skillinfo_f.lub` (compiled) hands `c_AddNeedSkillList` for a job, so whether the client
    lists only the direct needs was taken from the C++ alone.

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
- The owner's decision, 2026-11-21: a deal's zeny is written once, in the dark of the client's top text
  (`web/hud/exchange.rs exchange_zeny`). The client makes three texts (`CUIExchangeUI::OnCreate`,
  `UpdateZeny`): one in the colour of the amount's size a pixel to the left, one in that colour in place, the
  dark one over both a pixel lower, which leaves an edge of colour round the digits. The two coloured texts
  are not drawn, so the size of the amount is not told by colour.
- Refining at the smith NPCs works the classic way only with `feature.refineui: off`
  (`roweb/server/conf/battle_conf.txt`; in effect since the map server's restart of 2026-11-21, seen in the
  game). With rAthena's default (`rathena/conf/battle/feature.conf:85`, on) `npc/merchants/refine.txt:527`
  calls `refineui()`: the server sends ZC_OPEN_REFINING_UI (0x0aa0), a window of a later client that is in
  `BACKLOG.md` and is not made, and holds the player (`sd->state.refineui_open`, `pc_cant_act`): no walking,
  no NPC, until the map is changed, since this client never answers CZ_CLOSE_REFINING_UI. Seen so before the
  restart; nothing was built for it.
- What an NPC says that is taller than its window (a smith's warning before a risky refining: seven lines, the
  last "Do you still want to refine?") is cut by the window's lower edge until it is moved, as in the client
  with the English service's font: a line is 15 and 6 there (`CCreateLineInfo::ArrangeLines`, Arial of 15 for
  the family 0 at 12, `TextContext::SetFont`), so seven lines end 145 down in a viewer of 140, and
  `UITagTextViewer::AddText` does not move the text to its end. The viewer has the client's bar and the wheel
  and the arrows move it 30 (`scrolling.rs`).
- The storage stands where this client has always had it, not at the client's 360, 200
  (`web/hud/windows/storage.rs`).
- Opening a stall: what is wrong is said in the chat in red, not in message boxes
  (`web/app/clicks/vending.rs stall_opened`).
- The scroll bar (`scrolling.rs Bar`, `web/app/scrollbars.rs`, 2026-11-27) is the client's in its pictures,
  its handle's size and place, its arrows, its pages and the drag of its handle; the list beside it is still
  scrolled by the page, so: a list of rows found between two rows is taken as on the nearer; the pointer over
  the bar is not the client's (`SetCursorAction(2)`); a bar's length is its list's, where the client's window
  gives its own (`height - 38`, `height - 58`, `height - 87`: the same wherever it was compared). Lists that
  still have no bar, or the browser's: the emotions, the chat tab's list of kinds, the World Map's list, the
  castles of a guild, a chat room's members, the quest windows and the Adventure Guide (`CUIVScrollBar`, 7
  wide, `.viewer`), the achievements (`.achbar`), the mail, an item's and a skill's description, and the
  windows of the replay, the party booking and the combos (`.bar`, the browser's bar in the client's
  pictures). The chat's list is still moved by the wheel as a page moves it, not three lines a notch.

### Chat
- The chat's tabs (`chat_tabs.rs`, 2026-11-26). A tab is pulled out of the window once the pointer is more
  than 5 away from where it was pressed, every time; the client (`UITabControl::OnMouseMove`) pulls it out at
  the first move of the pointer, and after the first time only at a move more than 5 to the left of or above
  where the last one was pulled out (the owner asked that a tab not come out by accident). A tab's own
  window kept out of sight is brought wholly into the screen (`own_shown`); the client only puts one past
  the right side against it and one past the bottom at 100. The own windows lie over the pieces of the screen
  that are always there and under the windows that are opened; in the client they come to the front when
  pressed. The window that sets what a tab lists is put away when a tab goes out, comes back or is taken
  away; the client's follows the open tab. The names' widths are the page's, so the tabs are a little wider
  or narrower than the client's, and the limits of a name fall a letter sooner or later. Escape does not put
  away the line a name is typed in (a press on a tab does). Page Up and Page Down do not turn the list of a
  tab's own window.
- The count of players by the chat's tabs ("N online") is this project's own: the Windows client has no such
  indicator, only `/who`. It is fed by CZ_REQ_USER_COUNT, asked as each map begins and at every keep-alive,
  twelve seconds apart (`web/app/actions/{travel,link}.rs`; before 2026-11-21 every 36 seconds, so two players
  could read different counts for that long). A browser slows the timers of a tab that is not looked at, so
  a hidden tab's count can be older.
- `/packets` and its window: a developer's list of the map server's packets nobody reads; the client has
  none (`web/app/commands.rs`, `web/hud/unread.rs`, `net/unread.rs`, `press/unread.rs`).
- Whisper blocks (`/ex name`) last one visit; the client keeps them with the account (`web/app/commands.rs
  whisper_from`).
- The whisper window is of one size; the client's can be made larger (`web/hud/whispers.rs`).
- A coloured broadcast with a letter height of nought is drawn at 12; the client would draw nothing
  (`net/broadcast.rs coloured`).
- The rows of the chat's list (`chat.rs rows`, `web/hud/chat.rs chat_lines`): a row is 94 LETTERS where the
  client counts the bytes of the server's writing (a Korean letter is two); a line with a link to an item is
  not cut at the 94 letters (the client cuts through the link's own letters there); a row is cut by its width
  at the box less 16 whenever it is wider than that (the client asks whether it is wider than the box less
  20, and then cuts it at the box less 16); what
  stands in brackets is kept whole up to 40 letters, where the client asks whether it is narrower than the
  box; the letters' widths are the browser's. The `^RRGGBB` marks the client takes out of a line of the chat
  (`RemoveTextColors`) are left in it.
- "Allow all party invitations." and "Equipment information not open to the public." are said at every
  change of map, as the Windows client says them: its `CGameMode::ReceiveToggleNotice` (ZC_PARTY_CONFIG) and
  `ReceiveEquipmentViewState` (ZC_CONFIG_NOTIFY) write the line whenever the packet comes, and rAthena sends
  both after every map change (`clif_parse_LoadEndAck`, `if( sd->state.changemap )`). The owner found the
  repeating pair a bug (2026-11-26); it is left as the client has it until he decides otherwise.
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
- The window of the party's instance (`web/hud/instances.rs`, `web/hud/windows/instances.rs`): Escape puts it
  away as its cross does (what the client's Escape does to it was not found); the left button of its title
  bar is drawn and does nothing, as in this client's other windows (the client's folds the window to its
  title); the lines about entering are stood about their middle whatever their number, where the client
  stands a single line 55 down (its texts are never a single line); the place it was dragged to is kept as
  every window's is here, not in the client's two numbers of its own.
- The owner's decision, 2026-11-21: the menu on another player offers a guild's master "Send an Guild Alliance
  Request" only while that player's guild is not allied already, "Set this guild as an Antagonist" only while
  it is not opposed already, and neither on a player of no guild (`crate::guild::relation_rows`,
  `web/hud/player_menu.rs`). The client offers both on anyone who is not of the master's guild and looks at
  nothing else (`CGameMode::OpenActorMenu`, `Source/Contents/World/WorldObjects.cpp:814-828`).
- The small windows of the party's members (`party/mini.rs`, `web/hud/party/mini.rs`, 2026-11-26). A new
  one goes under the HEIGHT of the strip of menu icons (238), as `UIWindowMgr::GetMiniPartyUsableArea` reads,
  not under where the strip ends: with the strip open under Basic Information the first window stands over
  its lower icons. A member pulled out of the party window is carried as the picture of its job alone; the
  client carries the whole row (`UIDragMiniPartyWnd::Paint`). The whole text is in the page's own tip
  always; the client shows it by the pointer only when it is cut. With no free cell no window is made (the
  client makes one in a cell of no place). A press on a member uses only a skill of help that is aimed; the
  client's `TargetActorByAid` also takes another skill with Shift held or on a map where players fight. The
  gauge is not shown for a member whose life is not known or who is offline (the client moves it away in
  `UM_REFRESHITEM` and back in `Paint`: which of the two wins was not made out). The two narrower forms
  (`m_sizeMode` 1 and 2, `img_partymini_bg_mini.tga`) are not made: nothing in the client was found to ask
  for them. Line 3236 "Mini party windows can't be overlapped." is said by the client for a cell out of the
  grid, which `FindCell` never gives: it is not said here either. Not seen with two players.
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
- When the player's own guild is broken up, the flags of its castle go bare at once (`world::guild_gone`). The
  Windows client keeps the emblem on a flag in sight (`SetGuildInfo` refuses a guild of 0, the flag keeps its
  texture), and rAthena sends nothing for the flags of a castle left without an owner (`agit_main.txt`,
  `OnRecvCastle`: only an owned castle's flags are told `FlagEmblem`). For anyone who is not of the guild the
  flags in sight keep the emblem until they come into sight anew, as in the client: nothing tells them.
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
- Never seen in the game: a female character of a reborn job with its weapon and shield, a Gypsy, a
  disguise and `setnpcdisplay` (`web/world/actors.rs reshaped`; it turns one sprite into another and knows
  nothing of a change to or from the hidden NPC, job 111). The files were all asked of the gateway
  (`tools/sprite_files.py`). The rest was seen on a male character (`docs/DONE.md`).
- A mercenary's body shows no weapon apart from what its one sprite has drawn into it: so it is in the
  client (`CNpc`, one sprite), but nobody compared a striking Spearman with the Windows client.

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
- Graffiti's box of words; what a Talkie Box says when someone steps on it (`net/skill_units.rs talkie`): never
  seen in the game (the Talkie Box's own box was).
- Every coded effect of `client/src/web/world/prims/*.rs` (40 files) and `prim/**`, except the few seen:
  bow attack, Double Strafe, Arrow Shower, Improve Concentration, Blitz Beat, Detect, Ankle Snare, Land Mine,
  Skid Trap, Claymore, the level-up and job-change effects, Fire Wall.
- Effects played from files (`effect.rs`, `effect/**`, `web/world/launch.rs`) beyond those skills.- Everything the audit of the classic game added (`docs/DONE.md`, section 4 "What the audit … found" and
  section 18 "Effects of classic maps"): the sounds of 44 effects and of 82 classic skills, about 95 effects
  drawn by code, the units of the later classic jobs, the monk's combos, the emitters of Prontera, Morocc
  and Comodo, the invisible NPCs as places of effects. Look first at: the sounds of Sanctuary and of a
  Bard's song, Meteor Storm, Heaven's Drive, an NPC's `specialeffect` 4, 54 and 71, the waterfalls of
  Yggdrasil, the ghosts of Niflheim, the dust of Morocc, a potion's effect 7 (now `Exit`).

- Volcano, Deluge and Violent Gale (`web/world/prims/grounds.rs flare`) hide those standing in them almost
  wholly: 49 low walls of light, each mixed in at up to 180 of 255, all drawn after the bodies. The client
  draws them after the bodies too (their `renderFlag` 5 puts them in `RenderQueues::queue_01EC`, drawn after
  the sprites, which write no depth: `GameObject.cpp:883`, `AlphaQueuesDX9.cpp WritesDepth`), so the order is
  the client's; whether its walls are as dense was not compared with a picture of the Windows client (seen
  here 2026-11-27 in Volcano, from two tabs).
- A member's job in the party window after that member, or the one looking, came into the party or the game
  anew is what rAthena's char server read when it loaded the party (`int_party.cpp`: the class is read from
  the table once, `mapif_parse_PartyChangeMap` renews the map and the level only): a job changed since then
  shows as the old one until the server loads the party anew. ZC_NOTIFY_MEMBERINFO_TO_GROUPM, which the map
  server sends as the job changes, is followed (seen 2026-11-27 from both tabs). The server's, not this
  client's.
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

- The blue word that goes with the total of a kick (`web/world/combat/landing.rs`, `Sign::Word`) is the words'
  sprite tinted blue; the client draws `이팩트\bluemsg.spr` by `msg.act` (`MAKETYPE_BLUE_EFFECT`), tinted the
  same. Not seen in the game.

### Items
- Making: forge, mend, identify, warps list (`choosing.rs`, `net/choosing/**`, `web/hud/choosing.rs`;
  setting a card was seen); viewing another's equipment (`net/viewequip.rs`, `web/hud/viewequip.rs`).

### Shops, storage, cart, vending
- The panel over the basket of what is bought (`web/hud/shops.rs shop_compare_window`): beside an NPC's shop
  the client's code never tells it of an item to compare, so it shows what is worn twice; whether the
  Windows client of this version really does only that was not looked at. Its place is not kept between
  visits (the client keeps it in the registry). The marks are drawn 8 by 8 where GDI's region leaves the last
  row and column out.
- Of an NPC's shop and the storage (seen otherwise): "Toggle Item Amount", the storage's order box, its
  eight small buttons that show a tab apart, a guild's storage. The cart (`cart.rs`, `web/hud/cart.rs`,
  `web/world/cart.rs`), vending both sides (`vending.rs`, `web/hud/vending.rs`, `vending/buying.rs`).

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
- Of pets, homunculus and mercenary (seen otherwise, `docs/DONE.md`): a skill of a homunculus or a
  mercenary used from its list or the hotkey bar, a homunculus's own level and evolution, a pet's accessory
  and talk, renaming any of them, a user's own AI script (`ai.rs`, `ai/**`, `web/app/ai.rs`,
  `web/world/ai.rs`).
- In the three windows, and in Monster Info, words of the English table run into what stands after them
  ("NameMina", "Expiration11/21", "IntimacyShy", "Element" into its value): compared 2026-11 and left, for it
  is the client's own. The places are fixed in its code, none follows the word (`UIMerInfoWnd::Paint`: words
  at 100, the name at 135, the date at 155; `UIHomunInfoWnd::Paint`: "Intimacy" at 100, ":" at 140, the value
  at 150; `UIPetInfoWnd::Paint`: values at 145; `UIMonsterInfoWnd::Paint`: words at 260, values at 300), and
  the words are as wide here as there: the client writes them in Arial of 13 pixels (`TextContext::SetFont`,
  language 1: family 1, height 12 + 3; `FontEntry::CreateSystemFont`), whose widths by the font's own table
  for that size (`hdmx`) are Name 34, Expiration 58, Intimacy 49, Element 48, against 34.7, 57.8, 48.4 and
  47.7 measured on the page. The places were made for the Korean words, which are two or three letters.
- A pet's emotion stands 20 over its feet as anyone's (`web/world/frame/sprites.rs emoting`), which for a
  Poring is well over it; not compared with the Windows client.

### NPC scripts, books, quests, guide
- Books (`books.rs`, `web/hud/books.rs`); scripted pictures, marks and cut-ins (`net/scripted.rs`,
  `web/hud/scripted.rs`); the Adventure Guide (`guide.rs`, `web/hud/guide.rs`, `guide/pages.rs`); the Tipbox
  beyond its first opening (`web/hud/tips.rs`).

### Maps, world
- A copy of a map made for one party: `1@orcs` was entered with a party of two (2026-11-27, see `DONE.md`
  26) and loads; `1@tower` only by `@warp`. The client's files have no minimap picture for either
  (`유저인터페이스\map\1@orcs.bmp`, `1@tower.bmp`: none in the archives), so none is shown. Not seen: what is
  in such a copy (its NPCs and monsters were not played), the queue for one, one ending by its time.
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
- How `UITabControl` draws the mark `SetFlag` puts on the skill window's tab "Etc." while it has a skill
  not looked at yet (`UINewSkillListWnd::OnCreate`, `UM_REFRESH_EXTRA_SKILL_TAB`): the tab is drawn plain
  (`web/hud/skills.rs skill_window`).
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

### Chat
- What the client's `/pk` (`TT_REQ_KILLER_RANK`) sends: here it asks CZ_REQ_RANKING for the fourth kind, as
  the server takes it and as the client's table of headings has a row for (`web/app/commands/beside.rs`);
  the client's table of packets also has the older CZ_KILLER_RANK (0x0237).

### Party, guild
- What the chat says when a party gets another leader (nothing is said): `web/app/incoming/party.rs`.
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
- After the link dropped (the server restarted, or a hidden tab timed out before `web/app/beats.rs`), the
  login screen came up three times without some of its pictures (2026-11-27): the box "Disconnected from
  server." without its ground or its OK, once the login window without its ground too; the texts stood
  where they belong and a press where OK is closed the box, and the next screens were whole. The elements
  had the right addresses and the pictures loaded when asked for by hand; the browser's list showed one
  request for `btn_ok.bmp` cut off (`ERR_ABORTED`) as the screen was written. Not explained: writing the
  music's hint into the corner without writing the page anew (tried, taken out again) changed nothing.
- The window that leaves a party (`ReceivePartyMemberLeft`) closes the instance's window too since
  2026-11-27 (`incoming/instances.rs instance_forgotten`): by reading only.
- The message box of a question (`hud/boxes.rs confirm_box`) is the client's `UIMessageBox` since 2026-11
  (title bar "Message", three lines of 18 cut at 37 letters, its bar and the wheel for more: `message_box.rs`;
  seen in the game with the homunculus's question of six lines). Left: the lines are cut by letters where the
  client cuts by bytes of its code page (the same for Latin and Cyrillic, not for Korean); the boxes of
  `message_box` (questions others ask, `win_msgbox.bmp`) still let the page break their words.
- A guild's castles in the game: only the empty list was seen ("Castles Owned :" with nothing under it, on a
  new guild); a castle cannot be given to a guild without the server's owner.
- The readied arrow's picture and name lie over the figure's head in the Equipment window. Read once more in
  `UIEquipWnd::Paint`: the client draws them after the figure, so over it, the picture at (115, 64) of the
  window and the name from 115, the figure's feet at (141, 149); the name was 5 too low here and is put
  right, which brings it nearer the face, not off it. No picture of the original window is in `../rodecomp`
  or `docs/`: to compare with a video (`client/src/web/hud/equipment.rs equipment_ammunition`).
- A trap is pointed at only by a skill for traps (Remove Trap, Spring Trap). The client's pointer also becomes
  the sword over a Blast Mine, a Claymore Trap and an Ice Wall, which it lets be struck, and over every trap
  in a siege (`CGameMode::HandleActorPick`, the pick of the kind 2): not done. A hidden trap shows to its
  setter only, not to the party (`client/src/web/world/skill_units/traps.rs`).
- The traps' models: `CSkill::ShowModel` sets no scale and none is set here, so the size is the file's; on the
  screen a Skid Trap, a Sandman and a Claymore Trap are about 2 cells wide. Not compared by eye with the
  Windows client, and the models' own sizes were not read out of the files as numbers.
- A Talkie Box was not seen sprung: no monster was brought onto its one cell.
- Storm Gust and Magnus Exorcismus on a light floor: a single `StormGust.str` still whitens light sand at its
  height (seen 2026-11-26 in the Sograt Desert), as light added to a light floor must; what made it brighter
  than the client's is taken away (the file played again for every hit of every unit; the pictures kept in
  8 bits a colour). Not compared with the Windows client side by side. Still different from the client: (Storm
  Gust's spikes of ice, `CRagEffect::StormGust`, are drawn since 2026-11-27: eight, two at each of four ticks;
  seen with `@effect 89` only;) an effect's
  file draws its layer 0, which the client skips (empty in both files); the last key of a layer is applied a
  frame late (`effect/transform.rs`). Magnus Exorcismus was not looked at in the game after the change.
- A character's own level-up shows whenever ZC_PAR_CHANGE tells its level, also when a GM lowers it
  (`@blvl -1`): the client does not ask whether it went up either. `EF_ANGEL2` (338) and `EF_ANGEL3` (582),
  the Super Novice's and the Taekwon's, are not shown here (`effect_tables/effects_missing.txt`).
- A cart that stands behind its owner (as after every change of map, the owner looking at the camera) is all
  but covered by the body, as the client makes it a rope's length behind (`AM_SET_MASTER_POINTER`); its shadow
  shows. Whether the Windows client's cart shows more of itself there was not compared on a screen.
- What monsters shoot (`blow/shots.rs`): the monsters of later content in `CGameActor::LaunchAttackProjectile`
  and `CArrowEffect::SendMsg` are left out (Phylla, Petal, Corruption Root, Nyailo, Limacina and the rest), as
  are the later jobs of `SpawnMandraAttack` and all of `LaunchBulletSpriteEffect` (the Disguiser and others,
  none of the classic game: `BACKLOG.md`). A shot is one picture turned along its way; the client picks
  the sprite's action by the direction faced (`GetDir`) and steps its pictures after 8 frames. When the shot
  leaves was read as the frame handed to `SpawnArrowEffect` (`speed × attackMotion`); the decompiled call
  hands that number in a place `CArrowEffect::SendMsg` does not read it from, so this is the likely reading,
  not a certain one.
- Skills cast by monsters, looked at in the game only in part: seen in the packets and on the screen from a
  Marduk, a Marc, an Elder Willow — Fire Bolt (cast and bolts), NPC_FIREATTACK (its hit and number),
  NPC_WATERATTACK, NPC_MAGICALATTACK, NPC_STUNATTACK, NPC_BLINDATTACK (numbers; their hits' effects not told
  apart in the crowd), NPC_EMOTION (the server sends it as a cast of no time with an emotion); on 2026-11-26
  Teleport of Creamies (the effect left on the place and its sound), NPC_METAMORPHOSIS of Pupas (the cast is
  heard, no packet is left unread), an Osiris (its slaves called at its coming, Meteor Assault, Quagmire under
  the crowd), an Anubis (a cast on a place with its circle, Sonic Blow with its sound and numbers), the
  Mandragora's NPC_GROUNDATTACK (heard as a skill's hit). Not looked at: Fire Wall and Heaven's Drive by a
  monster (the Anubis did not cast its Fire Wall while it was watched), the poses of `monster_skills.rs`
  (of the classic monsters the table has almost only skills whose packets the client does not ask it for),
  the smoke at tick 35 of `EF_SUMMONSLAVE` (made, not caught on a picture), Baphomet. `world::cast` and the
  handlers of a skill's packets do not ask whether the source is a character, but for the casting pose, which
  only characters have.
- One struck while it walks may stand a cell away from where the server has it (seen after the fix of the
  moment a blow lands, 2026-11-26: none in 26 stops among a few Orc Warriors, 4 of 8 in a crowd of eight of
  them on a Peco Peco, each by one cell): the client shows every walk 72 milliseconds behind the server's
  clock and more (`server_time.rs`, its `GetServerTime`), while the blow lands in real time, and the stop is
  seen a frame late. This is the client's own; rAthena sends the walk anew for a while after a blow, and the
  next walk sets out from the server's cell.
- The attack is asked for again when nothing came of it for 1.5 seconds (`web/world/pursuit.rs chase`): the
  client asks once for each press (`isAttackRequest`, `CPlayer::UpdatePendingAction`) and again only when the
  server says the blow was asked from too far (`AM_RESUMEPROCEEDATTACK`). So after a skill used on the
  character itself in a fight (Endure, a potion is no skill) the fight goes on here about a second later,
  where the client stands idle until the monster is pressed again (the server ends the fight with any skill).
- On a Mac the key to sit is Alt+X, not Insert (`hotkeys.rs mac_instead`, this project's own).
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
- Merging the other launchers in `web/world/prims/*.rs`: no test; small numeric differences change effects.
  (`blows.rs cloud_streaks` / `cloud_spread` are one function now, `streaks_to_aim`, by the owner's decision
  of 2026-11-21: the same lots in the same order and the same figures, held against the two as they were.)
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
  `tables/items.rs`, `quests.rs`, `guide.rs` and `skill_needs.rs`, each in its own way (the last has the
  fullest reader of a line, `field`: its depth, its dash, its key and its value).
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
- Tidy of 2026-11-27: the three listeners of a gesture are wired once (`web/app/wiring.rs wire_gesture`: the
  windows dragged, the chat's tabs, the scroll bar's handle), and the wheel and the arrows move what an NPC
  said by one rule (`step_list`). Left: the small repeats of `web/world/pursuit.rs` (`chase` and `fetch` end
  alike: the order turned into a walk or a wait) and of `web/world/combat.rs` (the three lines every function
  of the world begins with); where the pointer was last is still kept by the tooltip (`web/app/tooltip.rs`)
  and by the world (`web/world/input.rs`), each for itself, since the world does not reach into the app.
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

### Written twice and left (`python3 tools/duplicates.py`: 6 groups, 55 lines, of 21 groups and 218)
- `scenery/makers.rs heard_as_it_was` (and its last lines against `earshot.rs`), `prim/tei/forest.rs light`:
  a test's own copy of the rule, kept whole to hold the rule to it. (The parts of the aura of the highest
  level are made once now, `Part::level99_glow` and `level99_swirl` of `prim/tei/max_level.rs`, by the
  effect and by its tests alike.)
- `net/notices/tests.rs a_line_takes_what_it_asks_for` is the test of `numbers.rs` once more: no test's name
  was to be lost.
- `web/world/prims/*.rs`: `known` / `thrown` / `run` begin alike in every file of effects (the table and the
  rule differ).
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
- A page that is not shown has its timers held back by the browser (one beat a second, and after some
  minutes one a minute), and the server ends a connection it has heard nothing from for a minute (rAthena's
  `stall_time`, `conf/packet_athena.conf`). So the two beats of the connection (the keep-alive and the watch
  on it) come from a worker, whose timers are not held back (`web/app/beats.rs`); the page's own timers beat
  only where no worker can be made. This is the browser's need: the Windows client has nothing of the kind.
  Before it, a tab left in the background lost the game: seen 2026-11-21, and timed 2026-11-27 (hidden at
  19:08:03, "Character logged off" in the server's log at 19:12:15, with every other hidden tab of the same
  browser in the same second). The watch still asks first and closes only when what it sent stays unanswered
  (`link.rs watch`, `ANSWER_WAIT`: tests only).

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
- The sparks of `AlphaDownRipples` (`client/src/web/world/effects.rs`).
- `CSkillInfo::UsesLevel` tables are not read: every used skill has the level arrows —
  `client/src/web/hud/skill_list.rs`.
- Aiming a skill at someone (`skills/aim.rs`, `web/world/input/press.rs aimed`): the rule of Shift is the
  client's for a monster and for another player, but two of its conditions are left out: a player who ails
  (`pick->healthState != 0`) is not taken for one a skill of attack goes on without Shift, and a plain attack
  on another player with Shift held, off the maps where players are fought, is not asked of the server (the
  client asks, and the server refuses it). The player's own character is picked when nobody else is under the
  pointer, and before a monster while players come first, not ranked among the other players by how near it
  is. Seen with a monster only: a second player was not tried.
- A skill aimed at the player's own character with the pointer is asked at once, not held until the
  character's own blow is over as the client holds it (`web/app/clicks/skills.rs own_aimed`; the client's
  `AM_PROCEEDSKILL` with its own ID).
- A character's swing and the moment its blow lands (`blow.rs attack_motion`, `web/world/combat.rs swing`):
  one who holds a weapon in each hand is taken as the client takes one with any other weapon (6 pictures in),
  where the client's `AttackMotionLength` gives it 3 as with a katar: which of its two attacks such a
  character shows was not ported (`attack_action`).
- Found in the playtest of the merchant, acolyte and thief lines and left:
  - no sign stands over the vendor's own head (the server tells the others only; the client was not found to
    make one for itself). The window of one's own stall is titled "My Shop : <its name>" since 2026-11
    (`vending::own_title`), by a test and by reading only: not seen in the game;
  - an NPC's shop shows a Merchant the discounted price alone ("38 Z"): whether the client writes both was
    not compared — `web/hud/shops.rs`;
  - the picture of a state without a time (Maximize Power, told 9999) looked washed out, as if its shade
    lay whole over it — `web/hud/statuses.rs`, `cooldowns::turns`;
  - "Random" of Teleport: the client itself would not know rAthena's "Random.gat" and would write its line
    for no map; here it is read as Random (`web/hud/choosing.rs`).
- Monster Info (`web/hud/fight.rs sense_window`): in English "Element" runs under what it names, the client's
  own places (260 and 300) being 40 apart; the monster itself is not drawn in the window's left.

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
- The party window's Lock (`ID_TOGGLE_PARTY_MOUSE_LOCK`, `m_lockMouse`: it keeps the window's buttons, its
  menu and the members' small windows from the pointer) — `client/src/web/app/clicks/party.rs`.
- `UIGuildHelperWnd` (the guild key with no guild opens nothing) and `WID_CREATEGUILDWND_MODE_1` (the window
  that disbands) — `client/src/web/app/clicks/guild.rs`.

### Items, exchange, companions
- The second `PriceColor` (a ground behind the zeny) — `client/src/web/hud/exchange.rs` (the first is not
  drawn there either now: the owner's decision, section 1).
- An egg's description — `client/src/web/app/incoming/pets.rs`.
- The other settings of `ZC_CONFIG` — `client/src/web/app/incoming/viewequip.rs`.

### Quests, maps, tips
- The menu on a quest under the right button — `client/src/web/hud/quests.rs`.
- The monster's picture in Monster Info — `client/src/web/hud/fight.rs`.
- World Map search and the Kafra's arrows; the large map's boxes for the marks of quests, towns and the guild,
  and its notes — `client/src/web/hud/worldmap.rs`.
- The Tipbox: a tip's picture and the button that links a tip into the chat — `client/src/web/hud/tips.rs`.
  Its links to tips and to items are pressed (`hud/texts.rs linked_with`, seen in the game); `<NAVI>` starts a navigation (`clicks/navigation.rs`); `<URL>` is shown and not pressed (the client hands the
  address to Windows, `OpenTextButtonLink`; every address of this `tipbox.lub` is `http://127.0.0.1/`). The
  other marks (`^nItemID^`, `^e[..]`, `<FONT>`, `<B>`, `<I>`) no text of this client's `tipbox.lub` has, and
  they are not read. A line with a picture stands 25 high with its words on the line's foot by the browser's
  own rule, not placed to the pixel as `ArrangeLines` does.
- Fifteen tips of `SystemEN/tipbox.lub` have words left in Korean, which show as signs (the file is in the
  Western code page with these parts in the Korean one, and the client's English service writes all of it in
  the Western one, so they are unreadable there too): all of the text in 4479 and 8242; names of links in
  2003, 4169, 4170, 4171, 4173, 4335, 9015, 10058; search words after `#` in 6214, 6221, 6236, 9170 (not
  shown). Left as the file is.
- A talk with an NPC by the keyboard (`client/src/talk.rs`, `web/app/clicks/talk.rs`): a menu always has a
  line picked (the client's list has none after a press on its empty ground, and its OK then does nothing);
  the list follows the picked line by the line's own height, not by a line of 16; a press of the mouse on a
  line draws the menu anew from its top.
- The game's menu (`client/src/web/hud/esc.rs`): its shape follows the character while it is open (the
  client's is that of the moment it opened); the menu is put away while the save point is asked about (the
  client leaves it behind its message box); the dead rise by a thing they carry, the client's state 580 is
  not looked at, nor a map of `isEventPVPMode`, which this client does not keep; a menu dragged stays where
  it was dragged to, where the client opens a new one at its own place each time.
- Fireworks weather (`pokjuk` of `etcinfo.txt`) — `client/src/web/world/scenery.rs`.

### Two players, found in the playtest of 2026-11-21/26 and left
- Not looked at with two players, for want of time: a pet and a homunculus following as the other sees them
  walk, Devotion and Providence, Warp Portal entered by the other, Sanctuary, Ice Wall, Safety Wall and
  Pneuma on the other, Strip and Steal Coin, a guild between the two accounts (inviting, positions, the
  notice, expelling: seen earlier with other accounts, `DONE.md` 11), a stall sold out, a deal refused for
  weight or a full inventory, marriage, adoption, mail.
- A chat room's wrong password and throwing a member out could not be seen: both test accounts are game
  masters, whom rAthena lets into any room (`PC_PERM_JOIN_ALL_CHAT`) and does not let be thrown out
  (`PC_PERM_NO_CHAT_KICK`). It needs an account of no group.
- A game master sees the HP bar of every player (rAthena's `disp_hpmeter`), so the bar under a party member
  could not be told from that.
- Packets no reader takes, each time a map is entered: ZC_CONFIG 0x02d9 (10 bytes) of the kinds the readers
  of `incoming/homunculus.rs` and `incoming/viewequip.rs` do not take. (ZC_ACH_UPDATE 0x0a24 and
  ZC_REPUTE_INFO 0x0b8d are read since 2026-11-26.)
- A press, a double press and a drag made by a script of the page (not by the mouse) are not always taken
  since the build of 2026-11-26 (a skill of the tree, a stall's sign, a thing dragged into a basket): only
  whoever drives the page by script meets it; the mouse works.

### Windows, found in the playtest of 2026-11 and left
- The lists of rows move by whole rows under the wheel as the client's (`scrolling.rs`, `hud/lists.rs
  rows_stepped`): the shop, storage, cart, inventory, vending, the lists to pick from, friends, party, guild
  members, the guild's table and skills, an NPC's menu, the Tipbox's list. Not so yet: the emotions, the
  options' rows, the chat tab's list of kinds, the World Map's list (how the client steps them was not read).
- A thing's tooltip is the client's since 2026-11-27 (`tooltip.rs`, `web/app/tooltip.rs`). Left with the
  browser's (`title`): the rows of the shops', the storage's and a stall's lists (the client says the name,
  the amount and the price there: `UIMerchantItemShopWnd::OnMouseHover` and its like), the quest marks, the
  party's and the mail's pictures, the chat's buttons. The colour of a thing with cards past its slots
  (`ItemCardColor`) and the yellow of one bound when worn are not given (no item of the classic game has
  either). A tooltip is put by its thing's corner in the page's measure, the client's by the cell the
  pointer is in: the same place for a cell.
- The question of an NPC's shop (buy, sell, cancel) has no words in it; the shop's list and the guild window
  open over Basic Information; the pet, homunculus and mercenary stand on the character after a warp.
- Enter in an NPC's menu was not seen to fail in any case tried (Kafra, smith, metal seller; the chat's line
  with and without the keyboard; the pointer on the window and off it); what the owner saw fail was not found
  beyond the double click and the boxes that ask, which are mended.

### Achievements and titles, reputation, probabilities (2026-11-26)
- **The server answers a reward the other way round.** rAthena sends ZC_REQ_ACH_REWARD_ACK with 1 for a reward
  given and 0 for one refused (`achievement_get_reward`); the client takes 0 for given
  (`CGameMode::ReceiveAchievementResult`) and is read so here. Nothing is lost by it: the server sends the
  achievement, or the whole list, again with the reward marked. But a title just gained is never coloured in
  the list of titles (`UIListBoxTitle`, 0x92C0FA), which the client does from that answer alone: by reading
  only.
- **Titles the server allows and the client does not list.** rAthena lets a title be worn once its
  achievement is completed (`achievement_get_titles`); the client lists it once the reward was taken
  (`ReceiveAchievementList`), and so does this one.
- This client's own: the names of the character itself and of players of no guild are asked of the server
  as the game is entered and as they come into sight (CZ_REQNAME), for the title told with them; the client
  asks when the pointer first comes over someone, so its page of titles shows none worn until then. The
  line of an achievement completed goes up to a second late (it is looked at with the watch on the
  connection). The tabs under an open tab are moved by the bar's two arrows only, not by its handle or the
  wheel. The list is scrolled by the browser, its bar in the client's pictures (WebKit only).
- Not found in the client: how `UITextButton` colours the names of the achievements kept in sight (black over
  white here, as the count beside them); `/achievement` (the client's line 3089 names it, its table of
  commands does not have it: not made); where the window `UIAchTracingWnd` is kept between visits (its
  `AchTracingWndInfo`): here it is not moved and not kept, and those kept in sight are forgotten at a reload.
- The date of an achievement completed lies over the box of its reward not taken yet, in the list and in the
  summary: so `UIAchListWnd::Paint` places them (263, 49 and about 276, 55). Not compared with a picture of
  the Windows client.
- Reputation: the classic server has no reputations (`db/re/reputation.yml` is Renewal's), so the window
  lists the two of the client's table that are always shown, with no points. For the rest the owner would
  add reputations to `server`'s `db/import/reputation.yml` (and scripts that give points). How the client
  draws the drop-down (`UIComboBoxDefWnd`: its arrow, its list's rim) and whether a picture button writes its
  label on itself (`btn_search` with line 3777: told when pointed at here) were not read.
- Probabilities: the page is the publisher's own web page (`https://probability.gnjoy.com/`, Korea's); there
  is no service of this project behind it. The icon does what the client's does; to lead it elsewhere the
  line `MSI_ETC_PROBABILITY_URL` of the client's `data\msgstringtable.csv` would have to say another address.
- The message table of this version has 4,405 lines in `msgstringtable.csv`, of which the gateway gives the
  4,023 of `msgstringtable.txt`: the lines past it are read one by one where a system needs them
  (`tables/probability.rs`), by names copied out of the client's compiled list.

### Bank, attendance check, cash shop (2026-11-26)
- **The attendance check's table is the server's, not the client's (this project's own, 2026-11-27)**: the
  client's `System\CheckAttendance.lub` has an event that is over (20260304 to 20260331) and twenty rewards of
  later content the classic item database does not have, and a server gives its own rewards whatever the
  client draws; the two agree on a Windows client only by writing that file anew, and the client's files are
  never changed here. So the gateway gives the browser the period of the server's own database that runs today
  (`server/db/attendance.yml`, the file Docker mounts as rAthena's `db/import/attendance.yml`;
  `--attendance <file>`), and the client's file only when there is none. The period is picked as the gateway
  starts: a gateway left running past the period's end, or started before a change of that file, shows the old
  one until it is started again.
- Cash shop, not made: dragging a thing into the list of what is bought and out of it, with the box that asks
  how many (`UICashShopWnd::DragAndDrop`, `UICashShopWnd_Cart::DragAndDrop`: a thing goes in one at a time
  by "Buy" and the plus); the double click on a kind that takes it out; the wheel and the thumb of the list's
  bar (its two arrows move it); the larger layout of the sale for one account (tab 9: rAthena ignores the
  request, the tab shows nothing); ZC_SE_CASHSHOP_UPDATE (0x08d3), which rAthena never sends; the second
  banner and the lists of the Stove launcher. The box before a purchase is the plain message box, not
  `UIPayConfirmWnd` with the item's picture, which was not looked at.
- Cash shop, the client's own: a hundred of a kind may be put in the list, and rAthena refuses more than 99
  (`cashshop_buylist`); a search keeps the page's size of the tab it was begun on.
- Attendance: the banners of `tb_checkattendance_banner.lub` are not drawn (the table is empty); the files
  `CheckAttendance_<server>.lub` for the Korean servers are not read; the days left are counted as the client
  counts them, by the player's own clock. In a letter (the reward's), a line of the message table named by
  its name in the client's code (`<MSG>MSI_…</MSG>`) is not written out, only one named by its number; the
  links to places and items in a letter are still shown as written.
- Bank: the client refuses a deposit and a withdrawal while a window of Lapine, reforming, enchanting or
  runes is open, and the window while one of refining or of the barter market is: none of those windows
  exists here. The window's field takes any letters, as the client's does.
- ZC_UI_OPEN's other windows are not read: a tip of the Tipbox (5), a quest's information (6), the stylist,
  the windows against macros, the grade enchanting.
- Escape closing these three windows was not found in the client's code and is taken from its other windows.
- The colours of the texts of the attendance check and the cash shop are read as `0xBBGGRR`
  (`TextContext::SetColor`): not compared with a picture of the Windows client.

### Battleground, Adventurer's Agency, Replay (2026-11-26)
- **For the owner**: the Adventurer's Agency talks to the game's web service, rAthena's `web-server`, which
  the Docker setup builds and runs since 2026-11-27 (the service `web`, port 8888 on 127.0.0.1). The gateway
  reaches it at `--web-server` (`127.0.0.1:8888` when not given): a gateway built before the agency was made
  has to be built and started anew.
- **A search by the note brings rAthena's web service down** (seen 2026-11-27; the Windows client would do the
  same to it): `partybooking_search` of `../rathena/src/web/partybooking_controller.cpp` puts the note into
  the query as `like '%note%'` and hands the query to `SqlStmt::Prepare`, which reads it as a `printf` format.
  The service ends, Docker starts it again within seconds, the board keeps what it showed (the gateway answers
  502). To put right in rAthena, by the owner: the two `%` of that line written `%%`. Its search also takes
  the two levels as they are (`minimum_level` = and `maximum_level` =), not as a range.
- `/organize name` typed while the party window is closed shows nothing: the setup window is drawn beside the
  party window only (seen 2026-11-27; the party's, left).
- rAthena's web service answers `/party/add`, `del`, `get`, `info`, `list`, `search`; not `/party/change`
  (`CAdventurerAgencyMgr::RequestChangeMaster`, sent when the party's leader changes): this client does not
  send it. Its `list` gives every notice at once and calls their number `totalPage`: the board shows the first
  ten and that number as its pages, as the Windows client would against the same service.
- The agency, against the client: the browser posts to the gateway, which hands the form on (a page cannot post
  to another origin); the name of the world is the name of the char server picked; a row's right button (a
  talk with the leader, `UIPartyInfoWnd::OnRBtnUp`) is not made (the window of a notice has the button "PM");
  the fields of the search and of a notice are named by hand on both sides (`agency::SEARCH_FIELDS`,
  `NOTICE_FIELDS`); the windows open where the client first puts them and are not kept from one session to the
  next.
- **The old party booking, against the client** (made: `docs/DONE.md`, section 26).
  - *Decided here against the client of 2026*: `/recruit` and `/booking` open its two windows. That client
    still has the windows, their packets and what the commands do (`TT_SEEK_PARTY`, `TT_REQ_RECRUIT`), but no
    word for either in its table of commands (`CSession`'s `m_commandIds`), its two hotkeys are hidden
    (`HK_PARTY_RECRUIT`, `HK_PARTY_BOOKING_LIST`: `HiddenInterfaceIndex`) and the menu's old icon is skipped:
    nothing a player does opens the booking there. The two words are those the client's own help of the
    booking still gives (`PartyBookingHelp` of `party_booking_config.lub`); the owner asked for them.
  - The windows first stand 510 across and 360 down as the client's, but are kept on the screen, and their
    places are not kept from one session to the next (the client keeps them in the registry). The window of
    one's own ad has no part in the order of windows (the client's `IsOrderFixed`) and so lies under the
    others here: opened in the same place as the list, it is under it until one of them is moved.
  - Not found in the client: what fills `PartyBookingMgr`'s names of regions and maps (the code that reads
    them is there, `MakeEntry`; here they are the lists the windows themselves read), so an ad of region 0
    is not shown (none is registered so); the height of a line of `UIComboBoxEx` (16 here) and how the
    words "LEVEL", "MAP", "JOB" are drawn (`SetText(.., 1)`: plain here). The box that drops a list
    (`web/hud/combos.rs`) is written once for the booking; the reputation's own drawing of the same box
    (`hud/reputation.rs`) is not moved onto it.
  - The time of an ad is what rAthena sends, the moment it was registered, shown in the browser's own time
    (the client names the field the time it expires).
  - The windows stay open across a change of map (the client makes its windows anew for each map and opens
    only that of one's own ad again); they close when the game is entered anew.
  - rAthena's own: the search goes by the ad's level being within fifteen UNDER the level sought
    (`party_booking_search`), the client's own filter of new ads by it being within fifteen OVER; the last ad
    of a page is given again at the head of the next (the list keeps it once). Neither is mended here.
  - The fields of the level are named by hand on both sides (`booking::LEVEL`, `OWN_LEVEL`).
  - Not seen in the game: see `docs/DONE.md` (an ad of another player, the next page, the server's refusals).
- Battleground, against the client: the window that asks whether to enter has no cross (the client's does
  nothing); the help opens beside the list (the client gives it no place); the list's windows are not kept
  from one session to the next. Not made: ZC_BATTLEFIELD_NOTIFY_HPSP (0x0baa, not sent by rAthena). Of
  the battleground's own chat (`/battlechat`, made: `docs/DONE.md`, section 26): the line of a speaker that is
  a monster by its number is not given the client's colour for it (`MonsterJobColor`); the client's count of
  the same line said again and again (`ShowRestrictedNotice`) and its silence under Frenzy
  (`IsBerserkActive`) are not made for it; a line heard was never seen in the game. Not seen in the game: everything from a battle that is ready onward (see
  `docs/DONE.md`).
- **Replay: where the browser decides.** There is no `Replay` folder: recordings are kept in the browser's
  IndexedDB (`roweb-replays`), and the list has three buttons of this project's own, "Save to a file", "Load
  a file" and "Delete" (their words are not the client's). The Windows client is started apart for replays
  (its replay mode, the list reached from its window of services, `UISelectServerWnd` with `ID_SELECT_REPLAY`,
  which this client does not have); here the login screen has the client's `btn_replay.bmp` under the login
  window, which opens the list in the login's place. The list has no Exit button (a page is not quit).
- **Replay: the file, section by section** (all of it by reading the decompiled client: there is no Windows
  client here to try a file in, and **nothing was tried against a real Windows client**, in either
  direction). The same in both: the text of 100 bytes, the signature and the keys, the table of 26 sections
  (each entry in the place of its number, the 25th place marked 111, the first section at 0x174), the
  cipher, section 2 (the header's tagged records), section 1 (the packets, each after frame, tick and size;
  only the packets the client's table marks for recording, `replay/recorded.txt`, and no chat without its
  option). A recording begins from a snapshot as the client's does, and so does a replay; per section:
  - **3 the character**: every member from 1010 to 1101 at its size. Filled: the account, the character, the
    job, the levels, the experience, the points, the stats and their costs, the fighting values, zeny, speed,
    the head and its colour, the clothes' colour (1063, which the decompiled client calls `m_bodyStyle`), the
    three headgears, the weapon and the shield (as item numbers, the left hand's half of 1061 nought), the
    body's state and the effect state (1068, 1069). Zeros: the cart's counts, the party's sharing, the
    ammunition's place (1050), the robe, the skill points of guild, homunculus and mercenary, the rest.
  - **4 the stats**: the bonuses, weight, HP, SP; zeros for the five rows of twelve numbers the decompiled
    client does not name; the account's name empty.
  - **6 quests**: fully, but a quest's time is written as a date in UTC (the client writes its own clock's).
  - **7 party and friends**: fully as the client keeps them (it keeps neither job nor level of a member: the
    character's own are filled in again at playback); the flag of the session and who asked into a party not.
  - **8 items**: the inventory, the cart, an open storage and the ten worn. Not written: what an item looks
    like worn (`sprite`), prices, a rented one's end, grade and random options (this client keeps none of
    them with an item); the other ten lists and five rows (shops, a deal, costumes: a costume worn stays in
    the inventory's list with where it is worn). Which accessory stands at 3 and which at 7 of the ten was
    not found in the client: the right one is put at 3. At playback the storage's list is not shown.
  - **9 companions**: the homunculus, the mercenary and the pet as far as this client keeps them (not what
    each fights and does, the pet's egg, the homunculus's accessory; its most hunger is written as 100); no
    elemental.
  - **10 memorial dungeon, 12 the battlegrounds' queue, 14 the shops**: only defaults (what a new session
    holds), though this client has the first two as windows: not kept past them.
  - **11**: the client leaves it empty and reads nothing of it. **This project's own**: one record in it,
    the character's sex (9014), which no section of the client's has (its session has it from the login). A
    file without it (the Windows client's) is played as a man.
  - **13 the hotkey bar**: the 36 places of the first bar; the two quick places empty.
  - **15 everyone in sight**: each from what the world holds of it now, as the packet 0x09FF where it stands
    or 0x09FD (setting out now from the cell it has come to) when it walks, a chat room's and a stall's sign
    after its owner, the portals as NPCs of the job 45. Not kept by the world and so nought: the third
    option (Fury and the like show only when told anew), a robe, a player's character number, the honour;
    another player's homunculus or mercenary is written as a pet (7); the places a script's effect plays at
    (the unseen NPC 139) and a buying stall's sign are left out; nothing for a bar that fills or a Madogear.
    At playback the chunks of older shapes (0xD0 to 0xD8 but the two) are passed by.
  - **16 things on the ground**: fully. **17 skills' units**: the unit, its cell, its kind in a byte and
    whether it is seen; who made it is nought (the world does not keep it); none with words. Read by a new
    reader of ZC_SKILL_ENTRY (0x011F). Not seen in the game: no unit stood there.
  - **18 the states' icons**: the state, the time left and the whole time; the three values nought.
  - **19 the windows, 20 the setting, 21, 23 the flag `isOnQuest`, 24 the navigation**: only defaults (no
    window is written or opened again; what the flag means was not found).
  - **22 names and guild**: the names of the players in sight with their party's and guild's, the
    character's own, its guild and the emblem's version; no emblem's file. At playback the names of party
    and guild are put under the players; the emblems are not.
  - **The head of the packets**: the two packets of the map's kind as they last came, the three lists of
    skills in the layout of 37 bytes (a skill's name in the client's files is left empty: this client has
    the names only as shown), and, this project's own, a ZC_NOTIFY_TIME with the server's clock, by which
    the replay's clock is set (the client's files have none there: such a file's clock is set by the first
    walk or time it tells). The fourth packet of the client (`BuildStatusReplayPacket`, in a mode of the
    session the decompiled client does not name) is not made.
  At playback the game is entered by a ZC_ACCEPT_ENTER made of the header (the client sets its session's
  place from the header without one); the way the character faced is written into the header and not shown
  (it begins facing south). One that walked as the recording began walks on from its cell; while a replay
  is held, such walks go on (see "while one plays"). The sounds of the states given back are heard at once.
  **A file of the Windows client here**: read as far as the above; never tried. **A file of this client in
  the Windows client**: laid out as its own by every size this page could find; never tried. Files this
  client wrote before the evening of 2026-11-27 began from a section of its own (the fifth place of the
  table, or the 24th under the number 124: who the character is and every packet of the map server since
  the game was entered) and are still played from it; new files have none.
- **Replay: the records of the windows** (CC_REPLAYPACKET 0x08ae; `replay/told.rs`, read back in
  `replay/told/read.rs`). The client writes what its dialogs, shops, lists and accept boxes were answered
  (`UIWindow::RecordResponse`, kind 900: its number of the window, its number of the command, three values),
  drops (901), the texts of a window's fields as its OK is pressed (902), tabs and boxes of the quest window
  (913), and plays them by sending the same to the same window. This client now writes and reads the same
  kinds for the windows it has, window and command as the client's:
  - the talk: "next" and "close" (16, 218, the third value 22 or 89), the menu's OK and Cancel (17, 184/185),
    the number and the words typed (902 of 56 or 100, then 184);
  - a shop: the question (25: 224, 223, 185), the baskets' OK and Cancel (23 and 24), the box "Toggle Item
    Amount" (22, 213, what it becomes), a row let go over a basket or back (901 onto 23, 24 or 22, from 3, 4 or
    5), the count box (902 of 15, then 15, 184, 100, the number, what was asked: 1 to 4);
  - the lists of a skill or an item: OK and Cancel of 39, 94, 99, 74, 79, 80, 107, 111, of the magnifier's (73)
    and the eggs' (90), the row where each window records it; a thing let go over a forging (901 onto 80);
  - the boxes that ask: a deal (32), an invitation to a party (35), to a guild (70), to be a friend (109);
  - a deal's window: a thing let go over it (901 onto 10011), its count (15 with 5 asked), its zeny (902);
  - the quest window: its opening and closing (909, 914, as before; the client itself writes no closing for
    that number), its four tabs and its box (913 of 10008).
  **Against the client, this project's own:** a menu's and a warp list's OK carry the row in the first value
  always (the client's does only when the row was double-clicked; its button leaves 0, and the row it picked
  is then not in its recording at all), and the list of spells (99) likewise, where the client records no
  row; a drop is written without where on the window it fell, prices, slot, refining, cards and options
  (nought), and is found again by the item's number or its place where it is carried; a text typed is written
  as this client sends it (UTF-8), not in the client's code page. **Still this project's kind 990** (the
  press's name; the Windows client passes it by), because the client records nothing for them: the deal
  window's OK, Trade, Cancel and its box of a screenshot (`CUIExchangeUI` has no `RecordResponse`), a slot of
  a forging emptied (the client has it dragged out), the cross of a script's picture (`UIIllustWnd`).
  **Not recorded at all:** a row only picked and a link to an item in a talk (nor does the client), the boxes
  of the party, guild and friends that ask anything but an invitation, the client's plain message boxes
  (`UIMessageBox`, whose number is not one of this client's windows), a chat room's settings (904), the
  navigation (900 of the window 203, 908, 910 to 912): the client's navigation records lean on its whole
  state written out (911, `CNavigationManager::WriteStates`) and on rows of its lists by their numbers, which
  this client's navigation keeps in another shape; its kinds are passed by when read. Kinds 903, 905 to 907
  are passed by too. **Tried:** every rule by tests; in the game the reading only, from a recording made up
  of an older one of this client (its 990 turned into the client's kinds, a number field and a shop put
  after it): the talk's "next", the menu answered, "close", the number typed into the field and sent, the
  shop's question answered, things let go over the basket and back with the count box answered, the
  basket's OK, each window opening and closing with no hand. **Not tried in the game:** the writing (no
  session was played with this change: the worker could not log in), the lists, the deal, the boxes that
  ask, the quest window's tabs; a file of the Windows client.
- **Replay: while one plays.** Held back as the client holds it, by reading `UIWindowMgr::ProcessShortcutKey`,
  `Replay::AllowsEvent` and the checks of `g_replayAnimationScaling`: the keys (the player's three; minimap,
  World Map, quests, commands; the character's windows by the recording's options), the presses of the page
  (the player, the menu's fold, the windows of status, items, equipment, skills, party and friends by the
  options; nothing else), every form (nothing typed is taken), Escape (closes the window in front, never
  opens the options), Enter. Simpler than the client: a window allowed is allowed whole (the client dims
  single buttons inside them); a press on the world is not held back (it sends nothing, there being no
  server; the client's `CWorld` check was not followed to its end). Not made: everyone's motions drawn slower
  or faster with the speed (`Replay::AnimationScale`, `DelayScale`: the packets come faster, walking and
  striking keep their own time); the waits of skills counted by the player's clock (`CooldownState`).
  Speeds of 8 and 16 times have their lines in the table and no button, as in the client.
- **Replay: the recording.** As the client: it ends when the character is moved to another map or enters the
  game anew, the recorder put away (the client's state 3, in which nothing more is written and from which a
  new recording is begun by hand); this client then also keeps the file with its length, where the Windows
  client leaves it unclosed and reads the length from its last record. It also ends, kept, when the
  connection does. The chat's line at Stop names the folder `\Replay\` and the file, there being no path.
  The sounds of what is caught up with at the beginning of a replay of an older file are all heard at once.
- **Replay: ordered and not done** (the owner's list of 2026-11-27): inside a window the one watching may
  open, single buttons dimmed as the client dims them; presses on the world held back; everyone's motions and
  walks drawn by the player's speed (`Replay::AnimationScale`, `DelayScale`) and standing still while it is
  held; the waits of skills by the player's clock; the navigation's records (908, 910 to 912); a recording
  of five minutes and more, one with a deal and with storage, played and looked at (the longest looked at
  is 1:13, with a shop).

### Navigation (2026-11-26)
- The tables are the server's own where it has them (2026-11-27): `server/navigation.sh` runs rAthena's
  `map-server-generator --generate-navi` once, in a container of its own (the same sources, conf, scripts and
  map cache as the map server's; an empty database of its own), and writes `navi_*_krpri.lub` into
  `server/navigation/pre-re/` (not committed: made data, as the map cache is); the gateway reads that folder
  (`--navigation-dir`) and gives the client's own tables only where it holds none. What the generator does
  not know, and so the navigation on this server does not either:
  - Only portals (`warp`, kind 200) are ways out. A Kafra's teleport, a ship, the airship, and every NPC that
    takes one somewhere when talked to are not in the table (the client's own has them as kinds 201, 204 and
    205), so the box "service" changes nothing, and a place reached only through such an NPC has no route:
    Morroc from Prontera or Payon ("Failed to find a path": every field around it leads into `moc_fild20`,
    which is left only through its Continental Guards), the islands, the towns over the sea. rAthena's way to
    add them is a label `OnNaviGenerate:` with `naviregisterwarp "<name>","<map>",<x>,<y>;` in the NPC's own
    script (as `npc/custom/warper.txt` has it); none of the server's scripts has one. Left for the owner to
    decide: such labels in scripts of the server's own, or these kinds taken over from the client's table.
  - A monster is listed once for each line that spawns it (three "Orc Zombie" on `orcsdun01`); the client's
    table has one row a kind.
  - No scrolls (`Navi_Scroll`) and no NPCs dressed as players (`navi_picknpc`): both tables are empty.
  - A map is called by its file's name; the gateway puts the client's name in its place where the client's
    table knows the map (this project's own choice). 584 maps of the server (guild houses, copies of
    dungeons) are not in the client's table and keep the file's name.
  - An NPC that a script moves as the server starts (`movenpc`) is listed where its script first put it. For
    the 35 that `server/npc/roweb_classic.txt` moves, `navigation.sh` writes the new cell into the
    generator's own copy of the scripts first; any other script that moves someone is not looked for.
  The tables are made anew by hand (`server/navigation.sh`) when the server's scripts, `map_conf.txt` or map
  cache change; nothing does it by itself, and nothing says they are old. For Renewal (`rules.sh renewal`)
  they are read from `server/navigation/re/`, which was not made or tried.
- Not made: the button "Read external file path information" (`NAVI_BUTTON_DATA`: the list of the `.txt`
  files of the game's `NavigationData` folder, each a list of places) and the private airship's button
  (`NAVI_BUTTON_PRIVATE_AIRPLANE`, `UIPrivateAirplaneWnd`, `CZ_PRIVATE_AIRSHIP_REQUEST`; the server has
  `feature.privateairship` and Renewal's items for it): both are drawn and do nothing. The scrolls of the
  route search (`Navi_Scroll`, kind 400, `includeObjects`) are not read: the client's table holds none.
- The window's own differences: the two lines that follow the pointer over the map to press a place on are
  not drawn; the picture of an NPC or a monster looked at stands still (the client plays it, 1500 ms a
  round) and is not drawn out to its space; an NPC that `navi_picknpc` dresses as a player is shown by its
  job's picture; the lists have the browser's scroll bar, not `Navi_scroll_*.bmp`; the box of kinds has one
  picture for its button whatever its state; the small window does not grow more solid under the pointer;
  the windows of the arrows and of the route open by where the navigation's window stood when one of them was
  last opened; the window's place is not kept between visits (the client keeps it and the set of arrows in the
  registry). The message of line 4195 (a map the search does not list) and of 4692 (inside a copy of a map)
  are not in this client's table: the first is not said, the second in this client's words.
- On the World Map: a press on a dungeon's rectangle opens the client's list of its floors, which is not
  made (only maps with a rectangle of their own are pressed); the line of a route is drawn plain, not with
  `wv_Airf2.bmp`.
- The mark at a way's end is lifted 0.4 over the ground as the mark of the cell under the pointer is, not
  drawn nearer by a hair of the depth; an arrow is looked for anew when the character has strayed at each of
  its steps, not at each frame.
- `Line::Unsupported` (the client's box "That does not support the navigation area") is said in the chat.
- Not wired yet: the quest window's picture of the NPC to go to (`UINpcSprWnd`, which is not made), the
  marks of quests on the minimap, the Adventure Guide's buttons that start a navigation
  (`UIAdventureGuide.cpp:1593`), the shop's (`UIChooseSellBuyWnd.cpp:859`) and `UIShortCutWnd.cpp:720`.
- ZC_NAVIGATION_ACTIVE (0x08e2) is read and tested natively; no script of the classic server sends it
  (`navigateto` is only in `npc/re`), so it was not seen in the game.

