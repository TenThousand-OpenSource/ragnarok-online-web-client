//! The friends list as rules: how many it holds, and the colour each friend's name is written in.

/// How many friends the list holds (the client's title, `%s(%d/%d)` with 40; rAthena's `MAX_FRIENDS`).
pub const MOST: usize = 40;

/// The colour a friend's name is written in, by the friend's place in the list (the client's
/// `CSession::MemberColor`, whose numbers are read here as red, green and blue).
pub fn colour(place: usize) -> &'static str {
    const COLOURS: [&str; 16] = [
        "#0a3278", "#007878", "#000000", "#780000", "#007800", "#787800", "#780078", "#783200", "#787878", "#000078", "#003232", "#320000", "#003200", "#323200", "#320032", "#780000",
    ];
    COLOURS.get(place).copied().unwrap_or("#323232")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_go_by_the_place_in_the_list() {
        assert_eq!(colour(0), "#0a3278");
        assert_eq!(colour(3), colour(15));
        assert_eq!(colour(16), "#323232");
        assert_eq!(colour(39), "#323232");
    }
}
