# Backlog: after the classic client is finished

The owner's order: first the classic (pre-renewal) client is made complete; what only Renewal or later content
uses waits here. Each line is something the Windows client has and this one does not, found by the audits of
2026-11-19 and in the code's own comments (the file named is where the comment says so). Workers add to it
what they meet and leave for later; nothing here is to be built until the owner says so.

What the classic game has and this client lacks is not here: that is a debt, and belongs in `docs/DEBTS.md`.

## Interface
- Cash shop, mileage, cash emotions (the tabs of emotions bought with cash, the search and Buy:
  `client/src/web/hud/emotions.rs`); auction; roulette; attendance; stylist and dress room.
- Achievements and titles; bank; reputation; rune and collection systems; probability table.
- Navigation (`UINavigationWnd`, NavigationData), and what the World Map does with it: a press on a map that
  starts navigation there, and the routes (`client/src/web/hud/worldmap.rs`); replay; adventurer agency and
  party booking (the button is there and does nothing: `client/src/web/app/clicks/party.rs`).
- The menu icons of these are drawn faded and pressed for nothing (`client/src/web/hud/basic.rs`):
  Battleground, Navigation, Bank, Replay, Achievements, Cash Shop, Attendance, Adventurer's Agency,
  Reputation, Probabilities.
- Clans.
- Shop search; barter and NPC markets.
- Refine UI, grade enchant, enchant, item reform, Lapine boxes, random options, select-package boxes.
- Item names: the grade after the refinement (`ITEM_INFO::Grade`, `client/src/item_names.rs`); an item's
  description: the lines of its random options, the comparison with what is worn
  (`UIItemCollection_ComparisonWnd`).
- Lists a skill opens: Shadow Spell's (ZC_SKILL_SELECT_REQUEST 0x0442 into `UISpellListWnd`), runes and new
  poisons through the forging list (the results 4 to 7 of ZC_ACK_REQMAKINGITEM:
  `client/src/net/choosing/making.rs`, `client/src/choosing.rs`), `UIChangeMaterialWnd`.
- Equip switch, inventory expansion, stack merging, item compare.
- Shadow gear, second costume, trait stats of 4th jobs, the skill delay window.
- Private airship; captcha and macro detector; OTP / second password windows; character slot move.
- Pet evolution; elementals and ABR; champion monsters (`monster_size_effect`).
- An NPC's pictures over the whole screen (`UIMultiIllustWnd`, the kinds from 10 on of ZC_SHOW_IMAGE:
  `client/src/net/scripted.rs`).

## Jobs after the second ones
- The later kinds of Super Novice in `IsSuperNoviceClass` (`client/src/jobs.rs`, `super_novice`).
- The size of the baby selves of the jobs after the third ones, 0.82, and of the Doram, 0.8
  (`CSession::GetSkillFactor`: `client/src/pose.rs`, `size`).
- `CPc::SelectStanceFrame` for the Star Emperors and their like, and `CPc::MapMotion` for the third jobs
  (`client/src/pose.rs`, `stance_head` and `weapon_shown`); the frames of the later jobs' skills in
  `CGameActor::ProcessSkillMessage` (Sura, Rebellion, the fourth jobs).
- The auras of the later jobs (`client/src/aura.rs`, `client/src/web/world/prims/max_level.rs`): the third
  jobs and their mounts at `BaseLevel3rd` (effects 1226 and 1227), the fourth jobs at `BaseLevel4th`
  (the table's `MaxLevelEffect4th`), the second Super Novice, the expanded jobs and the Doram (1022 and
  1023), a homunculus at `BaseLevelHomun` (978 and 979); and the other picture and colour
  `EFST_EQUIPED_FLOOR` gives the aura.
- What the later jobs show in place of spirit spheres (`client/src/spirits.rs`): the coins of a Rebellion and
  a Night Watch (`CRichsCoinEffect`), the weapons of a Dragon Knight (`CServantWeaponNewEffect`), the effect
  2109 of an Abyss Chaser; `CEnergyOrbEffect` (`SetCharm`, `EFST_GENSOU`); the kinds 4 and 10 to 13 of
  `CRagEffect::ElementalOrbit` (the coins of `rcoin1.bmp`, the four elemental spirits `EF_CHOOKGI_FIRE` and
  the three beside it: `client/src/prim/tei/spheres.rs`).
- The falcon's later kin (`client/src/falcon.rs`): the Wind Hawk's skills that send it flying, the Ranger's
  owl and the Wind Hawk's bird as sprites. Wargs.
