//! The party as rules: what the client decides of its members that no packet says.

/// The map a member is on without what the server writes after its name (`prontera.gat`): two members are on
/// one map when these are the same (the client's `UpdatePartyMemberInfo` cuts the same off both).
pub fn map_name(map: &str) -> &str {
    map.split('.').next().unwrap_or(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_members_are_on_one_map_by_its_name_alone() {
        assert_eq!(map_name("prontera.gat"), "prontera");
        assert_eq!(map_name("prontera"), "prontera");
    }
}
