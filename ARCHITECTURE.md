# Architecture

A map of the project for someone opening it for the first time. Read this page, then the module comment
(`//!`) at the top of the file you are about to change: every file says what it is for.

What is implemented and how far it was checked is `docs/DONE.md`; the debts, the deviations from the Windows
client and the known bugs are `docs/DEBTS.md`. The project is "classic first": what only Renewal has waits in
`BACKLOG.md` until the classic game is finished.

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

The game's art and data are not in the repository: the gateway reads them from a kRO client folder. The one
exception is rule 7's: small tables the client has compiled into it, written out with a note of their origin
(`client/src/effect_tables/`, `statuses.txt`, `commands.txt`…).

## The client in four layers

Each layer knows only the ones under it.

| Layer | Where | What it is | Needs a browser |
|---|---|---|---|
| 4. App | `client/src/web/app*`, `hud*`, `view.rs` and `view/`, `parts.rs` | State of the session, what each packet / click / key does, the game's windows (`hud`), the screens before the game (`view`), what every window is made of (`parts.rs`) | yes |
| 3. World | `client/src/web/world*`, `figure*`, `sound.rs` | The map in 3D, everyone on it, effects, the mouse over it; sprites as pictures; music and sounds | yes |
| under both | `client/src/web/mod.rs`, `socket.rs`, `data.rs` and `data/` | The entry point and what all of `web/` shares (`document`, `by_id`, `now`, `korean`, fetching); the WebSocket; the client's tables fetched from the gateway, asked by the world, the windows and the app alike as `data::…` | yes |
| 2. Protocol | `client/src/net/` | Packets: reading the servers' and writing the client's | no |
| 2. Rules | the other `client/src/*.rs` and their folders | What the client decides with no page and no packet, each with its tests (the groups are listed below) | no |
| 1. Formats | `client/src/{map,model,sprite,effect,bmp}.rs` and their folders | The game's file formats, parsed into plain structures | no |

The rules, as `client/src/lib.rs` declares them:

- *a system's rules*: `account`, `shops`, `inventory`, `storage`, `vending`, `exchange`, `mail`, `choosing`,
  `skills`, `pets`, `homunculus`, `mercenary`, `party`, `friends`, `pvp`, `chat`, `chat_rooms`, `chat_tabs`,
  `whispers`, `family`, `guide`, `quests`, `books`, `links`, `commands`, `cooldowns`, `statuses`, `hotkeys`,
  `item_names`, `jobs`, `maps`, `said`, `song_talk`, `screenshot`, `ai`;
- *how the world is drawn*: `pose`, `blow`, `body`, `route`, `server_time`, `prim`, `scenery`,
  `effect_tables`, `state_effects`, `skill_units`, `ailments`, `aura`, `spirits`, `scope`, `quake`, `earshot`,
  `names`, `cart`, `falcon`, `mounts`, `cursor`, `font`;
- *shared by all*: `bytes`, `vector`, `tables`, `markup`, `kept`, `numbers`, `minimap`, `press`, `stack`,
  `snap`, `i18n`.

The protocol and the rules are one layer: some rules take a type of `net/` and some readers of `net/` use a
rule. Layers 1 and 2 are plain Rust with unit tests (`cargo test -p roweb-client`); everything that touches the
page or the GPU is under `web/` and only builds for `wasm32`.

## Directory map

Every source file of the two programs on a line, with the first sentence of its module comment; open the file
for the rest. The map is written from the tree (`python3 tools/directory_map.py --write`) and `check.sh`
refuses a tree it no longer shows: after adding, renaming or re-describing a file, run it once.