- What the later jobs' skills leave on the ground: every unit kind of theirs, most of which the client hands
  to its `CSkillActionMgr` (`client/src/skill_units.rs`), and the models of the Ranger's and the Windhawk's
  traps (`client/src/skill_units/traps.rs`).
- The effects of the later jobs' states (`client/src/state_effects.rs` lists them with their numbers):
  `EFST_ORATIO`, `EFST_BANDING`, `EFST_VENOMIMPRESS`, `EFST_FROSTMISTY`, `EFST_COLD`, `EFST_SIREN`,
  `EFST_MANHOLE`, `EFST_SHADOWFORM`, `EFST_CAMOUFLAGE`, `EFST_STEALTHFIELD`, `EFST_ELEMENTAL_VEIL`,
  `EFST_GRANITIC_ARMOR`, `EFST_KYOMU`, `EFST_GENSOU`, `EFST_KAGEMUSYA`, `EFST_ZANGETSU`, `EFST_MAXPAIN`,
  `EFST_BURNT`, `EFST_MISTY_FROST`, `EFST_PRISON`, `EFST_LOCKON_LASER`, `EFST_HELLS_PLANT_ARMOR`, the states
  that launch an `EF_STATUS_…_UP` (2288 to 2300); the six songs whose effect is an `EF_CIRCLEPOWER` (865, 867,
  871, 873, 875, 877); `EFST_HANDICAPSTATE_DEEPBLIND`, which darkens the player's own screen (`EF_BLIND` with
  `EF_STATUS_STATE`); `EFST_MEIKYOUSISUI` (Moonlit's wind in a colour of its own); and what the client does
  through its other effects' manager (`g_effectMgr`: the balls of `EFST_SUMMON1` to `5`,
  `EFST_ROLLINGCUTTER`, the stances, the marks and the hats).
- The copies of a character of the later jobs' states (`EF_MAKEHALLU` 792 of `EFST_HALLUCINATIONWALK`,
  `EFST_STRIKING`, `EFST_WUGDASH`) and `EF_HALLUTIME` (793, `TexturedRisingOrbs` kind 3) that a `CHalluPC`
  launches at its 110th pass (`client/src/body/copies.rs`).

## World and characters
- RSW 2.6+ (`client/src/model/rsw.rs` reads 1.2 to 2.5), RSM2 models (`client/src/model/rsm.rs` reads 1.1 to
  1.5); the sun's lens flare; signboards (`SignBoardList`).
- Robes and wings (`RobeNameTable`), alternative outfits (`body2`), hat effects, damage skins. Of footprints
  a runner's are there (`CPc::UpdateFootprints` with its kind 2, `client/src/body.rs`); its kind 1, which
  `CPc::Render` asks for by another flag of the character's state, is not (what sets that flag was not
  looked up).
- Pillars of light on a rare drop; rental mounts.
- The newer clouds of a sky (`Cloud_Effect` of MapSkyData.lub: `CCloud`, effect 0x47A) and the fog of its
  background (`BG_Fog`), left out in `gateway/src/tables/scenery.rs`; the kinds of `CRagEffect::SkyCloud`
  only later maps use (7, 8, 10 and on).
- The effects later maps place in their `.rsw` (946, 690, 713, 1073 and on: lights, coloured torches, animated
  emitters; `EF_MAP_MAGICZONE` 650 and 651, which two classic maps, ice_dun03 and ice_dun04, place too: see
  `docs/DEBTS.md`). The plain emitters (974) are made, for the classic maps that place them (Prontera,
  Morocc, Comodo) and so for every map; of the `effecttool` files what no classic map uses is left: emitters
  turned by their own angles (`billboard_off`) or drawn over everything, the wind (`windEffectInfo`, effect
  2343), animated emitters, magic floors, the evil's paw, `ez2strInfo`, and the first version's `_map_Effect`.
- `EF_M01` (583): only skills of the third jobs launch it.

## Of the years before Renewal, put off with the rest
These were in the original game before Renewal, so by this file's own rule they may belong in
`docs/DEBTS.md` instead; they stand here until the owner decides, because they were put here on purpose.
- Battlegrounds queue and status; the instances window (memorial dungeons).
- Buying stores.

## Two servers
- Classic and Renewal as two separate servers on separate databases, chosen at login (asked for on 2026-11-18).
  Today there is one, switched by `rules.sh`; the list of servers is shown as the login server sends it
  (`client/src/web/app/incoming/account.rs`).
