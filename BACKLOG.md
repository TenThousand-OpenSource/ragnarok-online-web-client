# Backlog: after the classic client is finished

The owner's order: first the classic (pre-renewal) client is made complete; what only later content uses waits
here. Each line is something the Windows client has and this one does not, found by the audits of 2026-11-19.
Workers add to it what they meet and leave for later; nothing here is to be built until the owner says so.

## Interface
- Cash shop, mileage, cash emotions (buying); auction; roulette; attendance; stylist and dress room.
- Achievements and titles; bank; reputation; rune and collection systems; probability table.
- Navigation (`UINavigationWnd`, NavigationData); replay; adventurer agency and party booking.
- Battlegrounds queue and status; instances window (memorial dungeons); clans.
- Buying stores; shop search; barter and NPC markets.
- Refine UI, grade enchant, enchant, item reform, Lapine boxes, random options, select-package boxes.
- Item names: the grade after the refinement (`ITEM_INFO::Grade`); an item's description: the lines of its
  random options, the comparison with what is worn (`UIItemCollection_ComparisonWnd`).
- Lists a skill opens: Shadow Spell's (ZC_SKILL_SELECT_REQUEST 0x0442 into `UISpellListWnd`), runes and new
  poisons through the forging list (the results 4 to 7 of ZC_ACK_REQMAKINGITEM), `UIChangeMaterialWnd`.
- Equip switch, inventory expansion, stack merging, item compare, favourites tab.
- Shadow gear, second costume, trait stats of 4th jobs, the skill delay window.
- Private airship; captcha and macro detector; OTP / second password windows; character slot move.
- Pet evolution; elementals and ABR; champion monsters (`monster_size_effect`).

## World and characters
- GND 1.8/1.9, RSW 2.6+, RSM2 models; the sun's lens flare; signboards (`SignBoardList`); camera commands.
- Robes and wings (`RobeNameTable`), alternative outfits (`body2`), hat effects and footprints, damage skins.
- Wargs; auras above level 99; pillars of light on a rare drop; rental mounts.
- Granny models (`.gr2`: guardians and the Emperium of sieges) — needed for War of Emperium.
- The newer clouds of a sky (`Cloud_Effect` of MapSkyData.lub: `CCloud`, effect 0x47A) and the fog of its
  background (`BG_Fog`); the kinds of `CRagEffect::SkyCloud` only later maps use (7, 8, 10 and on).
- The effects later maps place in their `.rsw` (974, 946, 690 and on: lights, coloured torches, ghosts), and
  the emitters of the newer maps' `effecttool` files.

## Two servers
- Classic and Renewal as two separate servers on separate databases, chosen at login (asked for on 2026-11-18).