<!-- directory map: written by tools/directory_map.py --write; change a file's `//!`, not these lines -->
```
client/src/
  account.rs            The account as rules: what an ID, a password and an e-mail may be before a form is sent…
  ai.rs                 The scripts that move a homunculus and a mercenary
    host.rs               The functions the client provides to the script of a homunculus or a mercenary…
    prelude.lua           read by ai.rs
    sight.rs              What the functions the client provides to a script answer (`host.rs`), as rules over…
    tests.rs              tests
  ailments.rs           What someone's body state and health state do to how they are seen and heard
    marks.rs              The marks that stay by one who ails for as long as the state holds (`CMsgEffect`s the…
    tests.rs              tests
    veils.rs              The two veils over the screen of the player's own character
  aura.rs               The aura of the highest level: who shows which of the client's effects, as the client…
  blow.rs               When a blow lands: the moment into the striker's swing at which the one struck flinches,…
    flight.rs             How the numbers and words of a fight fly
    landing.rs            The hits a blow is made of: how many land, how long after the first each lands, and as…
    signs.rs              What a hit shows over the one struck as it lands, and what it sounds like there
  bmp.rs                Windows BMP decoder for the client's textures
  body.rs               What an effect makes of the body of the one it is on
    copies.rs             The copies of a character that an effect leaves standing
    drawn.rs              The drawings of a body's picture, as the flags and the counts that effects have set on…
    tests.rs              tests
  books.rs              The books that are read in a window of their own (the client's `UIBookWnd`)
  bytes.rs              Reading the game's files: one reader of little-endian numbers for every format (`.gat`,…
  cart.rs               The merchant's cart as it is drawn behind its owner
  chat.rs               The chat window as rules: the sizes it steps through
  chat_rooms.rs         Chat rooms as rules: what a room is and what its sign and its window say of it…
  chat_tabs.rs          The tabs of the chat window as the player has them
  choosing.rs           The lists a skill or a card opens, as rules
  commands.rs           The client's chat commands: the words typed after a slash, and what each asks for
  commands.txt          read by commands.rs
  cooldowns.rs          The waits before skills may be used again, as the client keeps them…
  cursor.rs             The game's mouse pointer: which of its pictures shows at a moment, as the client's…
  earshot.rs            How loud a sound of the world is by how far from the player's character it starts, and…
  effect.rs             The client's effect files (`.str`, "STRM")
    carried.rs            An effect file that a class of the client's own plays and carries
      tests.rs              tests
    flights.rs            The ways a carried effect file flies and is turned, each the rule of the class of the…
    played.rs             Which of the client's effects are played from files, and how
    strm.rs               The `.str` format ("STRM") as it lies in the file
    tests.rs              tests
    transform.rs          A layer of an effect file at a frame: its state, how the client brings it there from…
  effect_tables.rs      The tables of skills and effects the original client has compiled into it, written out of…
    effect_files.txt      read by effect_tables.rs
    effect_sounds.txt     read by effect_tables.rs
    effects_missing.txt   a table kept beside the code
    ground_scopes.txt     read by effect_tables.rs
    item_effects.txt      read by effect_tables.rs
    skill_effects.txt     read by effect_tables.rs
    skill_follow.txt      read by effect_tables.rs
    skill_hits.txt        read by effect_tables.rs
    skill_launch.txt      read by effect_tables.rs
    skill_motions.txt     read by effect_tables.rs
    skill_struck.txt      read by effect_tables.rs
  exchange.rs           A deal between two players as rules: the most zeny a side puts into it, the zeny typed…
  falcon.rs             The hunter's falcon as it is drawn by its owner
  family.rs             The family as rules: the lines the client says of a divorce and of an adoption that…
  font.rs               The size the client writes a text in, from what its windows ask for
  friends.rs            The friends list as rules: how many it holds, and the colour each friend's name is…
  guide.rs              The Adventure Guide as rules (the client's `CUIAdventureGuide`, `CAdventureGuideMgr`)
  homunculus.rs         The homunculus as rules: the word for its intimacy, when the chat says it is hungry, the…
  hotkeys.rs            The client's keys: which key does what, how a key is written, and what differs on a…
  i18n.rs               The client's own texts, in every language it speaks
    phrases.rs            The words and phrases of the game's windows in Russian, each by its English (`super::tr`)
    tests.rs              tests
  inventory.rs          The inventory as rules: its tabs and which a thing is under, what the window shows of…
    weight.rs             How heavy what is carried is, as the client tells of it
  item_names.rs         An item's name as the client composes it wherever it writes one…
    tests.rs              tests
  job_mounts.txt        read by mounts.rs
  jobs.rs               What the client knows of a job by its number alone
  kept.rs               What the game keeps in the browser between visits, as text
  lib.rs                roweb: a Ragnarok Online client for the browser
  links.rs              The links written into what an NPC says (the client's `UITagTextViewer`, whose lines…
    items.rs              The links to items in a line of chat (`<ITEML>…</ITEML>`)
  mail.rs               The mail (the client's RODEX) as rules: how the letters are kept and ordered as the…
  map.rs                A map's ground: GAT (where one can walk, and how high the ground is there) and GND (the…
    gat.rs                GAT: where one can walk on a map, and how high the ground is there
    gnd.rs                GND: the ground's shape, textures and baked light, and the mesh drawn from them
    paths.rs              The paths walked on a map's cells, and the direction of a step
    tests.rs              tests
  maps.rs               A map by its name, and the client's table of the music of each (`mp3nametable.txt`)
  markup.rs             The marks the client's texts carry, told apart from the text here once
  mercenary.rs          The mercenary as rules: the title of its window, which mercenary it is to its script, how…
  minimap.rs            The minimap as rules: where a cell of the map lies on it, which part of the map it shows…
  model.rs              What stands on a map's ground: RSW (the map's water, its sunlight, where each model…
    matrix.rs             The 4 by 4 matrices the models are put together and stood on the map with
    rsm.rs                RSM: a model of the classic maps (versions 1.1 to 1.5)
    rsw.rs                RSW: a map's water with its waves, its sunlight, where each model stands and how it…
    tests.rs              tests
  mounts.rs             What a job is drawn as in a state the server tells of
  names.rs              The name under someone the pointer is on
  numbers.rs            How the client writes a number in its windows (zeny, a price, a sum) and reads one typed…
  party.rs              The party as rules: what the client decides of its members that no packet says
  pets.rs               Pets as rules: the words of a pet's window for its hunger and its intimacy, what it feels…
  pose.rs               How a character is posed, as the client's actors decide it
  press.rs              What can be pressed on the page: every button, tab and row that does something, as a value
    account.rs            The presses before the game and on the way out of it
    books.rs              The presses of the books: the four buttons of the window a book is read in (the…
    cart.rs               The presses of the merchant's cart: its window and the looks it may be given
    chat.rs               The presses of the chat window (`UINewChatWnd`)
    chat_rooms.rs         The presses of chat rooms: the window that makes or changes a room (the client's…
    choosing.rs           The presses of the lists a skill or a card opens to pick one thing of (`net::choosing`)
    emotions.rs           The presses of the window of emotions (the client's `UICashEmotionListWnd`, Alt+L) and…
    exchange.rs           The presses of a deal between two players
    family.rs             The presses of the family: the answers to one who asks to adopt the character (the…
    fight.rs              The presses of what a fight opens: the OK of the window Sense shows a monster in (the…
    friends.rs            The presses of the friends list: the Friend side of the window it shares with the party…
    general.rs            The presses that belong to no one window
    guide.rs              The presses of the Adventure Guide (the client's `CUIAdventureGuide`)
    guild.rs              The presses of the guild: its window with its tabs, the list of members and the menu on…
    homunculus.rs         The presses of the homunculus: its window (the name, feeding, deleting, feeding by…
    hotkeys.rs            The presses of the Shortcut Settings window (the client's `UIHotKeyWnd`)
    inventory.rs          The presses of what is carried: the inventory and its tabs, the Equipment window, an…
    mail.rs               The presses of the mail (the client's RODEX)
    mercenary.rs          The presses of the mercenary: its window (dismissing it), what the window asks, the…
    party.rs              The presses of the party: its window with the list of members and the bar of buttons…
    pets.rs               The presses of the pet: the slot machine of taming, the list of eggs to hatch, the pet…
    player_menu.rs        The presses of the menu the right button opens on another player (the client's…
    quests.rs             The presses of the quests: the journal with its tabs, filters, search and rows, the…
    scripted.rs           The presses of what an NPC's script shows
    settings.rs           The presses of the Game Settings window (the client's `CUIGameSettingsUI`)
    shops.rs              The presses of an NPC's shop: the question whether to buy or to sell, and what is…
    skills.rs             The presses of the skills: the skill window with its tabs and the buttons by each…
    status.rs             The presses of Basic Information and the Status window
    storage.rs            The presses of the storage's window (the Kafra's, or a guild's)
    talk.rs               The presses of a talk with an NPC: "next" and "close" under what it says, a line of the…
    things.rs             What is done to the things in the windows other than pressing them
    tips.rs               The presses of the Tipbox (the client's `UITipboxWnd`, Alt+D, `/tip`)
    unread.rs             The presses of the list of packets this client does not read (a tool of this project's…
    vending.rs            The presses of a merchant's shop: the window a shop is made in, one's own shop while it…
    viewequip.rs          The presses of looking at another player's equipment
    whispers.rs           The presses of a talk with one player in a window of its own (the client's…
    worldmap.rs           The presses of the World Map (the client's `UIRoMapWnd`, Ctrl+~) and of the large…
  prim.rs               The client's effect primitives (`CEffectPrim`)
    casts.rs              More kinds made of parts. Two of walls of light round the place of the effect, each…
      rays.rs               `PP_CHEMICALPROTECTION` (54, `Cast::Rays`)
    change.rs             What a primitive may be told to become at a tick of its life (the client's…
    facing.rs             Ways on the ground and the camera's turn, as the client measures them, and how a sprite…
    flat.rs               The kinds that lie flat on the screen: how a picture and a kite move…
    fourfold.rs           The kind 167 of the client's primitives: a sprite drawn four times, once at each of the…
    glow.rs               The kind 153 of the client's primitives: a sprite standing on the ground that comes up…
    horn.rs               `PP_3DQUADHORN` (9): a spike of four sides, a picture on each, that is pushed along its…
    horn_tests.rs         tests
    life.rs               How a primitive lives: one tick of it, kind by kind (`CEffectPrim::OnProcess` and the…
    master.rs             What a primitive shares with the effect it belongs to (the client's `master`, a…
    outdoors.rs           Two kinds of sprite the effects of a map itself are made of
    particle.rs           The primitives that show a sprite: `PP_3DPARTICLE` (5), `PP_3DPARTICLEGRAVITY` (6),…
    particle_tests.rs     tests
    petals.rs             `PP_KOUENKA` (124): a sprite that rises wandering from side to side and fades…
    plate.rs              `PP_3DTEXTURE` (15): one picture standing in the world, moved by `MoveEffectPrimitive`…
    quarter.rs            Which of the four ways across the screen an effect points, by which several effects…
    slots.rs              The kinds made of parts. The client gives every primitive four `EffectPrimitiveSlot`s,…
    slots_tests.rs        tests
    solid.rs              How the primitives that stand in the world are shaped
    streak.rs             `PP_CHEMICAL3` (56): streaks of light that lie flat, 6 above the ground, along the way…
    tei.rs                The kinds the client builds corner by corner out of its four parts and draws with…
      aid.rs                `PP_FIRSTAID` (58) and the kind 67 of the client's primitives, two pictures that only…
      arcs.rs               `PP_HITLINE3` (100) of the client's primitives
      auras.rs              The kind 82 of the client's primitives: one picture over the one the effect is on,…
      beams.rs              The kinds 89 and 69 of the client's primitives
      blades.rs             The kind 159 of the client's primitives: thin needles of light that fly in from every…
      boomerang.rs          The kind `PP_SHIELDBOOMERANG` (42) of the client's primitives
      bounces.rs            The kind `PP_HITLINE` (70) of the client's primitives
      boxes.rs              The kind `PP_GRANDCROSS2` (37) of the client's primitives
      cage.rs               The kind `PP_GUARD` (61) of the client's primitives
      chasers.rs            The kinds 122 and `PP_STIN4` (125) of the client's primitives
      columns.rs            The kind 60 of the client's primitives as it lives when its columns are not of the…
      cones.rs              The kind `PP_MAGNUM2` (73) of the client's primitives
      drawn.rs              What a kind built of the client's parts shows now
      drifts.rs             The kind 146 of the client's primitives: walls of light most of the way round a…
      drops.rs              The kind 87 of the client's primitives: a square facing the screen that comes down…
      dust.rs               The kind 157 of the client's primitives: squares facing the screen that fall from…
      field.rs              `RenderFieldPanel`, whole: how the client draws a part of the kinds that are squares…
      fins.rs               The kinds 154 and 156 of the client's primitives
      flames.rs             The kinds 155 and 170 of the client's primitives
      flecks.rs             The kind `PP_TEIHIT2` (45) of the client's primitives
      flowers.rs            `PP_FLOWERCAST` (108) of the client's primitives
      forest.rs             `PP_FOREST` (66) of the client's primitives
      gust_rings.rs         The kind 126 of the client's primitives: a square of wind standing upright along the…
      gusts.rs              The kind 119 of the client's primitives: a square of wind lying level that spins and…
      halo.rs               The kind 39 of the client's primitives: a square of light lying level over the ground…
      icons.rs              The kind 35 and `PP_DOUBLEGUMGANG` (99) of the client's primitives
      launchers.rs          `PP_SMA` (120) and the kind 151 of the client's primitives, which show nothing…
      layers.rs             `PP_ASURACASTING` (68) and the kind 147 of the client's primitives
      lights.rs             The kind 145 of the client's primitives: a light that goes round the one its effect…
      links.rs              `PP_LINELINK` (38) of the client's primitives
      low_rings.rs          The kind 36 of the client's primitives: walls of light of ten pieces that lean…
      max_level.rs          The lights of the aura of the highest level
      more_tests.rs         tests
      orbit.rs              The kind `PP_INTIMIDATE` (43) of the client's primitives
      panels.rs             The kinds 115 and `PP_SHIELDBOOMERANG3` (116) of the client's primitives
      part.rs               A part of a primitive of the kinds built corner by corner (`prim/tei.rs`)
      peong.rs              `PP_PEONG` (94) and `PP_PEONGUP` (96) of the client's primitives, the puff of one who…
      pillars.rs            The kinds 137 and 140 of the client's primitives
      plates.rs             The kind 44 of the client's primitives: up to four long plates of light that shoot…
      quarters.rs           The kind `PP_GI_3` (34) of the client's primitives
      rect.rs               A rectangle as the client's `RenderTeiRect` draws one, and the ways the kinds of…
      ripples.rs            The kind 63 of the client's primitives: rings of light that widen from the place of…
      scans.rs              The kind 135 of the client's primitives: a thin line of light from the place of the…
      sizes.rs              The colours `RenderTeiRect` has for the bands whose primitive's `size` picks one (its…
      sky.rs                The kind 33 of the client's primitives: squares facing the screen that come up, stand…
      slashes.rs            `PP_SLASH1` (75): thin blades of light that shoot out along the ground from the place…
      souls.rs              `PP_SOULLINK` (109) and `PP_SOULPARTICLE` (111)
      spheres.rs            The kind 32 of the client's primitives: a sphere of light that goes round the one its…
      squares.rs            The kind 92 of the client's primitives: squares that lie level and slide along the…
      stretch.rs            The kind 141 of the client's primitives: a bar of light across the one the effect is…
      tests.rs              tests
      twinkles.rs           The kind 132 of the client's primitives: upright squares of light through the one the…
      winds.rs              `PP_WIND` (62), `PP_SPHEREWIND` (77) and `PP_SPHEREWIND2` (91)
    tests.rs              tests
    tint.rs               The colour of a primitive that changes as it lives
    turn.rs               How a primitive is turned in the world and where its middle is
    walls.rs              Three more kinds made of walls of light round the place of the effect, each wall a band…
    walls_tests.rs        tests
  pvp.rs                Fighting other players as rules: what holds on the map the character is on and who on it…
  quake.rs              The shake of the screen that some effects ask for (the client's `MM_QUAKE`, which…
  quests.rs             The quest journal's rules that need no page
    tests.rs              tests
  route.rs              A walk as the client shows it: the way cell by cell with the server's time at which each…
    tests.rs              tests
  said.rs               The lines of the chat window: the kinds the client gives them, by which the window's tabs…
  scenery.rs            What surrounds the player on a map besides its ground and its models, as rules that need…
    dark.rs               The darkening of the world at night: the client's `CWeatherEffect::StartDarken`,…
    fog.rs                The fog of a map: `data\fogparametertable.txt` as `CGameMode::ApplyMapFog` reads a…
    makers.rs             The effects and the sounds a map places (`model::MapEffect`, `MapSound`), each kept by…
    views.rs              The camera's view of a map: the figures `data\viewpointtable.txt` has for some maps…
  scope.rs              The circle on the ground under a place a skill is being cast on, as the client's…
  screenshot.rs         A picture of the screen as the client names and stamps it (`CMode::TakeScreenshot`,…
  server_time.rs        The server's clock as the client keeps it
    tests.rs              tests
  shops.rs              An NPC's shop as rules, those of the client's two windows of it…
  skill_units.rs        Skill units: what a skill leaves standing on the ground (a cell of Fire Wall, of…
    pulse.rs              The box of light that stands on a cell of Sanctuary and of Magnus Exorcismus
    traps.rs              The traps a Hunter lays: the units the client shows as a model standing on their cell…
  skills.rs             Skills as rules, and no packets: which line of the client's message table says why a…
  snap.rs               A window being moved docks to what is near it, as the client's do with `/window` on
  song_talk.rs          The lines of Frost Joke and Scream: the client's effects `EF_TALK_FROSTJOKE` (295) and…
  spirits.rs            The spirit spheres about a monk and the coins about a gunslinger, as rules
  sprite.rs             The client's sprites: SPR (the pictures), ACT (how they are laid out per action and…
    act.rs                ACT: how a sprite's pictures are laid out in each frame of each action, the frames'…
    canvas.rs             Drawing a frame of a sprite into an RGBA picture
    imf.rs                `.imf`: which of a character's body and head lies over the other in each frame of each…
    layers.rs             A frame of a sprite as the client's effects draw it
    spr.rs                SPR: a sprite's pictures, palette-indexed and truecolor, and PAL, a palette to recolour…
    tests.rs              tests
  stack.rs              The order the game's windows lie in, one over another
  state_effects.rs      The effects a state going on or off begins and ends on the one in it
    kept.rs               The effects a state of someone's keeps going for as long as it holds
    numbers.rs            The client's numbers of the effects the states begin and end (`EF_…`), named once for…
    own.rs                The player's own character arriving on a map
    seen.rs               Someone coming into sight: what the bits of its third option begin and end on it…
    tests.rs              tests
  statuses.rs           The states a character is in (its buffs and debuffs), as the client lists them at the…
  statuses.txt          read by statuses.rs
  storage.rs            The storage as rules, those of the client's window of it (`UIItemStoreWnd`)
  tables.rs             The shapes the client's tables of text come in, each read here once
  vector.rs             Vectors of three, as far as the game needs them
  vending.rs            A player's shop as rules: what the client lets a shop have and a thing cost, what the…
  whispers.rs           The rules of a talk with one player in a window of its own (the client's `UIWhisperWnd`)
  net/
    actors.rs             The packets about those in sight on the map, players, NPCs and monsters
      changes.rs            What changes about someone in sight: the state it is in, what it wears and holds,…
      ground.rs             The things that lie on the ground: one falling, one coming into sight, one gone, and…
      requests.rs           What the client asks for its own character among those in sight
      tests.rs              tests
    books.rs              A book an NPC's script has the player read (`readbook`)
    broadcast.rs          The server's announcements: ZC_BROADCAST and its coloured form, as the client reads…
    bytes.rs              Reading the numbers and the texts of a packet, and writing a name into one
    cart.rs               The merchant's cart: what lies in it, how full it is, things moved between it and the…
    chat_rooms.rs         Chat rooms: a player's room and the waiting room of an NPC, the sign of one over its…
    choosing.rs           The lists a skill or a card opens to pick one thing of, and the answers to each
      cards.rs              A card set into equipment: what the card can go into, the thing picked, and whether…
      making.rs             What a skill makes: the arrows to craft, the spells of Auto Spell, what can be…
      mending.rs            What a smith mends and betters: the broken things to repair, the weapons to refine,…
      tests.rs              tests
      warps.rs              The places of Teleport and Warp Portal: the list the skill opens, the place picked,…
    commands.rs           What the chat commands send to the map server and hear back
    companions.rs         What the packets of a pet and of a homunculus share
    config.rs             The switches the server keeps for a character (ZC_CONFIG, CZ_CONFIG)
    cooldowns.rs          The wait before a skill may be used again, as the map server tells of it
    emotions.rs           Emotions: the little pictures shown over someone's head
    exchange.rs           A deal between two players: asking for one, what each side puts into it, closing one's…
    family.rs             The family: what the server asks and tells of a wedding, a divorce and an adoption
    fight.rs              What the server says of a fight beside the blows themselves
    friends.rs            The friends list: the list sent on entering, who of it is in the game, asking someone…
      tests.rs              tests
    golden.rs             Every packet this client writes, built with fixed arguments and compared byte for byte…
    golden.txt            read by golden.rs
    guild.rs              The guild: what it is, who is in it and in which position, its notice, its chat, and…
      manage.rs             What the master of a guild does with it: the positions and who has which, the emblem,…
      members.rs            The guild's members and its positions as the server lists them, and those it has…
      requests.rs           What the client sends about the guild: asking for its tabs and names, making one,…
      tests.rs              tests
    homunculus.rs         The alchemist's homunculus: what the map server tells of it, and what its owner's…
      tests.rs              tests
    inventory.rs          The packets about what the character carries and knows
      favourites.rs         The inventory's fourth tab, "Fav": which of the things carried the player keeps under…
      hotkeys.rs            The hotkey bar as the server keeps it: its slots, and setting one
      skills.rs             The skills a character knows: the list, one added, one raised
      tests.rs              tests
      wearing.rs            Putting things on and taking them off, as the server answers and as the Equipment…
    link.rs               The connection itself: the client's watch on a connection gone silent, and why a server…
    mail.rs               The mail (the client's RODEX): the letters the map server lists, one read, its zeny and…
      requests.rs           What the client sends about the mail: opening and closing the mailbox, reading,…
      tests.rs              tests
    marks.rs              Marks on the minimap that the server tells of
    mercenary.rs          A hired mercenary: what the map server tells of it, and the command its owner gives it
    mod.rs                The game protocol, without any I/O: how long each packet is and cutting the byte stream…
    notices.rs            What the map server tells the player in a line of the chat, with a number or a name to…
      tests.rs              tests
    packet_lengths.txt    read by mod.rs
    packet_lengths_more.txt  read by mod.rs
    packet_names.txt      read by unread.rs
    packets.rs            The packets of the way in (the login server, the character server, entering the map…
      chars.rs              The character server: the list of characters, making and deleting one, and the map…
      tests.rs              tests
      zone.rs               The basics of the map server: entering a map, its clock, walking, the character's…
    party.rs              The party: who is in it, what they share, what they say to each other, and the packets…
      requests.rs           What the client sends about the party: making one, inviting and answering, leaving…
      tests.rs              tests
    pets.rs               Pets: what the map server tells of the player's own (everything about it, one thing…
      requests.rs           What the client asks of the map server about a pet
      tests.rs              tests
    pvp.rs                Fighting other players: what kind of map the character is on, who on it may be fought,…
      tests.rs              tests
    quests.rs             The quests: the journal the map server sends on entering, what it says as a quest is…
      tests.rs              tests
    scripted.rs           What an NPC's script has the client show and play beside its words
    shops.rs              Buying from and selling to NPC shops (rAthena, `clif_npcbuysell`, `clif_buylist`,…
    skill_units.rs        Skill units, what skills leave standing on the ground, as the map server tells of them…
    skills.rs             Skills in use: what the map server tells of a skill being cast, of its landing and of…
    spirits.rs            The spirit spheres of a monk and the coins of a gunslinger, as the server tells how…
    statuses.rs           The states someone is in (buffs and debuffs), as the map server tells of them going on…
    storage.rs            The Kafra's storage and a guild's, which the server opens, fills and closes with the…
    talk.rs               Talking to NPCs: what the player sends, and what the script answers (rAthena,…
    unread.rs             The packets of the map server that this client does not read yet, counted
    vending.rs            A merchant's shop (vending): opening one's own out of the cart, what is left in it and…
      requests.rs           What the client sends about a shop: opening one's own out of the cart and closing it,…
      tests.rs              tests
    viewequip.rs          Looking at what another player wears: asking the server, its answer with the look and…
      tests.rs              tests
    whispers.rs           Whispers: one heard, one sent and the answer to it
  web/
    app.rs                The app: the way into the game (login, accounts, character select and creation) and the…
      account.rs            Logging in, registering and recovering an account, and the questions asked on the way
      action.rs             Everything that happens, as a value: an event of the page, of the world, of a socket…
      actions.rs            What the app does about each action: the one place every event of the page and of the…
        bag.rs                Double clicks on things, things dragged out of a window and let go over another,…
        forms.rs              What the page's forms do when sent.
        link.rs               The connection: what comes of the socket, and keeping it alive
        pointer.rs            What the pointer did on the map, as the world reports it (`world::Hooks`)
        taming.rs             Taming a monster, and the menu on the player's own pet
        travel.rs             The character moving: over the cells of a map, and from map to map
      ai.rs                 The scripts of the homunculus and the mercenary at work (`crate::ai`)
      chat.rs               The chat window on the app's side: a line added to it, and its next size
      clicks.rs             What each thing on the page does when pressed
        account.rs            Before the game: the login, the servers, the characters and the making of one
        books.rs              The books that are read (`crate::books`, the client's `UIBookWnd`)
        cart.rs               The cart's window and what goes into the cart and out of it as the client moves it…
        chat.rs               The chat window, its tabs' own windows, and the window that sets what a tab lists
        chat_rooms.rs         Chat rooms: opening the window that makes one, what its buttons and the room's own…
        choosing.rs           The lists a skill or a card opens to pick one thing of
        emotions.rs           The window of emotions and the Shortcut List
        exchange.rs           A deal between two players: asking for one, answering, putting things and zeny in,…
        family.rs             The family: the answers to one who asks to adopt the character (the client's…
        fight.rs              What the buttons of the windows a fight opens do
        friends.rs            The friends list: what the buttons of its side of the window do, its key, the menu…
        general.rs            What belongs to no one window: the language, the sound, the minimap's buttons, the…
        guide.rs              The Adventure Guide.
        guild.rs              The guild: what its window's tabs, rows and buttons do, the menus in it, the…
          manage.rs             The guild: what its master does in its window and on other players — the…
        homunculus.rs         The homunculus window, what it asks, the list of the homunculus's skills, and the…
        hotkeys.rs            The Shortcut Settings window.
        inventory.rs          The windows of what is carried: the inventory, the equipment, an item's description
        mail.rs               The mail: opening the mailbox, its tabs, pages and search, reading a letter, taking…
          writing.rs            A letter being written: asking to write one, the receiver asked about, things…
        mercenary.rs          The mercenary window, what it asks, the list of the mercenary's skills, and the…
        party.rs              The party: what its windows' buttons do, the commands typed for it, and the menu on…
        pets.rs               Taming a monster, hatching an egg, and the pet window
        player_menu.rs        The menu on another player: opening it where the right button was pressed, and what…
        quests.rs             The quest journal: its tabs, filters, search and rows, the pane of one quest, and…
        scripted.rs           What is pressed of what an NPC's script shows, and the times such things run by
        settings.rs           The Game Settings window.
        shops.rs              An NPC's shop: its question, and what goes across its two windows, the list and the…
        skills.rs             The skill window, a skill's description, and the hotkey bar
        status.rs             Basic Information and the Status window.
        storage.rs            The storage's window: its tabs, its order, its search, and what goes into it and…
        talk.rs               The window of a talk with an NPC.
        tips.rs               The Tipbox.
        unread.rs             The list of packets this client does not read
        vending.rs            A merchant's shop: making one's own out of the cart and closing it…
        viewequip.rs          Looking at another player's equipment: asking for it, closing its window, and the…
        whispers.rs           A talk with one player in a window of its own
        worldmap.rs           The World Map and the large window of the map one is on
      commands.rs           The client's own commands: typed into the chat after a slash, or run from the Game…
        beside.rs             The client's commands beside those that switch a row of the Game Settings
      connection.rs         The connection to the login, character and map servers in turn, and the way from one…
      dragging.rs           Dragging the things in the windows (items, skills, what is on the hotkey bar), as the…
      hotkeys.rs            Which key does what for this player, and the setting of it in the Shortcut Settings…
      incoming.rs           What the servers say: each packet is read and the app's state, the windows and the…
        account.rs            What the login server and the character server say on the way in
        actors.rs             Everyone else on the map and the things lying on it
        ai.rs                 What the scripts of the homunculus and the mercenary need of the server's packets…
        books.rs              A book an NPC's script has the player read (`crate::net::books`)
        broadcast.rs          The server's announcements (ZC_BROADCAST and its coloured form)
        cart.rs               The merchant's cart: what the map server tells of it
        chat_rooms.rs         What the server says of chat rooms (`crate::net::chat_rooms`)
        choosing.rs           What the server says of the lists a skill or a card opens to pick one thing of, and…
        combat.rs             Blows and skills: who strikes whom, what is being cast, what it does when it lands
        commands.rs           What the server answers to the chat commands
        cooldowns.rs          The waits before skills may be used again, as the server tells of them
        emotions.rs           The emotions shown over heads.
        exchange.rs           A deal between two players: what the map server tells of it
        falcon.rs             The hunter's falcon: the skills that send it at something
        family.rs             What the server says of the family (`crate::net::family`)
        fight.rs              What the server says of a fight beside the blows (`net/fight.rs`)
        friends.rs            The friends list: what the map server tells of it
        guild.rs              The guild: what the map server tells of it
          members.rs            The guild's lists as the server tells them
        homunculus.rs         What the server tells of the alchemist's homunculus
        hotkeys.rs            The hotkey bar as the server keeps it: its list, told on coming in
        inventory.rs          The inventory: what is carried, picked up, used, dropped, worn and taken off
        link.rs               What the servers say of the connection itself
        mail.rs               The mail: what the map server tells of it
        marks.rs              Marks on the minimap that the server tells of
        mercenary.rs          What the server tells of a hired mercenary
        notices.rs            What the server tells the player in a line of the chat
        party.rs              The party: what the map server tells of it
        pets.rs               Taming a monster, hatching its egg, and what becomes of the pet
        pvp.rs                Fighting other players: what the map server tells of the map's kind and of the…
        quests.rs             The quest journal as the server tells it, what the chat says of it, and the marks…
        scripted.rs           What an NPC's script has the client show and play (`crate::net::scripted`)
        shops.rs              An NPC's shop: its question, its lists, and what it answers to a deal
        skill_units.rs        The units skills leave on the ground, and skills used on a place
        spirits.rs            The spirit spheres about a monk and the coins about a gunslinger
        status.rs             What the character is: its numbers (life, stats, experience) and the skills it knows
        statuses.rs           The states someone is in (buffs and debuffs) going on and off
        storage.rs            The storage, the Kafra's or a guild's: opened, filled, added to, taken from and…
        talk.rs               What is said: NPC dialogs and the chat. (Whispers
        travel.rs             Arriving on a map, the character's own steps, and being moved or sent back to the…
        unread.rs             A packet of the map server that none of the readers took
        vending.rs            Merchants' shops: what the map server tells of them
        viewequip.rs          Looking at another player's equipment: the server's answer, the player's own switch…
        whispers.rs           Whispers: one heard, and the server's answer to one sent
      keys.rs               The keyboard: on the windows before the game, and in the game
      page.rs               The page itself: its elements and form fields, what the browser keeps between visits,…
      screen.rs             Putting the app on the page: which screen shows, and the characters painted on it
      screenshot.rs         A picture of the screen, given to the player as a file (the client's `MM_SCREENSHOT`
      settings.rs           What the player has set in the Game Settings window, and its keeping
      state.rs              What the app keeps, in groups: one for each part of the game, each in a file of its…
        ai.rs                 What the app keeps of the scripts that move the homunculus and the mercenary…
        books.rs              What the app keeps of the books that are read (`crate::books`)
        broadcast.rs          What the app keeps of the server's announcements
        cart.rs               What the app keeps of the merchant's cart
        chat.rs               What the app keeps of the chat window.
        chat_rooms.rs         What the app keeps of chat rooms: the rooms whose signs it was told of, the room…
        choosing.rs           What the app keeps of the lists a skill or a card opens to pick one thing of
        commands.rs           What the app keeps for the chat commands
        cooldowns.rs          What the app keeps of the waits before skills may be used again
        emotions.rs           What the app keeps of the window of emotions and of the Shortcut List
        exchange.rs           What the app keeps of a deal between two players
        family.rs             What the app keeps of the family: who the character is married to, and who asks to…
        fight.rs              What the app keeps of a fight beside the blows
        friends.rs            What the app keeps of the friends list.
        guide.rs              What the app keeps of the Adventure Guide
        guild.rs              What the app keeps of the guild.
        homunculus.rs         What the app keeps of the alchemist's homunculus
        inventory.rs          What the app keeps of the inventory: what is carried, and the windows of it
        link.rs               What the app keeps of the connection for the watch on it
        mail.rs               What the app keeps of the mail.
        marks.rs              What the app keeps of the marks on the minimap that the server tells of
        mercenary.rs          What the app keeps of a hired mercenary: what the server told of it, its skills,…
        notices.rs            What the app keeps of what the server's notices tell
        party.rs              What the app keeps of the party.
        pets.rs               What the app keeps of the pets: the player's pet and its window, the eggs, the slot…
        player_menu.rs        What the app keeps of the menu on another player
        pvp.rs                What the app keeps of fighting other players
        quests.rs             What the app keeps of the quests.
        scripted.rs           What the app keeps of what an NPC's script shows and plays
        settings.rs           What the app keeps of the Game Settings and of the Shortcut Settings
        shops.rs              What the app keeps of dealing with an NPC's shop
        skills.rs             What the app keeps of the skills: those known, the one aimed, the hotkey bar
        status.rs             What the app keeps of the character itself
        statuses.rs           What the app keeps of the states the character is in
        storage.rs            What the app keeps of the storage: the Kafra's, or a guild's, which is the same…
        talk.rs               What the app keeps of a talk with an NPC
        tips.rs               What the app keeps of the Tipbox.
        unread.rs             What the app keeps of the packets it does not read
        vending.rs            What the app keeps of merchants' shops: the signs in sight, a shop being made,…
        viewequip.rs          What the app keeps of looking at another player's equipment
        whispers.rs           What the app keeps of the talks with one player each
        worldmap.rs           What the app keeps of the World Map and of the large window of the map one is on
      windows.rs            The order of the game's windows on the page (`crate::stack`)
      wiring.rs             Starting the client: the page's events (clicks, keys, drags, timers) are turned into…
    data.rs               The game's own tables, as the gateway gives them out of the client's files
      ai.rs                 The scripts that move a homunculus and a mercenary (`crate::ai`)
      aura.rs               What the client's settings say of the aura of the highest level (the gateway's…
      books.rs              The books (`crate::books`): which items are books, out of the client's…
      cards.rs              What the client's item naming reads beside the item table (the gateway's…
      commands.rs           The chat commands: the help the client prints for them, and what an option says when…
      figures.rs            How characters, monsters and NPCs are drawn, as the client's Lua says it
      guide.rs              The Adventure Guide: its pages, and who can wear what it recommends
      interface.rs          The interface's own tables: the client's messages, keys, options and emotions
      items.rs              The items: names, pictures and descriptions, asked for from the gateway as they are met
      job_bases.txt         read by jobs.rs
      job_sprites.txt       read by jobs.rs
      jobs.rs               The jobs and what is drawn for them: names, body sprites, headgear, weapons, and the…
      maps.rs               The maps: their music, what the client calls them, and who stands where in the towns
      pets.rs               The pets: their pictures, the actions of those that wear something, and what they say
      quests.rs             The quests: what the journal writes of each, what the client recommends, and how it…
      scenery.rs            What surrounds a map: the sky the client draws behind it (its `MapSkyData.lub`)
      skills.rs             The skills: what they are called, their pictures and descriptions, and where they…
      song_talk.rs          The lines of Frost Joke and Scream (`crate::song_talk`)
      statuses.rs           The states a character is in (buffs and debuffs)
      tips.rs               The tip box: the client's tips, each with its title and its pages
      worldmap.rs           The world map: the client's worlds, the picture of each, and where each map lies on it
    figure.rs             Characters as the windows show them: body, head and headgear sprites put together and…
      compose.rs            A character put together from its parts, as the client's…
      weapon.rs             The weapon a character holds: which sprite it is, and its picture by itself
    hud.rs                The windows of the game itself. This file has the list of them and what they all are…
      basic.rs              The Basic Information window with its gauges, and the strip of menu icons under it
      books.rs              The window a book is read in (the client's `UIBookWnd`), the line of a book read…
      boxes.rs              The boxes every system asks with, each written once
      broadcast.rs          The server's announcements across the top of the screen (the client's…
      buttons.rs            The buttons of the windows, each kind written once
      cart.rs               The merchant's cart: its window (`UIMerchantItemWnd`), the box that asks how many go…
      chat.rs               The chat window: its tabs, its lines and the line to type in
      chat_rooms.rs         Chat rooms on the page: the sign over a room's owner (the client's…
      checks.rs             The client's small boxes that are ticked and its round buttons of which one is on,…
      choosing.rs           The windows of the lists a skill or a card opens to pick one thing of, in the…
      colours.rs            The colours the windows share, as the page takes them
      cooldowns.rs          The shade the client lays over a picture while a time runs (`Source/UI/Cooldowns`)
      emotions.rs           The window of emotions, and the Shortcut List
      equipment.rs          The Equipment window (the client's `UIEquipWnd`)
      esc.rs                The game options (the client's `UIEscOptionWnd`), which stand where an NPC's window…
      exchange.rs           The windows of a deal between two players
      family.rs             The family on the page: the question of one who asks to adopt the character (the…
      fields.rs             The fields of the windows that are typed into, written once
      fight.rs              The windows a fight opens: what Sense tells of a monster (the client's…
      frames.rs             What the windows are framed with, each piece written once
      friends.rs            The friends list's windows: the Friend side of the window it shares with the party…
      gauges.rs             The gauges of the windows, their pieces written once
      guide.rs              The Adventure Guide (the client's `CUIAdventureGuide`)
        pages.rs              The pages of the Adventure Guide that list things
      guild.rs              The guild's windows: the guild window with its six tabs (the client's `UIGuildWnd`…
        expelled.rs           The guild window's fifth tab: those expelled (`UIGuildBanishedMemberWnd`)
        members.rs            The guild window's second tab: the members (`UIGuildMemberManageWnd`)
        notice.rs             The guild window's sixth tab: the notice (`UIGuildNoticeWnd`)
        positions.rs          The guild window's third tab: the positions (`UIGuildPositionManageWnd`)
        skills.rs             The guild window's fourth tab: the guild's skills (`UIGuildSkillWnd`)
      homunculus.rs         The homunculus window (the client's `UIHomunInfoWnd`), what it asks, the list of the…
      hotkeys.rs            The Shortcut Settings window: every key of the client, and the setting of another in…
      inventory.rs          The inventory and what belongs to its items
      items.rs              What the windows show of an item or a skill, written once
      lists.rs              The client's windows that list things a row to a thing (`UIItemBaseWnd` and those…
      mail.rs               The mail (the client's RODEX): the notice of mail not read (`UIOpenMailBoxWnd`), the…
        letter.rs             A letter of the mail on the screen: the one being read (the client's…
        list.rs               The mailbox's list (`UIRodexInboxWnd`): a row of it, and the search over it
      marks.rs              The marks on the minimap that the server tells of
      mercenary.rs          The mercenary window (the client's `UIMerInfoWnd`), what it asks, the list of the…
      messenger.rs          What the two sides of the client's `UIMessengerGroupWnd` share, the Party side…
      party.rs              The party's windows: the list of members with the bar of buttons under it (the Party…
        side.rs               The window beside the party window: the one that sets the party up or makes one…
      pets.rs               The pet window: what the pet is called, how hungry and how close it is, and the…
      player_menu.rs        The menu the right button opens on another player (the client's…
      pvp.rs                The standing on a PVP map: "rank / total" in the corner of the screen, in the digits…
      quests.rs             The quest journal (the client's `CUIRenewQuestUI`), the button that tells of a new…
        pane.rs               The pane of one quest, beside the journal
      scripted.rs           What an NPC's script shows on the page: the picture over the screen (the client's…
      settings.rs           The Game Settings window.
      shops.rs              An NPC's shop: the question whether to buy or to sell, and the client's two windows…
      skill_list.rs         The list of a homunculus's skills and of a mercenary's
      skills.rs             The skill tree, a skill's description, and the hotkey bar
      status.rs             The Status window: the six stats, what they give, and the points to raise them
      statuses.rs           The states the character is in (buffs and debuffs), listed down the right side of the…
      storage.rs            The storage's window (the client's `UIItemStoreWnd`
      tabs.rs               The tabs down a window's left side: the client's `UITabControl` in its layout 0, as…
      talk.rs               What is shown in the middle of the screen
      texts.rs              The texts of the windows: how the client asks for a font, a line at its place in that…
      tips.rs               The Tipbox: the client's tips, a page at a time
      unread.rs             The list of the packets of the map server that this client does not read yet
      vending.rs            A merchant's shop: the signs over the vendors' heads (`UIMerchantShopTitle`), the…
        buying.rs             Buying from a vendor: the vendor's list (`UIMerchantItemShopWnd`), the basket…
      viewequip.rs          Looking at another player's equipment: the window that shows it (the client's…
      whispers.rs           The window of a talk with one player (the client's `UIWhisperWnd`, 280 by 120), and…
      windows.rs            The windows of the game's screen as one list
        basic.rs              The entries of the list of windows for what is always in the corner of the screen
        books.rs              The entries of the list of windows for the books
        broadcast.rs          The entry of the list of windows for the server's announcements across the top of…
        cart.rs               The cart's entries of the list of windows
        chat.rs               The entries of the list of windows for the chat's tabs taken out into windows of…
        chat_rooms.rs         The entries of the list of windows for chat rooms
        choosing.rs           The entries of the list of windows for the lists a skill or a card opens to pick…
        emotions.rs           The entries of the list of windows for the window of emotions and the Shortcut List
        equipment.rs          The entry of the list of windows for the Equipment window
        exchange.rs           The entries of the list of windows for a deal between two players
        family.rs             The entry of the list of windows for the family
        fight.rs              The entries of the list of windows for what a fight opens
        friends.rs            The friends list's entries of the list of windows
        guide.rs              The Adventure Guide's entry of the list of windows
        guild.rs              The guild's entries of the list of windows
        homunculus.rs         The homunculus's entries of the list of windows
        inventory.rs          The items' entries of the list of windows
        mail.rs               The mail's entries of the list of windows
        mercenary.rs          The mercenary's entries of the list of windows
        party.rs              The party's entries of the list of windows
        pets.rs               The pets' entries of the list of windows
        places.rs             Where the windows stand, as their tags say it
        player_menu.rs        The entry of the list of windows for the menu on another player
        quests.rs             The quests' entries of the list of windows
        redraw.rs             The list of windows on the page: the screen written from it, and windows drawn anew…
        scripted.rs           The entry of the list of windows for what an NPC's script shows
        settings.rs           The entries of the list of windows for the Shortcut Settings and the Game Settings
        shops.rs              The entries of the list of windows for an NPC's shop
        signs.rs              The entry of the list of windows for the signs over heads (a chat room's, a shop's)
        skills.rs             The skills' entries of the list of windows
        status.rs             The entry of the list of windows for the Status window
        statuses.rs           The entry of the list of windows for the states down the right side of the screen
        storage.rs            The entries of the list of windows for the storage
        talk.rs               The entries of the list of windows for what is said
        tips.rs               The entry of the list of windows for the Tipbox
        unread.rs             The entry of the list of windows for the packets this client does not read…
        vending.rs            The entries of the list of windows for merchants' shops
        viewequip.rs          The entry of the list of windows for looking at another player's equipment
        whispers.rs           The entries of the list of windows for the talks with one player each
        worldmap.rs           The entries of the list of windows for the World Map and the large window of the…
      worldmap.rs           The World Map, and the large window of the map one is on
    mod.rs                Everything that needs the browser: the windows, the game world on the canvas, and the…
    parts.rs              The pieces every window on the page is made of, whichever screen it belongs to
    socket.rs             A connection to a game server, through the gateway's WebSocket bridge
    sound.rs              The sounds of the game, as the original client plays them
    view.rs               The windows, as HTML laid over the game's own pictures
      accounts.rs           The windows of the account: logging in, registering, recovering and resetting a…
      characters.rs         Choosing a server and a character. Making a character is `view/creation.rs`
      creation.rs           Making a character (the client's `UINewMakeCharWnd`)
      game.rs               The game's screen: the minimap in its corner with the character's place under it,…
    world.rs              The game world on the canvas: the map's ground in 3D, the player's character on it, the…
      actors.rs             What the server says of everyone else on the map and of the things lying on it
      ai.rs                 What the scripts of a homunculus and a mercenary are told of the map (`crate::ai::Seen`
      ailments.rs           Body states and health states as they show in the world (`crate::ailments`, the…
      aura.rs               The aura of the highest level, kept on those who show it
      camera.rs             The camera over the character and the view it gives
      cart.rs               The carts merchants pull: drawn behind each owner as the client's effect that follows…
      combat.rs             Blows and casts: what the app tells the world when someone strikes, casts, aims or is…
        landing.rs            The hits that land: the one struck flinches and stops, the hit sounds from where it…
      earshot.rs            The sounds of the world, each heard from where it starts
      effects.rs            The effects the client draws by code rather than from `.str` files
      falcon.rs             The falcons hunters keep: drawn by each owner as the client's effect that follows one…
      frame.rs              One frame of the world: everyone is moved to where they are now, the effects advance,…
        effects.rs            The effects of this frame, each kind turned into quads for the renderer
        marks.rs              What is laid on the ground and hung in the air this frame
        sprites.rs            Who and what is drawn standing in the world this frame
      gpu.rs                The GPU side: pipelines, textures and buffers, and the drawing of one frame
        draw.rs               The drawing of one frame: the map, on it the sprites and the numbers, and over them…
        land.rs               The drawing of the map itself: its ground, its models and its water, and what their…
        layers.rs             The sprites of the effects made of the client's primitives
        textures.rs           Pictures put on the GPU: a texture of a picture's bytes, the bind groups a pipeline…
      guild.rs              The guilds of the players in sight: which guild each is of, what the app has learnt…
      hooks.rs              What passes between the world and the page
      input.rs              The mouse over the world: walking, pressing someone, aiming a skill, turning and…
        pointer.rs            The pointer moves over the page, and a button is let go
        press.rs              A button of the mouse goes down on the world
        wheel.rs              The wheel over the world: the level of a skill being aimed, or the camera
      land.rs               The models and the water of a map on the GPU
      launch.rs             One of the client's effects begins on someone, or ends
      loading.rs            Reading a map from the client's files: ground, models, water, and the pictures the…
      math.rs               Vectors of three, as far as the world needs them (`crate::vector`), and the lots the…
      mover.rs              Those who stand and walk on the map: where each is at a moment, and which picture of…
      names.rs              What the server told of those in sight beside their names, for the name under whoever…
      overlays.rs           What is laid over the picture of the world once it is drawn
        bars.rs               The bars and words laid over the world: the life of the monsters being fought, the…
        pointer.rs            The picture of the mouse pointer over the world
      pick.rs               What the pointer is on: someone in the world, or a thing lying on the ground
      players.rs            What the app asks about the other players on the map
      pose.rs               How those in sight are posed and shown, as the server tells of each
      prims.rs              The effects the client draws by code, out of its primitives, under way in the world
        auras.rs              Pictures over the one an effect is on that face the screen (the kinds 82 and 141 of…
        blasts.rs             What bursts out over the ground round the one an effect is on, each with the…
        blows.rs              The effects of blows whose primitives are of the kinds the client builds out of its…
        bodies.rs             Effects of the client's that draw nothing of their own and change the body of the…
        bolts.rs              The mage's bolts and ice, each with the figures of its function of `CRagEffect`
        burst.rs              An effect drawn by code under way on someone (`Burst`)
        bursts.rs             Effects that burst out from the one they are on across the screen, each with the…
        course.rs             What an effect drawn by code remembers from tick to tick (`Course`
        drawn.rs              What an effect drawn by code shows now, a piece at a time (`Piece`), for the frame…
        falls.rs              What falls on the one an effect is on from high above, each effect its function of…
        grounds.rs            The effects the units of skills on the ground launch on themselves…
        gusts.rs              Squares of wind that spin and fly over the ground from the one who strikes (the…
        heals.rs              Healing and Resurrection, each with the figures of its function of `CRagEffect`
        held.rs               What a state shows on the one in it for as long as it holds (`crate::state_effects`…
        hits.rs               What a blow shows where it lands, each with the figures of its function of…
        launched.rs           Primitives as several files of effects launch them, each written once
        links.rs              Effects that link the one they are on to another actor with a tube of light that…
        marks.rs              Pictures that lie level over the ground under the one an effect is on, each with…
        max_level.rs          The aura of the highest level, three effects that live until they are ended…
        missiles.rs           What else flies or runs from the one who strikes to a place it is told, beside the…
        motes.rs              Small lights round the one an effect is on
        pillars.rs            Thin shafts of light that shoot up out of the ground (the kind 137 of the client's…
        places.rs             The effects a map places about itself (its `.rsw` lists them,…
        played.rs             Effects that are a sprite playing at a place, each with the figures of its arm of…
        prints.rs             A ring of eight sprites that spreads over the ground from round the one struck,…
        puffs.rs              `EF_PEONG` (411): the puff of one who lands from a leap (Lope
        rays.rs               Lines of light that go out from the one an effect is on, or fly in to that one,…
        shields.rs            The effects of walls, shields and what is thrown like one, whose primitives are of…
        songs.rs              The effects of the songs of minstrels and wanderers, each its function of…
        souls.rs              `EF_SOULLINK` (503): the link of a soul, as its arm of `CRagEffect::OnProcess` and…
        sounds.rs             Effects of the client's that are sounds and nothing drawn, each as its arm of…
        spirits.rs            The spirit spheres about a monk and the coins about a gunslinger
        sprays.rs             The effects of things struck or torn out of someone, that fly every way from a place
        sprites.rs            What of the client's effects is made of sprites (`PP_3DPARTICLE`, 5, and…
        struck.rs             What follows a blow on the one struck: effects only the client's tables of what…
        trails.rs             Lights that leave a trail of sparks behind them
        walls.rs              Walls of light round the one an effect is on, each with the figures of its function…
        waves.rs              Walls of light that fly from the one who strikes towards the one struck, one after…
        weather.rs            The weather: effects that fall over the whole view round the player's character,…
        winds.rs              Winds and bands that go round the one an effect is on, of the kinds the client…
      pursuit.rs            What the player's own character does by itself between the player's presses
      pvp.rs                Fighting other players: which map the character is on (`crate::pvp::Zone`) and who…
      quads.rs              The effects' quads of one frame, as the renderer takes them
      quests.rs             The marks over NPCs that have something for the character
      scene.rs              The map being shown and who is on it (`Scene`), and what is asked of it everywhere
      scenery.rs            What surrounds the player on a map besides its ground and its models…
        placed.rs             One frame of what surrounds the player: the makers of the effects a map places…
      scope.rs              The circles on the ground under the places skills are being cast on (`crate::scope`,…
      scripted.rs           What an NPC's script shows in the world: the bar that fills over someone (`progressbar`
      session.rs            Entering and leaving a map, and what the server says of the player's own character
      shaders.rs            The WGSL programs the world is drawn with
      signs.rs              The signs that stand over heads: of a chat room over the one it belongs to, of a shop…
      skill_units.rs        The units skills leave on the ground (`crate::skill_units`, the client's `CSkill`)…
        traps.rs              The traps standing on the map (`crate::skill_units::traps`, the client's…
      spirits.rs            The spirit spheres about a monk and the coins about a gunslinger, as the server tells…
      state.rs              What the world keeps between frames: the map and who is on it, what the player is…
      state_effects.rs      The effects that the states of someone begin and end on it (`crate::state_effects`)
      written.rs            The words of units on the ground: what a talkie box says when someone steps on it,…
gateway/src/
  accounts.rs           Accounts, for players who have none or lost their password
    recover.rs            A lost password: `POST /api/recover` e-mails a link to set a new one (the letter, and…
    rules.rs              What an account's ID, password and e-mail address may be
    tests.rs              tests
  bridge.rs             `GET /ws/<port>`: a WebSocket bridged to a TCP connection to the game server, because…
    guard.rs              The guard on the login server's port: failed logins counted per address and per account…
    lag.rs                The bridge's debug delay (`--lag`, `--jitter`)
  config.rs             What the gateway is started with: its options, each with what it is when not given, and…
  grf.rs                Reader for the client's GRF archives: version 0x200 ("Master of Magic") and 0x300 ("Event…
  images.rs             `GET /image/<client path>`: a picture out of the GRF as a PNG, for the parts of the…
  lib.rs                The pieces of the roweb gateway, as a library so they can be tested
  limiter.rs            Counting attempts per key (an address, an account name) inside a sliding window of time
  main.rs               roweb gateway: the one native process a browser client needs beside rAthena
  mapcache.rs           The server's map cache, made from the client's own maps
  public.rs             What the gateway does when it is reached from outside
  routes.rs             What the gateway answers out of the client's files
  tables.rs             The client's own tables, read out of its files for the browser
    ai.rs                 The scripts that move a homunculus and a mercenary
    aura.rs               What the client's settings of its service say of the aura of the highest level
    cards.rs              What the client's item naming reads beside the item table (`LoadItemNameTables`,…
    commands.rs           The chat commands: the help the client prints for them, and what its options say when a…
    figures.rs            What the client's Lua says of how characters, monsters and NPCs are drawn
    guide.rs              The Adventure Guide: its pages, and the tips its quests are shown by
      tests.rs              tests
    interface.rs          The tables of the interface: the emotions, the keys, the options of the Game Settings…
    items.rs              The items: names, pictures and descriptions, in the client's own words, in English and…
      tests.rs              tests
    job_codes.json        read by guide.rs
    jobs.rs               The jobs: what each is called, the sprite files of NPCs and monsters, the name a…
    maps.rs               What the client knows of the maps: their names and signs, and who stands where in the…
    pets.rs               The pets: the picture of each in its window, and the actions one is drawn with by what…
    quests.rs             The quests: what the journal writes of each, what the client recommends, and how it…
    scenery.rs            What surrounds a map: the sky the client draws behind it
    skills.rs             The skills: where they stand in the skill window, what they are called, what the client…
    statuses.rs           The states a character is in (its buffs and debuffs)
    tests.rs              tests
    tips.rs               The tip box: the client's tips, each with its title and its pages
    worldmap.rs           The world map: the client's worlds, the picture of each, and where each map lies on it
gateway/tests/
  accounts.rs           Registration and password recovery against the real services of `server/docker-compose.yml`
  bridge.rs             The WebSocket-to-TCP bridge, driven the way a browser drives it
  lag.rs                The bridge's debug delay (`bridge::Lag`, the gateway's `--lag` and `--jitter`)
client/web/index.html   the page: its elements and its one style sheet (a layer of the windows has one rule)
client/dist/            what build-web.sh makes of the client: the gateway serves it (not committed)
server/                 a Docker setup of rAthena and MariaDB for development: docker-compose.yml, Dockerfile,
                        conf/ (the server's settings, groups.yml, the test accounts of accounts.sql),
                        npc/ (the classic scripts put right for the client's maps: tools/client-tables/
                        classic_overlay.py writes them), mapcache/ and public/ (made locally, not committed)
translations/           GLOSSARY.md (terms, so that translators agree); src/ (English sources of the
                        translated batches); ru/ (Russian: items-*.json, skills-*.json)
docs/                   DONE.md (what is implemented, where from, how far checked), DEBTS.md (deviations
                        from the Windows client, bugs known, refactoring left), a picture for the README
ARCHITECTURE.md  CLAUDE.md  README.md  BACKLOG.md
                        this map and the rules; the rules in short for agents; what the project is and how
                        to run it; what waits for Renewal
build-web.sh            builds the client for the browser into client/dist
dev.sh                  the same again whenever a file of the client is saved; a page opened with ?dev reloads
check.sh                what a change passes before it is committed: tests, the browser build, tools/limits.py
rules.sh                switches the local server between the classic rules and Renewal's
public.sh               opens the local server to invited friends: the invitation word, a Cloudflare tunnel
tools/
  limits.py             rules 4, 10 and 12 as far as a script can hold them, and that this map is the tree's
  directory_map.py      writes this map
  split_module.py       moves functions of a file that grew too large into child modules
  packet-names.py       writes out what rAthena's headers call each packet (client/src/net/packet_names.txt)
  packet-lengths/       builds the table of packet lengths from rAthena's sources
  client-tables/        writes out tables compiled into the Windows client, each script one or two files:
                        skill_effects.py (client/src/effect_tables/), statuses.py (statuses.txt), commands.py
                        (commands.txt), job_bases.py (web/data/job_bases.txt), job_mounts.py
                        (job_mounts.txt), ground_scopes.py (effect_tables/ground_scopes.txt); and for the
                        server: classic_overlay.py (server/npc/roweb_classic*), placement_check.py
```
<!-- end of the directory map -->

