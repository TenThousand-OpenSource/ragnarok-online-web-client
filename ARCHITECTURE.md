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
  bytes.rs          reading the game's files: the one `Reader` of little-endian numbers for every format
                    (`take`, `u8`, `u16`, `u32`, `i32`, `f32`, `floats`, `count`, `name`); packets: net/bytes.rs
  vector.rs         vectors of three (`add`, `subtract`, `scale`, `cross`, `dot`, `length`, `normalize`, `along`)
  tables.rs         the shapes the client's text tables come in, each read once: `tabbed` (number, tab,
                    value), `sorted` (lines of one sort), `hexed` (`0xNNNN value`), `hashed` (fields that end
                    with `#`, with `figure` for one of their numbers)
  markup.rs         the marks in the client's texts told from the text: `^RRGGBB` and the tags of links
                    (`pieces`, `Piece`, `uncoloured`)
  kept.rs           what is kept in the browser between visits, as text: lists of pairs and of numbers,
                    parsed and written (`pairs`, `paired`, `numbers`, `numbered`, `entries`)
  minimap.rs        the minimap as rules: where a cell lies on it, the part shown when it is nearer than
                    whole, the steps of its zoom (`on_picture`, `corner`, `spot`, `placed`, `zoomed`)
  shops.rs  inventory.rs  storage.rs  vending.rs  exchange.rs  mail.rs  choosing.rs  skills.rs  pets.rs
  homunculus.rs  mercenary.rs  party.rs  guild.rs  friends.rs  pvp.rs  chat_rooms.rs  family.rs  guide.rs
                    a system's rules that are no packet and need no page, each with its tests (empty until
                    the rules are moved out of net/ and web/ into them)
  sprite.rs         .spr / .act: sprites and their animations (sprite/canvas.rs: a frame drawn into a picture,
                    each layer scaled, mirrored, turned and tinted; sprite/layers.rs: a frame as the client's
                    effects draw it, a quad to a layer; sprite/imf.rs: .imf, which of a body and its head lies
                    over the other in a frame, and the order a character's parts are drawn in)
  pose.rs           how a character is posed, as the client's actors decide it: the action and the frames of
                    a state (a skill used, a dance, a song, a shot readied) or of a monk's combo, the head
                    of a Taekwon in its kicks, the kind a weapon is shown as, when it stands ready, when its
                    weapon and shield show, how small a baby job is, where its head turns when it looks at a
                    place (the world's side: web/world/mover.rs, poses.rs)
  map.rs            .gat / .gnd: walkable cells, ground mesh, path finding
  model.rs          .rsw / .rsm: what stands on a map, and a model's triangles as it stands lit on the map (its
                    matrices in model/matrix.rs; model/rsw.rs: the map's water and waves, its sun, where its
                    models stand and how they play, the effects and sounds it places; model/rsm.rs: a model's
                    parts, the keys of their turning, the model at a frame, the frame an animation is at)
  scenery.rs        what surrounds the player on a map, as rules: reading the client's tables whose fields end
                    with `#`; in scenery/: makers.rs (when a placed effect is launched, how loud a placed sound
                    is), fog.rs (fogparametertable.txt), views.rs (viewpointtable.txt, the camera's easing and
                    the wheel's tilt), dark.rs (the darkening of a night)
  effect.rs         .str: the client's effect files, frame by frame, and where a file's pictures are
                    (effect/carried.rs: a file that a class of the client's carries — moved along a way from
                    one actor to another, turned on the screen, put aside, begun late or anew: the rules of
                    its place, its turn and its frame at a tick; effect/played.rs: which effects are played
                    from files and how — the one file of the client's table, one of several picked by lot,
                    files that are carried — the one answer to "is this effect a file", with its sounds)
  font.rs           the size the client writes a text in, from the family and height its windows ask for
  figures.rs        how the client writes a number in its windows (commas between its thousands, the colour
                    of a price: `grouped`, `price_colour`), reads one typed into them (`typed`), and writes
                    what a line of its message table asks for into the line (`filled`)
  blow.rs           when a blow lands: the frame of the swing and the time to it (the client's timings); in
                    blow/: landing.rs (the hits a blow or a skill is made of, when each lands and as what),
                    signs.rs (what a hit shows over whom, in which colour, and what it sounds like: by the
                    weapon, by what a struck job wears), flight.rs (how the numbers and words of a fight fly:
                    place, size and fading of each of the client's kinds)
  earshot.rs        how loud a sound of the world is by its distance (nothing from 250 units, whole within 40;
                    `loudness_within` for a sound with distances of its own),
                    and how fast the music fades (played by web/sound.rs, placed by web/world/earshot.rs)
  names.rs          the name under someone the pointer is on: its colour (an NPC's, a monster's by its level), a
                    title before it, the mark of a boss or a group beside it (web/world/names.rs)
  server_time.rs    the server's clock as the client keeps it: set from a time the server tells, read 72
                    milliseconds behind, corrected by the time every walk set out at
  route.rs          a walk as the client shows it: the way with the server's time at which each cell is
                    reached, where the walker is drawn at a time of that clock, a way told of too late, the
                    cell the server names for one to stand on; the waits between the client's own requests
                    to walk (the world's side: web/world/mover.rs, pursuit.rs)
  press.rs          what can be pressed on the page, as values: the one place its text is written and read
                    ── children in press/, a file a system (as app/clicks/), and things.rs: what is
                    double-clicked, dragged, what a thing dragged is let go over (`Drop`), and the forms ──
  scope.rs          the circle on the ground of a skill cast on a place: its size, turn and cells (drawn by
                    web/world/scopes.rs)
  aura.rs           the aura of the highest level: who shows which of its effects (the job, the level, the
                    hiding states, the options `/aura` and `/aura2`; kept by web/world/aura.rs)
  ailments.rs       body and health states (stone, frozen, poisoned, cursed…): the colour they give, the
                    sounds, the marks by the one ailing, the two veils over the screen (drawn by
                    web/world/ailments.rs)
  body.rs           what an effect makes of the body of the one it is on at a tick (`Touch`: its colour, its
                    flags and counts, its turn; `flagged`, `flashes`, `phased`, `tinted` for what many effects
                    do alike) and what a body keeps of it (`Body`): the one place the colour of someone is
                    kept, the one its states gave it among that; the footprints it leaves, the picture it is
                    held in (kept for everyone by web/world/ailments.rs). body/drawn.rs: the drawings of its
                    picture that follow (lit, outlined, shrunk, pressed, spun, shaken, blurred), its weapon
                    drawn by itself, a shadow that lies under every body (drawn by web/world/gpu/draw.rs);
                    body/copies.rs: the copies of a character that an effect leaves standing and the
                    afterimages of one who moves (`EF_MAKEBLUR`): how each fades; tests in body/tests.rs
  quake.rs          the shake of the screen some effects ask for (`MM_QUAKE`; web/world/camera.rs)
  state_effects.rs  the effects a state going on or off begins and ends on the one in it, those the third
                    option of one coming into sight begins, those begun anew on the player's own character
                    as it arrives on a map (`reapplied`), and which effects of a skill wait on another;
                    state_effects/kept.rs: the effects a bit of someone's state keeps going (Sight, Ruwach)
  song_talk.rs      the lines of Frost Joke and Scream: which file, and how a line is drawn from it
  spirits.rs        the spirit spheres of a monk and the coins of a gunslinger: which effect a job shows them
                    as, where each sphere of it begins (kept by web/world/spirits.rs)
  skill_units.rs    the units skills leave on the ground: which effect each kind launches and how often, the
                    effect at a place a skill is used on; skill_units/pulse.rs: Sanctuary's box of light;
                    skill_units/traps.rs: the traps, which are models (which model a kind is, how it stands
                    and springs, what it launches when it has sprung)
                    (kept and drawn by web/world/skill_units.rs)
  stack.rs          the order the game's windows lie in: one opened or pressed in front, Escape closes it
  cursor.rs         the game's mouse pointer: which of its pictures shows when (the client's timings)
  mounts.rs         what a job is drawn as in a state: on its mount, in its machine (job_mounts.txt)
  carts.rs          the merchant's cart behind its owner: which sprite a kind of cart is, how it is pulled on
                    its rope, which way it is seen and which picture of its roll shows; the looks offered for
                    it (drawn by web/world/cart.rs)
  falcon.rs         the hunter's falcon by its owner: which sprite it is, how it hovers and follows, how it
                    flies at what a skill sends it to and comes back, which picture shows (drawn by
                    web/world/falcon.rs)
  jobs.rs           what the client knows of a job by its number alone: whether it has Activity Points, is a
                    baby's, a Super Novice's, of the Taekwon line, a monster, a hireling
  said.rs           the lines of the chat window: the kinds the client gives them (which tabs list them) and
                    the colours it writes them in, each named once (`colour::{parts, css, hex}` for a
                    colour the server names)
  chat_tabs.rs      the chat's tabs as the player has them: what each lists, those taken out into windows of
                    their own, the text they are kept as (the client's `ChatWndInfo_U.lua`); the links to
                    items in a line of chat (`<ITEML>`), read and written
  snap.rs           a window being moved docks to the screen's edges and to the other windows (`/window`)
  screenshot.rs     a picture of the screen: how the client names its file and stamps it
  prim.rs           the client's effect primitives: the figures of one and the list of the kinds
  prim/             life.rs: one tick of a primitive's life, kind by kind, and how it fades; turn.rs: how one
                    is turned in the world, the client's sines of whole degrees; flat.rs: the triangles of the kinds that lie flat on the screen; solid.rs: those of the
                    kinds that stand in the world; particle.rs: the kinds that show a sprite (how they fly, fall,
                    go round and bow, which frame shows); horn.rs: the spike of four sides; slots.rs: the kinds
                    made of parts (motes that rise, walls of light); walls.rs: more walls of light (that
                    widen, spin, change their picture) and a level square that widens; master.rs: what a primitive shares with its
                    effect (the place aimed at, the word that one has landed) and the kinds that use it;
                    facing.rs: ways on the ground (and `Get_Angle`) and the camera's yaw as the client
                    measures them, and a sprite's turn on the screen; streak.rs: streaks of light flat over the ground that run
                    towards a place; quarter.rs: which of the four ways across the screen an
                    effect points, by which some pick a sprite's action; tint.rs: a colour that goes round; tests.rs, particle_tests.rs, horn_tests.rs, slots_tests.rs, walls_tests.rs:
                    their tests; change.rs: what a primitive becomes at a tick of its life, the speeds along
                    the three directions, the ring on the screen that widens; plate.rs: one picture standing in
                    the world; casts.rs: walls of light that rise once, or turn, and in casts/rays.rs rays in the flat
                    of the screen (each with its tests);
                    tei.rs: the kinds built corner by corner out of the client's parts kept whole
                    (`RenderTeiRect`): the one list of them (`Tei`) and which function lives and draws
                    each; in tei/: part.rs (a part, and the motions several kinds share: the wait before it
                    begins, a turn kept under a round, the end of one that has faded, the angle that wanders,
                    a tail of parts that follow) and rect.rs (a rectangle, and the ways they are built: a
                    place across the screen, a square that faces it, a band of pieces between a row of
                    edges); then a file to a kind or two: beams.rs (beams that open like a fan, a streak in five
                    sides), winds.rs (bands that go round), squares.rs (squares that slide or stand stacked),
                    panels.rs (squares that face the screen), arcs.rs (sparks that leap in a bow), their tests
                    quarters.rs and boxes.rs (the walls and the bars of Grand Cross), boomerang.rs, flecks.rs
                    and drops.rs (squares facing the screen that are thrown and come back, fly from one
                    struck, come down turning; field.rs draws them), orbit.rs (squares that fly out spinning
                    like coins), plates.rs (long plates that shoot out every way), cage.rs (half a ball of
                    panels before one who guards), bounces.rs (sparks that leap and leap on), cones.rs (walls
                    that lean out like a funnel), each with its tests; links.rs (a tube of light from one
                    actor to another, which it follows);
                    ripples.rs, columns.rs, low_rings.rs and halo.rs (the rings, the walls that turn and the
                    low walls of a Warp Portal and of the sage's grounds, a square that breathes), slashes.rs
                    (blades that shoot out along the ground), gusts.rs, gust_rings.rs and chasers.rs (squares
                    and wheels of wind that fly to someone, straight or round in a bow), scans.rs (a beam
                    that sweeps round), auras.rs (a picture over someone that faces the screen, and
                    `to_screen`), twinkles.rs (upright squares of light that grow and begin anew), drifts.rs
                    (walls that fly off one after the other), pillars.rs (shafts that shoot up; a picture
                    that runs through its frames), blades.rs (needles that fly in and whirl), layers.rs (a
                    picture drawn ten times over, larger and fainter: letters, signs), souls.rs (the light of
                    a soul link, its sparks, and the angle that wanders), fins.rs (two wedges towards a
                    place; a picture that hovers), stretch.rs (a bar drawn out across someone), dust.rs
                    (squares that fall and go out), each with its tests; sizes.rs (the colours of the bands
                    by a primitive's size), icons.rs (signs that flash up round the head of one in a fury,
                    lifted by where that one's name stands), peong.rs (the scraps and lights of a landing),
                    flowers.rs (walls arched like petals), flames.rs (lights whose way wanders), forest.rs
                    (shafts of light that slant down), launchers.rs (places that show nothing and ask for
                    another effect where they are, `Prim::asks`), aid.rs (the glitter of First Aid; a
                    footprint that fades), lights.rs (a light that goes round someone with upright squares
                    about it), sky.rs (squares that come and go over the whole view: the clouds and stars
                    of a sky), spheres.rs (a sphere of light that goes round someone: a spirit sphere),
                    max_level.rs (the lights of the aura of the highest level, particles of the client's
                    kept as parts: squares that breathe over the ground, bands that go round, lights that rise),
                    each with its tests;
                    glow.rs: a sprite standing on the ground with a rim of its own light; petals.rs: a sprite
                    that rises wandering (each with its tests); fourfold.rs: a sprite drawn four times, once
                    at each of its parts, every layer twice in colours made of its own (with its tests);
                    outdoors.rs: a sprite whose way wanders (a firefly) and one that falls wandering (a petal,
                    a leaf), with their tests
  effect_tables.rs  tables the client has compiled into it: which effect a skill or an item shows, which file
                    and sound an effect is, how the body moves for a skill (the files in effect_tables/, written
                    by tools/client-tables/skill_effects.py; skill_struck.txt is what each hit of a skill shows on
                    the one struck; effects_missing.txt lists what is not drawn yet)
  item_names.rs     an item's name as the client composes it wherever it writes one (`ITEM_INFO::DisplayName`):
                    its refinement, its cards' words before and after, its slots, who forged or brewed it,
                    the element and the strength of star crumbs, a named pet's egg (`display_name`, asked of
                    the tables through `Tables`; in the browser `data::item_title`); what an item is kept
                    from, as the client's tooltip lists it (`restrictions`)
  hotkeys.rs        the client's keys: key numbers, how a key is written, what a Mac lacks
  ai.rs             the scripts that move a homunculus and a mercenary: the client's own `AI\AI.lua` and
                    `AI_M.lua` run by a Lua interpreter in Rust (`lunita`), called with what is in sight
                    (`Sight`) and answered with what they ask for (`Order`); ai/host.rs: the functions the
                    client provides to a script (`GetV`, `Move`, `Attack`…), each a rule over the sight;
                    ai/prelude.lua: what the players' own scripts (`AI\USER_AI`) expect of Lua 5.1 besides
                    (its older names, files kept in memory, a clock), run before a script;
                    tests in ai/tests.rs (the client's own scripts run there when its folder is beside)
  talk.rs           PetTalkTable.xml: what pets say
  quests.rs         the quest journal's rules: its tabs, filters and search, how a hunt is written, what a
                    text shows of its links, which picture the mark over an NPC is
  statuses.rs       the states a character is in (buffs and debuffs): which are listed, in what order, where each
                    stands, how the time left is written (statuses.txt: the pictures and states the client has
                    compiled into it, written by tools/client-tables/statuses.py)
  cooldowns.rs      the waits before skills may be used again, and how a time is drawn over a picture
  commands.rs       the client's chat commands and what each asks for (commands.txt, written by
                    tools/client-tables/commands.py)
  links.rs          the links written into what an NPC says (`<ITEM>`, `<NAVI>`, `<URL>`…): taken out of a
                    text, and put back as what shows of each
  books.rs          the books read in a window of their own (`UIBookWnd`): which items are books, a text cut
                    into lines and pages, the page a book opens at, turning, reading aloud line by line
  whispers.rs       a talk with one player in a window of its own: what the window is called, where a new one
                    stands, how a line is written, what a command typed into it does
  net/
    mod.rs          framing a byte stream into packets (lengths from packet_lengths*.txt)
    bytes.rs        reading the numbers and texts of a packet and writing one: the one set of readers every
                    file here uses (`u16_at`, `u32_at`, `word`, `long`, the signed `i16_at`, `i32_at`, `short`,
                    `int`, `quad`, `text`, `records`, `body`, `id`…), the `Writer` of a packet with the two
                    ways a text goes into a field (`field`, `name24`; `field_whole_chars`), and what the
                    tests share (`testing`: `field`, `packet`, `sized`, `longs`, `assert_lengths`)
    golden.rs       a test: every packet this client writes, built with fixed arguments and compared byte
                    for byte with golden.txt (written by the builders as they were), so that rewriting a
                    builder cannot change its packet; fails for a builder that has no line in it
    packets.rs      the way in: what the packets share, and the login server   ── children in packets/ ──
      chars.rs        the character server            zone.rs   the basics of the map server
    units.rs        everyone on the map: appearing (with what they hold, how the head is turned, sitting or
                    lying), walking, turning, striking, rising, their spirit spheres, loot on the ground
                    (tests in units/tests.rs)
    items.rs        inventory and equipment, the number typed into the box that asks how many
                    (items/skills.rs: the skills known; items/hotkeys.rs: the hotkey bar; items/favourites.rs:
                    the inventory's tab "Fav"; items/wearing.rs: why a thing was not put on, the "off" button;
                    items/weight.rs: what the chat says as the weight carried passes a share; tests in
                    items/tests.rs)
    skills.rs       casting and landing of skills; an effect the server has shown on someone, what the
                    character recovers, a cast broken off, how far the weapon reaches
    skill_units.rs  the units skills leave on the ground coming and going
    pets.rs         the player's pet: its state, feeding, the commands given to it
    homunculus.rs   the alchemist's homunculus: everything about it, what changed, feeding; what its owner
                    asks for it (commands, a name, to walk, to strike, to come)
    mercenary.rs    a hired mercenary: everything about it, a number that changed, dismissing it
    quests.rs       the quest journal, a quest taken, advanced and dropped, the mark over an NPC
    statuses.rs     the states someone is in going on and off
    cooldowns.rs    the waits before skills may be used again
    commands.rs     what the chat commands send and hear back: who may whisper, how many play
    emotions.rs     the emotions shown over heads
    npc.rs          NPC dialogs, whispers, restart
    trade.rs        NPC shops: their lists, the deal and its answers, and the rules of the client's two windows
                    of a shop (what goes into the basket, the number typed into the box that asks how many)
    storage.rs      the storage, the Kafra's or a guild's: opened, filled, added to and taken from, the requests
                    that move a thing between it, the inventory and the cart, which tab a thing is under
    party.rs        the party: its members, how it shares, its chat, making, joining and leaving one
    pvp.rs          fighting other players: the map's kind, who is fought on it, the standing on a PVP map
    guild.rs        the guild: its packets' numbers and kinds, what it is and who belongs, its emblem,
                    skills, notice and chat as the server tells them
                    (guild/members.rs: the members, the positions, the expelled; guild/requests.rs: what
                    the client sends: making, inviting, answering, leaving, expelling, the chat, the notice;
                    guild/manage.rs: what its master does: the positions and who has which, the emblem
                    packed and checked, the guilds allied and opposed; tests in guild/tests.rs)
    choosing.rs     the lists a skill or a card opens to pick one thing of, and the answer to each: the
                    places of Teleport and Warp Portal and `/memo`'s answer, what arrows are made of, Auto
                    Spell's spells, what can be forged, brewed, cooked, repaired, a smith's refining, the
                    equipment a card goes into; a thing refined at an NPC; what goes into a forging; the name
                    of a character asked by its number
    exchange.rs     a deal between two players: asking, what each side puts in, closing, the trade
    mail.rs         the mail (RODEX): what the server tells: the mailbox listed, a letter read, its zeny
                    and things taken, a letter being written
                    (mail/store.rs: how the letters are kept and ordered, the colour of a title, the time
                    left, the fee; mail/requests.rs: what the client sends; tests in mail/tests.rs)
    friends.rs      the friends list: the list, who is in the game, asking and answering, removing a friend
    cart.rs         the merchant's cart: how full it is, a thing going in and coming out, the looks offered,
                    the moves between it and the inventory (those to and from the storage: storage.rs)
    vending.rs      a merchant's shop: a shop to be made, one's own list and what was sold, a vendor's sign
                    and list, buying; how a price is coloured, typed, and what is left of it
    viewequip.rs    looking at what another player wears, the switch that lets others look, ZC_MSG
    scripted.rs     what an NPC's script shows and plays: a picture over the screen, a mark on the minimap, the
                    bar that fills over someone and the answer to it, a sound, a piece of music, words over a head
    books.rs        a book an NPC's script has the player read (ZC_READ_BOOK)
    chat_rooms.rs   chat rooms: the sign of one, who is in the one sat in, making, entering, changing, leaving
    family.rs       the family: a wedding's confetti, a divorce, one who asks to adopt and the answer
    fight.rs        what the server says of a fight beside the blows: the combo's window, a blade caught,
                    Devotion's wards, the MVP and its reward, what Sense tells, a Star Gladiator's places and
                    missions, a skill changed or taken away, the words of a talkie box and of Graffiti
    unread.rs       the packets no reader took, counted by number, and what rAthena calls each
                    (packet_names.txt, written by tools/packet-names.py): the developer's list, `/packets`
    notices.rs      what the server tells in a line of the chat with a number or a name in it (an action
                    that failed, what a map forbids, ZC_MSG_VALUE / _SKILL / _COLOR, Gospel, experience, a
                    rented item, a thing damaged or bound): the packets, which line is said in which colour,
                    and a line filled as `sprintf` fills it (`filled`); tests in notices/tests.rs
    broadcast.rs    the server's announcements: the words, the colour its first letters name, the lines
                    across the top of the screen
    link.rs         the connection itself: the watch on a silent one (CZ_PING_LIVE), why a server ends one
    marks.rs        marks on the minimap the server tells of: the guild's members, the boss monster
  web/
    mod.rs          entry point, fetching files from the gateway, the client's Korean read (`korean`)
    socket.rs       the WebSocket
    data.rs         the client's tables fetched from the gateway: what is shared, and `load`  ── children in data/ ──
      jobs.rs  items.rs  skills.rs  pets.rs  interface.rs  maps.rs  guide.rs  statuses.rs  commands.rs
      worldmap.rs  tips.rs
      books.rs (which items are books, and the text of one: files of the client's data read as they are)
      cards.rs (the cards' words and illustrations, what an item is kept from, what a smith's things are
      made of, the names of makers as the server told them, and `item_title`: the one way a name is written)
      quests.rs  song_talk.rs (two files of the client's data read as they are: `crate::song_talk`)
      scenery.rs (the sky behind each map: its colour, its stars and clouds)
      figures.rs (the shadows' sizes by job and the order of what is worn on the head: `shadow`,
      `head_layers`)
      ai.rs (the scripts of a homunculus and a mercenary, fetched when one is first out)
      aura.rs (the highest level and the effects of its aura: `aura_table`)
                      each kind of table: what it keeps, how it is read (`read`), how it is asked (`data::…`)
    figure.rs       sprites as pictures: one frame of a monster's sprite, a character put together
                    (figure/compose.rs: body, head, weapon, its trail, shield and headgear in the client's
                    order for the direction and the frame);
                    figure/weapon.rs: which sprite a weapon is, and its picture by itself (`crate::body`)
    sound.rs        music (fading out and in at a change of map or screen) and sound effects; `sound::at`
                    plays a sound of the world as loud as its distance leaves it (`crate::earshot`)
    parts.rs        what every window is made of: places, pictures, and the marks of what can be pressed
    view.rs         the page outside the game's own windows: shared pieces    ── children in view/ ──
      accounts.rs     logging in, registering, recovering          characters.rs   server and character select, creation
      game.rs         the game's screen: the minimap and the character's place
    app.rs          the App and what the whole of it needs       ── children in app/ ──
      state.rs        what the App keeps, in groups: here what the whole game shares (the character, the
                      bag, the skills, the chat, an NPC's window); every other system's group in a file of
                      its name in state/: chat_rooms  choosing  commands  cooldowns  exchange  family  fight  friends  guide  guild  mail  party  scripted
                      pets  player_menu  pvp  quests  settings  shops  statuses  storage  viewequip  whispers
                      unread  notices  link  broadcast  marks  cart  vending  books
      wiring.rs       page events (click, key, timer) -> Action
      dragging.rs     taking a thing out of a window and letting it go, as the client does (its sprite under the pointer)
      actions.rs      which code sees to each Action (the one place everything arrives at); the longer ones in actions/
        forms.rs  bag.rs  link.rs (what comes of the socket, keeping it alive, the watch on it)  taming.rs  travel.rs
      clicks.rs       what each press does (crate::press): handed to its system in clicks/, as incoming/ is
      keys.rs         the keyboard
      hotkeys_set.rs  which key does what for this player; setting keys (kept in the browser)
      account.rs      login, registration, recovery
      connection.rs   login -> char -> map server, and back
      incoming.rs     packets from the servers: which server's reader a packet goes to, and the list of the
                      map server's readers (`MAP_SERVER`), a system each in incoming/
        way_in.rs (the login and the character server, a session thrown out)
        travel.rs  actors.rs  combat.rs  talk.rs  status.rs  inventory.rs  shops.rs  storage.rs  pets.rs
        quests.rs
        statuses.rs  cooldowns.rs  commands.rs  party.rs  exchange.rs  skill_units.rs
        pvp.rs  guild.rs
        fight.rs (what the server says of a fight beside the blows: `net/fight.rs`)
        friends.rs  viewequip.rs  whispers.rs (a whisper heard and the answer to one sent: to the window
        of the talk, or to the chat)
        mail.rs (what the server says of the mail; it overhears the state that counts the letters sent)
        cart.rs (the cart; it overhears the inventory's lists, of which the cart's is one kind, and the
        state that says who pushes which cart)  falcon.rs (it overhears the skills that send a falcon
        flying)  vending.rs (shops and signs; it overhears who goes out of sight)
        scripted.rs (what an NPC's script shows and plays)  chat_rooms.rs (signs, the room sat in, and what is
        said near while one sits in a room)  family.rs
        books.rs (a book a script opens)
        homunculus.rs  mercenary.rs (what the server says of the two)  ai.rs (it overhears who struck
        whom: the scripts of the two are told)
        spirits.rs (how many spirit spheres someone has; it overhears who goes out of sight)
        unread.rs (a packet nobody read is counted)  notices.rs (the server's lines for the chat)
        broadcast.rs (an announcement: into the chat and across the top of the screen)
        link.rs (ZC_PING_LIVE; the words for why a server ends the session)  marks.rs (the guild's
        members and the boss on the minimap)
        choosing.rs (the lists a skill or a card opens and how each choice ended; it overhears every packet
        to ask for the makers' names that a window looked up in vain)
      clicks/         account  talk  status  inventory  skills  chat  shops  storage  pets  hotkeys  settings  guide
                      quests
                      party (its buttons, its commands, its chat's `%`)  exchange  player_menu (the menu on
                      another player: opening it, and what its rows do)
                      guild (its window, the menus in it, its commands, its chat's `$`; what its master
                      does in guild/manage.rs: positions, the emblem's file, alliances, skills, the notice)
                      fight (the OK of the Monster Info window, the answer about a Star Gladiator's place)
                      friends (the Friend side of the party's window, its key, the menu on a friend, its
                      questions)  whispers (opening the window of a talk, what is typed into it, the options
                      of when one opens)  viewequip (asking what another wears, the box that lets others look)
                      choosing (a row picked, OK and Cancel of whichever list is open and what each
                      sends; a card double-clicked; what is dragged into a forging)
                      scripted (the cross of a script's picture, and the times its bar, sounds and marks run by)
                      chat_rooms (making a room, a sign entered, what is typed in a room, its menu)  family
                      mail (the mailbox: tabs, pages, search, reading, taking, deleting; the letter being
                      written in mail/writing.rs: the receiver asked about, things dragged in, sending)
                      cart (its window and key, what is dragged into it and out of it, the looks of Change
                      Cart)  vending (making a shop and closing it, a sign pressed, the basket at a vendor's)
                      emotions (the window of emotions and the Shortcut List: its ten lines kept and said with Alt and a digit)
                      worldmap (the World Map, the large window of the map one is on)  tips (the Tipbox)
                      homunculus  mercenary (the window of each, what it asks, the list of its skills,
                      the menu on it)
                      unread (the list of packets not read: `/packets` and its buttons)
                      books (a book asked for and read, its pages turned with the buttons and the arrow
                      keys, its bookmark kept in the browser, its lines said when it is read aloud)
      settings.rs     the Game Settings: the player's options, kept in the browser, and what they switch
      ai.rs           the scripts of the homunculus and the mercenary at work: called when due, told what is
                      in sight, obeyed; the commands the player gives the two, their skills used
      commands.rs     what is typed after a slash (commands/beside.rs: the options with a command and no row of
                      the Game Settings, `/set1`…, `/hi`, `/savechat`, the rankings, `/tip`, `/buildinfo`)
      screenshot.rs   a picture of the screen given to the player as a file: the world's canvas, the windows
                      over it and the client's stamp put together
      game.rs         using items and skills, hotkeys, chat lines, entering a map
      screen.rs       which screen shows; painting characters into windows
      windows.rs      the windows' order on the page (stack.rs), and Escape closing the one in front; which
                      windows have a part in it is read from hud/windows.rs. Also what every system's menus
                      and boxes share on the app's side: a place of the page in the stage's measure
                      (`on_stage`), opening a window there (`open_at`), putting every menu away
                      (`close_menus`), the number typed into a box that asks how many (`counted`)
      page.rs         DOM helpers (`written`: what stands in a field now), storage, fade and loading screen,
                      dragging windows
    hud.rs          the in-game windows: the list of their files  ── children in hud/ ──
      frames.rs       what windows are framed with: `art`, `fill`, `title_bar`, `panel`, `panel_ground`,
                      `panel_ground_round`, `side_rimmed`, `button_bar`, `system_box`, `cross`, `base_button`,
                      `figure_canvas`
      buttons.rs      a button of three pictures (`pictured`, `pictured_as` with `Kind::Submit`, `states`
                      with the `Convention` its files are named by, `picture_press`, `picture_press_sized`,
                      `press_button`) and the client's button with words (`basic_button`)
      texts.rs        `Font` and its usual sizes, a text at a place (`text`, `text_in`, `text_ending`,
                      `text_ending_in`, `text_centred`, `fitted`), `titled`, `outlined`, `coloured`, `count`
      checks.rs       check boxes and radio buttons (`check_words`, `check`, `radio`, `radio_marked`)
      gauges.rs       the pieces of a gauge (`filling` with `Red`, `numbers_over`, `plain_bar`)
      items.rs        an item's and a skill's picture by its number (`App::item_icon`, `icon_of`,
                      `skill_icon`), the grid of cells (`grid_of`, `grid_cell`)
      fields.rs       a field that is typed into (`field`, `numeric_field`)
      colours.rs      the colours the windows share
      basic.rs  inventory.rs  equipment.rs  skills.rs  status.rs  chat.rs  dialogs.rs  trade.rs  pets.rs
      hotkeys_set.rs  settings.rs  guide.rs
      boxes.rs        what every system asks with, each written once: the client's message box with a
                      question (`message_box`), the box that asks how many (`App::count_box`), a row of a
                      menu opened with the right button (`menu_row`, `MENU_BOUNDARY`), the list of rows
                      with a picture to pick one of (`pick_list`, `Picking`)
      lists.rs        the client's windows that list things a row to a thing (a shop's two, the storage's):
                      their frame, their list and a row of it (`list_frame`, `list_of`, `item_row`)
      storage.rs      the storage's window with its tabs, order and search, the windows that show one tab apart,
                      the one of what a search found, how many go in or out
      party.rs        the party window, its setup, invitation and questions, its marks on the minimap
      quests.rs       the quest journal, the button that tells of a new quest, the list of quests on the
                      screen, the NPCs' marks on the minimap; the pane of one quest in quests/pane.rs
      player_menu.rs  the rows of the menu on another player: a system with something to do with one adds its own
      pvp.rs          the standing on a PVP map, written in the corner of the screen
      guild.rs        the guild window, its first tab, the menus in it, what the guild asks; the other five
                      tabs (members, positions, skills, the expelled, the notice) in guild/tabs.rs
      exchange.rs     the window of a deal between two players, the question whether to deal, how many go in
      mail.rs         the notice of mail not read, the mailbox, what the mail asks, how many go into a letter;
                      the letter being read and the one being written in mail/letter.rs
      cart.rs         the cart's window, the Equipment window's button for it, the window of its looks; the
                      grid of things the windows made of `UIItemBaseWnd` share (`grid_of`, `grid_cell`)
      vending.rs      a shop's sign, the window a shop is made in and the one of what the cart has for it,
                      one's own shop, what it asks; the buyer's list, basket and question in vending/buying.rs
      choosing.rs     the windows of the lists a skill or a card opens, in the client's three shapes: the
                      places of a warp, rows with a picture (`boxes::pick_list`), lines under a title bar and
                      the thing being forged
      friends.rs      the Friend side of the party's window, the menu on a friend, the Friend Setup window, what
                      the friends list asks
      whispers.rs     the window of a talk with one player, and its menu
      viewequip.rs    the window that shows another player's equipment, the box that lets others look at one's own
      scripted.rs     the picture an NPC's script shows over the screen, its marks on the minimap
      chat_rooms.rs   the sign of a chat room, the window that makes or changes one, the room's window and
                      menu, the box that asks a room's password
      family.rs       the question of one who asks to adopt the character
      fight.rs        the Monster Info window of Sense, the question about a Star Gladiator's place
      homunculus.rs   the homunculus window, what it asks, the list of its skills, the menu on it
      mercenary.rs    the same of the mercenary
      skill_list.rs   the list of a homunculus's skills and of a mercenary's (the client's `UISkillListWnd`)
      tabs.rs         the tabs down a window's left side, their words a letter under a letter (the client's
                      `UITabControl` in its layout 0: the skill window's and the inventory's; `side_tabs`)
      unread.rs       the developer's list of the packets this client does not read (`/packets`)
      broadcast.rs    the server's announcements across the top of the screen (`UIBroadcastBalloon`)
      marks.rs        the guild's members and the boss monster on the minimap (written with the party's marks)
      statuses.rs     the states the character is in, listed down the right side of the screen
      cooldowns.rs    the shade of a time that runs, over a skill of the hotkey bar and over a state
      emotions.rs     the window of emotions and the Shortcut List
      worldmap.rs     the World Map and the large window of the map one is on
      tips.rs         the Tipbox
      books.rs        the window a book is read in, the line of one read aloud, "Read" and "Auto Read" on
                      an item that is a book
      windows.rs      the list of the screen's windows: one interface for drawing, redrawing, order, layer,
                      Escape. Here the type (`Window`, `Order`, `Layer`), what entries are written with
                      (`framed`, `middle`, `centred`, `staged`, `ordered`, `Window::question`,
                      `Window::count`, `Window::menu`) and the one list `ALL`; the entries in windows/, a
                      file to a system: scripted  chat_rooms  family  status  inventory  skills  talk  shops  storage  pets  settings  guide  party
                      player_menu  exchange  quests  guild  mail  friends  fight  viewequip  choosing  whispers (a window to
                      each of the talks that may be open at once)  unread  broadcast  cart  vending  books
    world.rs        the world: its types and what all of it needs     ── children in world/ ──
      state.rs        what the world keeps between frames
      session.rs      entering/leaving a map, the player's own character      (called by the app)
      actors.rs       everyone else and the loot                                (called by the app)
      aura.rs         the aura of the highest level kept on those who show it, held to `crate::aura` once
                      a frame
      ailments.rs     body and health states: the colour, the marks, the veils   (called by the app);
                      everyone's body as effects leave it, the footprints and the copies they leave
      skill_units.rs  the units of skills on the ground, their effects          (called by the app)
                      skill_units/traps.rs: the traps' models, read, stood on their cells and played
                      (the models themselves are land.rs's: `Models::raise`)
      combat.rs       blows and casts: who strikes, casts, aims, bends down, the window of a combo, a
                      blade caught                                              (called by the app);
                      combat/landing.rs: the hits that land, out of the one queue of them (`State::blows`):
                      each sounds from where it lands and shows its numbers and words (`crate::blow::signs`),
                      and tells `frame/sprites.rs` who stops, flinches and flashes; a number over someone
      earshot.rs      the sounds of the world, each heard from where the one it starts on stands (`heard`,
                      `heard_at`, and those due at a moment yet to come)
      names.rs        what the server told of those in sight beside their names (level, boss, title, group)
                      and the player's own level: what is written under one, its colour and its mark
      written.rs      the words of units on the ground: a talkie box's, Graffiti's      (called by the app)
      launch.rs       one of the client's effects begins on someone, or ends: played from its files
                      (`crate::effect::played`) or drawn by code (`prims::begin`)      (called by the app)
      state_effects.rs  the effects the states of someone begin and end (`crate::state_effects`): as a state
                      changes, as one comes into sight, on the player's own as it arrives  (called by the app)
      frame.rs        one frame: its steps in order                ── children in frame/ ──
        sprites.rs      who and what stands in the world, the shadows under them, the sounds of their
                        frames; the one struck stopping, flinching and flashing
        marks.rs        the cell under the pointer, the portals' rings, the numbers in the air
        effects.rs      each kind of effect turned into quads
      quads.rs        the effects' quads of a frame, as the renderer takes them
      overlays.rs     what is laid over the picture: its steps, the name, the numbers   ── children in overlays/ ──
        bars.rs  pointer.rs    the bars and words over heads; which pointer the mouse is and its picture
      pursuit.rs      what the character does by itself: walk on, fetch a thing, chase and strike, walk into reach
      pick.rs         what the pointer is on: someone, or a thing on the ground
      ai.rs           what the scripts of a homunculus and a mercenary are told of the map, the commands the
                      mouse gives the two with Alt held, the life under each          (called by the app)
      players.rs      what the app asks about the other players on the map: who one is, who has a name
      pvp.rs          the map's kind and the player's friends, as the app tells them: who is fought as a monster is
      guild.rs        the guilds of those in sight and the player's own: the names and the emblem written
                      under a player
      quests.rs       the marks over NPCs that have a quest                     (called by the app)
      cart.rs         the carts merchants pull, each a sprite that follows its owner (called by the app)
      falcon.rs       the falcons hunters keep, one by everyone whose state has its bit; a skill sends one
                      at something (`falcon_sent`)                              (called by the app)
      signs.rs        the signs over heads, of chat rooms and of shops: kept over their owners on the page
                      (called by the app; what a sign is made of is its system's, in `hud/`)
      scripted.rs     what an NPC's script shows in the world: the bar that fills over someone, a sound from
                      where someone stands                                      (called by the app)
      spirits.rs      the spirit spheres and coins about someone, as the server tells how many: the effect
                      of them ended and begun anew, told the count              (called by the app)
      input.rs        the mouse over the world: which event goes where   ── children in input/ ──
        press.rs  pointer.rs  wheel.rs   a button down; the pointer moved or a button up; the wheel
      gpu.rs          pipelines, textures and buffers   ── children ──
        draw.rs         the drawing of one frame, step by step: the map, the sprites and numbers, the effects
        layers.rs       the sprites of the effects of primitives: read as they are, a picture on the GPU to each
      loading.rs      reading a map's files
      land.rs         the map's models and water on the GPU: read, sorted by picture, the models whose parts
                      turn moved on every frame (drawn by gpu/land.rs, which also tells the map's programs
                      the eye, the fog, the night's light and the water's waves)
      scenery.rs      what surrounds the player: the effects and sounds a map places, its fog (`world::fog`),
                      sky, view and weather, the night; scenery/placed.rs: one frame of it (the makers launch,
                      the sounds play, the weather is kept going)
      camera.rs  effects.rs  shaders.rs
      math.rs         vectors of three, and the lots the world draws (`lot`, the client's `rand() % under`)
      mover.rs        those who stand and walk: each moved along its way by the server's clock every frame
                      (`crate::route`, `crate::server_time`), and which picture of it shows (`crate::pose`:
                      its motion, the frames of a state, standing ready, the head's turn)
      poses.rs        how those in sight are shown as the server tells of each: turning and the head's turn,
                      lying dead and rising, hiding with its puff of smoke, whether a shadow lies under one,
                      the look towards a place pressed                           (called by the app)
      prims.rs        an effect made of the client's primitives (`crate::prim`) under way on someone
                      (`Burst`, `Course`), the one way one is begun (`begin`), and the one list of the files
                      of effects (`files!`)   ── children, each with its `FILE` but the first ──
        launched.rs     primitives as several files launch them, each written once: a sprite thrown, bowed
                        or thrown level, the band of a hit, a primitive of parts and a part, the beams of
                        `TexturedQuadSpin`, the rings of a portal, the mark of `TexturedSkillAura`, the
                        square of `TexturedLevelRing`, the sparks of a soul's light; the names of the
                        sprites several files show
        hits.rs         what a blow shows where it lands: the ring and the flares of a hit, the hit of
                        cold, of Pierce and Sonic Blow, the dark hit
        struck.rs       what follows a blow on the one struck: Bowling Bash, Blitz Beat, the ball of
                        lightning, the spikes of Grimtooth, the sign of a curse
        bolts.rs        the mage's bolts and ice: effects that fly to or fall on the one struck (Soul
                        Strike and its dark twin, Fire Ball, the bolts, Frost Diver, Spear Boomerang)
        missiles.rs     what else flies or runs to a place it is told: the balls of Jupitel Thunder and Water
                        Ball, Grimtooth, the drain, the fists of Boost Knuckle, streaks over the ground, the
                        streaks of Sharp Shooting and Arrow Vulcan
        heals.rs        healing and Resurrection
        sprites.rs      effects made of sprites (Steal, Detoxify, Sight, Ruwach, Aqua Benedicta, Blessing,
                        Enchant Poison, Enchant Deadly Poison, Kouenka); it has the shared sprites read
        played.rs       a sprite that plays at a place: the gunslinger's, the ninja's and the elementals'
                        skills (`TexturedClassChange`), the lightning of Raigekisai, the dark breath, the palm
                        of Investigate and the spears of Hundred Spear
        bursts.rs       what bursts out from someone across the screen: Endure, Bash, Turn Undead, Adrenaline
                        Rush, the rings of Sonic Blow and Overthrust, the wedges of a call for slaves, the
                        rays of Chemical Protection
        blasts.rs       what bursts out over the ground: Magnum Break, Sightrasher, Earth Spike and its ice,
                        the splash of a ball of water
        motes.rs        small lights round someone: the streaks of Increase and Decrease Agility and of the
                        dances, sparks that loop out and back, the Warmths' motes
        walls.rs        walls of light round someone: the rings of a spell in every colour, the pillar of a
                        changed element, Defender's and Reflect Shield's walls, walls that spin away, the
                        swirl a song begins with
        winds.rs        winds and bands that go round someone: Wind Walk, the storm kicks, the bands of
                        leaves under the moonlight
        marks.rs        pictures lying over the ground: the mark of Sense, rings and squares that widen
        shields.rs      walls, shields and what is thrown like one: Grand Cross, Shield Boomerang, Shield
                        Chain, the spheres of Finger Offensive, the panels of a guard and of Preserve
        sprays.rs       what is struck or torn out of someone: the coins of Steal Coin, the gear of a Strip
                        (pictures of items among them), the sparks of Investigate and Back Stab, of a kick
                        and of Cart Termination, the streaks of Asura Strike and Raid
        blows.rs        blows of the later jobs: the streaks of Triple Attack, fans of beams (Sacrifice, Tiger
                        Fist, Head Crush, Joint Beat), the squares of Soul Breaker and Meteor Assault, the
                        sparks of a combo's end, the funnels of Spiral Pierce, the cross of Pressure
        grounds.rs      what the units of skills on the ground launch: ice, a pillar's rings, a wall of light
        sounds.rs       effects that are sounds and nothing drawn (Napalm Vulcan among them; and the two
                        that are a line of talk)
        bodies.rs       effects that draw nothing and change the body of the one they are on (`crate::body`):
                        the bodies of Berserk, Assumptio, Kaite, a Soul Link, the flashes, lights, kicks, the
                        weapon of Aura Blade and Mystical Amplification, the footprints of one who runs, the
                        copies left by an Emergency Escape
        links.rs        a tube of light between two actors that follows both: Soul Change, the strings of
                        Marionette Control, Soul Unity
        auras.rs        pictures over someone that face the screen: the flash of a hit, the clock of slowed
                        casting, the target of Analyze, the strokes of Body Painting, the ring of Deadly Infect
        gusts.rs        squares of wind that fly from the striker to the one struck: Estin, Esma, Huujin,
                        Kamaitachi
        rays.rs         lines of light out from someone or in to someone: the beams of True Sight, the blades
                        of a run that stops and of Kaizel, the beams of Infrared Scan, the needles of Prestige
                        and Inspiration
        songs.rs        the songs of minstrels and wanderers: the monsters that come up round one who sings,
                        the lights of the circles, the sign of Randomize Spell, the lines of Great Echo, the
                        note of Reverberation
        max_level.rs    the aura of the highest level: the squares and bands of the full aura, the lights
                        that rise, those of the baby jobs
        souls.rs        the link of a soul: the light that flies over the one linked, its sparks, the letters
        spirits.rs      the spirit spheres of a monk and the coins of a gunslinger, going round the one
                        who has them
        waves.rs        walls of light that fly to the one struck: the bubbles and waves of Sonic Wave, the
                        elemental's Wind Slash
        pillars.rs      shafts of light out of the ground: Tetra Vortex and its blast, Hesperus Lit
        falls.rs        what falls on someone from high above: the stones of Stone Rain
        held.rs         what a state shows on the one in it until it ends: the signs of Fury, Steel Body,
                        Power Up and the Warmths, the sign of Stop
        puffs.rs        the puff of one who lands from a leap
        trails.rs       lights that leave sparks behind them, each spark an effect the light's primitive asks
                        for (`Asked::launches`): the ball of Estun, the lights of Harmonize and Arrullo
        prints.rs       a ring of sprites that spreads over the ground: Fire Wave, Poison Buster
        places.rs       what a map places about itself: the smoke of a chimney, a firefly, the flame of a
                        torch, the Christmas lights, shafts of light through the trees
        weather.rs      what falls or drifts over the whole view round the player: snow, petals, leaves,
                        the clouds and the stars of a sky
gateway/src/
  main.rs           routes and start-up
  grf.rs            reading files out of GRF archives
  tables.rs         the client's Lua tables: running them, and everything the browser is given at once; what
                    the files of tables/ share (`state`, `exec`, `run`, `run_in`, `beside`,
                    `names_stand_for_themselves`, `whole`, `text`, `lines`, `names`, `numbered`, `by_number`,
                    `json_of`, `korean`, `latin`, `hash_fields`), tested in tables/tests.rs
    tables/           one file per kind: items  skills  interface  maps  guide  quests  (+ job_codes.json)
                      statuses (the states' pictures and descriptions)  commands (the help, what options say)
                      worldmap (the worlds of the World Map, where each map lies on them)  tips (the Tipbox)
                      scenery (the sky of each map: MapSkyData.lub)
                      figures (the shadows' sizes by job, the order of what is worn on the head)
                      ai (the Lua files of the client's `AI` folder, for the scripts of a homunculus and a
                      mercenary: `/tables/ai.json`, and with the player's own `/tables/ai-user.json`)
                      cards (the cards' words and illustrations, what may not be moved, a smith's materials)
                      aura (the highest level and the effects of its aura: `ExternalSettings_kr.lub`)
  images.rs         game pictures as PNG
  bridge.rs         WebSocket <-> TCP, only to the configured server ports; a debug delay each way
                    (`--lag`, `--jitter`: never where others play)
  accounts.rs       registration and password recovery
  limiter.rs        limits on attempts
  public.rs         reached from outside: the invitation word before anything is served, and visitors counted
                    by their own addresses behind a tunnel or a proxy
  mapcache.rs       rAthena's map cache
translations/
  GLOSSARY.md       terms, so that translators agree
  src/              English sources of the translated batches
  ru/               Russian: items-*.json, skills-*.json  ({"<id>": {"name", "description"}})
rules.sh            switches the local server between the classic rules and Renewal's (its image and database)
dev.sh              builds the client again whenever a file of it is saved; a page opened with ?dev reloads
public.sh           opens the local server to invited friends: the gateway asking for the invitation word,
                    and a Cloudflare quick tunnel to it
check.sh            what a change passes before it is committed: tests, the browser build, tools/limits.py
tools/
  packet-lengths    builds the packet length table from rAthena's sources
  packet-names.py   writes out what rAthena's headers call each packet (client/src/net/packet_names.txt)
  limits.py         the sizes of rules 4 and 10: no file past about 500 lines, no function past about 100; and
                    of rule 12: no name of the page written by hand, no window laid over the others by its
                    name in the style sheet; lines longer than 200 characters are counted in a note
  client-tables     writes out tables compiled into the original client (skill_effects.py: skills and their
                    effects; statuses.py: the states' pictures and what the body's and health's states stand for,
                    statuses.txt; commands.py: the chat commands and the options' messages, commands.txt; job_bases.py: the plain job of each mounted job, data/job_bases.txt; job_mounts.py: the
                    mounted job of each job and state, job_mounts.txt), the classic server's scripts put right for the client's maps (classic_overlay.py
                    writes server/npc/roweb_classic*), and a check that everyone the scripts place stands where the
                    client's map lets one stand (placement_check.py)
  split_module.py   moves functions of a file that grew too large into child modules (also a whole
                    `impl Trait for Type`)
```

## How things flow

**Something happens on the page.** `app/wiring.rs` turns the browser's event into an `Action` and calls
`App::handle` (`app/actions.rs`). Nothing else listens to the page, except the world's own mouse handling in
`world/input.rs`, which reports back through `world::Hooks` — again as Actions.

**The server says something.** `socket.rs` → `Action::Socket` → `App::received` (`app/connection.rs`) frames the
bytes with `net::mod` and hands each packet to `App::packet` (`app/incoming.rs`). The map server's packets go
through the `*_packet` functions of `app/incoming/`, a system each, in the order of the list `MAP_SERVER`;
each returns whether the packet was its own. A handler does three things, in this order: parse (`net::*`), update the App's state, and tell the world
(`world::*`) and the windows (`refresh_*` or `render`).

**The world.** `world.rs` holds one `State` (thread-local): the GPU, the `Scene` of the current map, and what is
going on in it. The app only *tells* it things through the public functions in `world/session.rs`,
`world/actors.rs`, `world/combat.rs`, `world/launch.rs` and `world/state_effects.rs`; it never reaches into the state. `world/frame.rs` runs once per
animation frame.

**The windows.** They are HTML strings built by `hud.rs` and `hud/*.rs` from the App's state and from the
original client's bitmaps, placed at the original's coordinates. `App::render` writes the whole page. The game's
windows are one list (`hud/windows.rs`): each entry has its element, when it is open, where it stands, what
draws it, its part in the order and its layer (how it lies among the others when it has no part in the order:
under the windows, over them as one that asks something, over those as a menu); the screen is written from
the list, `App::redraw` draws some of them anew
where they are (typing and scrolling survive; a closed one is emptied and hidden), and the order and Escape are
read from it. What can be pressed is a value (`crate::press`): its element carries the value's name
(`data-click`, `data-use` for a double click, `data-drag`, `data-submit`), written only by `web/parts.rs`, and
the app reads the value back and matches on it. `data-info` / `data-skill` (right click) carry numbers.

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
   calls the public functions of `world/{session,actors,combat,launch,state_effects}.rs`. Windows (`hud/`) only read the App's state.
3. **Parents hold what is shared, children hold the behaviour.** `app.rs`, `world.rs`, `hud.rs`, `data.rs` and
   the gateway's `tables.rs` have the types, the shared helpers and the list of children; a child starts with
   `use super::*;` and is about one thing.
4. **No file grows past about 500 lines.** One that does is cut by responsibility *in the same change* — not
   later.
5. **One state, owned in one place.** What the game remembers is a field of its system's group in `App`
   (`app.rs`), or of the world's `State`. No second copy, no hidden global except the tables in `data/` and
   a pointer's gesture while it lasts and what it left on the page: the window being dragged and where each
   was dragged to (`app/page.rs`), the thing being dragged (`app/dragging.rs`).
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
10. **No function grows past about 100 lines**, unless all it does is hand work out by kind (a `match` over
    packets or actions, whose arms are a line or two each). A longer one is cut into its steps, each a function
    named for what it does: `world/frame.rs` is the pattern, a list of steps and nothing else.
11. **Nothing is written twice.** What two places need is a function both call. Some that exist and are to be
    used: `crate::web::document()` and `by_id()` for the page; `App::send` and `App::send_on_map` for packets;
    `Scene::place_of` for where someone is; `Quads` and `nearer` for an effect's quads; `world::burst` to show
    one of the client's effects by its number, `prims::begin` to begin one drawn by code, `launch::files` for
    one played from files, `world::lot` for a lot; in a file of effects `Course::sound` and `Course::starts`
    for its sounds, `Touch::flagged` and `flashes` for what it does to its actor, `prims/launched.rs` for a
    primitive several effects launch alike; for a kind of `prim/tei/`, `Part::waits`, `wander`, `trail`,
    `Rect::band` and `Part::facing`; `parts::{at, picture, pressed, used, dragged, sent}` for what
    every window is made of; `hud::{text, text_in, Font}` for a text in the client's own size (`crate::font`);
    `hud::pictured` (and `picture_press`, `press_button` over it) for a button of three pictures;
    `hud::{message_box, menu_row}` and `App::count_box` for a question, a menu's row and the box that asks how
    many; `App::words` for a line of the client's message table; `crate::said` for the kind and the colour of a
    line of the chat; `parts::picture` and `web::image_url` for the address of a picture; `net::bytes` for
    reading a number, a text or a name out of a packet and `net::bytes::Writer` for writing one;
    `crate::bytes::Reader` for reading a file of the game; `crate::vector` for vectors of three;
    `crate::tables` for a table of text and `crate::markup` for `^RRGGBB` and the tags in a text;
    `crate::kept` for what the browser keeps; `crate::figures::{grouped, typed, filled, price_colour}` for
    numbers written, typed and filled into a line; `crate::jobs` for what a job is by its number;
    `web::korean` for a text of the client's files; `hud/{frames, buttons, texts, checks, gauges, items,
    fields, colours}.rs` for what windows are made of; the gateway's `tables::{state, exec, whole, text,
    lines, numbered, by_number, latin, hash_fields}`. Before writing a helper, look for one.
12. **Nothing on the page is named by hand.** What can be pressed is a value of `crate::press`, drawn through
    `web/parts.rs` (`pressed`, `used`, `dragged`, `sent`) and matched as a value where it is handled, with no
    arm for "anything else": a press nobody handles, or a name nobody draws, does not compile. A window is an
    entry of `hud/windows.rs`, and the page's style sheet (`client/web/index.html`) never names one: how a
    window lies over the others is its entry's `layer`, for which the sheet has one rule a layer. A field of a
    box that asks is named from its window (`Window::field`). `tools/limits.py` refuses a `data-click="…"`
    written out, and a rule of the style sheet that gives a window's ID a `z-index`.

Before finishing, and before any commit: `./check.sh`. It runs the tests, the build for the browser with no
warnings, and `tools/limits.py`, which holds rules 4 and 10 (a function that is a plain dispatcher is named in
its `DISPATCHERS`, with the reason there) and what a script can hold of rule 12. A change is not committed
until it ends with "all checks passed".

## How to…

**…add a table from the client's files**: read it in `gateway/src/tables/<kind>.rs` and add it to
`tables::for_browser`; on the client, keep and read it in `web/data/<kind>.rs` (`read`, called from
`data::load`) and bump the number in `data::load`'s URL so browsers fetch the tables anew.

**…handle a new packet from the map server**
1. Its length must be in `client/src/net/packet_lengths.txt` (generated) or `packet_lengths_more.txt` (by hand).
2. Add a parser with a unit test in the fitting `client/src/net/*.rs`, reading with `net::bytes`. No packet is
   read or built by hand under `web/`.
3. Add an arm to the fitting `client/src/web/app/incoming/*.rs`; a new system's `<system>_packet` is one more
   name in `MAP_SERVER` (`app/incoming.rs`).
4. If the world must show it, add or call a function in `world/{session,actors,combat}.rs` (an effect by its
   number is `world::burst`, whatever the effect is made of).

**…send a packet**: a builder function next to the parsers in `net/`, called from `app/clicks/*.rs`, `app/keys.rs`
or `app/actions.rs` as `self.send_on_map(&…)`.

**…say something in the chat**: `self.chat_line(words, colour)`, or `chat_line_of(kind, words, colour)` for a
line of a kind the chat's tabs tell apart; the kinds and the colours are `crate::said` (`RED`, `YELLOW`,
`CYAN`… are in scope everywhere under `app/`). A colour only one line has is written where the line is said.

**…add or change a window**: what is in it in `hud/<system>.rs`; an entry for it in its system's file of
`hud/windows/` and its name in `ALL` of `hud/windows.rs` (its element, when it is open, where it stands, what
draws it, what Escape does to it if it is to lie in the order, and its `layer`: `Layer::Usual` for a plain
window) — nothing else has to know of it, the style sheet least of all; its state as a field of its system's
group in `App` (`app/state.rs` for `Hero`, `Bag`, `Skills`, `Chat`, `Dialog`; `app/state/<system>.rs` for
every other, and a new system gets a file and a group of its own); its texts through `i18n::tr`, or
`App::words` for a line of the client's message table. To show a change in it,
`self.redraw(&[&wnd::<WINDOW>])`; `render()` writes the whole screen.

**…add a window that asks something** (it lies over the windows that are in the order): its entry has
`layer: Layer::Asks`. Three kinds are written once and only named by a system:
- *a question with OK and Cancel*: the entry is `Window::question("<system>-asks", open, App::<system>_question)`;
  the drawing is `message_box(Asked::Offer or Asked::Sure, &words, Press::Yes, Press::No)` (`hud/boxes.rs`;
  the two layouts are the client's two); what is asked is an `Option<…Ask>` in the system's group, and the
  two presses are arms of `app/clicks/<system>.rs`.
- *how many of a thing*: the entry is `Window::count("<system>-count", open, App::<system>_count_window)`; the
  drawing is `self.count_box(&windows::<WINDOW>, counting, Form::…, Press::…)` with `counting` the place in the
  inventory and how many there are, kept in the system's group; the form is a variant of
  `press::things::Form` whose arm in `app/actions/forms.rs` reads the number with
  `Self::counted(&wnd::<WINDOW>, most)` (the rule itself is `net::items::count_typed`, tested).
- *a menu opened with the right button*: what it opens on is a variant of `press::things::Menu`, drawn with
  `menued(..)`; it arrives as `Action::MenuOn(what, at)` (`app/actions.rs`), where the system keeps
  `Some((Self::on_stage(at), what))` in its group and shows the menu (`self.open_at(&wnd::<MENU>,
  &wnd::menu_style(at))`, or `render()`); the entry is `Window::menu("<system>-menu", open, |app, _|
  menu_frame(<where it was opened>), App::<system>_menu)`; its rows are `menu_row(&words, Press::Do(entry))`
  with `MENU_BOUNDARY` between groups; a `close_<system>_menu` that puts it away is called from
  `App::close_menus` (`app/windows.rs`), so a press elsewhere closes it.

**…add a button, a tab, a row that is pressed**: a variant in the system's list in `client/src/press/<system>.rs`
(name it as nothing else is named: a test holds that); draw it with `pressed(Press::…)` or a helper that takes a
press (`picture_press`, `press_button`); the compiler then asks for its arm in `app/clicks/<system>.rs`. What is
double-clicked, dragged or sent as a form is in `press/things.rs`, drawn with `used`, `dragged` and `sent`; so
is what the right button opens a menu on (`menued`, which arrives as `Action::MenuOn` with the place). A
window that opens where the pointer is, or at a place decided as it opens, is put there with `App::open_at`
(`app/windows.rs`): `redraw` alone leaves a window where the screen was last written with it.

**…let a window take the things dragged**: a variant of `press::things::Drop`, written on the window (or on a
part of it) with `dropped(Drop::…)`; what is let go over it arrives as `Action::LetGo(dragged, onto)` in
`App::let_go` (`app/actions/bag.rs`), which hands it to the window's system (`shop_let_go`, `storage_let_go`,
`tab_let_go`, `cart_let_go`, `vending_let_go`). A thing taken out of a window is a variant of `Drag`, drawn with `dragged(..)`.

**…add a row to the menu on another player**: an `Entry` in `client/src/press/player_menu.rs`, its place and
words in `hud/player_menu.rs` (the client's order: `CGameMode::OpenActorMenu`), and what it does in
`app/clicks/player_menu.rs`.

**…add an effect**: one of the client's effects is shown by its number, `world::burst(effect, on_whom, blow,
delay)` (`world/launch.rs`), whatever it is made of; a `.str` file by its name alone is `world::effect("name.str",
on_whom)`. Which of the two an effect is, is asked in one place, `crate::effect::played`:
- *a plain `.str` file* needs no code: the client's table has it (`effect_tables/effect_files.txt`, with its
  sound in `effect_sounds.txt`).
- *an effect the client draws by code* goes to `world/effects.rs` (geometry) or is made of the client's
  primitives: a function in the file of `world/prims/` whose effects are of its sort (hits, bolts, walls of
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
`prim/tei/` with two functions for a part, each its function of the client line for line: one tick of its
life (named in `Prim::tick_parts`) and its rectangles (named in `Prim::rects`), with its tests beside them.
What kinds share is taken from `tei/part.rs` (`waits`, `under_a_round`, `over_once_gone`, `wander`,
`fly_level`, `follow`, `trail`, `Prim::head_and_tail`) and `tei/rect.rs` (`Rect::new`, `Rect::piece`,
`Rect::band` with `edges`, `to_screen`, `Part::facing`); a motion or a way of building rectangles that a
second kind needs is moved there, not written again.

**…add an effect a map places, or a kind of weather**: an effect of a map's `.rsw` is launched at its place
by its maker (`world/scenery/placed.rs`) whatever it is made of: one played from files needs nothing, one
drawn by code is an arm of `world/prims/places.rs` (it is told the map's four figures: `Course::figures`).
What falls over the whole view is an arm of `world/prims/weather.rs`, launched on the player's own character by
a server's map flag (`world::burst`) or by the client's own table (`scenery::weather`).

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
spot made for it (`skill_units::spot`, the client's `LaunchDummyEffect`).

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
- Types live in the parent module (`app.rs`, `world.rs`, `hud.rs`), behaviour in the children; a child starts
  with `use super::*;`.
- A file that passes ~500 lines is split by responsibility (`tools/split_module.py` does the moving). The
  tests of a file that has many are in `<file>/tests.rs`.
- No game data is committed.

## Build, run, test

```
cargo test -p roweb-client                       # formats and protocol
./build-web.sh                                   # client -> client/dist (wasm + page)
./dev.sh                                         # the same, again at every save; open the page with ?dev
cargo run --release -p roweb-gateway             # serves http://localhost:8080
docker compose -f server/docker-compose.yml up   # a local rAthena
```

Test accounts for the local server are in `server/conf/accounts.sql`.

## Known debts

- One function is over 100 lines, a plain dispatcher (`actions::handle`): it is named in `tools/limits.py`,
  which lets no other through.
- Three states of the box that asks how many (`Bag::throwing`, `Exchange::counting`, `Mail::counting`) and
  three windows for it, where the client has one window: the box, its place and the reading of its number
  are written once, but each system still keeps its own.
- Some lines of the client's message table are said through `data::message(..)` with the English beside them
  and not through `App::words`, which also translates: they are to be looked at one by one.
- What needs no page is being moved out of `web/` so that it can be tested (`cursor.rs`, `prim.rs`, the rules
  in `net/`); the windows and the world's drawing are still checked by hand in the game.
- A window's inside is still written as a string of HTML by its drawing function, and written whole when it
  changes: there is a list of windows with one interface (`hud/windows.rs`), not a tree of elements compared and
  patched. By choice: the windows are small, and each is placed by the client's own coordinates.
