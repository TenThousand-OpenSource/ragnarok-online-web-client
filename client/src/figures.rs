//! How a character is put together as rules, with the client's tables given (`web/data/figures.rs` has them
//! of the gateway): the order of what is worn on the head.

use std::collections::HashMap;

/// The priorities of what a character wears on its head (the views, as the packets list them: low, top,
/// middle) seen from a direction (`GetLayerPriority`): the lower is drawn first, -1 not at all. Nothing worn
/// in a place has -1. `usual` is the priority of each place when the table does not name the view worn there,
/// `layers` those of the views it names: for each of the three places, by direction.
pub fn head_layers(direction: usize, headgear: [u16; 3], usual: [i32; 3], layers: &HashMap<u16, [[i32; 8]; 3]>) -> [i32; 3] {
    std::array::from_fn(|slot| match headgear[slot] {
        0 => -1,
        view => layers.get(&view).map_or(usual[slot], |rows| rows[slot][direction % 8]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_on_the_head_has_the_tables_priority_or_the_usual_one() {
        let rows = [[1, 2, 3, 4, 5, 6, 7, 8], [11, 12, 13, 14, 15, 16, 17, 18], [21, 22, 23, 24, 25, 26, 27, 28]];
        let layers: HashMap<u16, [[i32; 8]; 3]> = [(5, rows)].into_iter().collect();
        let usual = [300, 200, 100];
        assert_eq!(head_layers(0, [0, 0, 0], usual, &layers), [-1, -1, -1]);
        assert_eq!(head_layers(3, [9, 0, 9], usual, &layers), [300, -1, 100]);
        // A view the table names: its row for the place it is worn in, by direction.
        assert_eq!(head_layers(0, [5, 5, 5], usual, &layers), [1, 11, 21]);
        assert_eq!(head_layers(7, [0, 5, 9], usual, &layers), [-1, 18, 100]);
        // A direction past the eighth goes round.
        assert_eq!(head_layers(10, [5, 0, 0], usual, &layers), [3, -1, -1]);
        assert_eq!(head_layers(2, [1, 2, 3], usual, &HashMap::new()), usual);
    }
}