## How things flow

**Something happens on the page.** `app/wiring.rs` turns the browser's event into an `Action` and calls
`App::handle` (`app/actions.rs`). Nothing else listens to the page, except the world's own mouse handling in
`world/input.rs`, which reports back through `world::Hooks` — again as Actions.

**The server says something.** `socket.rs` → `Action::Socket` → `App::received` (`app/connection.rs`) frames the
bytes with `net::mod` and hands each packet to `App::packet` (`app/incoming.rs`). The map server's packets go
through the `*_packet` functions of `app/incoming/`, a system each, in the order of the list `MAP_SERVER`;
each returns whether the packet was its own (a reader that only overhears a packet of another system gives
back `false`: the table over `MAP_SERVER` says who overhears what, and the order of the list holds it). A
handler does three things, in this order: parse (`net::*`), update the App's state, and tell the world
(`world::*`) and the windows: `self.redraw(&[&wnd::X])` draws the named windows anew, `render()` writes the
whole screen. Three shorthands over `redraw` remain: `refresh_values` (the windows of the character's values),
`refresh_items` (those that show items and skills) and `refresh_chat`.

**The world.** `world.rs` holds one `State` (thread-local): the GPU, the `Scene` of the current map, and what is
going on in it. The app only *tells* it things, and asks it things, through what `web/world.rs` re-exports
(its `pub use` lines): everything public of `session.rs`, `actors.rs`, `combat.rs`, `launch.rs` and
`players.rs`, and the functions it names one by one of the other children (`state_effects`, `skill_units`,
`signs`, `guild`, `ai`, `pvp`, `quests`, `scenery::fog`…), with `Hooks` and `Options` of `hooks.rs`. Nothing
else of the world is seen from outside, and nobody reaches into the state. `world/frame.rs` runs once per
animation frame.

**The windows.** They are HTML strings built by `hud.rs` and `hud/*.rs` from the App's state and from the
original client's bitmaps, placed at the original's coordinates. `App::render` writes the whole page. The game's
windows are one list (`hud/windows.rs`): each entry has its element, when it is open, where it stands, what
draws it, its part in the order and its layer (how it lies among the others when it has no part in the order:
under the windows, over them as one that asks something, over those as a menu); the screen is written from
the list, `App::redraw` draws some of them anew
where they are (typing and scrolling survive; a closed one is emptied and hidden), and the order and Escape are
read from it. What can be pressed is a value (`crate::press`): its element carries the value's name
(`data-click`, `data-use` for a double click, `data-drag`, `data-drop`, `data-menu`, `data-submit`), written
only by `web/parts.rs`, and the app reads the value back and matches on it. `data-info` / `data-skill` (right
click) carry numbers.

**One system, one name.** A system of the game (pets, shops, skills…) is the same word in every layer:
`pets.rs` (its rules that need no page), `net/pets.rs` (its packets), `press/pets.rs` (what can be pressed in
it), `app/incoming/pets.rs` with `pets_packet` (what the server says of it), `app/clicks/pets.rs` with
`pets_pressed` (what its buttons do), `app/state/pets.rs` and `App::pets` (what is remembered of it),
`hud/pets.rs` (its window) and `hud/windows/pets.rs` (the window's entry). What belongs to no system (the
connection, the account, the screen shown) is a plain field of `App`.

The exceptions that remain, so that nobody looks in vain:
- the character's own values ("status"): the packets are in `net/packets/zone.rs` with the other basics of the
  map server, the windows are two (`hud/basic.rs`, `hud/status.rs`), and the group is `Hero`, `App::hero`
  (`app/state/status.rs`);
- the inventory's group is `Bag`, `App::bag` (`app/state/inventory.rs`), and the Equipment window is a file of
  its own (`hud/equipment.rs`);
- the packets of the skills known and of the hotkey bar are children of the inventory's
  (`net/inventory/skills.rs`, `net/inventory/hotkeys.rs`); `net/skills.rs` is the casting and landing of
  skills, read by `app/incoming/combat.rs`; the list of skills known is read by `app/incoming/status.rs`;
- an NPC's window is `Dialog`, `App::dialog` (`app/state/talk.rs`), a plain field and no group; the keys are
  `Hotkeys`, `App::keys`, kept in `app/state/settings.rs`; the scripts of a homunculus are `Scripts`, `App::ai`;
- `press/general.rs` and `app/clicks/general.rs` have the presses of no one window; `press/things.rs` what is
  double-clicked, dragged, dropped on, opened a menu on and sent as a form.

## The rules every change follows

These hold for everyone who writes here, people and agents alike. A change that breaks one is not done.

1. **Find the home before writing.** A thing belongs to a *system* (pets, items, skills, the guide, the
   settings…) and to a *layer*. Its code goes into that system's file of that layer; if the system is new, it
   gets its own files under the same name in every layer it touches (see "One system, one name"). Nothing is
   added to a file because it happens to be open.
2. **A layer knows only the layers under it.** `net/` and the formats never touch the page. The world never
   reaches into the app: it reports through `world::Hooks`. The app never reaches into the world's state: it
   calls what `web/world.rs` re-exports, and a function the app needs of the world is added to those `pub use`
   lines. Windows (`hud/`) only read: the App's state, the tables (`data::…`), and of the world what it
   re-exports for asking.
3. **Parents hold what is shared, children hold the behaviour.** `app.rs`, `world.rs`, `hud.rs`, `data.rs` and
   the gateway's `tables.rs` have the shared helpers and the list of children; a child starts with
   `use super::*;` and is about one thing. A type lives with the child that owns it (`Scene` in
   `world/scene.rs`, `State` in `world/state.rs`, a system's group in `app/state/<system>.rs`, `Window` in
   `hud/windows.rs`); `App` itself is in `app.rs`.
4. **No file grows past about 500 lines.** One that does is cut by responsibility *in the same change* — not
   later.
5. **One state, owned in one place.** What the game remembers is a field of its system's group in `App`
   (the groups are in `app/state/<system>.rs`, `App` in `app.rs`), or of the world's `State`
   (`world/state.rs`). No second copy, no hidden global except: the tables in `data/`; what `figure.rs` has
   read and painted and what `sound.rs` plays; and a pointer's gesture while it lasts and what it left on the
   page: the window being dragged, where each was dragged to and whether windows dock (`app/page.rs`), the
   thing being dragged (`app/dragging.rs`).
6. **The original client is the reference, and the comment says where from.** Behaviour, sizes and texts come
   from the client's code and files; the comment names the function or file (`UIItemIdentifyWnd`,
   `Guide_Monster.lub`). What was not found there, or was decided here against it, says so in the comment.
