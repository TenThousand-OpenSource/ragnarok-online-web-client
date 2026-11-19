//! The server's map cache, made from the client's own maps.
//!
//! rAthena does not read the client's files while it runs: where one can walk comes from `map_cache.dat`, made
//! from some client at some time. When that differs from the files the players' client draws, characters walk
//! through walls that are drawn and stop at walls that are not. Written to rAthena's `db/import/map_cache.dat`,
//! this file takes the place of the server's own for every map the client has.
//!
//! The format and the rules for a cell follow rAthena's `src/tool/mapcache.cpp`.

use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::grf::Grf;

/// The longest map name the server keeps, with its terminator (`MAP_NAME_LENGTH`).
const NAME_LENGTH: usize = 12;

/// One map's cells as the server keeps them: the GAT's cell types, with walkable cells under the water's surface
/// marked as water (type 3).
pub fn cells(gat: &[u8], water_level: Option<f32>) -> Result<(i16, i16, Vec<u8>), String> {
    if gat.len() < 14 || &gat[..4] != b"GRAT" {
        return Err("not a GAT file".to_owned());
    }
    let size = |at: usize| i32::from_le_bytes(gat[at..at + 4].try_into().unwrap());
    let (width, height) = (size(6), size(10));
    let count = usize::try_from(width).ok().zip(usize::try_from(height).ok()).map(|(width, height)| width.saturating_mul(height)).ok_or(
        "a GAT of negative size"
    )?;
    if width > i32::from(i16::MAX) || height > i32::from(i16::MAX) || gat.len() < 14 + count.saturating_mul(20) {
        return Err("a GAT larger than its file".to_owned());
    }
    let cells = (0..count)
        .map(|cell| {
            let at = 14 + cell * 20;
            // The height of the cell's first corner, pointing down as the files keep it.
            let height = f32::from_le_bytes(gat[at..at + 4].try_into().unwrap());
            match u32::from_le_bytes(gat[at + 16..at + 20].try_into().unwrap()) {
                0 if water_level.is_some_and(|level| height > level) => 3,
                // Only the low byte, as the server's own tool keeps it: some maps carry flags above it.
                kind => kind as u8,
            }
        })
        .collect();
    Ok((width as i16, height as i16, cells))
}

/// The height of a map's water as its RSW gives it (pointing down), or `None` for a map that keeps none there.
pub fn water_level(rsw: &[u8]) -> Option<f32> {
    if rsw.len() < 6 || &rsw[..4] != b"GRSW" {
        return None;
    }
    let version = u16::from(rsw[4]) << 8 | u16::from(rsw[5]);
    // The water follows the header and the names of the map's other files; newer versions have more before it,
    // and the newest keep the water with the ground.
    let at = match version {
        0x0103 => 6 + 120,
        0x0104..=0x0201 => 6 + 160,
        0x0202..=0x0204 => 6 + 1 + 160,
        0x0205 => 6 + 4 + 1 + 160,
        _ => return None,
    };
    rsw.get(at..at + 4).map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
}

/// Writes the cache of every map in the client's archives. Returns how many maps it holds.
pub fn write(grf: &Grf, mut out: impl Write) -> std::io::Result<usize> {
    let mut names: Vec<String> = grf
        .paths()
        .filter_map(|path| path.strip_prefix("data\\")?.strip_suffix(".gat").map(str::to_owned))
        .filter(|name| !name.contains('\\') && name.is_ascii() && name.len() < NAME_LENGTH)
        .collect();
    // Maps that are another map's files under a name of their own (instances of dungeons, mostly).
    names.extend(
        grf.alias_names()
            .filter_map(|name| name.strip_suffix(".gat"))
            .filter(|name| !name.contains('\\') && name.is_ascii() && name.len() < NAME_LENGTH)
            .map(str::to_owned)
    );
    names.sort();
    names.dedup();

    let mut body = Vec::new();
    let mut count = 0usize;
    for name in &names {
        let Ok(gat) = grf.read(&format!("data\\{name}.gat")) else { continue };
        let water = grf.read(&format!("data\\{name}.rsw")).ok().and_then(|rsw| water_level(&rsw));
        let (width, height, cells) = match cells(&gat, water) {
            Ok(map) => map,
            Err(error) => {
                eprintln!("mapcache: {name}: {error}");
                continue;
            }
        };
        let mut packed = ZlibEncoder::new(Vec::new(), Compression::best());
        packed.write_all(&cells)?;
        let packed = packed.finish()?;

        let mut field = [0u8; NAME_LENGTH];
        field[..name.len()].copy_from_slice(name.as_bytes());
        body.extend(field);
        body.extend(width.to_le_bytes());
        body.extend(height.to_le_bytes());
        body.extend((packed.len() as i32).to_le_bytes());
        body.extend(packed);
        count += 1;
    }
    // The header: the file's size, the number of maps, and two bytes the server's compiler pads it with.
    out.write_all(&((body.len() + 8) as u32).to_le_bytes())?;
    out.write_all(&(count as u16).to_le_bytes())?;
    out.write_all(&[0, 0])?;
    out.write_all(&body)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gat(cells: &[(f32, u32)], width: i32) -> Vec<u8> {
        let mut file = b"GRAT\x01\x02".to_vec();
        file.extend(width.to_le_bytes());
        file.extend((cells.len() as i32 / width).to_le_bytes());
        for (height, kind) in cells {
            for _ in 0..4 {
                file.extend(height.to_le_bytes());
            }
            file.extend(kind.to_le_bytes());
        }
        file
    }

    #[test]
    fn keeps_cell_types_and_marks_water() {
        // Heights point down: 5 is below a surface at 2, -5 above it.
        let file = gat(&[(-5.0, 0), (5.0, 0), (5.0, 1), (0.0, 5)], 2);
        assert_eq!(cells(&file, Some(2.0)).unwrap(), (2, 2, vec![0, 3, 1, 5]));
        // A map without water has no water cells.
        assert_eq!(cells(&file, None).unwrap().2, [0, 0, 1, 5]);
        assert!(cells(b"GRSW", None).is_err());
        assert!(cells(&file[..40], None).is_err());
    }

    #[test]
    fn finds_the_water_level_by_version() {
        let mut rsw = b"GRSW\x02\x01".to_vec();
        rsw.extend([0u8; 160]);
        rsw.extend(12.5f32.to_le_bytes());
        assert_eq!(water_level(&rsw), Some(12.5));
        // One byte more before it from 2.2 on.
        let mut newer = b"GRSW\x02\x02\x00".to_vec();
        newer.extend([0u8; 160]);
        newer.extend(7.0f32.to_le_bytes());
        assert_eq!(water_level(&newer), Some(7.0));
        // 2.6 keeps the water with the ground.
        assert_eq!(water_level(b"GRSW\x02\x06"), None);
        assert_eq!(water_level(b"GRAT\x01\x02"), None);
    }
}
