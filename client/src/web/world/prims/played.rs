//! Effects that are a sprite playing at a place, each with the figures of its arm of `CRagEffect::OnProcess`
//! and of the function that arm calls:
//!
//! - a sprite that plays once on the one the effect is on (`TexturedClassChange`): `EF_STONERING` (797),
//!   `EF_HAMICASTLE` (570),
//!   `EF_HAMIBLOOD` (571), `EF_M02` (584, Full Buster), `EF_M07` (589), `EF_KIRIKAGE` (630), `EF_KASUMIKIRI`
//!   (632), `EF_ISSEN` (633), `EF_DESPERADO` (637), `EF_RAPIDSHOWER` (643), `EF_MAGICALBULLET` (644),
//!   `EF_SPREADATTACK` (645), `EF_TRACKING` (647), `EF_TRIPLEACTION` (648), `EF_SPR_MASH` (729);
//!   `EF_NPC_EARTHQUAKE` (666), which shakes the screen three times besides (`MM_QUAKE`); and what things
//!   that are used show: `EF_ITEM_THUNDER` (576) to `EF_ITEM_RAIN` (580), the five boxes of a mood;
//!   `EF_POK_LOVE` (682) to `EF_POK_CHRISTMAS` (686), the firecrackers of the feasts; `EF_ITEM318` (707);
//! - the hearts of chocolate and their like, a sprite that plays once on the one the effect is on
//!   (`Vallentine`): `EF_VALLENTINE` (363) and `EF_VALLENTINE2` (364), the first and the second action of one
//!   sprite; `EF_05VAL` (509); `EF_ITEMFAST` (519), the sign of a potion of speed. (`EF_RO2YEAR`, 545, is the
//!   same function with `ro2effect.spr`, which this client's files do not have: it is left out.)
//! - `EF_EL_GUST` (947) to `EF_EL_HEATER` (958), `EF_SILENT_BREEZE` (961), `EF_MAGMA_FLOW` (962) and
//!   `EF_NPC_WIDEWEB` (1053): a sprite at the feet of the one it is on (`TexturedClassChange`, kinds 70 to 83
//!   and 87);
//! - `EF_TATAMI` (631), `EF_KAEN` (634) and `EF_LIGHTNING_S` to `EF_FLARE_S` (638 to 642): the sprite on a
//!   cell of Tatami Gaeshi, of Kaensin and of Ground Drift (`TexturedClassChange`, kinds 16, 19 and 21 to 25),
//!   which the units launch on themselves (`crate::skill_units`);
//! - `EF_POISONHIT` (53): the hit of poison, a sprite that plays once on the one struck (in `OnProcess` itself);
//! - `EF_THUNDERSTORM2` (622): the lightning of Raigekisai (in `OnProcess` itself);
//! - `EF_DARKBREATH` (212): the dark breath (in `OnProcess` itself);
//! - `EF_CHIMTO` (267): the palm of Investigate at the one struck (`CHIMTO`, flag 0);
//! - `EF_CHIMTO2` (723): the spears of Hundred Spear at the one struck (`CHIMTO`, flag 12);
//! - `EF_FVOICE` (606), `EF_WINK` (607): the cry of Pang Voice and the kiss of Wink of Charm at the one they
//!   are meant for (`CHIMTO`, flags 2 and 1). (Their sounds the client starts in its handler of the skill.)
//!
//! All of these follow the one they are on (their functions take their parent's place every tick); the two of
//! `CHIMTO` put their sprite at the place they are told. But for the palm and the breath the sprites are drawn
//! layer by layer in the colours their files give the layers (`pattern & 0x200`). The client also sets a bit of
//! the same number in how such a sprite is drawn (`renderFlag & 0x200`); what that bit does was not found, and
//! it is left out.
//!
//! The client places the sounds of most of these where the effect is (`PlayEffectWave`); here a sound is only
//! started. `ReceiveSkillUse` and `PlaySkillEffect` start more sounds beside some of these effects (the
//! castling of `HAMI_CASTLE`): those are the skill's, not the effect's, and are not here.