7. **Data comes from the client's files through the gateway**, each kind with a file in `gateway/src/tables/`
   and its mirror in `client/src/web/data/`. No game data is typed into the code or committed, except small
   tables the client has compiled into it, which are written out with a note of their origin.
8. **What can be tested without a browser is.** A parser in `net/`, a rule in a native module
   (`client/src/<system>.rs`), a format or a table of the gateway gets a unit test with the change; a rule
   found under `web/` that needs no page is moved out into its native module and tested there. What only shows
   on the page is looked at in the game before it is called done, and the report says what was and was not
   looked at.
9. **A change leaves the documents true.** A new file begins with a `//!` that says what it is, and
   `python3 tools/directory_map.py --write` puts it into the directory map above (`check.sh` refuses a map that
   is not the tree's); a new kind of thing is added to "How to…"; a deviation from the Windows client, or a bug
   left, to `docs/DEBTS.md`; a system made or looked at in the game, to `docs/DONE.md`.
10. **No function grows past about 100 lines**, unless all it does is hand work out by kind (a `match` over
    packets or actions, whose arms are a line or two each). A longer one is cut into its steps, each a function
    named for what it does: `world/frame.rs` is the pattern, a list of steps and nothing else.
11. **Nothing is written twice.** What two places need is a function both call. Before writing a helper, look
    for one. Those in wide use, by layer:
    - *no browser*: `net::bytes` for reading a number, a text or a name out of a packet and
      `net::bytes::Writer` for writing one (`net::bytes::testing` for their tests); `crate::bytes::Reader` for
      a file of the game; `crate::vector` for vectors of three; `crate::tables` for a table of text;
      `crate::markup` for `^RRGGBB` and the tags in a text; `crate::kept` for what the browser keeps;
      `crate::numbers::{grouped, typed, filled, price_colour}` for numbers written, typed and filled into a
      line; `crate::jobs` for what a job is by its number; `crate::maps::bare` for a map's name without its
      ending; `crate::minimap` for places on the minimap; `crate::said` for the kind and the colour of a line
      of the chat; `crate::font` for the client's sizes of text.
    - *primitives* (`prim/`): for a kind of `prim/tei/`, `Part::waits`, `wander`, `fly_level`, `trail`
      (`tei/part.rs`), `Rect::band` and `Part::facing` (`tei/rect.rs`); `Slot::waits` for a kind of
      `prim/slots.rs`; `prim::testing` for what their tests share.
    - *all of `web/`* (`web/mod.rs`): `document()` and `by_id()` for the page, `now()` for the one clock of
      the page, `korean` for a text of the client's files, `image_url` and `asset_url` for the address of a
      picture or a file; `sound::at` for a sound heard from a place; `data::item_title`, `data::skill_name`,
      `data::map_name` for the one way a name is written.
    - *the world* (`web/world/`): `Scene::place_of` for where someone is, `Scene::mover` / `mover_mut` (the
      player's own for `None`) and `one` / `one_mut` (by ID, told the player's own) for someone,
      `Scene::view` for the camera's view; `State::forget_fights` for what a map left behind;
      `Gpu::sprite_bind` and `effect_binds` for a picture on the GPU; `Quads` and `nearer` for an effect's
      quads; `lot` (`world/math.rs`) for a lot. An effect by its number is `world::burst` (`launch.rs`),
      whatever it is made of: under it `launch::files` begins one played from files and `prims::begin`
      (`prims/burst.rs`) one drawn by code (`Burst::begin` is another thing: the effect told, before its first
      tick, which way its actor faces). In a file of effects: `Course::sound` and `Course::starts` for its
      sounds, `Touch::flagged` and `flashes` for what it does to its actor, `prims/launched.rs` for a
      primitive several effects launch alike, and `prims.rs`'s own `lot` (the same lot as a figure of a
      primitive); a part's own lot is `Part::lot`. `world::sign` puts a sign over someone.
    - *what windows are made of*: `parts::{at, picture, pressed, used, dragged, dropped, menued, sent}`
      (`web/parts.rs`); `hud::{text, text_in, Font}` for a text in the client's own size; `pictured` for a
      button of three pictures, `pictured_by` with a `Convention` for one whose files are named by a rule, and
      over them `picture_press`, `submit_press`, `press_button` (`hud/buttons.rs`); `title_bar`, `system_box`
      (`hud/frames.rs`); `App::item_icon`, `icon_of`, `skill_icon` (`hud/items.rs`); `message_box`,
      `menu_row`, `pick_list`, `App::count_box` and `count_box_of` (`hud/boxes.rs`); `list_frame`, `list_of`,
      `item_row` (`hud/lists.rs`); `side_tabs` (`hud/tabs.rs`); the other pieces in `hud/{checks, gauges,
      fields, colours}.rs`; `escape` (`app/page.rs`) for a text put into HTML.
    - *windows as entries* (`hud/windows.rs`): `Window::usual`, `Window::pick`, `Window::question`,
      `Window::count`, `Window::menu` for the usual shapes of an entry, `Window::field` for the name of a
      box's field.
    - *the app*: `App::send` and `App::send_on_map` for packets; `App::words` for a line of the client's
      message table, `chat_line` / `chat_line_of` for a line of the chat and `App::say_filled` for a line of
      the table said with its blanks filled; `App::open_at`, `App::on_stage`, `App::counted` and
      `close_menus` (`app/windows.rs`); `App::own_account`.
    - *the gateway* (`gateway/src/tables.rs`): `state`, `exec`, `whole`, `text`, `text_only`, `lines`,
      `numbered`, `by_number`, `latin`, `hash_fields`.
