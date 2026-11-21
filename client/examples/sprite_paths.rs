//! Answers `tools/sprite_files.py`: which files the client's rules name for a job. A request a line on the
//! standard input, its fields parted by tabs, an answer a line on the standard output:
//!
//! - `body`, `weapon`, `shield`, `imf` `<job> <sex>`: `crate::dress` (a weapon's and a shield's are what its
//!   files begin with);
//! - `palette <job> <sex> <colour told> <korea: 1 or 0>`: the palette of the colour shown for the one the
//!   server tells (`dress::clothes_colour`, `dress::palette`), `-` for the sprite's own colours;
//! - `shown <job> <kind of weapon>`: the kind whose sprite the job shows for it (`pose::weapon_shown`);
//! - `creature <job> <name in the client's table, or nothing>`: `sprite <path>`, `character`, `model <file>`
//!   or `-` (`creatures::shape`).

use roweb_client::creatures::{self, Shape};
use roweb_client::{dress, pose};

fn answer(line: &str) -> Option<String> {
    let mut fields = line.split('\t');
    let (what, job) = (fields.next()?, fields.next()?.parse::<u16>().ok()?);
    let mut number = || fields.next().and_then(|field| field.parse::<u16>().ok());
    Some(match what {
        "body" => dress::body(job, number()? as u8),
        "weapon" => dress::weapon(job, number()? as u8),
        "shield" => dress::shield(job, number()? as u8),
        "imf" => dress::imf(job, number()? as u8),
        "palette" => {
            let (sex, told, korea) = (number()? as u8, number()?, number()? == 1);
            dress::palette(job, sex, dress::clothes_colour(job, told), korea).unwrap_or_else(|| "-".to_owned())
        }
        "shown" => pose::weapon_shown(job, number()?).to_string(),
        "creature" => match creatures::shape(job, line.split('\t').nth(2).filter(|name| !name.is_empty())) {
            Some(Shape::Sprite(path)) => format!("sprite\tdata/sprite/{path}"),
            Some(Shape::Model(file)) => format!("model\t{file}"),
            Some(Shape::Character) => "character".to_owned(),
            Some(Shape::Hidden) => "hidden".to_owned(),
            None => "-".to_owned(),
        },
        _ => return None,
    })
}

fn main() {
    for line in std::io::stdin().lines().map_while(Result::ok) {
        println!("{}", answer(&line).unwrap_or_else(|| "?".to_owned()));
    }
}
