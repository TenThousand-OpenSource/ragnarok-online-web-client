//! The probabilities as rules: what the menu's icon of them does (`UIMenuIconWnd::SendMsg`,
//! `MENUICON_PROBABILITY`). The client has no window for it: it asks whether to visit the publisher's page of
//! the probabilities (its line `MSI_ETC_PROBABILITY_WEB_CONNECT`) and on a yes hands the address of its line
//! `MSI_ETC_PROBABILITY_URL` to Windows to open. The lines are given by the gateway.

/// The address the question's yes opens, if the line is one of the web: anything else is opened by nobody
/// here (the client hands whatever the line says to `ShellExecuteA`).
pub fn address(line: &str) -> Option<&str> {
    let line = line.trim();
    (line.starts_with("https://") || line.starts_with("http://")).then_some(line).filter(|line| !line.contains(['"', '\'', '<', '>', ' ']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_address_of_the_web_is_opened() {
        assert_eq!(address("https://probability.gnjoy.com/"), Some("https://probability.gnjoy.com/"));
        assert_eq!(address(" http://example.org/a?b=1 "), Some("http://example.org/a?b=1"));
        assert_eq!(address(""), None);
        assert_eq!(address("javascript:alert(1)"), None);
        assert_eq!(address("file:///etc/passwd"), None);
        assert_eq!(address("https://a.b/\" onclick=\""), None);
    }
}