12. **Nothing on the page is named by hand.** What can be pressed is a value of `crate::press`, drawn through
    `web/parts.rs` (`pressed`, `used`, `dragged`, `dropped`, `menued`, `sent`) and matched as a value where
    it is handled, with no arm for "anything else": a press nobody handles, or a name nobody draws, does not
    compile. A window is an entry of `hud/windows.rs`, and the page's style sheet (`client/web/index.html`)
    never names one: how a window lies over the others is its entry's `layer`, for which the sheet has one
    rule a layer. The field of a box that asks how many is named from its window (`Window::field`).
    `tools/limits.py` refuses a `data-click`, `data-use`, `data-drag` or `data-submit` written out anywhere but
    in `web/parts.rs`, and a rule of the style sheet that gives a window's ID a `z-index`. Not held yet: other
    fields that are typed into are still named by hand on both sides (`field("exchange-zeny")`,
    `field("quest-search")`, the account's forms…), and no script refuses that.

Before finishing, and before any commit: `./check.sh`. It runs the tests, the build for the browser with no
warnings, and `tools/limits.py`, which holds rules 4 and 10 and what a script can hold of rule 12. Its
`DISPATCHERS`, for a function over 100 lines that is a plain dispatcher, is empty: no function is that long
now. A change is not committed until it ends with "all checks passed".

## How to…

**…add a table from the client's files**: read it in `gateway/src/tables/<kind>.rs` and add it to
`tables::for_browser`; on the client, keep and read it in `web/data/<kind>.rs` (`read`, called from
`data::load`) and bump the number in `data::load`'s URL (`/tables.json?v=N`) so browsers fetch the tables
anew. What several files of `gateway/src/tables/` need is in `tables.rs` (rule 11).

**…handle a new packet from the map server**
1. Its length must be in `client/src/net/packet_lengths.txt` (generated) or `packet_lengths_more.txt` (by hand).
2. Add a parser with a unit test in the system's file of `client/src/net/`, reading with `net::bytes`. No
   packet is read or built by hand under `web/` (so it is now: no `from_le_bytes` and no `packet.get(` there).
3. Add an arm to the system's reader, `<file>_packet` in `client/src/web/app/incoming/<file>.rs`. A new
   system's reader is one more name in `MAP_SERVER` (`app/incoming.rs`; the list's length is written in its
   type). Its place in the list matters when it overhears a packet of another system, or another overhears
   one of its own: read the table over the list and add a row to it.