use super::*;
use crate::prim::quarter;

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[], sprites: &[&SPRITES, &KINDS_FROM_70, &ON_CELLS] };

/// The sprites the effects here show, among the client's sprites (the client names them `이팩트\name.spr`).
const SPRITES: [&str; 37] = [
    "이팩트/poisonhit",
    "이팩트/vallentine",
    "이팩트/05vallentine",
    "이팩트/fast",
    "이팩트/item_thunder",
    "이팩트/item_cloud",
    "이팩트/item_curse",
    "이팩트/item_zzz",
    "이팩트/item_rain",
    "이팩트/폭죽_러브",
    "이팩트/폭죽_화이트데이",
    "이팩트/폭죽_발렌타인",
    "이팩트/폭죽_생일",
    "이팩트/폭죽_크리스마스",
    "이팩트/mobile_ef04",
    "이팩트/fvoice",
    "이팩트/wink",
    "이팩트/어스퀘이크",
    "몬스터/stapo",
    "이팩트/캐슬링",
    "이팩트/블러드러스트",
    "이팩트/m_ef02",
    "이팩트/m_ef07",
    "이팩트/그림자베기",
    "이팩트/안개베기",
    "이팩트/일섬",
    "이팩트/데스페라도",
    "이팩트/래피드샤워",
    "이팩트/매지컬불릿",
    "이팩트/스프레드",
    "이팩트/트래킹",
    "이팩트/트리플액션",
    "이팩트/mashofa",
    "이팩트/썬더스톰",
    "이팩트/헌드레드스피어",
    INVESTIGATE,
    BREATH,
];

/// The sprite of Investigate and that of the dark breath (`이팩트\DarkBreath.spr`).
const INVESTIGATE: &str = "이팩트/침투경_";
const BREATH: &str = "이팩트/darkbreath";

/// The sprites of `TexturedClassChange`'s kinds 70 to 83 and 87, in that order.
const KINDS_FROM_70: [&str; 15] = [
    "이팩트/거스트",
    "이팩트/블래스트",
    "이팩트/아쿠아플레이",
    "이팩트/업히벌",
    "이팩트/와일드스톰",
    "이팩트/칠리에어",
    "이팩트/커즈드소일",
    "이팩트/쿨러",
    "이팩트/트로픽",
    "이팩트/파이로테크닉",
    "이팩트/페트롤로지",
    "이팩트/히터",
    "이팩트/silent_brz",
    "이팩트/magma_flow",
    "이팩트/wideweb",
];

/// The sprites of `TexturedClassChange`'s kinds 16, 19 and 21 to 25, in that order: the mat of Tatami Gaeshi,
/// the fire of Kaensin and the five spheres of Ground Drift (wind, dark, poison, water, fire).
const ON_CELLS: [&str; 7] = [
    "이팩트/다다미",
    "이팩트/화염진",
    "이팩트/라이트닝스피어",
    "이팩트/블라인드스피어",
    "이팩트/포이즌스피어",
    "이팩트/프리징스피어",
    "이팩트/플레어스피어",
];

/// Which kind of `TexturedClassChange` an effect is, counted from its kind 70 (the kind 87 comes after the
/// kind 83 here); nothing for any other effect.
fn ground_sprite(effect: u32) -> Option<usize> {
    match effect {
        947..=958 => Some(effect as usize - 947),
        961 | 962 => Some(effect as usize - 961 + 12),
        1053 => Some(14),
        _ => None,
    }
}

