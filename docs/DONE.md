# What is done

What roweb has implemented after the Windows client, system by system: where the code is, where in the
decompiled client (`rodecomp`, beside this project) it was read from, and how far it has been verified.

Each system ends with its **state**, one of three:

- **seen in the game** — the owner or the lead looked at it in the running game and it behaved as in the
  Windows client. This list is short on purpose: it holds only what someone actually looked at, not what is
  believed to work.
- **tests only** — its rules (`client/src/<x>.rs`) or its packets (`client/src/net/<x>.rs`) have unit tests,
  but nobody has looked at it in the game.
- **by reading only** — written from the decompiled code, with neither a test nor a look in the game. This is
  the usual state of drawing code: no file under `client/src/web/` has a test.

What is known to be wrong or missing is in `docs/DEBTS.md`.

How to read the sections:

- Client paths are under `roweb/client/src/`; gateway paths are written from `gateway/src/`.
- A system's name is the same word in every layer: rules `<x>.rs`, packets `net/<x>.rs`, the app
  `web/app/{incoming,clicks,state}/<x>.rs`, windows `web/hud/<x>.rs`, the world `web/world/<x>.rs`, tables
  `gateway/src/tables/<x>.rs` with `web/data/<x>.rs`.
- Not repeated below: every window also has its entry in `web/hud/windows/<x>.rs`, and what can be pressed in
  it is a value in `press/<x>.rs` (each with a test that it is written and read back).
- "Taken from" names the Windows client's classes and functions as the code's own comments name them. A path
  after a name is the `rodecomp` file it was found in with `grep`; the decompiled files are grouped oddly
  (a window's `SendMsg` is often in a file named after another window), so search by the name, not the path.
  A name without a path was not looked up. Packet layouts are rAthena's (`rathena/src/map/clif.cpp`,
  `packets_struct.hpp`); the `net/` files name the `clif_…` function of each.

## 1. Way in and characters

Login, registration and password recovery, the server list, character select, creation and deletion.

**Files**
- Rules: `account.rs`, `jobs.rs`; the names of the files of a character's parts (body, weapon, shield, `.imf`,
  clothes' palette, each by the client's own table): `dress.rs`, its tables `web/data/job_files.txt` and
  `job_sprites.txt`
- Packets: `net/packets.rs`, `net/packets/chars.rs` (tests in `net/packets/tests.rs`)
- App: `web/app/account.rs`, `web/app/connection.rs`, `web/app/screen.rs`, `web/app/actions/forms.rs`,
  `web/app/{incoming,clicks}/account.rs`
- Windows: `web/view/accounts.rs`, `web/view/characters.rs`, `web/view/creation.rs`; the character painted in
  them: `web/figure.rs`, `web/figure/compose.rs`, `web/figure/weapon.rs`
- Tables: `gateway/src/tables/jobs.rs`, `tables/figures.rs`; `web/data/jobs.rs`, `web/data/figures.rs`
- Gateway (accounts are this project's own, the game protocol has none): `gateway/src/accounts.rs`,
  `accounts/rules.rs`, `accounts/recover.rs`, `bridge/guard.rs`, `limiter.rs`

**Taken from**
- `UINewSelectCharWnd` (`Source/UI/Login/UINewSelectCharWnd.h`), `UINewMakeCharWnd`, `UINewMakeCharWnd::Paint`
  (`Source/UI/Login/UINewMakeCharWnd.cpp`), `UILoginWnd::OnCreate`, `UIServerListBox`
  (`Source/UI/Lists/UIListBox.h`)
- `CSession::IsDoramJob` (`Source/Client/Session/Session.cpp`), `CSession::IsTreeGroupThree`
  (`Source/Contents/Jobs/JobInheritance.cpp`), `jobinheritlist.lub`
- `CPc::SelectStanceFrame`, `CPc::MapMotion`, `CPc::ApplyLayerEffects` (`Source/Contents/Actors/Player.cpp`),
  `SpriteLayerRenderer::Render`, `CSession::m_robeNames`
- The names of the parts' files: `CSession::InitResourceNameTables` (`m_bodyJobNames`, `m_imfNames`,
  `m_shieldJobNames`, `m_palJobNames`, the last by `g_serviceType`) and `CSession::InitJobSpriteNames`
  (`m_jobSpriteNames`, the weapons) in `Source/Client/Session/Session.cpp`, written out by
  `tools/client-tables/job_files.py`; `CSession::GetSpriteJob` (`Source/Contents/Sprites/SpriteJob.cpp`),
  `EquipmentResourceMgr::GetWeaponResourceName`, `GetShieldResourceName`, `GetBodyImfName`, `GetBodyPalName`
  (`Source/Contents/Jobs/JobInheritance.cpp`, `Actors/Resources/*.cpp`), `CSession::GetBodyStyle`
  (`Session.cpp:1909`: the colour shown for the one told); the service from `clientinfo.xml`
  (`ReadClientInfo`, the gateway's `korea`)

**State**
- Login and character creation: seen in the game.
- The names of the parts' files: tests (`dress.rs`), and every classic job, sex, kind of weapon, shield and
  colour asked of the gateway by `tools/sprite_files.py` (what is still missing there, the client's files do
  not have either). Seen in the game (2026-11-21, a male character): each reborn second job after `@job`
  (4008 to 4013, 4015 to 4020) with the weapon of its kind and, where it carries one, a Guard, in battle
  stance and striking, the head on the body; the requests show the weapon, the shield and the `.imf` taken
  from the folders of the plain second jobs, none missing. `@dye` on a Swordman, a Knight, a Clown and a
  High Thief: colours 2 to 4 each differ, 0 and 1 are the sprite's own, a High Thief wears its palette 1.
  A reborn second job has no palette 4 in the client's files (`크라운_남_4.pal`, `로드나이트_남_4.pal`…: the
  gateway answers 404, as `tools/sprite_files.py` says), so it stays in its own colours there. A female
  character and a Gypsy were not seen: their files were only asked of the gateway (all there).
- What an ID, a password and an e-mail may be, the packets of the login and character servers, the gateway's
  accounts, guard and limiter: tests only (`account.rs`, `net/packets/tests.rs`, `gateway/src/accounts/tests.rs`,
  `limiter.rs`).
- The forms of registering and recovering, deleting a character: by reading only.

## 2. Movement

Walking, the server's clock, routes, pursuit, and everyone else in sight walking and changing their look.