4. If the world must show it, call a function `web/world.rs` re-exports, or add one to the fitting child of
   `world/` and to those `pub use` lines (an effect by its number is `world::burst`, whatever the effect is
   made of).

**…send a packet**: a builder function next to the parsers in `net/`, written with `net::bytes::Writer`,
called from `app/clicks/*.rs`, `app/keys.rs` or `app/actions/*.rs` as `self.send_on_map(&…)`. A new builder
needs its line in `net/golden.rs` (the test `every_builder_is_here` fails until it has one) and in
`golden.txt`, written as below.

**…change a packet the client sends**: change its builder in `net/`. The test in `net/golden.rs` builds every
packet with fixed arguments and compares it byte for byte with `net/golden.txt`, so it fails now, as it
should. When the change is meant, write the file anew with `ROWEB_GOLDEN=write cargo test -p roweb-client
golden` and read what changed in it: only the lines of that builder may differ. `golden.txt` is never written
anew for any other reason.

**…add a rule that needs no page** (what a number typed means, which rows a list shows, when a button is
dim…): a function in the system's native module, `client/src/<system>.rs` (a new one is declared in
`client/src/lib.rs`), with its test beside it; the window (`hud/<system>.rs`) or the app
(`app/clicks/<system>.rs`, `app/incoming/<system>.rs`) calls it as `crate::<system>::…`. A rule is named where
it lives: no `pub use` brings it under another path.

