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
- Rules: `account.rs`, `jobs.rs`
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

**State**
- Login and character creation: seen in the game.
- What an ID, a password and an e-mail may be, the packets of the login and character servers, the gateway's
  accounts, guard and limiter: tests only (`account.rs`, `net/packets/tests.rs`, `gateway/src/accounts/tests.rs`,
  `limiter.rs`).
- The forms of registering and recovering, deleting a character: by reading only.

## 2. Movement

Walking, the server's clock, routes, pursuit, and everyone else in sight walking and changing their look.

**Files**
- Rules: `route.rs` (tests `route/tests.rs`), `server_time.rs` (tests `server_time/tests.rs`), `map/paths.rs`,
  `map/gat.rs`, `pose.rs`, `mounts.rs`, `cursor.rs`
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

**State**
- Walking, movement by the server's clock under lag, being knocked back by Fire Wall, the sprites of a job
  after a job change: seen in the game.
- The clock, a route's times, the path search, poses, mounts, the pointer's pictures, the packets of those in
  sight: tests only.
- Pursuit (walking on towards someone to attack or to pick up, `web/world/pursuit.rs`): by reading only.

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
- When a blow lands, how many hits it is made of, what each shows and sounds like, how the numbers fly, the
  colour of a name, who is fought on a PVP map, the packets: tests only.
- Death, the bars over heads, the two windows, the standing on a PVP map: by reading only.

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
- Tests only: how the kinds of primitive live and are shaped (`prim/tests.rs`, `prim/particle_tests.rs`,
  `prim/slots_tests.rs`, `prim/walls_tests.rs`, `prim/horn_tests.rs`, `prim/tei/tests.rs` and the tests in the
  kinds' own files), the `.str` format and how a carried file flies (`effect/tests.rs`,
  `effect/carried/tests.rs`), units, traps and the circle under a cast, what states begin, end and keep
  (`state_effects/tests.rs`, `ailments/tests.rs`, `body/tests.rs`), the aura, spirit spheres, the packets.
- By reading only: every other skill's effect as it is drawn (all of `web/world/prims/` beyond the skills
  above), casting bars and aiming, the aura and the spirit spheres on the screen.

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
- Tabs, weight, how an item is named, item links, how numbers are written and typed, the packets: tests only.
- Another player's equipment, the Fav tab, dragging between windows: by reading only (their packets have
  tests).

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

**State**: tests only (what may go into a forging, and every packet); the windows are by reading only.

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

**State**: tests only (the rules of the four and their packets); the windows, the cart and the signs in the
world are by reading only.

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

**State**: seen in the game, with zeny and with items. Its rules and packets also have tests.

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

## 11. Guild

**Files**
- Rules: none of its own (there is no `guild.rs`)
- Packets: `net/guild.rs`, `net/guild/members.rs`, `net/guild/manage.rs`, `net/guild/requests.rs` (tests
  `net/guild/tests.rs`)
- App: `web/app/incoming/guild.rs`, `web/app/incoming/guild/members.rs`, `web/app/clicks/guild.rs`,
  `web/app/clicks/guild/manage.rs`, `web/app/state/guild.rs`
- Windows: `web/hud/guild.rs`, `web/hud/guild/{members,positions,skills,expelled,notice}.rs`
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

**State**
- Making a guild, inviting, leaving, expelling, the emblem, the notice: seen in the game.
- Positions, alliances and oppositions, the guild's skills, the list of the expelled, the guild's chat: tests
  only for their packets; their windows are by reading only.

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
- World: `web/world/ai.rs` (what a script is told of the map)
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

**State**: tests only (the functions the client gives a script and a script run in a test, the words of the
three windows, the packets); the windows are by reading only. No pet, homunculus or mercenary is on the seen
list.

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

**State**
- The Tipbox opening as the game is first entered: seen in the game.
- Where a cell lies on the minimap, a map's name and music, what the Guide shows a character of a level, the
  marks' packets, the gateway's tables of the Guide and the world map: tests only.
- The World Map, the Adventure Guide's pages, the marks on the minimap: by reading only.

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

**State**
- Tests only: reading GAT, GND, RSW, RSM, SPR, ACT and IMF, a map's fog and view, the darkening at night,
  the makers of a map's effects and sounds.
- By reading only: everything drawn (ground, models, water, fog, sky, weather, night, camera). The map is on
  the screen behind everything on the seen list, but it is not itself on that list: nobody compared it part
  by part with the Windows client.

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

**State**: tests only for how loud a sound is by its distance (`earshot.rs`) and the music table (`maps.rs`);
playing itself (`web/sound.rs`) is by reading only.

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

## Between the systems

Not systems of the list above, but a newcomer will meet them at once.

- The menu the right button opens on another player: `web/hud/player_menu.rs`,
  `web/app/{clicks,state}/player_menu.rs`, `press/player_menu.rs`. Taken from `CGameMode::OpenActorMenu`. Its
  rows lead into equipment, deal, party, guild, whisper, friend, mail and block. State: seen in the game.
- What every window is made of: `web/hud.rs`, `web/hud/windows.rs`, and `web/hud/{frames,buttons,checks,fields,
  gauges,tabs,texts,boxes,colours,lists,items}.rs`; `web/parts.rs`, `web/view.rs`. State: by reading only.
- The order and docking of windows: `stack.rs`, `snap.rs`, `web/app/windows.rs`. State: tests only.
- The app's frame: `web/app.rs`, `web/app/action.rs`, `web/app/actions.rs`, `web/app/incoming.rs`,
  `web/app/clicks.rs`, `web/app/state.rs`, `web/app/wiring.rs`, `web/app/page.rs`; `press.rs`.
- Texts and tables: `i18n.rs` (tests `i18n/tests.rs`), `font.rs`, `tables.rs`, `net/bytes.rs`,
  `gateway/src/tables.rs` (tests `gateway/src/tables/tests.rs`). State: tests only.
