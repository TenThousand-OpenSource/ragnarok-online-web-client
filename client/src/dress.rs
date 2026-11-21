//! The names of the files a character's parts are in, as the client names them: the body's, the weapon's and
//! the shield's sprites, the `.imf` that orders body and head, the palette of the clothes.
//!
//! The client keeps a table of names for each part, all compiled into it and all numbered by "sprite job"
//! (`CSession::GetSpriteJob`): `m_jobSpriteNames` for the weapon (`CSession::InitJobSpriteNames`), and
//! `m_shieldJobNames`, `m_imfNames` and `m_palJobNames` (`CSession::InitResourceNameTables`). They differ from
//! the bodies' table: a Lord Knight has a body of its own and holds a Knight's sword and shield, and its head
//! goes over or under its body as a Knight's. They are written out by `tools/client-tables/job_files.py` into
//! `web/data/job_files.txt`.
//!
//! The job asked with is the one the character is drawn as (`crate::mounts::drawn_job`: a knight on its bird
//! is job 13). The client's `GetSpriteJob` takes that from the job the character's costume is of (the
//! packets' `body`), which a classic server never has other than the job itself; costumes are not built.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The client's tables, each by sprite job.
#[derive(Default)]
struct Tables {
    weapon: HashMap<u16, &'static str>,
    shield: HashMap<u16, &'static str>,
    imf: HashMap<u16, &'static str>,
    palette: HashMap<u16, &'static str>,
    /// The palettes a client of another service than Korea's names differently.
    palette_abroad: HashMap<u16, &'static str>,
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut tables = Tables::default();
        for line in include_str!("web/data/job_files.txt").lines().filter(|line| !line.starts_with('#')) {
            let mut fields = line.split('\t');
            let (Some(table), Some(job), Some(name)) = (fields.next(), fields.next().and_then(|job| job.parse().ok()), fields.next()) else { continue };
            let table = match table {
                "weapon" => &mut tables.weapon,
                "shield" => &mut tables.shield,
                "imf" => &mut tables.imf,
                "palette" => &mut tables.palette,
                _ => &mut tables.palette_abroad,
            };
            table.insert(job, name);
        }
        tables
    })
}

/// The number a job has in the client's tables of names: the later jobs moved down to follow the first ones
/// (`GetSpriteJob` with its `spriteIndex`).
fn sprite_job(job: u16) -> u16 {
    if job > 3950 { job - 3950 } else { job }
}

fn sex_word(sex: u8) -> &'static str {
    if sex == 1 { "남" } else { "여" }
}

fn race(job: u16) -> &'static str {
    if crate::jobs::doram(job) { "도람족" } else { "인간족" }
}

/// The sprite of a job's body (`GetBodyResourceName`: the race, `몸통`, the sex, then the name of the bodies'
/// table, `crate::jobs::body_sprite`, with the sex).
pub fn body(job: u16, sex: u8) -> String {
    format!("data/sprite/{}/몸통/{sex}/{}_{sex}", race(job), crate::jobs::body_sprite(job), sex = sex_word(sex))
}

/// What the files of a job's weapons begin with (`GetWeaponResourceName`: the race, then the table's
/// `folder\name`, then the sex): the weapon's own ending follows, `_검` or `_1101`.
pub fn weapon(job: u16, sex: u8) -> String {
    let name = tables().weapon.get(&sprite_job(job)).copied().unwrap_or("초보자\\초보자");
    format!("data/sprite/{}/{}_{}", race(job), name.replace('\\', "/"), sex_word(sex))
}

/// What the files of a job's shields begin with (`GetShieldResourceName`): `_<item>_방패` follows for a shield
/// with a sprite of its own, or the ending of its view (`_가드`).
pub fn shield(job: u16, sex: u8) -> String {
    let name = tables().shield.get(&sprite_job(job)).copied().unwrap_or("초보자\\초보자");
    format!("data/sprite/방패/{}_{}", name.replace('\\', "/"), sex_word(sex))
}

/// The file that says which of a job's body and head lies over the other (`GetBodyImfName`).
pub fn imf(job: u16, sex: u8) -> String {
    format!("data/imf/{}_{}.imf", tables().imf.get(&sprite_job(job)).copied().unwrap_or("초보자"), sex_word(sex))
}

/// The colour a character's clothes are shown in, of the colour the server tells (`CSession::GetBodyStyle`,
/// through which `CPc::SetBodyPalette` and `SetSprAct2` put what the packets say): from 2 on, that one. Under
/// it the client shows the first reborn jobs (High Novice to High Thief, 4001 to 4007) in their palette 1 —
/// that is all that tells them from the plain first jobs, whose sprites they wear — and everyone else in the
/// sprite's own colours, colour 1 too. `job` is the character's job, not what it rides.
pub fn clothes_colour(job: u16, told: u16) -> u16 {
    match told {
        0 | 1 => u16::from((4001..=4007).contains(&job)),
        told => told,
    }
}