**…say something in the chat**: `self.chat_line(words, colour)`, or `chat_line_of(kind, words, colour)` for a
line of a kind the chat's tabs tell apart (`app/chat.rs`); a line of the client's message table with blanks
(`%s`, `%d`) is `say_filled(kind, line, &[…], colour)`. The kinds and the colours are `crate::said` (`RED`,
`YELLOW`, `CYAN`…). A colour only one line has is written where the line is said.

**…add or change a window**: what is in it in `hud/<system>.rs`; an entry for it in its system's file of
`hud/windows/` and its name in `ALL` of `hud/windows.rs` (its element, when it is open, where it stands, what
draws it, what Escape does to it if it is to lie in the order, and its `layer`: `Layer::Usual` for a plain
window) — nothing else has to know of it, the style sheet least of all; its state as a field of its system's
group in `App` (`app/state/<system>.rs`: `Hero`, `Bag`, `Skills`, `Chat`, `Dialog` are in
`state/{status,inventory,skills,chat,talk}.rs`; a new system gets a file and a group of its own); its entry is
written with `Window::usual` if it is a plain window in the order; its texts through `i18n::tr`, or
`App::words` for a line of the client's message table. To show a change in it,
`self.redraw(&[&wnd::<WINDOW>])`; `render()` writes the whole screen.