/// How many ticks an effect lives (the client's `EffectLifetime`) and the sound it starts with (none: empty);
/// nothing for an effect that is not one of these. The sounds an effect starts later are in `sounds`.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        53 => (50, ""),
        212 => (500, ""),
        267 => (16, "effect/mon_침투경.wav"),
        363 | 364 | 509 | 519 => (100, "effect/vallentine.wav"),
        576..=580 | 707 => (100, ""),
        // (The client starts this one at the listener's own place, wherever the firecracker is.)
        682..=686 => (100, "effect/itempokjuk.wav"),
        570 | 584 | 589 | 643 | 645 | 647 | 648 => (100, ""),
        571 => (100, "effect/h_blood_lust.wav"),
        606 | 607 => (100, ""),
        622 => (100, "effect/ef_thunderstorm.wav"),
        630 => (100, "effect/그림자베기.wav"),
        631 => (100, ""),
        634 | 638..=642 => (23, ""),
        632 => (100, "effect/안개베기.wav"),
        633 => (100, "effect/일섬.wav"),
        637 => (200, ""),
        644 => (100, "effect/매지컬블릿.wav"),
        666 => (100, "effect/earth_quake.wav"),
        723 => (100, "hundred.wav"),
        729 => (100, "effect/t_에너지방출.wav"),
        797 => (300, "effect/wl_siennaexecrate.wav"),
        947..=958 | 961 | 962 => (100, ""),
        1053 => (999, ""),
        _ => return None,
    })
}

/// Whether an effect is one the client has the striker launch and then tells the place of the one struck
/// (`CSession::PlaySkillEffect`: Investigate, Full Buster, Spread Attack, Hundred Spear).
fn thrown(effect: u32) -> bool {
    matches!(effect, 267 | 584 | 606 | 607 | 645 | 723)
}

/// The sounds an effect starts after its first tick, each with its tick, as its arm of `CRagEffect::OnProcess`
/// does.
fn sounds(effect: u32) -> &'static [(i32, &'static str)] {
    match effect {
        584 => &[(5, "effect/풀버스터.wav")],
        637 => &[(10, "effect/데스페라도.wav")],
        643 => &[(20, "effect/래피드샤워.wav")],
        648 => &[(25, "effect/트리플액션.wav")],
        _ => &[],
    }
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    course.starts(sounds(effect));
    let first = course.tick == 0;
    // `EF_NPC_EARTHQUAKE` shakes the screen at its first, its 35th and its 70th tick.
    if effect == 666 && matches!(course.tick, 0 | 35 | 70) {
        course.quake = Some(0);
    }
    match effect {
        212 if first => prims.push(dark_breath()),
        267 if first => prims.push(palm(course)),
        606 if first => prims.push(chimto_once("이팩트/fvoice", course)),
        607 if first => prims.push(chimto_once("이팩트/wink", course)),
        622 if first => prims.push(thunder(course.lasts)),
        723 if first => prims.push(spears(course)),
        631 | 634 | 638..=642 if first => prims.push(on_cell(effect, course.lasts)),
        _ => prims.extend(played_once(effect, course).or_else(|| ground_sprite(effect).filter(|_| first).map(|kind| class_change(kind, course.lasts)))),
    }
}

/// A sprite as `TexturedClassChange` and its like launch one: it lasts as long as its effect, and every layer
/// of its frames is drawn in the colour its file gives it.
fn sprite(name: &'static str, lasts: i32, ticks_a_frame: i32) -> Prim {
    let mut sprite = Prim::new(Shape::Particle3d, name);
    (sprite.duration, sprite.anim_speed, sprite.own_colours) = (lasts, ticks_a_frame, true);
    sprite
}

