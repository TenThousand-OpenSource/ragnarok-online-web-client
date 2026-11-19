//! The shapes the client's tables of text come in, each read here once: the tables this client has compiled
//! into it (`effect_tables/*.txt`, `statuses.txt`, `commands.txt`, `job_mounts.txt`, `net/packet_lengths.txt`)
//! and the client's own whose fields end with `#` (`fogparametertable.txt`, `viewpointtable.txt`).
//!
//! A line that is not of the table's shape is passed by, in every one of them.

/// A table of lines `number<TAB>value`: each line as what stands before its first tab and what stands after
/// it. Nothing is parsed: the caller knows what its numbers and values are.
pub fn tabbed(table: &str) -> impl Iterator<Item = (&str, &str)> {
    table.lines().filter_map(|line| line.split_once('\t'))
}

/// A table of lines `sort<TAB>field<TAB>field…` that keeps several sorts of lines together: the lines of one
/// sort, each as its fields after the first.
pub fn sorted<'a>(table: &'a str, sort: &'a str) -> impl Iterator<Item = Vec<&'a str>> {
    table.lines().filter_map(move |line| {
        let mut fields = line.split('\t');
        (fields.next() == Some(sort)).then(|| fields.collect())
    })
}

/// A table of lines `0xNNNN value`: each line as its number, written in sixteens, and what follows the first
/// blank after it, without the blanks around.
pub fn hexed(table: &str) -> impl Iterator<Item = (u16, &str)> {
    table.lines().filter_map(|line| {
        let (number, value) = line.strip_prefix("0x")?.split_once(' ')?;
        Some((u16::from_str_radix(number, 16).ok()?, value.trim()))
    })
}

/// The records of one of the client's tables whose fields end with `#` each: a name, in small letters, and so
/// many figures after it, as `CSession::LoadMapViewParameters` reads them. A field is what stands between the
/// `#` before it, or the start of its line, and its own `#`; a record may go on over several lines, and what
/// follows the last `#` of a line (a `//` and what it says) is no field.
pub fn hashed(table: &str, figures: usize) -> Vec<(String, Vec<f64>)> {
    let mut records = Vec::new();
    let mut fields: Vec<&str> = Vec::new();
    for line in table.lines() {
        let mut rest = line;
        while let Some((field, after)) = rest.split_once('#') {
            fields.push(field.trim());
            rest = after;
            if fields.len() == figures + 1 {
                records.push((fields[0].to_ascii_lowercase(), fields[1..].iter().map(|field| figure(field)).collect()));
                fields.clear();
            }
        }
    }
    records
}

/// A figure of such a table: a number as C's `atof` reads it (as far as it is one, nought if it is none), or
/// one written in sixteens (`0xff12121e`), which the client reads with a function of its own.
pub fn figure(field: &str) -> f64 {
    if let Some(hex) = field.strip_prefix("0x").or_else(|| field.strip_prefix("0X")) {
        return u32::from_str_radix(hex, 16).map_or(0.0, f64::from);
    }
    let end = field
        .char_indices()
        .find(|&(at, character)| !(character.is_ascii_digit() || character == '.' || (at == 0 && (character == '-' || character == '+'))))
        .map_or(field.len(), |(at, _)| at);
    field[..end].parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tabbed_line_is_cut_at_its_first_tab() {
        let table = "12\tfire.str\nno tab here\n7\ta\tb\n";
        assert_eq!(tabbed(table).collect::<Vec<_>>(), [("12", "fire.str"), ("7", "a\tb")]);
        // As the tables of effects are asked: the value of a number.
        assert_eq!(tabbed(table).find(|(number, _)| number.parse() == Ok(7)).map(|(_, value)| value), Some("a\tb"));
    }

    #[test]
    fn the_lines_of_one_sort_are_told_from_the_others() {
        let table = "icon\t1\t30\tfile.tga\ncommand\twho\tTT_WHO\nicon\t2\n\nicons\t9";
        assert_eq!(sorted(table, "icon").collect::<Vec<_>>(), [vec!["1", "30", "file.tga"], vec!["2"]]);
        assert_eq!(sorted(table, "command").collect::<Vec<_>>(), [vec!["who", "TT_WHO"]]);
        assert_eq!(sorted(table, "none").count(), 0);
    }

    #[test]
    fn a_hexed_line_is_a_number_and_what_follows_it() {
        let table = "0x0064 55\n0x0069 -1 \n# a note\n0xzz 1\n0x0073\n0x09FD ZC_SET_UNIT_IDLE";
        assert_eq!(hexed(table).collect::<Vec<_>>(), [(0x0064, "55"), (0x0069, "-1"), (0x09fd, "ZC_SET_UNIT_IDLE")]);
    }

    #[test]
    fn hashed_records_are_a_name_and_their_figures() {
        let table = "// the fog of maps\nProntera.rsw#\n0.2#0.9#0xff12121e#1.5f#\nmoc_fild01.rsw#0#1#255#x#   // a note\nleft.rsw#1#";
        assert_eq!(
            hashed(table, 4),
            [("prontera.rsw".to_owned(), vec![0.2, 0.9, f64::from(0xff12121eu32), 1.5]), ("moc_fild01.rsw".to_owned(), vec![0.0, 1.0, 255.0, 0.0])]
        );
    }

    #[test]
    fn a_figure_is_read_as_far_as_it_is_a_number() {
        assert_eq!((figure("-12.5"), figure("+3"), figure("40abc"), figure("abc"), figure("")), (-12.5, 3.0, 40.0, 0.0, 0.0));
        assert_eq!((figure("0x10"), figure("0XfF"), figure("0xzz"), figure("1-2")), (16.0, 255.0, 0.0, 1.0));
    }
}
