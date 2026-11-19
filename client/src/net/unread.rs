//! The packets of the map server that this client does not read yet, counted: a tool of this project's own for
//! whoever builds it (the original client has nothing of the kind), shown by the chat command `/packets`. Here is
//! what each is called; the count of them is `crate::unread`.
//!
//! `packet_names.txt` is what rAthena's headers call each packet, written out by `tools/packet-names.py`.

use std::collections::HashMap;
use std::sync::OnceLock;

static NAMES: OnceLock<HashMap<u16, &'static str>> = OnceLock::new();

/// What rAthena calls the packet of that number, where its headers give it a name.
pub fn name(id: u16) -> Option<&'static str> {
    NAMES.get_or_init(|| crate::tables::hexed(include_str!("packet_names.txt")).collect()).get(&id).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packets_are_named_as_rathena_names_them() {
        assert_eq!(name(0x0984), Some("ZC_EFST_SET_ENTER"));
        assert_eq!(name(0x0081), Some("SC_NOTIFY_BAN"));
        assert_eq!(name(0xffff), None);
    }
}