/// `CRagEffect::TexturedClassChange`: at one tick of the effect (its 30th for Triple Action, else its first) a
/// sprite at the feet of the one it is on, that plays its frames once, each for so many ticks, and ends with
/// the last. Which sprite, how long a frame stays and which action plays is the function's own for each kind:
///
/// - Full Buster (kind 8) and Spread Attack (28) pick the action by the quarter the effect points in, from the
///   striker to the one struck, as the camera sees it;
/// - Kirikage (15) and Kasumikiri (17) draw one of two actions by lot;
/// - Issen (18) stands 6 above the feet;
/// - Sienna Execrate (60) is the monster Stapo, 8 above the feet, in its first action or its eighth by lot.
///
/// `EF_POISONHIT` in `CRagEffect::OnProcess` launches the same sprite, `PoisonHit.spr` at one and a half times
/// its size.
///
/// `CRagEffect::Vallentine` launches the same sprite (but for a bit of `renderFlag`, see the head of this
/// file): its kinds 0 and 1 are the first and the second action of `vallentine.spr`.
fn played_once(effect: u32, course: &Course) -> Option<Prim> {
    let (name, ticks_a_frame) = match effect {
        53 => ("이팩트/poisonhit", 2),
        363 | 364 => ("이팩트/vallentine", 2),
        509 => ("이팩트/05vallentine", 2),
        519 => ("이팩트/fast", 4),
        576 => ("이팩트/item_thunder", 2),
        577 => ("이팩트/item_cloud", 2),
        578 => ("이팩트/item_curse", 2),
        579 => ("이팩트/item_zzz", 2),
        580 => ("이팩트/item_rain", 2),
        682 => ("이팩트/폭죽_러브", 4),
        683 => ("이팩트/폭죽_화이트데이", 4),
        684 => ("이팩트/폭죽_발렌타인", 4),
        685 => ("이팩트/폭죽_생일", 4),
        686 => ("이팩트/폭죽_크리스마스", 4),
        707 => ("이팩트/mobile_ef04", 4),
        570 => ("이팩트/캐슬링", 2),
        571 => ("이팩트/블러드러스트", 2),
        584 => ("이팩트/m_ef02", 4),
        589 => ("이팩트/m_ef07", 4),
        630 => ("이팩트/그림자베기", 4),
        632 => ("이팩트/안개베기", 4),
        633 => ("이팩트/일섬", 2),
        637 => ("이팩트/데스페라도", 4),
        643 => ("이팩트/래피드샤워", 4),
        644 => ("이팩트/매지컬불릿", 2),
        645 => ("이팩트/스프레드", 4),
        647 => ("이팩트/트래킹", 2),
        648 => ("이팩트/트리플액션", 2),
        666 => ("이팩트/어스퀘이크", 4),
        729 => ("이팩트/mashofa", 4),
        797 => ("몬스터/stapo", 4),
        _ => return None,
    };
    if course.tick != if effect == 648 { 30 } else { 0 } {
        return None;
    }
    let mut sprite = sprite(name, course.lasts, ticks_a_frame);
    sprite.repeats = false;
    let points = quarter(course.yaw, course.shared.aim[0], course.shared.aim[2]);
    sprite.action = match effect {
        364 => 1,
        584 => [0, 3, 2, 1][points],
        645 => [0, 6, 4, 2][points],
        630 if lot(2) == 0.0 => 4,
        632 if lot(2) == 0.0 => 2,
        797 if lot(2) == 1.0 => 7,
        _ => 0,
    };
    if effect == 633 {
        sprite.put.1 = -6.0;
    }
    if effect == 797 {
        sprite.put.1 = -8.0;
    }
    if effect == 53 {
        sprite.size = 1.5;
    }
    Some(sprite)
}

/// `CRagEffect::TexturedClassChange`, its kinds 70 to 83 and 87 (`kind` counts them from 70): at its first tick
/// a sprite at the feet of the one it is on, for as long as the effect lives, each layer of its frames in the
/// colour and alpha its file gives it (`pattern & 0x200`), mixed in by that alpha. The twelve of the
/// elementals' skills (70 to 81) and Silent Breeze (82) play their frames over and over, Magma Flow (83) and
/// the wide web (87) once, ending with their last; the first twelve show a new frame every other tick, the
/// last three every fourth.
fn class_change(kind: usize, lasts: i32) -> Prim {
    let mut sprite = sprite(KINDS_FROM_70[kind], lasts, if kind < 12 { 2 } else { 4 });
    sprite.repeats = kind < 13;
    sprite
}