/// The palette of a job's clothes in a colour (`GetBodyPalName`), none for the sprite's own colours (0:
/// `CPc::SetBodyPaletteName` leaves then). `colour` is the one shown (`clothes_colour`); `korea` whether the
/// client is set up for Korea, which names the second jobs' palettes after the jobs, where the other services
/// borrow the palettes of the first second jobs (a Crusader a Knight's).
pub fn palette(job: u16, sex: u8, colour: u16, korea: bool) -> Option<String> {
    if colour == 0 {
        return None;
    }
    let (tables, sprite_job) = (tables(), sprite_job(job));
    let abroad = if korea { None } else { tables.palette_abroad.get(&sprite_job) };
    let name = abroad.or_else(|| tables.palette.get(&sprite_job)).copied().unwrap_or("초보자");
    let folder = if crate::jobs::doram(job) { "도람족/body" } else { "몸" };
    Some(format!("data/palette/{folder}/{name}_{}_{colour}.pal", sex_word(sex)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reborn_job_holds_the_weapon_and_shield_of_the_job_it_was() {
        // A Lord Knight, a High Priest, an Assassin Cross, a Paladin: the folders of the Knight, the Priest…
        assert_eq!(weapon(4008, 1), "data/sprite/인간족/기사/기사_남");
        assert_eq!(weapon(4009, 0), "data/sprite/인간족/프리스트/프리스트_여");
        assert_eq!(weapon(4013, 1), "data/sprite/인간족/어세신/어세신_남");
        assert_eq!(shield(4008, 1), "data/sprite/방패/기사/기사_남");
        assert_eq!(shield(4015, 0), "data/sprite/방패/크루세이더/크루세이더_여");
        // The plain jobs have their own, and a job the tables do not have is a Novice.
        assert_eq!(weapon(7, 1), "data/sprite/인간족/기사/기사_남");
        assert_eq!(weapon(0, 0), "data/sprite/인간족/초보자/초보자_여");
        assert_eq!(shield(60000, 1), "data/sprite/방패/초보자/초보자_남");
        // A Summoner's are among the Doram's.
        assert!(weapon(crate::jobs::SUMMONER, 1).starts_with("data/sprite/도람족/"));
    }

    #[test]
    fn a_body_has_the_name_of_the_bodies_table() {
        assert_eq!(body(4008, 1), "data/sprite/인간족/몸통/남/로드나이트_남");
        assert_eq!(body(0, 0), "data/sprite/인간족/몸통/여/초보자_여");
        assert!(body(crate::jobs::SUMMONER, 0).starts_with("data/sprite/도람족/몸통/여/"));
    }

    #[test]
    fn body_and_head_are_ordered_as_the_tables_job() {
        assert_eq!(imf(4008, 1), "data/imf/기사_남.imf");
        assert_eq!(imf(4009, 0), "data/imf/프리스트_여.imf");
        assert_eq!(imf(4013, 1), "data/imf/어세신크로스_남.imf");
        // A Lord Knight on its bird as a Knight on its; a Super Novice as a Novice.
        assert_eq!(imf(4014, 1), "data/imf/페코페코_기사_남.imf");
        assert_eq!(imf(23, 0), "data/imf/초보자_여.imf");
    }

    #[test]
    fn the_clothes_palette_has_the_name_of_the_clients_service() {
        // Korea's client has a palette for each second job; the others borrow.
        assert_eq!(palette(14, 1, 2, true).as_deref(), Some("data/palette/몸/크루_남_2.pal"));
        assert_eq!(palette(14, 1, 2, false).as_deref(), Some("data/palette/몸/기사_남_2.pal"));
        assert_eq!(palette(4009, 0, 3, true).as_deref(), Some("data/palette/몸/하이프리스트_여_3.pal"));
        assert_eq!(palette(4009, 0, 3, false).as_deref(), Some("data/palette/몸/프리스트_여_3.pal"));
        assert_eq!(palette(15, 1, 4, false).as_deref(), Some("data/palette/몸/프리스트_남_4.pal"));
        // Where the two agree, and the sprite's own colours.
        assert_eq!(palette(7, 1, 2, true), palette(7, 1, 2, false));
        assert_eq!(palette(7, 1, 0, true), None);
    }

    #[test]
    fn the_first_reborn_jobs_wear_their_first_palette() {
        assert_eq!((clothes_colour(4001, 0), clothes_colour(4007, 1), clothes_colour(4007, 3)), (1, 1, 3));
        // A Swordman, a Lord Knight, a Baby Novice: colour 1 is the sprite's own.
        assert_eq!((clothes_colour(1, 1), clothes_colour(4008, 0), clothes_colour(4008, 1), clothes_colour(4023, 1)), (0, 0, 0, 0));
        assert_eq!((clothes_colour(1, 2), clothes_colour(4008, 7)), (2, 7));
    }
}
