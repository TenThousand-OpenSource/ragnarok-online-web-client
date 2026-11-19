//! A picture of the screen as the client names and stamps it (`CMode::TakeScreenshot`,
//! `Source/Client/Mode/ModeScreenshot.cpp`).

/// The name of the file: what the picture is of, if it was said (a deal's "[Trade_<partner>]"), then
/// "screen", the character's name and a number of three figures. The signs a file's name may not have become
/// `_`. The client's ending is `.jpg`; this client makes a PNG.
pub fn file_name(caption: &str, character: &str, number: u32) -> String {
    let name: String = format!("{caption}screen{character}{number:03}").chars().map(|sign| if "\\/:*?\"<>|".contains(sign) { '_' } else { sign }).collect();
    format!("{name}.png")
}

/// What the client writes into the picture's bottom left corner: the date with the day of the week in
/// English, and the time ("%d/%02d/%02d/%s  %02d: %02d: %02d"). `weekday` counts from Sunday, 0.
pub fn stamp((year, month, day): (u32, u32, u32), weekday: u32, (hour, minute, second): (u32, u32, u32)) -> String {
    const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    format!("{year}/{month:02}/{day:02}/{}  {hour:02}: {minute:02}: {second:02}", DAYS[weekday as usize % 7])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_is_named_and_stamped_as_the_client_does() {
        assert_eq!(file_name("", "Poring", 7), "screenPoring007.png");
        assert_eq!(file_name("[Trade_A/B]", "Vi:k", 12), "[Trade_A_B]screenVi_k012.png");
        assert_eq!(stamp((2026, 2, 9), 1, (7, 5, 30)), "2026/02/09/Monday  07: 05: 30");
    }
}