/// `CRagEffect::TexturedClassChange`, its kinds 16 (`EF_TATAMI`), 19 (`EF_KAEN`) and 21 to 25
/// (`EF_LIGHTNING_S` to `EF_FLARE_S`): at its first tick a sprite on the cell, each layer of its frames in the
/// colour its file gives it. The mat stands 6 over the ground and plays once, six ticks a frame, ending with
/// its last; the fire and the spheres stand 1 over it and play over and over for the 23 ticks the effect
/// lives (their unit launches the next as this one ends), the fire a frame every 3 to 7 ticks by lot, the
/// spheres every other tick. (These kinds take the bit `renderFlag & 0x200` off again, which the others
/// keep: see the head of this file.)
fn on_cell(effect: u32, lasts: i32) -> Prim {
    let (kind, ticks_a_frame, up) = match effect {
        631 => (0, 6, 6.0),
        634 => (1, lot(5) as i32 + 3, 1.0),
        _ => (effect as usize - 636, 2, 1.0),
    };
    let mut sprite = sprite(ON_CELLS[kind], lasts, ticks_a_frame);
    (sprite.repeats, sprite.put.1) = (effect != 631, -up);
    sprite
}

/// `EF_THUNDERSTORM2` in `CRagEffect::OnProcess`: the lightning of Thunder Storm two and a half times its size,
/// 4.5 above the feet of the one the effect is on, its frames two ticks apiece over and over for as long as the
/// effect lives.
fn thunder(lasts: i32) -> Prim {
    let mut thunder = sprite("이팩트/썬더스톰", lasts, 2);
    thunder.put.1 = -4.5;
    thunder.size = 2.5;
    thunder
}

/// `EF_DARKBREATH` in `CRagEffect::OnProcess`: the breath, `DarkBreath.spr` in red, 20 above the one struck, a
/// new frame every tick, for 65 ticks, fading over the last 5. (The client draws it over everything.)
fn dark_breath() -> Prim {
    let mut breath = Prim::new(Shape::Particle3d, BREATH);
    breath.duration = 65;
    breath.put.1 = -20.0;
    (breath.size, breath.anim_speed) = (0.8, 1);
    breath.colour = [255.0, 0.0, 0.0];
    breath.fade_out = 60;
    breath
}

/// `CRagEffect::CHIMTO`, what its flags share: the sprite stands at the place the effect was told (that of the
/// one struck), its action picked by the quarter the effect points in as the camera sees it.
///
/// The client launches a primitive of its kind 46 for it, which every tick is put at the place its first part
/// holds and otherwise fades, keeps its trail and plays its frames as a sprite of kind 5 does
/// (`CEffectPrim::OnProcess`), and is drawn as one (`RenderSegmentSprites2`): it is a sprite of kind 5 here,
/// one that does not move and stays in the world where it was launched.
fn chimto(mut sprite: Prim, course: &Course) -> Prim {
    let aim = course.shared.aim;
    sprite.action = [2, 1, 0, 3][quarter(course.yaw, aim[0], aim[2])];
    (sprite.put, sprite.deep, sprite.left) = ((aim[0], -aim[1]), aim[2], Some(course.walked));
    sprite
}

/// `CRagEffect::CHIMTO` with a flag of nothing: the palm of Investigate, a sprite two and a half times its
/// size, at an alpha of 200, for the 16 ticks the effect lives, two ticks a frame.
fn palm(course: &Course) -> Prim {
    let mut palm = Prim::new(Shape::Particle3d, INVESTIGATE);
    (palm.duration, palm.anim_speed, palm.alpha, palm.size) = (course.lasts, 2, 200.0, 2.5);
    chimto(palm, course)
}

/// `CRagEffect::CHIMTO` with its flag of 12: the spears of Hundred Spear, a sprite in the colours of its file
/// that plays once, six ticks a frame.
fn spears(course: &Course) -> Prim {
    chimto_once("이팩트/헌드레드스피어", course)
}

/// `CRagEffect::CHIMTO` with its flags of 1, 2 and 12: a sprite in the colours of its file that plays once, six
/// ticks a frame.
fn chimto_once(name: &'static str, course: &Course) -> Prim {
    let mut once = sprite(name, course.lasts, 6);
    once.repeats = false;
    chimto(once, course)
}
