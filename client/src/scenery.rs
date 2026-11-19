//! What surrounds the player on a map besides its ground and its models, as rules that need no page: the
//! effects and sounds a map places and when each is launched (`makers.rs`), the fog (`fog.rs`), the view of
//! the camera a map sets and how the camera eases to what is asked of it (`views.rs`), and the darkening of a
//! night (`dark.rs`). The world's side of all of it is `web/world/scenery.rs`.

pub mod dark;
pub mod fog;
pub mod makers;
pub mod views;

/// The records of one of the client's tables whose fields end with `#` each (`fogparametertable.txt`,
/// `viewpointtable.txt`): a name and so many figures after it, as `CSession::LoadMapViewParameters` reads
/// them. A field is what stands between the `#` before it, or the start of its line, and its own `#`.
pub fn records(table: &str, figures: usize) -> Vec<(String, Vec<f64>)> {
    let mut records = Vec::new();
    let mut fields: Vec<&str> = Vec::new();
    for line in table.lines() {
        let mut rest = line;
        while let Some((field, after)) = rest.split_once('#') {
            fields.push(field.trim());
            rest = after;
            if fields.len() == figures + 1 {
                records.push((fields[0].to_ascii_lowercase(), fields[1..].iter().map(|field| number(field)).collect()));
                fields.clear();
            }
        }
    }
    records
}

/// A figure of such a table: a number as C's `atof` reads it, or one written in sixteens (`0xff12121e`), which
/// the client reads with a function of its own.
fn number(field: &str) -> f64 {
    if let Some(hex) = field.strip_prefix("0x").or_else(|| field.strip_prefix("0X")) {
        return u32::from_str_radix(hex, 16).map_or(0.0, f64::from);
    }
    let end = field.char_indices().find(|&(at, character)| !(character.is_ascii_digit() || character == '.' || (at == 0 && (character == '-' || character == '+')))).map_or(field.len(), |(at, _)| at);
    field[..end].parse().unwrap_or(0.0)
}

/// The name such a table knows a map by (`CSession::GetFogParameters`, `GetMapViewParameters`): the map's file,
/// and of a copy made for one party (`0011@gef`) the name as the client cuts it: without the three characters
/// that stand four before the `@` (`1@gef`).
pub fn table_name(map: &str) -> String {
    let mut name = format!("{}.rsw", map.to_ascii_lowercase().trim_end_matches(".rsw").trim_end_matches(".gat"));
    if let Some(at) = name.rfind('@').filter(|&at| at >= 4) {
        name.replace_range(at - 4..at - 1, "");
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_table_is_read_field_by_field() {
        let table = "// a note\n2@exds.rsw#\n0.23#\n0.5#\n0xff12121e#\n0.3#\n\nYuno.rsw#\t250#\t250#";
        assert_eq!(records(table, 4), [("2@exds.rsw".to_owned(), vec![0.23, 0.5, 4_279_374_366.0, 0.3])]);
        assert_eq!(records("yuno.rsw#\t230#\t170#\t500#\t-360#\t360#\t45#\t-25# -60#\t-35#\t\n", 9)[0].1, [230.0, 170.0, 500.0, -360.0, 360.0, 45.0, -25.0, -60.0, -35.0]);
    }

    #[test]
    fn a_map_is_looked_up_by_its_file() {
        assert_eq!(table_name("prontera"), "prontera.rsw");
        assert_eq!(table_name("Yuno.gat"), "yuno.rsw");
        assert_eq!(table_name("1@gef"), "1@gef.rsw");
        assert_eq!(table_name("0011@gef"), "1@gef.rsw");
    }
}