**Files**
- Rules: `route.rs` (tests `route/tests.rs`), `server_time.rs` (tests `server_time/tests.rs`), `map/paths.rs`,
  `map/sight.rs` (tests `map/sight/tests.rs`), `map/gat.rs`, `pose.rs`, `mounts.rs`, `cursor.rs`; what is drawn for an object that is no player, by its
  job alone (a character's body, or one sprite and its folder): `creatures.rs`
- Packets: `net/packets/zone.rs`, `net/actors.rs`, `net/actors/changes.rs`, `net/actors/ground.rs`,
  `net/actors/requests.rs` (tests `net/actors/tests.rs`)
- App: `web/app/incoming/travel.rs`, `web/app/incoming/actors.rs`, `web/app/actions/travel.rs`,
  `web/app/actions/pointer.rs`
- World: `web/world/mover.rs`, `web/world/pursuit.rs`, `web/world/pose.rs`, `web/world/session.rs`,
  `web/world/actors.rs`, `web/world/players.rs`, `web/world/pick.rs`, `web/world/input.rs`,
  `web/world/input/{pointer,press,wheel}.rs`, `web/world/overlays/pointer.rs`
- Gateway: `gateway/src/mapcache.rs` (the server's map cache), `bridge/lag.rs` (the debug delay `--lag`,
  `--jitter`)

**Taken from**
- `CSession::SetServerTime`, `CSession::GetServerTime` (`Source/Contents/Time/ClientTiming.cpp`),
  `CGameMode::SendSyncRequest` (`Source/Network/Packets/GameTickPacket.cpp`)
- `CGameActor::FindRoute` (`Source/Contents/Movement/ActorMovementRoute.cpp`), `CPathFinder::FindPath`
  (`Source/Contents/Movement/TerrainPathSearch.cpp`), `CPathFinder::BuildMovementRoute`
  (`Source/Contents/Movement/MovementRoute.cpp`); the server's search is rAthena's `path.cpp`
- `CGameActor::ProcessState`, `CGameActor::ProcessMotionWithDist` (`Source/Contents/Actors/GameActor.cpp`),
  `CGameActor::SetState`, `CRenderObject::SetFrameSequence` (`Source/Contents/Actors/RenderObject.cpp`)
- `CPlayer::UpdatePendingAction`, `CPlayer::SendMsg`, `CPc::SetState` (`Source/Contents/Actors/Player.cpp`);
  the messages `AM_MOVE`, `AM_STOPMOVE`, `AM_STANDENTRY`, `AM_OUTOFSIGHT`
- `CGameMode::ProcessGroundClick` (`Source/Client/Mode/GameMode.cpp`), `CMode::DrawCursor`
  (`Source/Client/Mode/ModeCursorDrawing.cpp`)
- Who is what: `CGameMode::SpawnMovingActor` (`Source/Client/Mode/GameModePackets.cpp:14402`: object type 0
  is a `CPc`, the rest `NewActor`), `CGameMode::NewActor` (`GameMode.cpp:8279`: `CCharacterNpc` for
  `IsPlayerJob`, a model for `.gr2`, else `CNpc`), `CNpc::SetSprJob` (`Source/Contents/Sound/SoundMaker.cpp:926`:
  `NPC\` for `IsNpcJob`, `인간족\몸통\` for 6017-6046, `homun\` for 6001-6052, else `몬스터\`),
  `CSession::IsNpcJob` and `IsPlayerJob` (`Source/Contents/Jobs/JobInheritance.cpp:112`, `:720`),
  `CCharacterNpc::SendMsg` (`Source/Contents/Actors/RenderObject.cpp:2728`), ZC_NPCSPRITE_CHANGE →
  `CNpc::SetSprAct` (`GameModePackets.cpp:26252`)

**State**
- Walking, movement by the server's clock under lag, being knocked back by Fire Wall, the sprites of a job
  after a job change: seen in the game.
- The clock, a route's times, the path search, poses, mounts, the pointer's pictures, the packets of those in
  sight: tests only.
- Pursuit (walking on towards someone to attack or to pick up, `web/world/pursuit.rs`): by reading, and:
- Going round what is in the way to attack, use a skill or pick up (this project's own, see DEBTS;
  `map/sight.rs`: the server's line of sight `path_search_long` and reach `battle_check_range`, the way to
  the nearest cell in reach): tests for the line, the reach, the cell chosen, a cell somebody stands on and
  giving up. Seen in the game (2026-11-26, moc_fild03, the packets sent written down): a Hunter with a bow at
  250,253 and a Red Plant at 260,258 behind the tree on 255..256,254..255 — one walk request to 250,254 and one
  attack, the arrows land (with `/farwalk` off, as before and as the client: the attack asked every 1.5 s,
  no answer, the character stands — seen on moc_fild12 at 247,266 by the stone lantern); from 262,266 at a
  plant on 254,280 beyond the rock, out of reach and not seen — a walk to 260,266 and the attack; Double
  Strafe from 251,254 at a plant on 263,254 behind the tree — a step to 251,253 and the skill; a Resin lying
  behind the tree picked up from 252,254 (a walk to 256,256 round it, then the pickup); a Knight with bare
  hands from 262,266 at a plant on 258,280 — one walk of 13 cells round the rock to 257,279 and the attack;
  on moc_fild12 a Knight from 248,266 round the stone lantern (251..253,265..267). Not seen in the game: a way
  longer than one stretch in a chase, giving up, a target that walks.
- The sprite of one that is no player, by its job: tests (`creatures.rs`), and every monster of the classic
  database, every sprite the classic scripts give an NPC, the homunculi and the mercenaries asked of the
  gateway by `tools/sprite_files.py` (none missing). Seen in the game (2026-11-21): the eight homunculi
  6001 to 6008, each drawn, the Lif following and fighting what Alt and the right button name twice; the
  three mercenaries of the scrolls 12153, 12163 and 12173, each a whole figure that follows and fights; the
  monsters of the museum (`yuno_in03`: Deviace, Seal, Sage Worm, Penomena) with their names under the
  pointer. `setnpcdisplay` was not seen.

## 3. Fight

Blows, the numbers and words that fly, flinching, death, the name under the pointer, PVP maps.

**Files**
- Rules: `blow.rs`, `blow/landing.rs`, `blow/signs.rs`, `blow/flight.rs`, `names.rs`, `pvp.rs`
- Packets: `net/actors/requests.rs` (to attack), `net/fight.rs`, `net/pvp.rs` (tests `net/pvp/tests.rs`)
- App: `web/app/incoming/combat.rs`, `web/app/{incoming,clicks,state}/fight.rs`,
  `web/app/{incoming,state}/pvp.rs`
- Windows: `web/hud/fight.rs` (Monster Info of Sense, the MVP's reward), `web/hud/pvp.rs`
- World: `web/world/combat.rs`, `web/world/combat/landing.rs`, `web/world/overlays/bars.rs`,
  `web/world/names.rs`, `web/world/pvp.rs`, `web/world/frame/sprites.rs`

**Taken from**
- `CGameActor::ProcessHitMessage`, `CGameActor::ProcessSkillMessage`, `CGameActor::SpawnDamageDigits`,
  `CGameActor::GetAttackMotion` (`Source/Contents/Actors/RenderObject.cpp`); the messages `AM_ATTACKED`,
  `AM_WILL_BE_ATTACKED`, `AM_MAKE_NUMBER`, `AM_MAKE_MISS`, `AM_MAKE_CRITICAL_EFFECT`
- `CMsgEffect`, `CMsgEffect::OnProcess` (`Source/Contents/Effects/Core/MsgEffect.cpp`), `CArrowEffect`
  (`Source/Contents/Effects/Skills/ArrowEffect.h`)
- `CSession::GetHitWaveName` (`Source/Contents/Sound/HitSounds.cpp`), `CSession::GetWeaponHitWaveName`
- `CGameMode::HandleActorPick` (`Source/Client/Mode/GameMode.cpp`), `CGameMode::ActorNameColor`
  (`Source/Client/Mode/GameModeNames.cpp`), `UINameBalloonText::SetName` (`Source/UI/Text/UIBalloonText.cpp`)
- `CGameMode::ReceiveMapProperty` (`Source/Client/Mode/GameModeNotices.cpp`), `CMapPropertyMgr`
  (`Source/Contents/Map/MapPropertyMgr.h`), `UIMonsterInfoWnd`, `CCorpse::OnProcess`

**State**
- Attacking with a bow: seen in the game.
- What monsters shoot (`blow/shots.rs` with its tests, `world::arrow`, `web/world/frame/effects.rs arrows`;
  `CGameActor::LaunchAttackProjectile`, `CArrowEffect::SendMsg`, `CArrowEffect::OnProcess`): a monster sends
  an arrow, or a thing of its own, by its job alone, leaving on the frame of its swing the blow is on; a shot
  from nearer than 8 cells flies a cell a frame and is there sooner (a character's arrow too). Seen in the
  game: the arrows of an Orc Archer and of a Kobold Archer at the player's character. The other sprites
  (Peach Tree, Evil Cloud Hermit, Stone Shooter, Wootan Shooter, Parasite, the Poton Canons, Lady Tanee, Ragged
  Zombie): the files are there; not seen in the game.
- What comes up under one a Mandragora or its like strikes (`blow/shots.rs root`, `root_picture` with their
  tests, `world::arrow`, `web/world/frame/sprites.rs roots`; `CGameMode::SpawnMandraAttack`,
  `CMandraAttackEffect::SendMsg` and `OnProcess`): the first action of the job's sprite played once where the
  one struck stands, from the moment the attack begins. Seen in the game, 2026-11-26: the root of a Mandragora
  and the tentacles of a Hydra at the player's character. Drosera, Nepenthes, Thanatos Odium, Mavka and
  Entweihen: by the table only.
- The playtest of 2026-11-26 (effects, sounds, sprites of the world), seen and heard in the game unless said:
  a smith's refining shows and sounds one effect, not two (ZC_NOTIFY_EFFECT does nothing for the player's own
  character, `ReceivePartyMemberEffect` with `GetGameActorByAID`); the player's own level-up is ZC_PAR_CHANGE's
  (`status.rs level_effect`, `job_level_effect`, `CGameMode::ReceiveParChange`: the angel and `levelup.wav`
  once, the job level's light); `effect_sounds.txt` has one line for a sound written in both branches of an
  option (`tools/client-tables/skill_effects.py`; Detoxification was heard once before and after, Lex Divina
  five times, as the client plays it); Hiding and coming out of it puff smoke (seen slowed down), Cloaking
  does not, as in the client; Chase Walk and the summoner of `EF_SUMMONSLAVE` puff it too (by reading; not
  seen); a weapon's hit sounds by the client's own table, in which a katar is `_hit_mace.wav` (heard: knife
  `_hit_sword.wav`, Jur `_hit_mace.wav`; the Taekwon's bare hands by test only); the cart is drawn after
  relogging and after changes of map and of its kind, with its shadow; a unit on the ground that strikes
  shows its own hit only (`skill_units.rs struck_by_unit`, `CSkill::SendMsg`), so Storm Gust's file plays
  once (heard once where it was three times); the pictures of an effect's file are kept as the client keeps
  them (`bmp.rs as_the_client_keeps`; test only); one who teleports away or logs out leaves the client's
  effect on its place (`pose.rs left_behind`, `CGameMode::ReceiveActorDisappear`; seen and heard of Creamies).
- When a blow lands and how long its swing is shown (`blow.rs pace`, `lands_after`, `swing_lasts`,
  `attack_motion`; `web/world/combat.rs swing`): `attackMotion × render.speed × 24` milliseconds, the speed
  being the one the sprite gives the action times the blow's over 432 (`CGameActor::ProcessHitMessage`,
  `CRenderObject::SetAction`, `ProcessMotion`); a character's `attackMotion` is the figure of its job
  (`CSession::AttackMotionLength`), not a frame of its sprite. What was wrong: the speed the sprite gives the
  action was left out, so an Orc Warrior's blow landed 168 ms into its swing where the client and the server
  (`ClientAttackMotion`, with `synchronize_damage`) have 672, and its swing was shown in half its time: the
  character was stopped, or not stopped, half a second apart from the server, and walked on while the server
  held it. Tests; seen in the game (2026-11-26, a Knight among Orc Warriors, with the packets and `@where`
  beside the client's cell): before, 7 stops of 10 a cell or three away from the server; after, none in 26
  (see DEBTS for the cell that is still left in a crowd).
- The fight as the player's character keeps it (`web/world/pursuit.rs`; `CPlayer::UpdatePendingAction`,
  `CPlayer::SendMsg` in `Source/Contents/Actors/Player.cpp`): a monster struck down is asked nothing more of
  (it was asked for every 1.8 s until its corpse was gone); a place pressed during the character's own blow
  is walked to as the blow ends (`isPreengageStateOfMove`; the press was lost before), and no other walk is
  asked for during a blow; a skill waits until the swing is over (`skills::aim::asked_now`, no figure of this
  project's in it now). After a skill aimed at someone or at a place the client does not go on fighting
  (every way out of `UpdatePendingAction` ends in `AM_CANCELATTACK`): the same here. Seen in the game with
  the packets: no request after the death; Bash asked for 544 ms after a blow of `amotion` 271 and not
  refused; a press on the ground 115 ms into a blow walked 437 ms later, as the swing ended; a Red Potion
  and Endure from the hotkey bar in a fight went at once.
- How many hits a blow is made of, what each shows and sounds like, how the numbers fly, the
  colour of a name, who is fought on a PVP map, the packets: tests only.
- The burst of a critical blow and the word of luck (`web/world/combat/landing.rs shown`): they are the
  third and the fourth action of the WORDS' sprite (`이팩트\msg`: its red spiky burst, shaking from side to
  side over 15 pictures, and the balloon with "LUCKY!!"), not of the digits' (`이팩트\숫자`, which has one
  action: nothing was drawn, so a critical blow was a yellow number alone). `CGameMode::SpawnMsgEffect`
  (`Source/Contents/World/WorldObjects.cpp:1037`, `MAKETYPE_CRITICAL_EFFECT` and `MAKETYPE_LUCKY_EFFECT`)
  names them by `GetDamageSpriteName`, which `CSession::LoadDamageSkinSettings`
  (`Source/Contents/Time/ClientTiming.cpp:803`) fills with the second name of `GetDamageResourceName`:
  `BaseMsg` of `damageskin\damageskinlist.lub`. The rest of a critical blow was as the client has it
  already: the mark greyed (`0xFFAAAAAA`) from three times its size, the number yellow whoever is struck
  (`CGameActor::SendMsg`, message 93), from five times to 1.3, both drifting for 70 frames, `EF_HIT2` on the
  one struck with its sound, the striker's motion that of any blow (`ProcessHitMessage`). Seen in the game
  (2026-11-26, a Hunter with 100 LUK on a Ghoul, frames taken off the canvas): the red burst behind the
  yellow number, shrinking and drifting with it. The word of luck and a monster's critical blow on the
  player: by reading only.
- Whom the pointer picks (`skills/aim.rs rank`, `sprite/canvas.rs drawn`, `web/world/pick.rs`;
  `CGameActor::SetRenderInfo`): a homunculus comes before a player and after a monster (it was ranked as a
  pet, after everyone, so its owner beside it took the pointer), and the rectangle of one drawn is that of
  what its picture shows, as the client's is that of the layers it draws (a character's was the whole sheet
  it is put together on, 157 by 195, about four times its body). Tests; seen in the game: a Lif standing
  half behind its owner is named over all of its 40-pixel box, the owner's box went from 168×204 to 84×96
  pixels of a page 1080 high.
- What the pointer shows over the world when it is there no more (`web/world/input/pointer.rs left`,
  `still_over_world`, asked from `overlays.rs pointed`): the name under it, the mark of its cell and the
  pointer's picture all go by where the pointer is over the world, which was only told by the mouse moving.
  Now the pointer leaving the page takes it off the world (and the words of a skill aimed with it), and every
  frame asks whether the page has the world under the pointer: a window that opens under a still mouse takes
  the name away, and gives it back when it closes (the client asks its windows before
  `CGameMode::HandleActorPick` every frame). Someone walking away or vanishing under a still mouse was
  right already (asked every frame). Seen in the game (2026-11-26): the name of an NPC under the pointer
  went as the Skill Tree and Equipment opened over it by their keys and came back as they closed; the
  pointer leaving the page was tried with the page's own `mouseleave` sent by hand, not with a hand's mouse.
- Death, the bars over heads, the two windows: by reading only. The standing on a PVP map: seen in the game
  (2026-11-26, "1 / 1" in the corner).

## 4. Skills and effects

Casting, skill units and traps, the effect primitives, played `.str` files, the effects of states, the aura,
the falcon, spirit spheres.

**Files**
- Rules, skills: `skills.rs`, `scope.rs`, `skill_units.rs`, `skill_units/pulse.rs`, `skill_units/traps.rs`,
  `quake.rs`, `song_talk.rs`, `effect_tables.rs` (+ `effect_tables/`)
- Rules, effects: `prim.rs` and `prim/` (the kinds of `CEffectPrim`; `prim/tei/` holds those built corner by
  corner), `effect.rs` and `effect/` (`strm.rs`, `transform.rs`, `played.rs`, `carried.rs`, `flights.rs`)
- Rules, states on a body: `state_effects.rs` and `state_effects/`, `ailments.rs` and `ailments/`, `body.rs`
  and `body/`, `aura.rs`, `falcon.rs`, `spirits.rs`
- Packets: `net/skills.rs`, `net/skill_units.rs`, `net/spirits.rs`, `net/inventory/skills.rs`
- App: `web/app/incoming/combat.rs`, `web/app/incoming/skill_units.rs`, `web/app/incoming/falcon.rs`,
  `web/app/incoming/spirits.rs`, `web/app/incoming/status.rs`, `web/app/{clicks,state}/skills.rs`
- Windows: `web/hud/skills.rs` (the skill tree, a skill's description, the hotkey bar)
- Tables: `gateway/src/tables/skill_needs.rs` ↔ `web/data/skill_needs.rs`: the SP of each level and what a
  job's skill asks for, by the rules the server plays (its `skill_db.yml`, `skill_tree.yml`; the owner's
  decision of 2026-11-21, `docs/DEBTS.md`). Used by the hotkey bar's tip ("… Use Lv n (Sp : n)",
  `SkillTooltipFlat`), the list of a companion's skills ("Sp : n", `UISkillListWnd`), the tree's marks of the
  skills asked for (`UINewSkillListWnd::UpdateRequiredSkills`, `DrawSkillTree`) and the plus
  (`skills::raises`, after `IncreaseSkillLevel`). State: seen in the game on a Magician (2026-11-21): the marks while
  a skill is pointed at (Fire Wall: Fire Ball 5, Sight 1) and the bar's tip ("Cold Bolt Use Lv 5 (Sp : 20)");
  tests only: the plus on a skill whose needs are lacking, the marks kept by the right button with the
  coloured tab, the companion's list.
- World: `web/world/prims.rs` and `web/world/prims/` (one file per family of effects), `web/world/effects.rs`,
  `web/world/launch.rs`, `web/world/skill_units.rs`, `web/world/skill_units/traps.rs`, `web/world/scope.rs`,
  `web/world/state_effects.rs`, `web/world/ailments.rs`, `web/world/aura.rs`, `web/world/falcon.rs`,
  `web/world/spirits.rs`, `web/world/written.rs`, `web/world/frame/effects.rs`, `web/world/quads.rs`,
  `web/world/gpu/layers.rs`
- Tables: `gateway/src/tables/skills.rs`, `tables/aura.rs`; `web/data/skills.rs`, `web/data/aura.rs`,
  `web/data/song_talk.rs`

**Taken from**
- `CRagEffect`, `CRagEffect::OnProcess` (`Source/Contents/Effects/Core/RagEffect.cpp`): one arm of it per
  effect number (`EF_…`)
- `CEffectPrim`, `CEffectPrim::OnProcess` (`Source/Contents/Effects/Primitives/EffectPrimitiveMotion.cpp`),
  `MoveEffectPrimitive`, `RenderTeiRect` (`Source/Contents/Effects/Primitives/EffectPrimitiveRendering.cpp`)
- `CSession::PlaySkillEffect` (`Source/Contents/Jobs/JobInheritance.cpp`): which effect a skill plays
- `CSkill` (`Source/Contents/Actors/Skill.h`), `CSkill::OnProcess`, `CSkill::ShowModel`,
  `CGameActor::PlayGroundSkillEffect` (`Source/Contents/Actors/RenderObject.cpp`)
- `CGroundScope`, `CGroundScope::Render` (`Source/Contents/Actors/GroundScope.h`)
- `CGameActor::SetStatusStates` (`Source/Contents/Actors/RenderObject.cpp`),
  `CGameMode::ApplyActorStatusEffects`, `CGameMode::ReceiveSkillUse`
  (`Source/Client/Mode/GameModePackets.cpp`), `CPlayer::ReapplySkillEffects`
- `CLevel99Effect` (`Source/Contents/Effects/Skills/Level99Effect.h`), `CHalluPC`
  (`Source/Contents/Actors/Pc.h`), `CMsgEffect` (the falcon and what else follows its owner; `AM_MAKE_BIRD`)
- `UINewSkillListWnd` (`Source/UI/Skill/UINewSkillListWnd.h`), `UISkillDescribeWnd`, `UIShortCutWnd`
- The message `MM_QUAKE`; the files `BA_frostjoke.txt`, `DC_scream.txt`

**State**
- Seen in the game: Double Strafe, Arrow Shower, Improve Concentration, Blitz Beat, the flight of Detect, the
  falcon by its owner, Ankle Snare set and sprung, Land Mine set and sprung, Skid Trap and Claymore Trap set,
  traps standing as models, Play Dead, the level-up effect, the skill tree with its points.
- Seen in the game in the playtest of the Hunter (Zombies and Orc Warriors walked into the traps, on
  `pay_fild05` and in Izlude): Skid Trap set (with its sign, effect 69) and sprung (its model plays, the
  monster is moved by the server's ZC_HIGHJUMP), Flasher sprung (the flash, effect 99, is drawn; the monster
  is blinded), Sandman sprung (the monster sleeps), Freezing Trap sprung (the monster in ice), Shockwave Trap
  set (its effect plays over it without end) and sprung, Blast Mine sprung (its number and flash) and left
  alone (the server only takes it away: nothing goes off), Claymore Trap sprung (number and fire), Talkie Box
  set through the box of its words (by OK and by Enter; not sprung: no monster was brought onto its cell),
  Spring Trap on a trap (the falcon flies to it, the trap's model plays), Remove Trap on a trap (the character
  walks into reach, the trap is taken back), Detect (the falcon and a faint light; nobody hidden was there),
  Charge Arrow (the cast under the target, the number; the server moves the monster 6 cells), Arrow Crafting
  (the list, Trunk made into arrows), the falcon's own Blitz Beat while shooting, the rows of Beast Bane,
  Falconry Mastery and Steel Crow in the tree. The sizes of the traps' models on the screen: about 2 cells
  wide for Skid Trap, Sandman, Claymore; about 1.5 for the six-sided ones and the Talkie Box
  (`CSkill::ShowModel` gives a model no scale: `C3dActor` begins at 1, and so does `Models::raise`).
- Seen in the game in the playtest of the swordsman's and the mage's lines (used on Pupas, Dragon Eggs and a
  Barricade; looked at in still pictures, so what moves fast was seen in part): Bash, Magnum Break, Provoke,
  Endure, Auto Berserk; Bowling Bash, Pierce, Spear Stab, Spear Boomerang, Brandish Spear on a Peco, Twohand
  Quicken, Auto Counter, Charge Attack; Auto Guard with the guard of a blow warded off, Grand Cross with the
  numbers on its caster, Holy Cross, Shield Boomerang, Reflect Shield, Defender, Shrink, Spear Quicken (the
  body's colour and the copies left while walking); the three bolts at level 10 with their totals, Soul
  Strike, Napalm Beat, Fire Ball, Fire Wall, Frost Diver with the ice and its breaking, Stone Curse's two
  stages, Safety Wall, Thunder Storm, Sight, Energy Coat; Ice Wall, Sense with the Monster Info window; a
  skill's description, its place on the hotkey bar and its use by the bar's keys; the lines "Skill has
  failed.", "Insufficient SP" and "The skill cannot be used with this weapon."; a stat raised in the Status
  window. Not seen there: Heal, Cure, Providence and Devotion (they want someone to be used on), what a
  blow knocks back, a cast broken by a blow (once, unclearly), Pierce's hits by size and their times
  (tests only), an Ice Wall walked into or struck.
- Tests only: how the kinds of primitive live and are shaped (`prim/tests.rs`, `prim/particle_tests.rs`,
  `prim/slots_tests.rs`, `prim/walls_tests.rs`, `prim/horn_tests.rs`, `prim/tei/tests.rs` and the tests in the
  kinds' own files), the `.str` format and how a carried file flies (`effect/tests.rs`,
  `effect/carried/tests.rs`), units, traps and the circle under a cast, what states begin, end and keep
  (`state_effects/tests.rs`, `ailments/tests.rs`, `body/tests.rs`), the aura, spirit spheres, the packets.
- Seen in the game (2026-11-21, play5): the aura of the highest level, full as the client starts it (the
  light on the ground and the three rings under the character, the blue lights over it) on dark ground
  (Niflheim) and light (`guild_vs3`), `/aura` to the lights alone and back; that the effects of a map left do
  not go on on the next (`world::enter` forgets them: the snow of `xmas` fell on every map after it); the
  effects a script sends, each showing and ending: 4, 5, 6, 9, 12, 20, 32, 34, 36, 50, 53, 54, 57, 59, 63,
  71, 72, 99, 118, 120, 141, 159, 224, 304, 314, 328 (with Asura's letters), 342, 343, 348, 362, 363, 364,
  373, 563 (looked at in small pictures: that something shows and ends, not each shape against the client).
- By reading only: every other skill's effect as it is drawn (all of `web/world/prims/` beyond the skills
  above), casting bars and aiming, the aura and the spirit spheres on the screen.

**What the audit of the classic game found silent or not drawn** (added after it; none of it seen in the game)
- Sounds of effects that are files: `tools/client-tables/skill_effects.py` now reads the second switch of
  `CRagEffect::OnProcess` too (the effects under 300 and 374 to 490, a step deeper than the first), every
  sound of an arm with its tick; `effect_tables/effect_sounds.txt` grew by 100 lines of 87 effects, 44 of them
  files that were silent (Thunder Storm, Angelus, Sanctuary, Kyrie, Magnus, Gloria, the traps, Endow, Strip,
  the MVP…). `effect_tables::effect_sounds`, `effect/played.rs plain`. An arm that hands the effect to a
  function of its own (`if (!this->Aspersio())`, `this->TwoHandQuicken();`) is read one step in too
  (`timed_sounds` of the tool): that gave the sounds of Aspersio (86, tick 70), Lex Divina (87, five), Storm
  Gust (89), `EF_GLASSWALL` (13), the bomb of a Fire Pillar (102, tick 30), Two-Hand and Spear Quicken (130,
  250), Improve Concentration (153), Cart Revolution (170, ticks 7 and 20), the berserk potion (220), Magic
  Rod (244). State: heard in the game (2026-11-21, by the files the page plays and their moments, not by
  ear): `@effect` 83, 30, 112 (b at once, a at tick 15), 68, 86, 87, 89, 130, 153, 170, 244, 102, 13, 220;
  by real skills Angelus, Kyrie, Magnificat, Gloria, Impositio (tick 60), Sanctuary, Meteor Storm (a sound a
  meteor), Heaven's Drive, Fire Wall, Sight, Sightrasher, Quagmire (by lot, again at each launch), Steel Body
  (the handler's and the state's sound together, as in the client); the potions 501, 502, 505, 506. Lord of
  Vermilion and a waiting Fire Pillar are silent, as in the client. Sounds from afar came quieter (0.26 to
  0.72), one's own whole.
- Sounds a skill's handler starts beside its effects (`g_soundMgr->PlayWave` in `CGameMode::ReceiveSkillUse`,
  `CSession::PlaySkillEffect`, `CGameActor::PlayGroundSkillEffect`, `CGameMode::ReceiveNotifyGroundSkill`):
  the generated `effect_tables/skill_sounds.txt` (238 sounds of 228 skills, 82 of them classic: the songs and
  dances, Berserk, Aura Blade, Gospel, Basilica, Magic Power, the Taekwon's, the Gunslinger's…),
  `effect_tables::skill_sounds`, played by `App::skill_sounded` (`web/app/incoming/combat.rs`) through
  `world::sound_on` and by `world::ground_skill`. State: tests for the table; Steel Body's heard in the game
  (2026-11-21), the songs' and the others' by nobody yet.
- The monk's combos (`blow/combos.rs`, `App::combo_shown`; `CGameActor::ProcessSkillMessage`,
  `RenderObject.cpp:5688`), Spring Trap and Remove Trap on a trap (`skill_units/traps.rs used_with`,
  `world::skill_unit_used`). State: tests only for the rules of the combos; the two skills for traps were
  seen in the game after traps became things to point at: a skill the server calls one for a trap (its
  kind 32, the client's `skillUseType` 5) is aimed at a trap standing on the ground and used on the trap's
  own ID (`web/world/pick.rs trap_at`, `web/world/input/press.rs`, `web/world/pursuit.rs`, `world::aim_traps`;
  `CSkill::SetRenderInfo`, `CActorPickNode::Pick`, `CGameMode::HandleActorPick`, the pick of the kind 2).
- Effects drawn by code, each from its function of `CRagEffect` (the comment over each says which):
  `prims/fields.rs` (Detect 119, Heaven's Drive 142, Ganbantein 223, Volcano 225, Deluge 236, Violent Gale
  237, Land Protector 238, Demonstration 302, EF_PORTAL3 341, Gospel 370, Basilica 374, Spider Web 404,
  Slim Potion Pitcher 497 to 499, Hermode 517, Gravitation 522, Suiton 620, the shake of Meteor Storm 92,
  whose files are `effect/played.rs`), Fog Wall's box (`skill_units/pulse.rs`, kind 2), `prims/played.rs`
  (Tatami, Kaensin, Ground Drift 631, 634, 638 to 642; the hit of poison 53; the sprites items show: 363, 364,
  509, 519, 576 to 580, 682 to 686, 707), `prims/hits.rs` (4, 5, 32, 50), `prims/lights.rs` (9, 63, 72, 100,
  438), `prims/bursts.rs` (Flasher 99), `prims/heals.rs` (14; 7 is now `Exit`, as in the client),
  `prims/sprites.rs` (20), `prims/sprays.rs` (227, 262), `prims/puffs.rs` (462), `prims/blows.rs` (376),
  `prims/missiles.rs` (512), `prims/walls.rs` (the cast circles 12, 54 to 57, 59 through `world::burst`; 328,
  342, the heart 343), `prims/arrivals.rs` (6, 34, 36, 304, 314), `prims/scenes.rs` (71, 118, 362, 563),
  `prims/winds.rs` (224), `prims/marks.rs` (159), `prims/auras.rs` (348), `prims/pillars.rs` (581),
  `prims/bodies.rs` (Cloaking 120, with `Touch::alpha`). New kinds of primitive with their tests:
  `Spike` of `prim/horn.rs`, `Cast::Fire`, `Walls::Rise`, `Shape::Ghost` (`prim/ghost.rs`),
  `Shape::CastingLow` (`prim/low_walls.rs`), and in `prim/tei/`: `map_pillars.rs`, `sheets.rs`, `spray.rs`,
  `floating.rs`, `spikes.rs`, `domes.rs`, `bottom.rs`, `fences.rs`, `cubes.rs`. `skill_units.rs` has the
  rows of the later classic kinds of units (177 to 197) and of `ground_effect` (459, 478, 483, 484; 491 on
  the caster). State: tests only for the primitives and the rules; drawn by reading only.
- An NPC that is there without being seen or pressed (jobs 139 and 32767) is a place effects play on
  (`web/world/actors.rs unit_shown`, `Scene::spots`): by reading only.
- The hidden NPC (job 111, `JT_HIDDEN_NPC`) is there with nothing drawn, no shadow either, and is pressed,
  pointed at and named as any NPC: `creatures.rs` (`Shape::Hidden`, with its test), `Look::Unseen`
  (`web/world/mover.rs`), its rectangle for the pointer in `web/world/pick.rs` (`touched`), the emotion it
  may show in `web/world/frame/sprites.rs` (`emoting`). Taken from `CNpc::Render` and `CNpc::SetState` at
  `JT_HIDDEN_NPC` (`Source/Contents/Sound/SoundMaker.cpp:1048, 1598`), `CSession::IsVisibleJob`
  (`Source/Contents/Jobs/JobInheritance.cpp:1212`), `CGameActor::SetRenderInfo`
  (`Source/Contents/Actors/RenderObject.cpp:869`). State: seen in the game (2026-11-21): the plates by the
  museum's monsters in `yuno_in03` open their texts when pressed; before, nothing there could be pressed.

**The words of a Talkie Box and of Graffiti** (added with the classic gaps)
- Files: `skills.rs` (`words_asked`), `net/skills.rs` (`use_on_ground_with_words`, 0x0367 of 31 bytes, with its
  lines in `net/golden.txt`), `web/app/clicks/skills.rs` (`skill_aimed`, `skill_words_typed`),
  `web/hud/skills.rs` (`skill_words_window`), the entry `SKILL_WORDS` of `web/hud/windows/skills.rs`.
- Taken from: `CPlayer::SendMsg`, `AM_REQ_GROUND_SKILL` (`Source/Contents/Actors/Player.cpp:3846`),
  `UITalkboxTrapInputWnd` (`Source/UI/Dialogs/UIMessageBox.cpp:824, 1505, 1986`), `UIGraffiStrboxWnd`
  (`Source/UI/Skill/UIGraffiStrboxWnd.cpp`, `Source/UI/Acceptance/UIChooseSellBuyWnd.cpp:635, 5404`),
  `CGameMode::SendMsg`, `MM_USE_SKILL_TOGROUND_WITH_TALKBOX` (`Source/Client/Mode/GameMode.cpp:3526`); the
  packet's length from rAthena's `clif_shuffle.hpp` and `TALKBOX_MESSAGE_SIZE`.
- State: tests for which skills ask, for how many letters, and the packet; the box of a Talkie Box was seen
  in the game (opened, typed into, sent by OK and by Enter, the box of wood then stands there). Graffiti's
  box, and what a Talkie Box says when stepped on (ZC_TALKBOX_CHATCONTENTS), by reading only.

**The area a skill warns of while it is cast** (added with the classic gaps)
- Files: `scope.rs` (`warned_cells`, `warned_alpha`, `WARNED_HALF`), `net/skills.rs` (`warned`, 0x0a41),
  `web/world/scope.rs` (`area_warned`, the planes drawn; `warning_plane.tga` is read when first wanted),
  `web/app/incoming/combat.rs`, `gateway/src/tables/skills.rs` (`scale`: the skills' `SkillScale`) with
  `web/data/skills.rs` (`skill_scale`).
- Taken from: `case skillscale` (`Source/Client/Mode/GameModePackets.cpp:28162`),
  `CGameMode::SpawnSkillWarningPlanes` (`Source/Client/Mode/GameModeHandlers.cpp:1922`),
  `CWarningPlaneEffect::AddPlane`, `IsAlive`, `OnProcess`, `Render`
  (`Source/Contents/Effects/Skills/MonsterSkillEffects.cpp:605`), `GetSkillScale` of
  `skillinfoz\skillinfo_f.lub` over `SkillScale` of `skillinfolist.lub` (68 skills have one).
- State: tests only (the cells, the pulse, the packet, the gateway's table); the planes on the ground are by
  reading only.

**The action a monster is shown in for a skill** (added with the classic gaps)
- Files: `monster_skills.rs` (`shown`: the rule, with `jobs::monster`), `gateway/src/tables/monster_skills.rs`
  (`data\MonsterSkillInfo.xml`: 96 monsters, a table only a client set up for Korea finds, as this one is)
  ↔ `web/data/monster_skills.rs` (`data::monster_skill`, asked by the name of the job's sprite in small
  letters), `web/world/combat.rs` (`struck`, where a skill lands) and `web/world/mover.rs` (`Mover::shows`,
  `Pose::loops`).
- Taken from: `CSession::GetMonsterSkillInfo` (`Source/Contents/Monster/MonsterXmlActions.cpp:43`),
  `CSession::LoadXmlTables` (`Source/Contents/Tables/ClientXmlTables.cpp:16`), its two callers that act on it,
  `CGameMode::ReceiveNotifySkill` and `ReceiveActorSkillAction` (`Source/Client/Mode/GameModePackets.cpp:4178,
  4252`: `SetAction(action * 8 - 8, 1, loopType == 0)`, for a monster that is not the player's own
  character), `CRenderObject::SetAction` and `ProcessMotion` (`Source/Contents/Actors/RenderObject.cpp:507,
  343`: `MT_ONETIME`, `MT_LOOP`), `DecodeMessageRot` (`Source/Contents/Tables/MessageTable.cpp:5040`).
  `ReceiveNotifyGroundSkill` (`GameModeNotices.cpp:4303`) asks the table and does nothing with the answer,
  and ZC_USE_SKILL does not ask: so here.
- State: tests only (the rule, the gateway's reading of the file); the monsters in their actions are by
  reading only. To see it: `@monster 1101` (Baphomet Jr.) or an Orc Warrior (1023, which smokes with skill 195
  round and round) and let it use its skills; `@spawn`ed Yoyo and Choco heal and strike in actions 6 to 8.

**Aiming a skill at someone: Shift, `/noshift`, whom the pointer picks** (added in the playtest of the merchant,
acolyte and thief lines)
- Files: `skills/aim.rs` (`support`, `players_first`, `rank`, `goes_on_monster`, `goes_on_player`, with tests),
  `net/inventory/skills.rs` (`Skill::on_friend`), `web/world/pick.rs` (`pointed_at` by rank),
  `web/world/state.rs` (`State::players_first`, `aim_goes_on`, `shift`), `web/world/input/press.rs` (`aimed`),
  `web/world/input.rs` (Shift told by the keys), `web/world/overlays/pointer.rs` (the pointer of a skill),
  `web/world/hooks.rs` (`Options::no_shift`, `Hooks::aimed_own`), `web/app/clicks/skills.rs` (`own_aimed`).
- Taken from: `IsSupportSkill` (`Source/Contents/Skills/SkillInfo.cpp:690`), `CGameActor::SetRenderInfo`
  (`Source/Contents/Actors/RenderObject.cpp:869`: the ranks, `g_pickInvert`, `g_pickModifierKey`),
  `CGameMode::HandleActorPick` and `ProcessActorClick` (`Source/Client/Mode/GameMode.cpp:10499, 8132`),
  `MM_HANG_USE_SKILL` (`GameMode.cpp:2901`), `MM_USE_SKILL` (`GameMode.cpp:2078`: the ways of use by a
  skill's kind).
- What was wrong: the skills held back from a monster without Shift were the list of another function
  (`IsOptionalSelfTargetSkill`), so Decrease Agility, Turn Undead and Lex Aeterna could not be used on a
  monster without Shift; an attack went on a monster with Shift held too, and on another player without; the
  pointer showed a skill's second picture over any monster; a skill for a friend (Heal, Blessing, Potion
  Pitcher, Repair Weapon) could not be aimed at the player's own character at all; with a skill of help the
  pointer still picked a monster before a player.
- Also here: a skill told to be used waits out the character's own blow (`skills::aim::asked_now`,
  `web/world/pursuit.rs reach`; `CPlayer::UpdatePendingAction`, `Source/Contents/Actors/Player.cpp:2973`): asked
  for in the middle of a fight it was refused by the server every time ("Cannot use … due to cooldown delay",
  or nothing at all).
- State: tests for the rules. Seen in the game (2026-11-21, a Priest and an Assassin): Heal on a Zombie with
  Shift held hurts it (the number, the Heal effect, its sound) and without Shift is not used; the pointer over
  a monster is the skill's first picture with Shift up and its second with Shift held, changing as the key
  goes down and up; with `/ns` on ("[no shift] option activated. [ON]") Heal goes on a monster without Shift;
  Decrease Agility, Lex Divina, Lex Aeterna, Turn Undead and Holy Light go on a monster without Shift; Heal,
  Blessing, Increase Agility, Kyrie Eleison, Status Recovery, Cure and Potion Pitcher on the player's own
  character; Heal with Shift and Sonic Blow in the middle of a fight are used. Not tried: another player.

**A thing thrown** (added in the same playtest)
- Files: `prim/tei/thrown.rs` (the client's primitive 53, with tests), `web/world/prims/thrown.rs` (the
  effects 298, 299, 308, 541, 600, 613 to 616), `tools/client-tables/skill_effects.py` (reads
  `LaunchTargetEffect` in `CSession::PlaySkillEffect`: `effect_tables/skill_hits.txt` has Acid Terror, Throw
  Stone, Venom Knife and the Ninja's four), `web/app/incoming/combat.rs` (`skill_used`: Potion Pitcher).
- Taken from: `CRagEffect::ThrowItem` (`Source/Contents/Effects/Core/RagEffect.cpp:12130`),
  `CEffectPrim::PrimThrowItem` (`Source/Contents/Effects/Primitives/EffectPrimitiveMotion.cpp:2130`),
  `RenderFieldPanel`, `CGameMode::LaunchTargetEffect` (`Source/Contents/World/WorldObjects.cpp:399`), the arm
  of `AM_POTIONPITCHER` in `CGameMode::ReceiveSkillUse` (`Source/Client/Mode/GameModePackets.cpp:18658`).
- State: tests for the primitive. Seen in the game: the green bottle of Acid Terror flying in its bow to a
  Pupa with the fainter ones behind it. Potion Pitcher was only thrown at the character itself, where
  nothing flies (no way to go: the client's own sum gives the same). The stone, the knives and the coin were
  not looked at.

**What else the playtest of the three job lines changed** (2026-11-21)
- A monster's life bar stays once its life was told, healed whole or not (`bars::shown`, test;
  `AM_UPDATE_MONSTER_GAGE`, `Source/Contents/Actors/GameObject.cpp:4400`). It hid a bar that was full, so a
  monster healed back to all its life lost its bar until it was hurt again. Seen in the game: a Pupa struck,
  healed to full with Shift and Heal, keeps its bar full.
- The coins of Mammonite (`web/world/prims/sprites.rs coins`; `CRagEffect::Coin`,
  `Source/Contents/Effects/Core/RagEffect.cpp:9147`): an effect that is a file now begins what its function
  draws by code besides (`web/world/launch.rs burst`). Seen in the game: the coins fly up out of the bag.
- "Random" in the list of Teleport, which rAthena names "Random.gat" (`web/hud/choosing.rs
  choose_place_window`): built, not looked at again.
- Seen in the game as they are, nothing changed: the cart behind a Merchant and its four looks by Change Cart,
  the cart window (put, take, the count box, weight), Vending (the window, the price, the name, the list said
  in the chat, another worker's character bought a potion: "Red Potion 1 sold."), Item Appraisal (its list,
  "Item Appraisal has completed successfully."), Mammonite (the zeny goes), Cart Revolution, Crazy Uproar,
  Hammer Fall, Adrenaline Rush, Weapon Perfection, Power Thrust, Maximize Power, forging (the two lists, the
  effect and sound of success), Pharmacy (its list, the effect of success), Bio Cannibalize and Summon Marine
  Sphere; Heal, Blessing, Increase and Decrease Agility, Angelus, Cure, Ruwach, Teleport (its list), Warp
  Portal (the list with the place kept by `/memo`, the portal), Pneuma, Signum Crucis, Holy Light, Kyrie
  Eleison (the blows it takes, its guard), Sanctuary (it hurts the Zombies), Magnus Exorcismus, Turn Undead,
  Lex Divina, Lex Aeterna, Status Recovery, Magnificat, Safety Wall; Steal ("Stone Heart - 1 obtained."),
  Envenom (the monster turns purple), Hiding (the burst, the body gone, the shadow left), Throw Stone, Pick
  Stone, Sonic Blow (its eight numbers), Enchant Poison; the skill trees of all eight jobs with their second
  page, dragging to the hotkey bar, the descriptions; an axe, a dagger, a mace and a katar in the hand.

## 5. Items, inventory, equipment

What is carried and worn, an item's name and description, the character's own numbers, looking at what
another player wears.

**Files**
- Rules: `inventory.rs`, `inventory/weight.rs`, `item_names.rs` (tests `item_names/tests.rs`),
  `links/items.rs`, `numbers.rs`
- Packets: `net/inventory.rs`, `net/inventory/wearing.rs`, `net/inventory/favourites.rs` (tests
  `net/inventory/tests.rs`), `net/viewequip.rs` (tests `net/viewequip/tests.rs`)
- App: `web/app/{incoming,clicks,state}/inventory.rs`, `web/app/{incoming,clicks,state}/status.rs`,
  `web/app/{incoming,clicks,state}/viewequip.rs`, `web/app/actions/bag.rs`, `web/app/dragging.rs`
- Windows: `web/hud/inventory.rs`, `web/hud/equipment.rs`, `web/hud/status.rs`, `web/hud/basic.rs`,
  `web/hud/viewequip.rs`; shared by every window that shows items: `web/hud/items.rs`, `web/hud/lists.rs`
- Tables: `gateway/src/tables/items.rs`, `tables/cards.rs`; `web/data/items.rs`, `web/data/cards.rs`

**Taken from**
- `UIItemWnd` (`Source/UI/Items/UIItemWnd.h`): `OnCreate`, `Paint`, `SendMsg`, `DragAndDrop`,
  `RefreshItems`; `UIItemBaseWnd` (`Source/UI/Items/UIItemBaseWnd.h`), `UIItemDropCntWnd`
- `UIEquipWnd` (`Source/UI/Items/UIEquipWnd.h`): `OnCreate`, `Paint`, `OnLBtnDown`, `UpdateCharacterView`
- `UIBasicInfoWnd` (`Source/UI/Status/UIBasicInfoWnd.h`), `UIMenuIconWnd::OnCreate`
  (`Source/UI/Menu/UIMenuIconWnd.cpp`), `UIStatusWnd::Paint`
- `ITEM_INFO::DisplayName` (`Source/Contents/Items/ItemNaming.cpp`), `cardprefixnametable.txt`,
  `cardpostfixnametable.txt`, `itemslotcounttable.txt`, `itemmoveinfov5.txt`
- `CGameMode::IsOverWeight` (`Source/Client/Mode/GameMode.cpp`), `CGameMode::ReceiveWearEquipAck`,
  `UIChatHisBox::RenderItemLinks`, `UINotifyItemObtainWnd`

**State**
- The inventory, Equipment and Status windows, putting on a bow and arrows: seen in the game.
- Seen in the game in the playtest of the Hunter: headgear put on and taken off by a double click (top, middle
  and lower at once: Hat, Sunglasses, Flu Mask), taken off by dragging it out of the Equipment window, put on
  by letting it go over the Equipment window, arrows readied the same way and changed for others (the name
  under their picture follows). A thing of the inventory let go over the Equipment window is put on
  (`inventory.rs worn_by_drop` with its test, `press::things::Drop::Equipment`, `clicks/inventory.rs
  worn_let_go`; `UIEquipWnd::DragAndDrop`). The readied arrows' name begins 5 above where it was
  (`hud/equipment.rs equipment_ammunition`; `UIFrameWnd::DrawTextLines` begins the first of a name's two
  lines half a line above the place handed to `DrawItemName`).
- Tabs, weight, how an item is named, item links, how numbers are written and typed, the packets: tests only.
- Another player's equipment, the Fav tab, dragging between windows: by reading only (their packets have
  tests).

**The buttons that tell of a level gained** (added with the classic gaps)
- Files: `web/app/incoming/status.rs` (`level_changed`), `web/app/clicks/status.rs` and `clicks/skills.rs`
  (the presses, and the windows that put the buttons away as they open), `web/hud/status.rs` (`level_told`,
  `told_place`), `web/hud/skills.rs` (`job_level_told`), the entries `LEVEL_TOLD` and `JOB_LEVEL_TOLD`;
  the marks on the menu's icons and on its folded bar in `web/hud/basic.rs` (`menu_icons`).
- Taken from: `CGameMode::ReceiveParChange`, `SP_BASELEVEL` and `SP_JOBLEVEL`
  (`Source/Client/Mode/GameModePackets.cpp:5576, 5637`), `UIWindowMgr::EnsureWindow` at
  `WID_NOTIFYLEVELUPWND`, `WID_NOTIFYJOBLEVELUPWND`, `WID_STATUSWND` and `WID_SKILLLISTWND`
  (`Source/UI/Window/UIWindowMgr.cpp:5846, 4917, 4931`), `UINotifyLevelUpWnd::OnCreate` and `SendMsg`,
  `UINotifyJobLevelUpWnd::SendMsg` (`Source/UI/Acceptance/UIChooseSellBuyWnd.cpp:7737, 6846, 5601`),
  `UIMenuIconWnd` (`ID_NOTICE`, `notificationButtons`, `notificationOpenButton`).
- State: seen in the game (2026-11-21): after `@blvl` the button in the lower right corner and the mark on
  the Status icon, after `@jlvl` the one in the lower left corner and the mark on the Skill icon; a press on
  each opens Status or the Skill Tree and the button goes. The marks on the folded bar were not looked at.

## 6. Cards and making

The lists a skill or a card opens to pick one thing of: setting a card, forging, brewing and cooking, crafting
arrows, Auto Spell, refining, repairing, identifying, the places of Teleport and Warp Portal.

**Files**
- Rules: `choosing.rs`
- Packets: `net/choosing.rs`, `net/choosing/cards.rs`, `net/choosing/making.rs`, `net/choosing/mending.rs`,
  `net/choosing/warps.rs` (tests `net/choosing/tests.rs`); identifying is in `net/inventory.rs`
  (`ZC_ITEMIDENTIFY_LIST`, `ZC_ACK_ITEMIDENTIFY`)
- App: `web/app/{incoming,clicks,state}/choosing.rs`
- Windows: `web/hud/choosing.rs`

**Taken from**
- `UIItemCompositionWnd`, `UIItemIdentifyWnd`, `UIMakingArrowListWnd` (all three in
  `Source/UI/Items/UIItemIdentifyWnd.h`)
- `UIMakeTargetListWnd` (`Source/UI/Items/UIMakeTargetListWnd.h`), `UIMakeTargetProcessWnd`
  (`Source/UI/Items/UIMakeTargetProcessWnd.h`), `metalprocessitemlist.txt`
- `UIItemRepairWnd` (`Source/UI/Items/UIItemRepairWnd.h`), `UIWeaponRefineWnd`
  (`Source/UI/Items/UIWeaponRefineWnd.h`), `UISpellListWnd` (`Source/UI/Items/UISpellListWnd.h`),
  `UIChooseWarpWnd` (`Source/UI/Navigation/UIChooseWarpWnd.h`)
- `CGameMode::ReceiveItemEffectResult`, `CGameMode::ReceiveCardCompositionResult`,
  `CGameMode::ReceiveRefineResult` (`Source/Client/Mode/GameModeNotices.cpp`)

**State**: tests (what may go into a forging, and every packet). Seen in the game (2026-11-21): a card
double-clicked lists the gear with a free slot of its kind, OK compounds it, the thing takes the card's
word before its name ("Hurricane Javelin [3]", "Lucky Cotton Shirt [1]"), its description shows the card in
its first slot, the card's own description opens from there, and "view" opens its illustration. The
windows of making, forging, repairing and refining are by reading only.

**Refining at the smith NPCs** (looked at in the game, 2026-11-21, rAthena's `npc/merchants/refine.txt` with
`feature.refineui: off`)
- Files: `web/app/incoming/choosing.rs` (`item_refined`), `net/choosing/mending.rs` (`item_refined`), the talk
  with the NPC (section 14), the notice of the thing given anew (`incoming/inventory.rs`, `tell_obtained`).
- Taken from: `CGameMode::ReceiveRefineResult` (`GameModeNotices.cpp`: `EF_REFINEOK` 154 / `EF_REFINEFAIL` 155
  on the player, lines 498 / 499 in 0xFFFF00), `RagEffect.cpp` (`bs_RefineSuccess.str` with
  `effect\bs_refinesuccess.wav` at tick 15, `bs_RefineFailed.str` with `effect\bs_refinefailed.wav` at tick
  5), `ReceivePartyMemberEffect` for ZC_NOTIFY_EFFECT 3 and 2; `AddPickedUpItemEx` and
  `UINotifyItemObtainWnd::SendMsg` for the notice's name (`DisplayName(0)`).
- State: seen in the game at Hollgrehenn (`prt_in`), weapons of levels 1 to 3 (Knife, Stiletto, Arc Wand), a
  Cotton Shirt and a Guard, 69 refinings in a row and more by hand: the menu of what is worn, the price and
  the ore asked, the warning past the safe level, both effects on the character, "Upgrade success!!" /
  "Upgrade failed!!", the name with its "+N" in the Equipment window and in the notice ("+5 Stiletto [2] - 1
  obtained."), ATK and DEF in the Status window, the thing gone from what is worn when it breaks; the
  refusals (no ore, a thing that cannot be refined, +10); the purifier Dietrich (5 rough ores for one).
  Nothing of it is left in `/packets` (ZC_ACH_UPDATE, of the achievements, is). Heard on 2026-11-26:
  `bs_refinesuccess.wav`, once a refining. Not told apart at that size: the weapon's and the shield's
  picture on the character after a break. Not looked at: a level 4 weapon, the "+N" in the description
  window, on the ground, in a deal and in a shop's list.

## 7. Shops, storage, cart, vending

An NPC's shop, the Kafra's storage and a guild's, the merchant's cart, a merchant's own shop.

**Files**
- Rules: `shops.rs`, `storage.rs`, `cart.rs`, `vending.rs`
- Packets: `net/shops.rs`, `net/storage.rs`, `net/cart.rs`, `net/vending.rs`, `net/vending/requests.rs`
  (tests `net/vending/tests.rs`)
- App: `web/app/{incoming,clicks,state}/shops.rs`, `…/storage.rs`, `…/cart.rs`, `…/vending.rs` (the same three
  folders for each)
- Windows: `web/hud/shops.rs`, `web/hud/storage.rs`, `web/hud/cart.rs`, `web/hud/vending.rs`,
  `web/hud/vending/buying.rs`
- World: `web/world/cart.rs` (the cart behind its owner), `web/world/signs.rs` (the sign over a vendor)

**Taken from**
- `UIItemShopWnd` (`Source/UI/Items/UIItemShopWnd.h`), `UIItemPurchaseWnd`
  (`Source/UI/Items/UIItemPurchaseWnd.h`), `UIItemSellWnd` (`Source/UI/Items/UIItemSellWnd.h`),
  `UIItemPurchaseWnd::SendMsg`, `UIItemDropCntWnd::SendMsg`
- `UIItemStoreWnd` (`Source/UI/Items/UIItemStoreWnd.h`), `UIItemStoreSubWnd`, `UIItemStoreFindWnd`
- `UIMerchantItemWnd` (`Source/UI/Shop/UIMerchantItemWnd.h`), `UISelectCartWnd`
  (`Source/UI/Items/UISelectCartWnd.h`), `CMsgEffect::FollowOwner`
  (`Source/Contents/Effects/Core/MsgEffect.cpp`), the message `AM_MAKE_CART`
- `UIMerchantShopMakeWnd`, `UIMerchantItemMyShopWnd`, `UIMerchantItemShopWnd`, `UIMerchantItemPurchaseWnd`,
  `UIMerchantShopTitle`, `UIMerchantMirrorItemWnd` (each in `Source/UI/Shop/<its name>.h`)

**State**: tests (the rules of the four and their packets). Seen in the game (2026-11-21): an NPC's shop
(the Tool Dealer and the Armor Dealer of `prt_in`): the box of buy, sell and cancel, the goods with their
pictures, names and prices, a row dragged into the basket with the box that asks how many, the total, "You
do not have enough zeny." for more than can be paid, buying, and selling two kinds at once. The Kafra's
storage: its fee, a thing put in and taken out with the box that asks how many, the count, the tab of
weapons, the search (by the letters as written: "Pot", not "pot") with a thing taken out of the window of
what was found, the server's refusal of a thing that cannot be stored. The cart, a merchant's own shop, a
guild's storage and the signs in the world are by reading only.

**The panel that compares what is worn, over the basket of an NPC's shop** (added with the classic gaps)
- Files: `shops.rs` (`compared`, `change`, `Worn`), `gateway/src/tables/cards.rs` (`worn`:
  `data\itemslottable.txt` and `data\itemparamtable.txt`, 900 items up to 5019) ↔ `web/data/cards.rs`
  (`data::item_worn`), `web/hud/shops.rs` (`shop_compare_window`), `web/hud/windows/shops.rs` (`SHOP_COMPARE`).
- Taken from: `UIItemParamChangeDisplayWnd` (`Source/UI/Items/UIItemParamChangeDisplayWnd.cpp`; its `Paint`,
  `SumEquippedStats`, `FindEquippedItem` and `SendMsg` at `Source/UI/Acceptance/UIChooseSellBuyWnd.cpp:657,
  2574, 2108, 7977`, `DrawParamChangeArrow` at 640), `LoadItemNameTables`
  (`Source/Contents/Items/ItemNaming.cpp:676, 681`), `UIWindowMgr::EnsureWindow`
  (`Source/UI/Window/UIWindowMgr.cpp:5767, 5936`: opened with the basket of what is bought, 280 by 34),
  `ItemParamChangeDisplayWndInfo::Reset` (`Source/Contents/UserData/NormalItemSlots.cpp:669`: 280, 246), closed
  at `MM_RELEASE_QUEST` (`Source/Client/Mode/GameMode.cpp:3292`), `EquipSlotDecoder::Decode`
  (`Source/Contents/Items/ClientItemLists.cpp:694`: the places in the order of the server's bits).
- The window is told of an item to compare only by `UIItemCollectionWnd::SendMsg`
  (`UIChooseSellBuyWnd.cpp:12656`) when a description was opened with 1, which `UIItemShopWnd2`,
  `UIItemSellWnd` and `UIItemSellWnd2` do (`Source/UI/Edit/UIEditCtrl.cpp:7042, 7077, 7112`) and the list of an
  NPC's shop, `UIItemShopWnd`, does not (`UIEditCtrl.cpp:7129, 5433`). So beside an NPC's shop the panel says
  the figures of what is worn twice, with the grey bar between: the rule for an item pointed at is written
  and tested, and nothing calls it with one.
- State: tests (the sums, the comparison, the gateway's reading of the two files); seen in the game
  (2026-11-21) over the basket of Prontera's Armor Dealer, "ATK 25 = 25  DEF 3 = 3" with a Sword and a
  Cotton Shirt on. To see it: talk to any NPC that sells, choose Buy; put on and take off a weapon or a guard
  with the shop open and the first figures follow.

## 8. Exchange

A deal between two players.

**Files**
- Rules: `exchange.rs`
- Packets: `net/exchange.rs`
- App: `web/app/{incoming,clicks,state}/exchange.rs`
- Windows: `web/hud/exchange.rs`

**Taken from**
- `CUIExchangeUI` (`Source/UI/Exchange/UIExchangeUI.h`): `OnCreate`, `OnProcess`, `SubmitZeny`
  (`Source/UI/Exchange/UIExchangeUI.cpp`)
- `UIExchangeAcceptWnd` (`Source/UI/Acceptance/UIJoinPartyAcceptWnd.h`), `UIItemDropCntWnd`
- `CGameMode::SendMsg` with `MM_REQ_EXCHANGE_ITEM`, `MM_ADD_EXCHANGE_ITEM`, `MM_EXEC_EXCHANGE_ITEM`

**State**: seen in the game, with zeny and with items. Its rules and packets also have tests. The zeny written once,
without the client's coloured edge (the owner's decision of 2026-11-21): by reading only.

## 9. Chat and chat rooms

The chat window and its tabs, whispers, the server's announcements, emotions, chat commands, chat rooms.

**Files**
- Rules: `chat.rs`, `chat_tabs.rs`, `said.rs`, `markup.rs`, `whispers.rs`, `chat_rooms.rs`, `commands.rs`
  (with `commands.txt`)
- Packets: public chat in `net/packets/zone.rs`; `net/whispers.rs`, `net/broadcast.rs`, `net/emotions.rs`,
  `net/commands.rs`, `net/chat_rooms.rs`
- App: `web/app/chat.rs`, `web/app/commands.rs`, `web/app/commands/beside.rs`, `web/app/incoming/talk.rs`
  (the chat's lines), `web/app/{clicks,state}/chat.rs`, `web/app/{incoming,clicks,state}/whispers.rs`,
  `web/app/{incoming,clicks,state}/chat_rooms.rs`, `web/app/{incoming,clicks,state}/emotions.rs`,
  `web/app/{incoming,state}/broadcast.rs`, `web/app/{incoming,state}/commands.rs`
- Windows: `web/hud/chat.rs`, `web/hud/whispers.rs`, `web/hud/broadcast.rs`, `web/hud/emotions.rs`,
  `web/hud/chat_rooms.rs`
- World: `web/world/signs.rs` (the sign of a room over its owner)
- Tables: `gateway/src/tables/commands.rs`, `tables/interface.rs` (messages, emotions); `web/data/commands.rs`,
  `web/data/interface.rs`

**Taken from**
- `UINewChatWnd` (`Source/UI/Chat/UINewChatWnd.h`), `UISubChatHisWnd`, `UIChatHisBox`
  (`Source/UI/Lists/UIChatHisBox.h`), `UIBattleMsgOptionWnd`
  (`Source/UI/GameSettings/UIBattleMsgOptionWnd.h`), `SaveData\ChatWndInfo_U.lua`
- `CSession::GetTalkType` (`Source/Client/Session/Session.cpp`), `CGameMode::ExecuteTalkType`
  (`Source/Client/Mode/GameMode.cpp`)
- `UIWhisperWnd` (`Source/UI/Chat/UIWhisperWnd.h`), `UIWindowMgr::ShowWhisperFrom`
  (`Source/UI/Window/UIWindowMgr.cpp`), the message `UIM_MAKE_WHISPER_WINDOW`
- `UIBroadcastBalloon`, `UIBroadcastBalloon::AddMessage` (`Source/UI/Text/UIBroadcastBalloon.cpp`),
  `CGameMode::ReceiveTaggedBroadcast` (`Source/Client/Mode/GameModePackets.cpp`)
- `UIChatRoomWnd`, `UIChatRoomMakeWnd`, `UIChatRoomChangeWnd`, `UIChatRoomTitle` (`Source/UI/Chat/`),
  `UIPasswordWnd` (`Source/UI/Login/UIPasswordWnd.h`)
- `UICashEmotionListWnd` (`Source/UI/CashEmotion/UICashEmotionListWnd.h`), `UIEmotionWnd`

**State**: tests only (tabs, the kinds of line, marks in a text, whispers, rooms, commands, and the packets);
every window is by reading only. Nothing of this system is on the seen list.

**A line too long for the chat's list is cut into rows** (2026-11-26)
- Files: `chat.rs` (`rows`, `held`, `ROW`, `BAR`, with tests), `web/hud/chat.rs` (`chat_lines`, `chat_row`),
  the page's `.chat-log div` (`client/web/index.html`).
- Taken from: `UINewChatWnd::SendMsg`, its `UM_ADDCHAT` (`Source/UI/Chat/UINewChatWnd.cpp:290`: the colours
  taken out, `WrapPlainText(.., 0x5E)`, then for each row `AddText` when it is no wider than the tab less 20
  and `AddTextWrapped` when it is); `WrapPlainText` (`Source/UI/Text/TextWrapping.cpp`); `UIChatHisBox::
  AddTextWrapped` (`Source/UI/Lists/UIListBox.cpp:138`: the box less 16) and `UIWindow::WrapPixelText`
  (`Source/UI/Window/UIWindow.cpp:1647`: cut by width between any two letters, what stands in `[...]` kept
  whole). Every row is a line of its own to the list (14 high, scrolled by rows). The announcement across
  the top of the screen was cut already (`WrapPlainText(.., 0x48)`, `net/broadcast.rs`).
- What was wrong: a line was written in one row whatever its length and ran out of the window.
- State: tests (the 94 letters, the brackets); seen in the game (2026-11-26): `@kami` and
  `@localbroadcast` of 180 letters and a long line said aloud are two rows in the chat at its first and
  second size, in the Battle tab, and more rows in a tab's own window 300 wide; a row of wide letters is
  cut by its width, a `[...]` goes to the next row whole; the list scrolls and stays at its end. The
  banner of each is three lines. Whisper, party and guild lines go the same way and were not said in the
  game.

**The away message, `/am` and `/automsg`** (added with the classic gaps)
- Files: `whispers.rs` (`away_reply` and the lines of the message table), `web/app/clicks/whispers.rs`
  (`away_switched`, `away_typed`, `away_answers`), `web/app/incoming/whispers.rs`, `web/app/commands/beside.rs`
  (`TT_AUTO_MSG_ON_OFF`), `web/app/keys.rs` (Escape), `web/hud/whispers.rs` (`away_window`), the entry `AWAY`
  of `web/hud/windows/whispers.rs`.
- Taken from: `CGameMode::ExecuteTalkType`, `TT_AUTO_MSG_ON_OFF` (`Source/Client/Mode/GameMode.cpp:9409`),
  `UIAutoMessageWnd` (`Source/UI/Acceptance/UIChooseSellBuyWnd.cpp:6247, 9368, 9639`),
  `CGameMode::ProcessWhisper` (`Source/Client/Mode/GameModeNotices.cpp:7672`), `UIWindowMgr::ProcessKeyDown`
  at `VK_ESCAPE` (`Source/UI/Window/UIWindowMgr.cpp:2464`), `LoginMode.cpp:2237` (the message at first).
- State: tests (what is answered, and to what not); seen in the game (2026-11-21): `/am` opens the box,
  its OK says the line 942, `/am` again says 943 and leaves an open box open, as the client does. The
  answer to a whisper was not seen (it needs a second player).

**The rankings by number, the points of fame, what a game master hears** (added with the classic gaps)
- Files: `commands.rs` (`ranking_lines` with the headings and colours of each kind, `ranking_named`,
  `fame_line`, `stat_dump`), `net/commands.rs` (`ranking_by_number` 0x0af6, `fame_gained` 0x097e, `kicked`
  0x00cd, `gm_stats` 0x0214; 0x01e0 is taken and does nothing, as in the client),
  `web/app/incoming/commands.rs` (`ranking_looked`, `ranking_said`), `web/app/state/commands.rs` (`ranking`),
  `Action::RankingLooked`, `web/data/cards.rs` (`character_name`: the names by number, which the items'
  makers use too).
- Taken from: `CGameMode::ReceiveShowRankingSchedule` (`Source/Contents/Items/ClientItemLists.cpp:1837`),
  `CSession::SetShowRankingSchedule`, `CShowRankingScheduler::Process` and `Show`, `RankingBoardStyles`
  (`Source/Contents/Events/ShowRankingScheduler.cpp:38, 80, 98, 153`), `CPlayerNameDictionary::Lookup`
  (`Source/Contents/Actors/PlayerNameDictionary.cpp:38`), `CGameMode::ReceiveRankingList` and
  `RankNoticeStyles` (`Source/Client/Mode/GameModeNotices.cpp:7496, 7517`), `CGameMode::ReceiveTypedNotice`
  (`GameModeNotices.cpp:2476`: ZC_UPDATE_RANKING_POINT), `CGameMode::ReceiveStatDump`
  (`Source/Client/Mode/GameModePackets.cpp:728`), `case ID_ZC_ACK_DISCONNECT_CHARACTER`
  (`GameModePackets.cpp:25066`).
- The ranking told with names (0x097d) was written with one heading and all in yellow; it has the client's
  four headings, its rule of 43 signs and its two colours a kind now.
- State: tests (the lines and colours, which names are waited for, the packets); seen in the game
  (2026-11-21): `/blacksmith`, `/alchemist`, `/taekwon` and `/pk` each write their heading, ten lines of
  "Nameless" (nobody has points on the test server) and the player's own points. `/pk` asks for the fourth
  kind of the same request (`web/app/commands/beside.rs`, `TT_REQ_KILLER_RANK`). A ranking with names in
  it, the wait of three seconds and "Fetching rank..." are by reading only.

**The words that are not let be written** (added with the classic gaps)
- Files: `manner.rs` (`Words::allows`, `masked`), `gateway/src/tables/manner.rs` (`data\manner.txt`, 1403
  words) ↔ `web/data/manner.rs` (`data::foul`, `data::starred`); and a call in each place the client asks
  its `g_forbiddenWords`:
  - *refused*: a chat room's title (`chat_rooms.rs wrong`, line 14 in the message box); a stall's title
    (`web/app/clicks/vending.rs`, 14); what is typed for an NPC (`clicks/talk.rs`, 14 in the chat, red); a
    pet's and a homunculus's new name (`clicks/pets.rs`, `clicks/homunculus.rs`, 2948); the guild's notice
    (`clicks/guild/manage.rs`, 14); a party's name (`clicks/party.rs party_make_asked`, 14); a guild's name
    (`clicks/guild.rs`, 3335 in the chat); a letter's title and text (`clicks/mail/writing.rs`, 3028); a new
    character's name (`web/app/connection.rs`, this client's own words for a bad name).
  - *written with stars*: what is typed into the chat's line, before anything else is done with it
    (`clicks/chat.rs chat_typed`), and a whisper sent (`clicks/whispers.rs whisper_send`); what the
    character said and what a player in sight says, the party's and the guild's chat, the announcements
    (`incoming/talk.rs`, `party.rs`, `guild.rs`, `broadcast.rs`); the lines of a whisper's window and of a
    chat room, the room's sign and title (`clicks/whispers.rs`, `clicks/chat_rooms.rs`,
    `hud/chat_rooms.rs`); a stall's sign (`hud/vending.rs`); the party's name as kept and in an invitation,
    the guild's in an invitation and in a request to ally, the guild's notice, the reason one left or was
    expelled (`incoming/party.rs`, `incoming/guild.rs`, `hud/guild.rs`, `hud/guild/expelled.rs`); under
    someone the pointer is on, the party's, the guild's and the position's names and a homunculus's name
    (`web/world/guild.rs`, `web/world/names.rs`).
- Taken from: `ForbiddenWords::Load`, `Allows`, `Contains`, `Mask`
  (`Source/Contents/Tables/ForbiddenWords.cpp:54, 89, 125, 132`), `CCommonObject::LoadTokens`
  (`Source/File/CommonObject.cpp:27`), and the callers: `CGameMode::SendMsg` at `MM_CHATMSG`,
  `MM_CONFORM_MAKEGROUP`, `MM_WHISPERMSG`, `MM_USE_SKILL_TOGROUND_WITH_TALKBOX`, `MM_REQ_MAKE_GUILD`,
  `MM_SEND_MAIL`, `MM_CHAT_PARTY`, `MM_CHAT_GUILD` (`Source/Client/Mode/GameMode.cpp:1884, 2229, 3227, 3543,
  3905, 4081, 4280, 4544`); `ReceiveActorChat`, `ReceiveBroadcast`, `ReceivePartyChat`, `ReceiveGuildChat`,
  `ReceiveTaggedBroadcast`, `ReceiveColorBroadcast`, `ReceiveGuildNotice`, the party's lists and the three
  invitations (`Source/Client/Mode/GameModePackets.cpp:11352, 13612, 11404, 11616, 16955, 16853, 12046,
  17267…17407, 25315, 25911, 25955`), `ReceiveGuildMemberExpelled`, `ReceiveGuildMemberLeft`
  (`GameModeNotices.cpp:7086, 7180`); `UIChatRoomMakeWnd`, `UIChatRoomChangeWnd`, `UINpcTextEditDialogWnd`,
  `UIWhisperWnd`, `UIJoinGuildAcceptWnd` (`Source/UI/Acceptance/UIChooseSellBuyWnd.cpp:5842, 5946, 1062, 4705,
  7631`), `UIChatRoomWnd` (`Source/UI/Window/UIFrameWnd.cpp:909, 958`), `UIChatRoomTitle`
  (`Source/UI/Chat/UIChatRoomTitle.cpp:173`), `UIMerchantShopTitle` (`Source/UI/Shop/UIMerchantShopTitle.cpp:115`),
  `UIMerchantShopMakeWnd` (`Source/UI/Edit/UIEditCtrl.cpp:11818`), `UIHomunInfoWnd`, `UIPetInfoWnd`,
  `UIMakeCharWnd` (`Source/UI/Dialogs/UIMessageBox.cpp:3621, 3830, 4187`), `UIGuildNoticeWnd`,
  `UIGuildBanishedMemberWnd` (`Source/UI/Guild/UIGuildWnd.cpp:2023, 1855`), `UINameBalloonText::SetName`
  (`Source/UI/Text/UIBalloonText.cpp:350…398`), `CGameMode::UpdateNameBalloon`
  (`Source/Contents/World/WorldObjects.cpp:467`).
- The file is Korea's, with Latin words in it too ("sex", "ass", "cum", "shit"…), and the client finds a
  word inside a longer one: "class" is sent as "cl***", "Assassin" as "***assin", and an @command with such
  a word in it is sent with the stars too. That is the Windows client's behaviour with these files.
- Found on the way and put right: the words of another player were hung over nobody's head, the ID being
  read two bytes early (`net::talk::npc` for ZC_NOTIFY_CHAT); it is `net/packets/zone.rs chat_speaker` now.
- State: tests only (the rule, the gateway's reading of the file, the title of a chat room, the speaker of
  a line); every place it is applied in is by reading only. To see it: type "this is shit" into the chat
  (it comes back "this is ****"); make a chat room or a stall called "sex" (the box says "Foul Language
  Detected."); `/organize sex`; rename a pet to "bitch".

## 10. Party

**Files**
- Rules: `party.rs`
- Packets: `net/party.rs`, `net/party/requests.rs` (tests `net/party/tests.rs`)
- App: `web/app/{incoming,clicks,state}/party.rs`
- Windows: `web/hud/party.rs`, `web/hud/party/side.rs`, `web/hud/messenger.rs` (what the Party and Friend
  sides of one window share)

**Taken from**
- `UIMessengerGroupWnd` (`Source/Contents/Messenger/MessengerWindows.h`): `OnCreate`, `Paint`, `SendMsg`;
  `UIJobIconButton`
- `UIPartySettingWnd`, `UIPartyInvitationToWnd` (`Source/UI/Party/UIPartySettingWnd.h`),
  `UIJoinPartyAcceptWnd` (`Source/UI/Acceptance/UIJoinPartyAcceptWnd.h`)
- `CGameMode::ReceivePartyRoster` (`Source/Client/Mode/GameModeHandlers.cpp`),
  `CGameMode::DrawMiniMapMembers` (`Source/Client/Mode/GameModeMiniMap.cpp`)
- The messages `MM_MAKE_GROUP`, `MM_REQ_JOIN_GROUP`, `MM_JOIN_GROUP`, `MM_REQ_LEAVE_GROUP`,
  `MM_REQ_EXPEL_GROUP_MEMBER`

**State**: tests only (the packets, and one rule); the windows are by reading only.

**A thing a member obtained** (added with the classic gaps)
- Files: `net/party.rs` (`picked`, 0x0b67), `web/app/incoming/party.rs` (`party_obtained_said`),
  `web/app/state/party.rs` (`obtained`), `said.rs` (`PARTY_ITEM`, the chat tabs' kind 9, "Party member's
  obtained item").
- Taken from: `CGameMode::ReceivePartyItemPickup` (`Source/Client/Mode/GameModeNotices.cpp:1172`): line 1276
  of the message table with the member's name and `ITEM_INFO::DisplayName(0)`, in 0xFFFF00, kind 9; nothing
  for the player's own account or for one not of the party.
- State: tests only (the packet); the line is by reading only.

## 11. Guild

**Files**
- Rules: `guild.rs` (the castles: the sizes of their list and menu, where the menu opens, the words of
  their two boxes; the guilds allied and opposed: the rows of their two lists, how wide a row is picked and
  marked, which guild a row shows)
- Packets: `net/guild.rs`, `net/guild/members.rs`, `net/guild/manage.rs`, `net/guild/requests.rs` (tests
  `net/guild/tests.rs`), `net/guild/castles.rs` (with its tests)
- Tables: `gateway/src/tables/castles.rs` ↔ `web/data/castles.rs` (the client's `AgitConfig.lub` and
  `AgitConfig_F.lub`: each castle's name and the two costs of going there, by the client's number of it)
- App: `web/app/incoming/guild.rs`, `web/app/incoming/guild/{members,castles}.rs`, `web/app/clicks/guild.rs`,
  `web/app/clicks/guild/{manage,castles}.rs`, `web/app/state/guild.rs`
- Windows: `web/hud/guild.rs`, `web/hud/guild/{members,positions,skills,expelled,notice,castles}.rs`
- World: `web/world/guild.rs` (the guild and emblem of each player in sight)

**Taken from**
- `UIGuildWnd` and its tabs `UIGuildInfoManageWnd`, `UIGuildMemberManageWnd`, `UIGuildPositionManageWnd`,
  `UIGuildSkillWnd`, `UIGuildBanishedMemberWnd`, `UIGuildNoticeWnd` (all in `Source/UI/Guild/UIGuildWnd.h`);
  `UIGuildMemberManageWnd::DrawMember` (`Source/UI/Guild/UIGuildWnd.cpp`)
- `UIJoinGuildAcceptWnd`, `UIAllyGuildAcceptWnd` (`Source/UI/Acceptance/UIJoinPartyAcceptWnd.h`),
  `UIGuildLeaveReasonDescWnd`, `UIGuildBanReasonDescWnd` (`Source/UI/Guild/UIGuildHelperWnd.h`)
- `CGuildMgr` (`Source/Contents/Guild/GuildMgr.h`), `CGameMode::ReceiveGuildLeave2`
  (`Source/Client/Mode/GameModeNotices.cpp`)
- The messages `MM_REQ_GUILD_MENU`, `MM_REQ_JOIN_GUILD`, `MM_REQ_BAN_GUILD`, `MM_SEND_GUILD_EMBLEM`,
  `MM_GUILD_NOTICE`, `MM_REQ_CHANGE_MEMBERPOS`; rAthena's `clif_validate_emblem`
- The castles (there is no window of their own in the client; the audit's `UIAgitWnd` is the name of a file):
  the list in the first tab and its menu, `UIListBoxAgit` and `UIListBoxAgitMenu` (`Source/UI/Guild/UIAgitWnd.cpp`),
  made and filled by `UIGuildInfoManageWnd::OnCreate` and `RefreshAgitList` (`Source/UI/Guild/UIGuildWnd.cpp`);
  the box before going to one, `UIAgitMessageBox` (`InitializeMessageBox` in `UIGuildWnd.cpp`);
  `GuildInfo::agitIds` (`Source/Contents/Guild/GuildInfo.cpp`); `CGameMode::ReceiveGuildAgitList`
  (`GameModeNotices.cpp`), `ReceiveAgitNotice` and `ReceiveGuildAgitMoveResult` (`GameModePackets.cpp`);
  `MM_REQ_MOVE_GUILD_AGIT` and `MM_REQ_AGIT_INVESTMENT` (`GameMode.cpp`); the lines 333, 1090, 3629 to 3633
  and 3640 of the message table; rAthena's `clif_guild_castle_list`, `clif_guild_castleinfo`,
  `clif_guild_castle_teleport_res` and the two `clif_parse_guild_castle_*`

**State**
- Making a guild, inviting, leaving, expelling, the emblem, the notice: seen in the game.
- Alliances and oppositions, seen in the game (two test guilds, 2026-11-21): a second `/guild` refused ("You
  are already in a Guild."); an alliance asked from the menu on the other master, the question window,
  refused and accepted, both guilds' lists filled; an opposition set and shown; either ended with the right
  button on its row and "Delete", from either side of an alliance, both lists emptied; a member leaving with
  a reason and a master expelling with one, the lines in the chat of both, the Expel List tab showing the
  name and the reason; `/breakguild` with a wrong name (the box of line 366) and with the guild's own (the
  guild gone, its window shut). The right button on a row of the two lists is as `UIGuildInfoManageWnd::
  OnRBtnUp` and `Paint` have it (`guild.rs`, with tests): taken from 201 to 287 across, on any of the three
  rows of a list, the row marked 80 wide from 204 and staying marked while the tab is open; the left button
  and the pointer over a row do nothing there, in the client as here; the menu has its one row for the
  master only. Not seen: the limit of alliances (it takes four guilds) and a non-master's right button on a
  row. What an allied or an opposed guild's members look like: nothing of their own in the client (no colour
  of the name, no mark; `GuildInfo::HasRelatedGuild` is asked only for whom a press fights during a siege,
  which is `web/world/pvp.rs`), and nothing here.
- While a siege runs the server lets no alliance be made or ended and no opposition ended: rAthena's
  `guild_reqalliance` and `guild_delalliance` answer with their own lines in the chat ("Alliances cannot be
  made / broken during Guild Wars!") whenever `agit_flag` is set, which is the server's rule and no fault of
  the menu. The local server's siege runs by `npc/guild/agit_controller.txt`'s days and hours, and is on
  again after a restart of the map server inside them; a game master ends it with `@agitend`.
- Positions, the guild's skills, the guild's chat: tests only for their packets; their windows are by
  reading only.
- The castles (ZC_GUILD_AGIT_INFO 0x0b27, ZC_REQ_ACK_AGIT_INVESTMENT 0x0b2d, ZC_REQ_ACK_MOVE_GUILD_AGIT 0x0b2e,
  CZ_REQ_MOVE_GUILD_AGIT 0x0b28, CZ_REQ_AGIT_INVESTMENT 0x0b2c): tests only for the packets, the table and
  the rules; the list, its menu and the two boxes are by reading only: in the game only a new guild's
  empty list was seen (2026-11-21, "Castles Owned :" and nothing under it). The
  words the server sends of the castles ("None Taken"…) are no longer written: the client's list lies over
  them. The classic server answers the list and the economy and defence; it does not move the character
  (`db/pre-re/castle_db.yml` has no `WarpEnabled` for any castle, see `DEBTS.md`).

## 12. Friends, family and mail

The friends list, the mail (the client's RODEX), and the family (wedding, divorce, adoption), which the code
keeps as a system of its own named `family`.

**Files**
- Rules: `friends.rs`, `mail.rs`, `family.rs`
- Packets: `net/friends.rs` (tests `net/friends/tests.rs`), `net/mail.rs`, `net/mail/requests.rs` (tests
  `net/mail/tests.rs`), `net/family.rs`
- App: `web/app/{incoming,clicks,state}/friends.rs`, `web/app/{incoming,clicks,state}/mail.rs`,
  `web/app/clicks/mail/writing.rs`, `web/app/{incoming,clicks,state}/family.rs`
- Windows: `web/hud/friends.rs`, `web/hud/messenger.rs`, `web/hud/mail.rs`, `web/hud/mail/list.rs`,
  `web/hud/mail/letter.rs`, `web/hud/family.rs`

**Taken from**
- `UIMessengerGroupWnd` (its Friend side), `UIFriendOptionWnd` (`Source/Contents/Messenger/MessengerWindows.h`),
  `UIJoinFriendAcceptWnd` (`Source/UI/Acceptance/UIJoinPartyAcceptWnd.h`)
- `CGameMode::ReceiveFriendList` (`Source/Client/Mode/GameModePackets.cpp`), `UIWindowMgr::RequestAddFriend`
  (`Source/UI/Window/UIWindowMgr.cpp`), `CSession::IsFriendName`, `CSession::m_friends`
- `UIOpenMailBoxWnd`, `UIRodexInboxWnd`, `UIRodexContentsWnd`, `UIRodexComposeWnd` (each in
  `Source/UI/Rodex/<its name>.h`), the messages `MM_OPEN_WRITE_RODEX`, `MM_SEND_MAIL`
- `UIBabyAcceptWnd`, `UIBabyAcceptWnd2` (`Source/UI/Acceptance/UIJoinPartyAcceptWnd.h`), the message
  `MM_JOIN_BABY`

**State**
- The mail icon (the notice of mail not read): seen in the game.
- The rest of the mail (the mailbox, reading, writing, taking what is in a letter), the friends list, the
  family: tests only for the rules and packets; the windows are by reading only.

## 13. Pets, homunculus, mercenary

Taming and the pet window, the alchemist's homunculus, a hired mercenary, and the Lua scripts that move the
last two.

**Files**
- Rules: `pets.rs`, `homunculus.rs`, `mercenary.rs`; the scripts: `ai.rs`, `ai/host.rs`, `ai/sight.rs`,
  `ai/prelude.lua` (tests `ai/tests.rs`)
- Packets: `net/pets.rs`, `net/pets/requests.rs` (tests `net/pets/tests.rs`), `net/homunculus.rs` (tests
  `net/homunculus/tests.rs`), `net/mercenary.rs`, `net/companions.rs`
- App: `web/app/ai.rs`, `web/app/{incoming,state}/ai.rs`, `web/app/{incoming,clicks,state}/pets.rs`,
  `…/homunculus.rs`, `…/mercenary.rs` (the same three folders), `web/app/actions/taming.rs`
- Windows: `web/hud/pets.rs`, `web/hud/homunculus.rs`, `web/hud/mercenary.rs`, `web/hud/skill_list.rs`
- World: `web/world/ai.rs` (what a script is told of the map); the sprite each is drawn with, by its job:
  `creatures.rs` (see "The map world")
- Tables: `gateway/src/tables/pets.rs`, `tables/ai.rs`; `web/data/pets.rs`, `web/data/ai.rs`

**Taken from**
- `CMercenaryAI`, `CMercenaryAI::Process`, `CMercenaryAI::RegisterFunctions`, `CMercenaryAI::Load`
  (`Source/Contents/Actors/MercenaryAI.cpp`); the client's `AI\AI.lua`, `AI\AI_M.lua`, `AI\Const.lua`,
  `AI\USER_AI\AI.lua`
- `UIPetInfoWnd` (`Source/UI/Status/UIPetInfoWnd.h`), `UIPetEggListWnd`
  (`Source/UI/Items/UIItemIdentifyWnd.h`), `CSession::InitPetActionTable`
  (`Source/Contents/Pet/PetStatus.cpp`), `PetTalkTable.xml`
- `UIHomunInfoWnd` (`Source/UI/Status/UIHomunInfoWnd.h`), `UIMerInfoWnd` (`Source/UI/Status/UIMerInfoWnd.h`),
  `UISkillListWnd` (`Source/UI/Skill/UISkillListWnd.h`)
- `CGameMode::Zc_Mer_Par_Change`, `CGameMode::GetClosestValidCell`
  (`Source/Contents/Messenger/ClientMessenger.cpp`); the messages `MM_PET_ACT`, `MM_COMMAND_HOMUN`

**State**: tests (the functions the client gives a script and a script run in a test, the words of the
three windows, the packets). Seen in the game (2026-11-21): a homunculus follows its owner, walks to and
fights a monster named twice with Alt and the right button; its window on Alt+R with its figures, Feed
(the question, then the server's line for no food), del (the question, then it is gone), the list of its
skills. A mercenary called by a scroll follows, fights what Alt and the left button name twice, its window
on Ctrl+R with its figures and the end of its contract, Dismiss with its question. A pet hatched from its
egg by the Incubator (the list of eggs), follows, its window on Alt+J with its picture, the list of
commands, Feed (question, the line of the food left, the emotion) and Performance. Not seen: a skill of
either used, a pet's accessory and its talk, a renaming, a mercenary's kills and faith changing.

## 14. NPC scripts and books

What an NPC says and asks, and what its script shows and plays beside its words: cut-ins, marks on the
minimap, gauges, sounds, books.

**Files**
- Rules: `links.rs` (the links in what an NPC says), `books.rs`
- Packets: `net/talk.rs`, `net/scripted.rs`, `net/books.rs`
- App: `web/app/{incoming,clicks,state}/talk.rs`, `…/scripted.rs`, `…/books.rs` (the same three folders)
- Windows: `web/hud/talk.rs`, `web/hud/scripted.rs`, `web/hud/books.rs`
- World: `web/world/scripted.rs` (the bar that fills over someone)
- Tables: `web/data/books.rs`

**Taken from**
- `UISayDialogWnd` (`Source/UI/Dialogs/UISayDialogWnd.h`), `UIChooseWnd` (`Source/UI/Dialogs/UIChooseWnd.h`),
  `UINpcEditDialogWnd`, `UINpcTextEditDialogWnd`
- `UITagTextViewer` (`Source/UI/Text/UITagTextViewer.h`), `CCreateLineInfo`
  (`Source/UI/Text/CreateLineInfo.h`), `UIWindow::WrapPixelText`
- `UIIllustWnd`, `UIMultiIllustWnd` (`Source/UI/Items/UIIllustWnd.h`), the message `MM_SHOW_IMAGE`
- `UIBookWnd` (`Source/UI/Dialogs/UIBookWnd.h`): `OnCreate`, `OnProcess`, `SendMsg`, `LoadBookmark`;
  `CGameMode::ReceiveBookOpen` (`Source/Client/Mode/GameModeHandlers.cpp`), `data\bookitemnametable.txt`
- `CGameMode::DrawMiniMap` (`Source/Client/Mode/GameModeMiniMap.cpp`), `UIGage`, `UITransBalloonText`
- `CGameMode::ReceiveSound` (`Source/Client/Mode/GameModeSounds.cpp`), `CGameMode::PlayBgm`
  (`Source/Client/Mode/GameModeHandlers.cpp`)

**State**
- The window of a talk with an NPC: seen in the game.
- A menu's choices are the client's: one with no text is not listed and not counted
  (`CGameMode::ReceiveWindow17Entries`); before 2026-11-21 it was counted, and the smith's menu of what is
  worn sent a number the server dropped the player for. Seen in the game, and a test (`net/talk.rs`).
- A talk by the keyboard (2026-11-21; rules `talk.rs` with tests, `web/app/clicks/talk.rs talk_key`, called
  first of the game's keys in `web/app/keys.rs`; `Use::Answer` for the double click): Enter presses OK of the
  box that asks, else OK of the menu, else "next" or "close", and goes nowhere else
  (`UIWindowMgr::ProcessEnterKey`, `UIFrameWnd::SendMsg` at `UM_DEFPUSH`); the arrows up and down move the
  menu's picked line and stop at its ends, the first line picked as it opens (`ConfirmActiveDialog`,
  `SubmitActiveDialog`, `UIListBox::SendMsg`), or with no menu move what was said by 10
  (`TagTextViewer::UIScroll::SendMsg`); "next" and a menu's OK are not taken twice in 300 ms
  (`m_itemSendTick`); an empty box is not sent; a double click on a line is its OK
  (`UIListBox::OnLBtnDblClk`); Escape does nothing in a talk. Seen in the game with the Kafra of Prontera:
  next, the menu, the arrows to both ends with the list following, a choice, the next menu, "close", all by
  the keyboard, and Enter with a line typed in the chat went to the talk and left the line unsent. The boxes
  for a number and for words, the double click and the 300 ms: by tests and reading only.
- Links, books, the packets of talk, of what a script shows, and of a book: tests only.
- Cut-ins, marks, gauges, the sounds a script plays, the book's window: by reading only.

## 15. Quests

**Files**
- Rules: `quests.rs` (tests `quests/tests.rs`)
- Packets: `net/quests.rs` (tests `net/quests/tests.rs`)
- App: `web/app/{incoming,clicks,state}/quests.rs`
- Windows: `web/hud/quests.rs`, `web/hud/quests/pane.rs`
- World: `web/world/quests.rs` (the marks over NPCs)
- Tables: `gateway/src/tables/quests.rs`; `web/data/quests.rs`

**Taken from**
- `CUIRenewQuestUI` (`Source/UI/Quest/UIRenewQuestUI.h`): `OnCreate`, `OnInit`, `UpdateQuestLists`,
  `MatchesFilter`; `CUIQuestList`, `CUIQuestItem` (the same file)
- `CRenewQuestMgr`, `CRenewQuestMgr::IsInFilter` (`Source/Contents/Quest/RenewQuestMgr.cpp`),
  `QuestClassificationInfo.lub`
- `CUIOngoingQuestInfo` (`Source/UI/Quest/UIOngoingQuestInfo.h`), `CUIRecommendedQuestInfo`
  (`Source/UI/Quest/UIRecommendedQuestInfo.h`), `UINotifyQuestWnd`, `UIQuestDisplay`
- `CGameMode::ReceiveAddQuest` (`Source/Client/Mode/GameModePackets.cpp`), `CNpc::UpdateMinimapMarker`

**State**
- The quest window: seen in the game.
- Tabs, filters and search, the packets, the gateway's table: tests only.
- The pane of one quest, the list on the screen, the marks over NPCs: by reading only.

## 16. Minimap, world map, tips

The minimap and the marks on it, the World Map, the Tipbox, the Adventure Guide.

**Files**
- Rules: `minimap.rs`, `maps.rs`, `guide.rs`
- Packets: `net/marks.rs` (guild members and the boss on the minimap)
- App: `web/app/{incoming,state}/marks.rs`, `web/app/{clicks,state}/worldmap.rs`,
  `web/app/{clicks,state}/tips.rs`, `web/app/{clicks,state}/guide.rs`
- Windows: `web/view/game.rs` (the minimap in its corner), `web/hud/marks.rs`, `web/hud/worldmap.rs`,
  `web/hud/tips.rs`, `web/hud/guide.rs`, `web/hud/guide/pages.rs`
- Tables: `gateway/src/tables/maps.rs`, `tables/worldmap.rs`, `tables/tips.rs`, `tables/guide.rs`;
  `web/data/maps.rs`, `web/data/worldmap.rs`, `web/data/tips.rs`, `web/data/guide.rs`

**Taken from**
- `CGameMode::DrawMiniMap`, `CGameMode::DrawMiniMapMembers` (`Source/Client/Mode/GameModeMiniMap.cpp`),
  `UIMiniMapWnd` (`Source/UI/Navigation/UIMiniMapWnd.h`), `UIMinimapZoomWnd`
  (`Source/UI/Navigation/UIMinimapZoomWnd.h`), `CGameMode::ReceiveBossMonsterInfo`
- `UIRoMapWnd` (`Source/UI/Navigation/UIRoMapWnd.h`), `CWorldViewMgr`
  (`Source/Contents/World/WorldViewData.h`), `worldviewdata_list.lub`, `Towninfo.lub`, `mapInfo_true.lub`
- `UITipboxWnd` (`Source/UI/Dialogs/UITipboxWnd.h`), `System\tipbox.lub`
- `CUIAdventureGuide`, `CUIIntro` (`Source/UI/AdventureGuide/UIAdventureGuide.h`), `CAdventureGuideMgr`,
  `CAdventureGuideMgr::IsAvailable` (`Source/Contents/AdventureGuide/AdventureGuideMgr.cpp`),
  `CJobCodeForItem::IsJobAllowed` (`Source/Contents/Items/JobCodeForItem.cpp`)
- `mp3nametable.txt`
- A map's names and whether its sign shows: `data\mapNameTable.txt` (`CSession::LoadMapNameTable`,
  `Source/Contents/Map/MapNameTable.cpp`: a name and no sign) and, written over it, `mapInfo_true.lub`
  (`CMapInfoMgr::LoadScript`, run after it: `ClientMessenger.cpp:6346`; its name and its `notifyEnter`), so a map
  of both shows its sign as the Lua says (the owner saw Geffen's in the Windows client; 2026-11-21 the order of
  the two was put right here, tests only): `gateway/src/tables/maps.rs`
- A copy of a map made for one party (`0011@tower`) looked up as the map (`maps::original`):
  `ResourceMgr::Resolve` (`Source/Resource/ResourceAliases.cpp:17`), `CMapInfoMgr::FindMapInfo`,
  `CSession::GetBgmFileName`, `IsIndoorMapName`, `GetFogParameters`, `GetMapViewParameters`, `ScriptMapName`
  (`Source/File/MapName.cpp`), `CurrentMapScriptName`; used by `web/world/loading.rs` (files, indoors, fog,
  view, sky, effects), `web/data/maps.rs` (music, name, sign), `web/view/game.rs` (the minimap's picture)

**State**
- The Tipbox opening as the game is first entered: seen in the game.
- Where a cell lies on the minimap, a map's name and music, what the Guide shows a character of a level, the
  marks' packets, the gateway's tables of the Guide and the world map, the names of `mapNameTable.txt`, the
  name of a copy of a map: tests only (no copy of a map was entered in the game: every instance of the
  classic server asks for a party of two). `1@tower` itself, by `@warp`: seen loading, with its music.
- The World Map, the Adventure Guide's pages, the marks on the minimap: by reading only.
- The Tipbox, 2026-11-21, seen in the game: it opens and closes by Alt+D, its cross, Close and Escape and is
  dragged by its title; the box "Open on Startup" emptied keeps it shut at the next entry (after a reload
  too), and the choice is read again as the game is entered (`tips_at_start`), so a box emptied in another
  tab holds; an item's picture stands in the line where a tip writes `^i[number]` (`markup::pictured`, tests;
  `CCreateLineInfo::ParseItemImage`, `LayoutItemImage`, `UIItemImage`: 25 by 25, named when pointed at, its
  description by the right button), the items asked of the gateway as a tip is shown; a tip's text begins 20
  higher under a title of one line (`UITipboxWnd::RefreshTip`).

## 17. Options and hotkeys

Game Settings, Shortcut Settings, the game's menu, screenshots, what the browser keeps between visits.

**Files**
- Rules: `hotkeys.rs`, `screenshot.rs`, `kept.rs`
- Packets: `net/config.rs` (the switches the server keeps), `net/inventory/hotkeys.rs` (the hotkey bar)
- App: `web/app/settings.rs`, `web/app/hotkeys.rs`, `web/app/keys.rs`, `web/app/screenshot.rs`,
  `web/app/{clicks,state}/settings.rs`, `web/app/{incoming,clicks}/hotkeys.rs`
- Windows: `web/hud/settings.rs`, `web/hud/hotkeys.rs`, `web/hud/esc.rs`
- Tables: `gateway/src/tables/interface.rs` (keys and options); `web/data/interface.rs`

**Taken from**
- `CUIGameSettingsUI`, `CUIOptionItem`, `CUIOptionItem::RunFunction` (`Source/UI/GameSettings/GameSettingsUI.h`),
  `CGameSettingsMgr` (`Source/Contents/Configuration/GameSettingsMgr.h`), `GameSettings.lub`
- `UIHotKeyWnd`, `UIHotKeyWnd::Paint` (`Source/UI/GameSettings/UIHotKeyWnd.h`), `CUserHotkeyMgr`
  (`Source/Contents/Hotkeys/UserHotkeyMgr.h`), `UIWindowMgr::ProcessHotkeyAction`
  (`Source/UI/Window/UIWindowMgr.cpp`), `hotkey.lub`
- `UIEscOptionWnd` (`Source/UI/GameSettings/UIEscOptionWnd.h`)
- `CMode::TakeScreenshot` (`Source/Client/Mode/ModeScreenshot.cpp`), the message `MM_SCREENSHOT`

**State**: tests only (which key does what and how it is written, a screenshot's name and stamp, what is
kept, the switches' packet); the three windows are by reading only.

**The game's menu and the menu of the dead** (2026-11-21, seen in the game)
- Files: `web/hud/esc.rs`, its entry `ESC` in `web/hud/windows/talk.rs`, `press/general.rs` (`EscRise`),
  `web/app/clicks/general.rs`, `numbers.rs` (`coded`), `pvp.rs` (`RAISING`, `Zone::raises`, `raises_with`),
  `net/packets/zone.rs` (`rise`, CZ_STANDING_RESURRECTION 0x0292).
- Taken from: `UIEscOptionWnd` (`OnCreate`, `Paint`, `LayoutButtons`, the constructor's three shapes,
  `SendMsg`), `UIWindowMgr::MakeWindow` at `WID_ESCOPTIONWND` (185 of 640 across, 300 of 480 down),
  `ShortcutFilename` (`Source/File/ShortcutFilename.cpp`), `CGameMode::HasResurrectionItem`, `MM_REBIRTH`
  (`Source/Client/Mode/GameMode.cpp`), the death in `GameModePackets.cpp:11788`. The 2026 client shows this
  window to the dead; its older `UIRestartWnd` and `UIRebirthWnd` are still in its code, and nothing opens
  them.
- Seen: dying (`@die`) opens it at once, where the client puts it, titled "Game Options (code)"; Escape and
  the menu's icon open and close it; it is dragged by its title; "Return to last save point" asks first, and
  cancel brings the menu back, OK returns the character; with a Token of Siegfried carried a third button,
  Resurrection, raises the character where it lies. Tests: the code, the maps that let the dead rise, the packet.

**The Hotkey Guide** (added with the classic gaps)
- Files: `hotkeys/guide.rs` (the client's table of keys, the list, how keys are written), `web/hud/hotkeys/guide.rs`,
  `web/app/clicks/hotkeys.rs` (`Guide`, `GuideClose`, `GuideSettings`), the entry `KEY_GUIDE` of
  `web/hud/windows/settings.rs`, `basic_button_holding` in `web/hud/buttons.rs` with the page's `.held`
  (`client/web/index.html`); the keyboard among the menu's icons opens it (`web/hud/basic.rs`).
- Taken from: `UIHotkeyGuideWnd` (`Source/UI/GameSettings/UIHotkeyGuideWnd.cpp`: `OnCreate`, `Paint`,
  `OnProcess`, `SendMsg`, `DrawKey`, `DrawKeyLabel`), `UIMenuIconWnd::SendMsg`, `ID_OPEN_HOTKEY`
  (`Source/UI/Menu/UIMenuIconWnd.cpp:432`), `UIHotKeyWnd::SendMsg`, `UM_TAB` from the guide
  (`Source/UI/GameSettings/UIHotKeyWnd.cpp:683`), `UIWindowMgr::EnsureWindow` at `WID_HOTKEYGUIDEWND`
  (`UIWindowMgr.cpp:7049`); the pictures of `data\texture\유저인터페이스\hotkey_guide\`.
- State: tests (the table, the list's places, how keys are written); seen in the game (2026-11-21): the
  guide opens from the keyboard icon with its keyboard and its lists, and Shortcut Settings opens from the
  Esc menu with its four tabs. Changing a key in it was not tried.

## 18. The map world

A map's ground, models, water, fog, sky, weather, night, the camera, and the effects and sounds a map places.

**Files**
- Formats: `map.rs`, `map/gat.rs`, `map/gnd.rs` (tests `map/tests.rs`), `model.rs`, `model/rsw.rs`,
  `model/rsm.rs`, `model/matrix.rs` (tests `model/tests.rs`), `sprite.rs`, `sprite/{spr,act,imf,canvas,layers}.rs`
  (tests `sprite/tests.rs`), `bmp.rs`, `bytes.rs`
- Rules: `scenery.rs`, `scenery/fog.rs`, `scenery/views.rs`, `scenery/dark.rs`, `scenery/makers.rs`,
  `prim/outdoors.rs`
- World: `web/world.rs`, `web/world/scene.rs`, `web/world/state.rs`, `web/world/loading.rs`,
  `web/world/land.rs`, `web/world/camera.rs`, `web/world/scenery.rs`, `web/world/scenery/placed.rs`,
  `web/world/prims/weather.rs`, `web/world/prims/places.rs`, `web/world/frame.rs`,
  `web/world/frame/{sprites,marks,effects}.rs`, `web/world/hooks.rs`
- GPU: `web/world/gpu.rs`, `web/world/gpu/{draw,land,layers,textures}.rs`, `web/world/shaders.rs`
- Tables: `gateway/src/tables/scenery.rs` (the sky); `web/data/scenery.rs`
- Gateway: `gateway/src/grf.rs`, `routes.rs`, `images.rs` (the client's files, as they are or as PNG)

**Taken from**
- `CGameMode::ApplyMapFog` (`Source/Contents/World/WorldObjects.cpp`), `CSession::GetFogParameters`
  (`Source/Client/Session/Session.cpp`), `CRendererDX9::ApplyFog` (`Source/Render/DX9/RendererFogDX9.cpp`),
  `fogparametertable.txt`
- `CGameMode::InitializeCamera` (`Source/Client/Mode/GameMode.cpp`), `CView::Apply`
  (`Source/Contents/Camera/Camera.h` has `CView`), `CSession::GetMapViewParameters`, `viewpointtable.txt`
- `CWeatherEffect::ApplyMapWeather`, `CWeatherEffect::StartDarken`, `CMapSkyMgr::Apply`
  (`Source/Contents/Map/MapSkyMgr.cpp`), `MapSkyData.lub`, `etcinfo.txt`
- `CWorld::LoadMapEffects` (`Source/Contents/World/WorldEffects.cpp`), `CEffectMaker::OnProcess`
  (`Source/Contents/Effects/Core/EffectMaker.cpp`), `CSoundMaker::OnProcess`
  (`Source/Contents/Sound/SoundMaker.cpp`)
- `CRagEffect::Snow`, `CRagEffect::Sakura`, `CRagEffect::SkyCloud`, `CRagEffect::ForestLight`
- `CRenderObject::DrawQuad` (`Source/Contents/Actors/GameObject.cpp`), `CImfRes`
  (`Source/Resource/Sprites/ImfResource.h`)
- The layouts of GAT and GND follow korangar's (said in `map/gat.rs`, `map/gnd.rs`)

**Effects of classic maps added after the audit** (none seen in the game)
- `prims/places.rs`: the pillars of light 231, 247, 259, 260 (`CRagEffect::TexturedCrossStaged`), the great
  portals of Juperos 561, 562 (`TexturedMagicRing(char)`), the ghosts and bats of Niflheim 358 to 360
  (`Ghost`, `PrimGhost`), the smoke 373; `prims/waterfalls.rs`: 349 to 356 (`Waterfall`, `FrostScatter`) and
  557 to 560 (`WaterfallSheets`).
- The particle emitters a map places (effect 974, `CEmitterEffect`) as its script sets them
  (`Lua Files\EffectTool\<map>.lub`, `EffectLists::LoadMapEffectScript`, `SetEmitterInfo`): the system
  `emitters` — `gateway/src/tables/emitters.rs` (asked for a map at a time, `GET /tables/emitters/<map>`),
  `web/data/emitters.rs`, `emitters.rs` (the rule of an emitter and its particles, `EmitterEffect.cpp`),
  `web/world/scenery/emitters.rs`, `web/world/gpu/emitters.rs`. Of the classic maps the `.rsw` places them on
  Prontera (3), Morocc (81) and Comodo (2) only; the scripts of the other classic maps set emitters their
  `.rsw` does not place, so the client shows nothing there either. State: tests for the gateway's reader
  (against the client's files) and the rule; drawn by reading only. The owner's gateway must be rebuilt and
  started anew for the table; until then no emitter is shown.
- Seen in the game (2026-11-21, each at 60 frames a second, the longest frame 18 ms): the dust of Morocc (a
  pale cloud, 81 emitters; the same rate with effects off), the pillars of Yuno and of `yuno_in02` (blue and
  green), the snow of `xmas`, Comodo, Niflheim, `nif_fild01`, Jawaii, `yggdrasil01`, `jupe_core`,
  `yuno_fild01`, Gonryun, Izlude, Hugel as they stand where the character came in. Not found in the views
  taken: the smokes of Prontera, the ghosts and bats of Niflheim, the waterfalls of Yggdrasil, the portals of
  Juperos (they are elsewhere on those maps).

**State**
- A model whose picture is a TGA (mixed in by its alpha) is bound through the layout of the pipeline that
  draws it (`web/world/land.rs batch`): bound through the other pipeline's, the browser refused every frame
  of such a map and it stayed black (Izlude). Seen in the game: Izlude is drawn.
- Tests only: reading GAT, GND, RSW, RSM, SPR, ACT and IMF, a map's fog and view, the darkening at night,
  the makers of a map's effects and sounds.
- By reading only: everything drawn (ground, models, water, fog, sky, weather, night, camera). The map is on
  the screen behind everything on the seen list, but it is not itself on that list: nobody compared it part
  by part with the Windows client.

### Granny models (guild flags, the Emperium, guardians, treasure boxes)

Whoever the client's table of jobs names with `.gr2` is drawn as a model of bones and skin: the guild flag
(722), the Emperium (1288, 1846, 3308), the guardians (1285–1287, 1829, 1830, 1899, 1900, 1949, 1950) and
the castles' treasure boxes (1324–1363 and the later ones, all `TREASUREBOX_2.gr2`).

**Files**
- Format: `model/granny.rs`, `model/granny/{file,oodle,coder,types,scene,pose}.rs`, `model/granny/bink.rs`,
  `model/granny/bink/{bands,wavelet,coder}.rs` (tests `model/granny/tests.rs`, `model/granny/bink/tests.rs`;
  `model/granny/section.bin` is 192 bytes of one of the client's files, for the test of the unpacking)
- Rules: `model/granny/shown.rs` (actions and their files, speed, turning, place, light, the pointer's reach)
- World: `web/world/granny.rs`; `Look::Model` in `web/world/mover.rs`, chosen in `web/world/actors.rs`;
  `web/world/guild.rs` (the emblems' pixels); `web/world/pick.rs`
- GPU: `web/world/gpu/granny.rs`, `GRANNY_SHADER` in `web/world/shaders.rs`
- App: `web/app/incoming/actors.rs`, `web/app/incoming/guild.rs` (a flag's emblem asked for; ZC_CHANGE_GUILD)

**Taken from**
- `CGameMode::NewActor` (`Source/Client/Mode/GameMode.cpp`), `C3dGrannyGameActor` (`SetSprAct`, `SetAction`,
  `Render`, `RenderLit`, `RenderShadowed`, `OnProcess`: `Source/Contents/Actors/RenderObject.cpp`),
  `C3dGrannyModelRes::Load`, `InstantiateModel`, `CreateTexture`, `PlayAnimation`, `GetMeshRenderData`,
  `ProjectBounds`, `GrannyAnimationLibrary::Load` (`Source/Resource/Models/AnimatedModelResource.cpp`,
  `ModelResource.cpp`)
- The file's layout and its types: nwn2mdk (`gr2_file.cpp`, Apache-2.0) and opengr2 (`gr2.c`, `typeinfo.c`,
  MPL-2.0), as descriptions. Neither reads Oodle0 (the packing of these files' sections) nor the Bink
  packing of the textures: both were read out of the client's own `granny2.dll` 2.1.0.5 (the functions are
  named in `coder.rs`, `oodle.rs`, `bink.rs` and its children).
- Curves and poses: Granny's own terms and the usual B-spline; not read out of the DLL.

**State**
- Tests: the unpacking of a section (a fixture of 192 bytes), the types, a curve, a skin with its bones, the
  rules of `shown.rs`, the pieces of the texture packing. With the client's files in a folder
  (`ROWEB_GR2=<folder>`): all six models and the fourteen files of other actions read, every texture unpacks.
- Seen in the game (2026-11-21, a build of this change alone on a gateway of its own, fog switched off with
  `/fog`): the two flags at the gate of a castle on `gef_fild13` with their cloth waving, a guardian (1285)
  and a treasure box (1324) called with `@monster`, the Emperium in its room of `gefg_cas01`; the Emperium's
  name under the pointer; a press on it walks the character to it.
- Seen in the game (2026-11-21, later): a guild's emblem on its castle's flags, in the middle of the cloth
  under its top (the 24 pixels in a texture of 32, `shown::emblem`).
- By reading only: the other actions of a guardian
  (walking, a blow, a flinch, the fall), a model's life bar, the fading of one struck down.

## 19. Sound

**Files**
- Rules: `earshot.rs`
- Page: `web/sound.rs` (the title music, a map's music, the effects' sounds)
- World: `web/world/earshot.rs`, `web/world/prims/sounds.rs`
- Tables: the music of each map is `maps.rs` (`music_table`, the client's `mp3nametable.txt`)

**Taken from**
- `CSoundMgr` (`Source/Contents/Sound/SoundMgr.h`): `PlayWave`, `PlayData2D`, `FadeThread`, `FadeIn`
  (`Source/Contents/Sound/SoundMgr.cpp`)
- `CGameMode::PlayBgm` (`Source/Client/Mode/GameModeHandlers.cpp`), `CGameMode::UpdateLoopSounds`
  (`Source/Client/Mode/GameMode.cpp`), `CLoginMode`
- `CRenderObject::PlayFrameSounds` (the sounds of a sprite's frames), `CSoundMaker`

Which sounds an effect and a skill start is section 4's (`effect_tables/effect_sounds.txt`,
`effect_tables/skill_sounds.txt`, both generated).

**State**: tests only for how loud a sound is by its distance (`earshot.rs`) and the music table (`maps.rs`);
playing itself (`web/sound.rs`) was checked in the game by the files asked for, their moments and loudness
(section 4), not by ear.

## 20. Server notices

The lines the server has the client say in the chat, the list of states (buffs and debuffs) at the right of
the screen, and the waits before skills may be used again.

**Files**
- Rules: `statuses.rs` (with `statuses.txt`), `cooldowns.rs`
- Packets: `net/notices.rs` (tests `net/notices/tests.rs`), `net/statuses.rs`, `net/cooldowns.rs`
- App: `web/app/{incoming,state}/notices.rs`, `web/app/{incoming,state}/statuses.rs`,
  `web/app/{incoming,state}/cooldowns.rs`
- Windows: `web/hud/statuses.rs`, `web/hud/cooldowns.rs`
- Tables: `gateway/src/tables/statuses.rs`; `web/data/statuses.rs`; the client's messages are in
  `tables/interface.rs`

**Taken from**
- `CGameMode::ReceiveFormattedMessage`, `CGameMode::ReceiveEquipItemDamaged`,
  `CGameMode::ReceiveRentalItemTime`, `CGameMode::Zc_Msg_Color` (`Source/Client/Mode/GameModePackets.cpp`),
  `CGameMode::ReceiveExpGainNotice` (`Source/Client/Mode/GameModeNotices.cpp`); rAthena's `clif_msg`
- `CGameMode::ReceiveStatusChangeEntry`, `CGameMode::ReceiveStatusNotice`
  (`Source/Client/Mode/GameModeHandlers.cpp`), `CRagEffect::StatusIconColumn`
  (`Source/Contents/Effects/Core/RagEffect.cpp`), `CSession::GetEFSTImgFileName`
  (`Source/Contents/Actors/Resources/StatusIconImages.cpp`), `CGameMode::ProcessStatusIconTooltip`
- `CSession::IsRequestReady`, `CSession::m_requestTimers` (`Source/Client/Session/Session.cpp`),
  `Source/UI/Cooldowns`

**State**: tests only (which line is said in which colour, the list of states, the waits, the packets); the
list and the shade on the screen are by reading only.

**Manner points, and the packets taken with nothing to show** (added with the classic gaps)
- Files: `notices.rs` (`manner_answer`, `manner_given`), `net/notices.rs` (0x014a, 0x014b),
  `web/app/incoming/notices.rs`. Taken from `case ID_ZC_ACK_GIVE_MANNER_POINT` and
  `case ID_ZC_NOTIFY_MANNER_POINT_GIVEN` (`Source/Client/Mode/GameModePackets.cpp:25768, 25783`).
- Taken and left without effect, each after reading the client's handler: 0x0283 `ZC_AID` and 0x01e0
  `ZC_ACK_ACCOUNTNAME` (an arm that does nothing, `GameModePackets.cpp:23432`), 0x0b1b (`ReceiveDeferredReset`,
  `GameModeHandlers.cpp:2037`: the hotkeys' lock and the queue of packets held back), 0x0b0b
  (`RefreshItemListWindow`, `Source/Contents/Items/ClientItemLists.cpp:670`: our windows are drawn as each
  list arrives), 0x0b18 (`ReceiveExtraInventorySlots`), 0x0a9b (`ReceiveSwapEquipmentList`), 0x0a23
  (`ReceiveAchievementList`) — the three last are of the Renewal pool (`BACKLOG.md`) — and 0x02ef
  `ZC_NOTIFY_FONT` (`GameModePackets.cpp:26919`; see `docs/DEBTS.md`). They are arms of
  `web/app/incoming/{travel,commands,inventory,notices,actors}.rs`.
- State: tests only (the manner lines and packets); by reading only otherwise.

## 21. Link watch

The watch on a connection gone silent, why a server ends a session, and this project's own list of the map
server's packets that are not read yet (`/packets`).

**Files**
- Packets: `net/link.rs`, `net/unread.rs`, `net/mod.rs` (packet lengths and cutting the stream, with
  `net/packet_lengths.txt`, `net/packet_lengths_more.txt`, `net/packet_names.txt`), `net/golden.rs` with
  `net/golden.txt` (every packet the client writes, compared byte for byte)
- App: `web/app/actions/link.rs`, `web/app/{incoming,state}/link.rs`,
  `web/app/{incoming,clicks,state}/unread.rs`, `web/socket.rs`
- Windows: `web/hud/unread.rs`
- Gateway: `gateway/src/bridge.rs` (the WebSocket bridged to the game server's TCP)

**Taken from**
- `CPingLiveSchedule::Process` (`Source/Client/Mode/GameMode.cpp`), `CGameMode::SendSyncRequest`
  (`Source/Network/Packets/GameTickPacket.cpp`)
- `CGameMode::ReceiveNotifyBan` (`Source/Client/Mode/GameModePackets.cpp`), `CLoginMode::ReceiveNotifyBan`
  (`Source/Client/Mode/LoginModeNotices.cpp`), `UIDisconnectedServerMsgWnd`
  (`Source/UI/Dialogs/UIDisconnectedServerMsgWnd.h`)
- The list of unread packets has no model in the Windows client; its names come from rAthena
  (`net/packet_names.txt`)

**State**: tests only (the watch, the reasons a session ends, counting unread packets, cutting the stream,
the golden packets); the `/packets` window is by reading only.

### Windows: steps of the wheel, the message box, tooltips (2026-11)
- Files: `scrolling.rs` (a notch of the wheel as rows, lines or 30 pixels: `UIItemBaseWnd::OnWheel` and its
  like, `UIListBox::OnWheel`, `UITagTextViewer::SendMsg`), `message_box.rs` (`UIMessageBox::Configure`,
  `WrapRichText`), `web/parts.rs stepped`, `web/app/wiring.rs` (`wire_wheel`, `wire_tooltip`, the double
  click counted on the second press: `press/things.rs doubled`), `web/hud/lists.rs` (`rows_stepped`,
  `lines_stepped`, `text_stepped`), `web/hud/boxes.rs confirm_box`, `web/hud/texts.rs linked_with`, `.bar` of `index.html`.
- Seen in the game: what a smith says with its bar, moved by the wheel and the arrows; the homunculus's
  question of six lines in the message box, three lines a notch; an NPC's shop moved three whole rows a notch;
  a double click on a line of an NPC's menu answering it (it did nothing before: the first press drew the line
  anew, and a page sends no double click then); the box that asks for a number taking the keyboard as it opens
  and Enter sending what was typed (it sent 0 before: the box had no keyboard); a tooltip of a menu icon gone
  when the pointer goes onto the world, and when its thing is taken away under a still pointer; an item's link
  in a tip opening the item, a tip's link the tip; the storage closed by a warp (it stayed, and could not be
  closed).
- Tests only: `vending::own_title` (one's own stall titled with its name), `links::Link::number`.

## Two players at once (playtest of 2026-11-21 and 2026-11-26)

Two accounts (`play6` a Crusader, `play7`) in two tabs through two copies of the gateway, each thing looked
at on both screens. **Seen in the game**, as the one who does it and as the other sees it:
- Coming into sight, walking (the same cells on both screens), sitting and standing, turning (`/bingbing`),
  an emotion, a line over the head and in both chats (wrapped over the head), a job changed, hair colour,
  headgear put on and taken off, lying dead and rising, hidden and cloaked (gone from the other's screen),
  a cart (`@cart`, the state of new clients: the old option bit draws none, as in the client), a falcon, a
  homunculus, the aura of level 99, the name with the party's under the pointer.
- On a Peco Peco, as the other sees it: standing, walking, turning, sitting (the bird crouches: the sprite's
  own sitting), striking with the sword, lying dead, getting off. No shield shows on a rider: the client's
  table names the folder `방패\신페코크루세이더` for it, of which the client's files have nothing.
- What is worn on the head and the shield (the owner's report of 2026-11-21, "glasses and shield not
  drawn"): nothing was lost on the way. The three places are drawn in the order of `TB_Layer_Priority.lub`
  (middle 100, top 200, low 300), so a Flu Mask lies over the lower rims of Glasses and little of them is
  left to see; alone or under a hat they show, on the own character, on the other's screen, and on one who
  comes into sight wearing them. A shield (and the weapon) is drawn only standing ready and striking, as the
  client's `IsReadyOrAttackAction` has it.
- Whispers: the window on both sides, the answer from it, `/ex` and `/in` with a name, `/ex` alone (the
  list), `/exall`, `/inall`, a name that is not there, `/am` with its automatic answer. The word filter
  ("this is ****" on both screens).
- Party: `/organize` with its setup, `/invite`, the question, refused and accepted, the list with levels,
  maps and HP on both, `%` chat, the mark on the minimap, EXP shared evenly and refused for the gap of
  levels, the leader handed over, a member expelled, `/leave`.
- Friends (by another hand on the same tabs): asked, accepted, removed, the list with its menu; "has logged
  out / in" told to the friend.
- A deal: asked from the menu on the player, refused, accepted, a thing and zeny from both sides, locked,
  traded, the sums right on both; called off after one side locked (what was put in comes back).
- Looking at the other's equipment from the menu.
- A chat room: made with a password (under four letters refused), its sign over the owner on the other's
  screen with the lock, entered, a line said in it, left; the room passing to who stays.
- A stall: the Vending skill, a thing from the cart with its price, the title; the sign over the seller on
  the other's screen, the stall opened by a press on it, two bought, "sold" and the buyer's name told to the
  seller, the zeny right on both.
- Skills on the other (a Priest on the Crusader): Heal, Blessing, Increase Agility, Kyrie Eleison, Aspersio
  (their effects on the caster's screen, the states with their times on the target's), Resurrection on the
  dead one; Lex Aeterna pressed on the friend without Shift goes nowhere.
- PVP (`pvp_y_1-2`): a member of the party is not fought, anyone else is with a plain press; the numbers of
  blows on both screens; death there with "Return to last save point"; the standing in the corner
  ("1 / 1") — it was never written (see below). `@duel`: invited, accepted, the opponent fought with a plain
  press on an ordinary map.
- Found and put right: the standing on a PVP map was never shown, the line being asked whether there was
  anything to write before the pictures it is written in were read (`web/app/incoming/pvp.rs rank_shown`).

## Between the systems

Not systems of the list above, but a newcomer will meet them at once.

- The menu the right button opens on another player: `web/hud/player_menu.rs`,
  `web/app/{clicks,state}/player_menu.rs`, `press/player_menu.rs`. Taken from `CGameMode::OpenActorMenu`. Its
  rows lead into equipment, deal, party, guild, whisper, friend, mail and block. State: seen in the game.
  The master's two rows (alliance, antagonist) are offered only for a relation that does not stand, and not
  on a player of no guild (`guild::relation_rows`, the owner's decision of 2026-11-21 against the client):
  tests only.
- What every window is made of: `web/hud.rs`, `web/hud/windows.rs`, and `web/hud/{frames,buttons,checks,fields,
  gauges,tabs,texts,boxes,colours,lists,items}.rs`; `web/parts.rs`, `web/view.rs`. State: by reading only.
- The order and docking of windows: `stack.rs`, `snap.rs`, `web/app/windows.rs`. State: tests only.
- The app's frame: `web/app.rs`, `web/app/action.rs`, `web/app/actions.rs`, `web/app/incoming.rs`,
  `web/app/clicks.rs`, `web/app/state.rs`, `web/app/wiring.rs`, `web/app/page.rs`; `press.rs`.
- Texts and tables: `i18n.rs` (tests `i18n/tests.rs`), `font.rs`, `tables.rs`, `net/bytes.rs`,
  `gateway/src/tables.rs` (tests `gateway/src/tables/tests.rs`). State: tests only.