**…add a window that asks something** (it lies over the windows that are in the order): its entry has
`layer: Layer::Asks`. Four kinds are written once and only named by a system:
- *a question with OK and Cancel*: the entry is `Window::question("<system>-asks", open, App::<system>_question)`;
  the drawing is `message_box(Asked::Offer or Asked::Sure, &words, Press::Yes, Press::No)` (`hud/boxes.rs`;
  the two layouts are the client's two); what is asked is an `Option<…Ask>` in the system's group, and the
  two presses are arms of `app/clicks/<system>.rs`.
- *how many of a thing*: the entry is `Window::count("<system>-count", open, App::<system>_count_window)`; the
  drawing is `self.count_box(&windows::<WINDOW>, counting, Form::…, Press::…)` with `counting` the place in the
  inventory and how many there are, kept in the system's group (or `count_box_of(..)` under it, for a box
  told its thing's name and number otherwise); the form is a variant of `press::things::Form` whose arm in
  `app/actions/forms.rs` calls the system's `<system>_counted`. There are two rules for the number typed,
  both tested: throwing away, the exchange and the mail read it with `Self::counted(&wnd::<WINDOW>, most)`
  (`crate::inventory::count_typed`); the shops, the storage, the cart and vending read the field
  (`field(&wnd::<WINDOW>.field())`) with `crate::shops::count_asked(typed, most, Asked::…)`, whose limits
  go by what is asked (buying, moving, storing).
- *a row to pick of a list with pictures*: the entry is `Window::pick(..)`, the drawing `pick_list(..)`
  (`hud/boxes.rs`).
- *a menu opened with the right button*: what it opens on is a variant of `press::things::Menu`, drawn with
  `menued(..)`; it arrives as `Action::MenuOn(what, at)` (`app/actions.rs`), where the system keeps
  `Some((Self::on_stage(at), what))` in its group and shows the menu (`self.open_at(&wnd::<MENU>,
  &wnd::menu_style(at))`, or `render()`); the entry is `Window::menu("<system>-menu", open, |app, _|
  menu_frame(<where it was opened>), App::<system>_menu)`; its rows are `menu_row(&words, Press::Do(entry))`
  with `MENU_BOUNDARY` between groups; a `close_<system>_menu` that puts it away is called from
  `App::close_menus` (`app/windows.rs`), so a press elsewhere closes it.

**…add a button, a tab, a row that is pressed**: a variant in the system's list in `client/src/press/<system>.rs`
(name it as nothing else is named: a test holds that); draw it with `pressed(Press::…)` or a helper that takes a
press (`picture_press`, `press_button`); the compiler then asks for its arm in the system's
`<system>_pressed` (`app/clicks/<system>.rs`). What is double-clicked, dragged or sent as a form is in
`press/things.rs`, drawn with `used`, `dragged` and `sent`: a double click arrives in `App::used`
(`app/actions/bag.rs`), a form in `app/actions/forms.rs`. So is what the right button opens a menu on
(`menued`, which arrives as `Action::MenuOn` with the place, in `App::menu_on` of the same `bag.rs`). A
window that opens where the pointer is, or at a place decided as it opens, is put there with `App::open_at`
(`app/windows.rs`): `redraw` alone leaves a window where the screen was last written with it.

**…let a window take the things dragged**: a variant of `press::things::Drop`, written on the window (or on a
part of it) with `dropped(Drop::…)`; what is let go over it arrives as `Action::LetGo(dragged, onto)` in
`App::let_go` (`app/actions/bag.rs`), which hands it to the window's system, each in its file of
`app/clicks/` (`shop_let_go`, `storage_let_go`, `tab_let_go` of the inventory, `cart_let_go`,
`vending_let_go`). A thing taken out of a window is a variant of `Drag`, drawn with `dragged(..)`.

**…add a row to the menu on another player**: an `Entry` in `client/src/press/player_menu.rs`, its place and
words in `hud/player_menu.rs` (the client's order: `CGameMode::OpenActorMenu`), and what it does in
`app/clicks/player_menu.rs`.

**…add an effect**: one of the client's effects is shown by its number, `world::burst(effect, on_whom, blow,
delay)` (`world/launch.rs`), whatever it is made of; a `.str` file by its name alone is `world::effect("name.str",
on_whom)`. Which of the two an effect is, is asked in one place, `crate::effect::played`:
- *a plain `.str` file* needs no code: the client's table has it (`effect_tables/effect_files.txt`, with its
  sound in `effect_sounds.txt`).
- *an effect the client draws by code* is made of the client's primitives (`world/effects.rs` beside them
  has a few drawn as plain geometry: the rings of a cast, the mark on its target, a portal's rings, the
  flash of a blow; nothing new goes there): a function in the file of `world/prims/` whose effects are of
  its sort (hits, bolts, walls of
  light…; no file is named for a worker or for a kind of primitive), named by that file's `known` (its lifetime
  and first sound: also a sound the client's handler starts beside the effect) and `run`; one that the striker
  launches towards the one struck is named in `thrown` there, and is then told that one's place
  (`world::Blow`, `Course`); the pictures it names go into the file's `PICTURES`. The sounds it starts later
  are a table of the file's, each with its tick (`fn sounds`, played with `course.starts(..)`), or
  `course.sound(..)` for one picked as it goes. What several files launch alike is in `prims/launched.rs`,
  written once: look there before writing a primitive's figures again. A new file of effects has those
  functions and a `FILE` of them, and is one name in `files!` of `world/prims.rs`: the loader and everything
  else are built from that list.
Run `tools/client-tables/skill_effects.py ../rodecomp client/src/effect_tables` after either (it reads the
numbers every `known` and `effect::played::played` names), and `./check.sh`.

**…add a primitive kind**: its figures are `Prim`'s (`prim.rs`), its name a variant of `Shape`, or of `Tei`
(`prim/tei.rs`) for a kind the client builds corner by corner out of its parts. A `Tei` kind is a file in
`prim/tei/` with two functions, each its function of the client line for line, and its tests beside them. One
is a tick of its life: `Prim::tick_parts` (`prim/tei.rs`) first asks `Prim::tick_whole`, which names the kinds
whose parts are ticked together, one looking at another, and otherwise calls `Part::tick_as` for every living
part, which names the kinds whose parts live each by itself; a new kind is an arm of one of the two. The
other is its rectangles, an arm of `Prim::rects` (`prim/tei/drawn.rs`).
What kinds share is taken from `tei/part.rs` (`waits`, `under_a_round`, `over_once_gone`, `wander`,
`fly_level`, `steer`, `follow`, `trail`, `Prim::head_and_tail`) and `tei/rect.rs` (`Rect::new`, `Rect::piece`,
`Rect::band` with `edges`, `to_screen`, `Part::facing`); a motion or a way of building rectangles that a
second kind needs is moved there, not written again.

**…add an effect a map places, or a kind of weather**: an effect of a map's `.rsw` is launched at its place
by its maker (`world/scenery/placed.rs`) whatever it is made of: one played from files needs nothing, one
drawn by code is an arm of `world/prims/places.rs` (it is told the map's four figures: `Course::figures`).
What falls over the whole view is an arm of `world/prims/weather.rs`, launched on the player's own character by
a server's map flag (`world::burst`) or by the client's own table (`weather` in `world/scenery.rs`).

**…add an effect that changes its actor** (colours it, lights or outlines it, turns it, holds it in a
picture): the effect says at each tick what it does, in `course.touch` (`crate::body::Touch`:
`flagged(flag::ADDS, r, g, b)`, `flashes(tick, colour)`, `phased`, `tinted`, `turn`); nothing else has to
know. A flag or a count the client has and `crate::body` does not yet is added there (`flag`, `Body::touch`,
and what is drawn of it in `body/drawn.rs`) with a test in `body/tests.rs`. An effect that draws nothing and
only does this is an arm of `world/prims/bodies.rs`. The colour of a body is kept in `Body` alone: a state's
through `Body::stated`, an effect's through its `Touch`.

**…add an effect begun by a state**: name it in `crate::state_effects::changed` for the state (the client's
`AM_STATUS_CHANGE`), in `seen` if a bit of the third option shows it on one coming into sight, in
`reapplied` if `CPlayer::ReapplySkillEffects` launches it anew on the player's own character (the app calls
`world::own_states_again` as the character arrives: `app/actions/travel.rs`), each with its test; the effect
itself is added as any other. One that a bit of the effect state keeps going is in `state_effects/kept.rs`.

**…add a played file that flies** (a file a class of the client's own moves, turns, delays or begins anew:
`CEZ2STRParticle`'s `m_angle`, `m_delay`, `m_loop`, a place set every tick): an arm of `played` in
`client/src/effect/played.rs`: its files, each with how it is carried (`crate::effect::carried`, where a new
way or turn gets its rule and its test), and its sounds with their ticks; a test beside it. Then run
`skill_effects.py`, which takes the effect out of the table of plain files. The world plays what `played`
gives (`launch::files`, `frame/effects.rs`); nothing there has to change for a new arm.

A primitive whose drawing function in the client launches another effect where it is says so in `Prim::asks`
(`prim/tei/launchers.rs`): the world launches what it asks for once a tick, on the same one, and tells it the
place (`Asked::launches`, `Burst::tell`). An effect on a place of the map, not on someone, is launched on a
spot made for it (`spot` in `world/skill_units.rs`, the client's `LaunchDummyEffect`).

**…put a sign over someone's head** (a chat room's, a shop's: the client's windows an actor carries): what
the sign is made of is HTML its system writes in `hud/<system>.rs`, 140 by 34 (`world::SIGN`), with what a
double click on it does as a `Use` (`used(..)`); the app hands it to the world with `world::sign(owner, own,
Board::…, Some(html))` and takes it away with `None`. The world keeps it over its owner frame after frame
(`world/signs.rs`) and drops it when the owner goes out of sight. A new kind of sign is a variant of `Board`
with its height over the head.

**…translate**: interface texts are in `client/src/i18n.rs`. Items and skills are JSON files in
`translations/ru/`; the gateway reads every `items-*.json` and `skills-*.json` at start.

## Conventions

- **The original client is the reference.** Sizes, coordinates, timings, file names and behaviour are taken from
  the original client (its decompilation and its data), and the comment says from where (`UIItemWnd`,
  `CArrowEffect`…). What was *not* found there and had to be decided here says so too.
- Comments say what a thing is and why, in plain words; packet layouts are written out where they are parsed.
- A parent module (`app.rs`, `world.rs`, `hud.rs`, `data.rs`) has what its children share and the list of
  them; a type lives with the child that owns it (`world/scene.rs`, `world/state.rs`,
  `app/state/<system>.rs`), behaviour in the children; a child starts with `use super::*;`.
- A file that passes ~500 lines is split by responsibility (`tools/split_module.py` does the moving). The
  tests of a file that has many are in `<file>/tests.rs`.
- A handler is named for its file: `<file>_packet` in `app/incoming/<file>.rs`, `<file>_pressed` in
  `app/clicks/<file>.rs`.
- No game data is typed into the code or committed, except small tables the client has compiled into it,
  written out with a note of their origin (rule 7).

## Build, run, test

```
cargo test -p roweb-client                       # formats, protocol and the rules (no browser needed)
cargo test                                       # the same and the gateway's tests
./build-web.sh                                   # client -> client/dist (wasm + page)
./dev.sh                                         # the same, again at every save; open the page with ?dev
cargo run --release -p roweb-gateway             # serves http://localhost:8080
docker compose -f server/docker-compose.yml up   # a local rAthena
./rules.sh classic|renewal                       # which rules the local server plays by (rebuilds, restarts it)
./check.sh                                       # before a commit: tests, the browser build, tools/limits.py
```

`./check.sh` must end with "all checks passed". `./rules.sh` writes `server/.env`; the gateway reads the same
file for the item database, so restart it afterwards.

The gateway's debug commands are `enum Command` in `gateway/src/config.rs`, given after its options in place
of serving: `stats` (how many files the archives hold), `ls <words>` (the files whose path has them),
`lua <file>,<file>` (runs datainfo files and lists the tables they leave), `dump <file>,<file> <global>`
(prints one of those tables as JSON), `mapcache <file>` (writes the server's map cache from the client's
maps), `items <id>,<id>` (what the item table says of them), `tables` (what the browser is given), `chat-tabs`
(the client's table of the chat's tabs).

Test accounts for the local server are in `server/conf/accounts.sql`.

## Known debts

The structural ones, as they stand now:

- **Seven boxes that ask how many**, where the client has one window: seven states (`Bag::throwing` and the
  `counting` of the exchange, the mail, the shops, the storage, the cart and vending) and seven entries
  written with `Window::count`. The box is drawn once (`count_box`, `count_box_of`), but the number is read by
  two rules (`crate::inventory::count_typed`, `crate::shops::count_asked`) and each system keeps its own.
- **Lines of the message table said beside `App::words`**: 22 calls of `data::message(..)` in 16 files under
  `web/` outside `App::words` and `say_filled`; a line said that way is not translated unless the caller sees
  to it. They are to be looked at one by one.
- **Overhearing depends on an order.** A reader of `app/incoming/` that overhears a packet of another system
  must stand before that system's in `MAP_SERVER` (`app/incoming.rs`); only the table written over the list
  holds that, no test and no type.
- **Three clocks in `app/`**: `page::now()` (the page's `web::now()`, as the servers count it),
  `page::epoch()` (seconds since 1970) and `js_sys::Date::now()` called where it is needed (17 places in
  `app/` and `hud/`). **Several timer drivers**: two intervals and `after(..)` in `app/wiring.rs`, the fade's
  timeout in `app/page.rs`, an interval of its own in `app/actions/taming.rs`; systems that run by time
  (`scripted`, `broadcast`) compare times themselves.
- **Two queues of sounds in the world** (`web/world/state.rs`): `sounds_due`, sounds by name and moment, and
  `heard`, sounds on someone heard from where that one is then (`world/earshot.rs`).
- **`web/world/effects.rs` beside the primitives**: a few effects drawn as plain geometry, not made of
  `crate::prim` as all the others are.
- **Fields named by hand**: only the field of a box that asks how many is named from its window
  (`Window::field`); the others are a string on both sides (rule 12).
- **Windows are strings of HTML**, written whole by their drawing functions when they change: there is a list
  of windows with one interface (`hud/windows.rs`), not a tree of elements compared and patched. By choice:
  the windows are small, and each is placed by the client's own coordinates.
- **Windows and the world's drawing are checked by hand** in the game: only what needs no page (layers 1
  and 2) has tests.

The full list, with the deviations from the Windows client and the bugs known, is `docs/DEBTS.md`; what is
implemented and how far it was checked is `docs/DONE.md`; what waits for Renewal is `BACKLOG.md`.
