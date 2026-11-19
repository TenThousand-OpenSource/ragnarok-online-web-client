//! What an account's ID, password and e-mail address may be: what rAthena's `login` table and its login packet
//! can hold. Each says what is wrong in a line for the player.

/// What rAthena's `login` table and login packet hold: 23 characters each, and 39 for the e-mail.
const ID_LENGTH: std::ops::RangeInclusive<usize> = 4..=23;
const PASSWORD_LENGTH: std::ops::RangeInclusive<usize> = 8..=23;
const EMAIL_MAX: usize = 39;

pub fn valid_id(id: &str) -> Result<(), &'static str> {
    if !ID_LENGTH.contains(&id.len()) || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        return Err("An ID is 4 to 23 letters, digits and underscores.");
    }
    Ok(())
}

pub fn valid_password(password: &str) -> Result<(), &'static str> {
    // Printable ASCII without spaces: what every keyboard and the game's login packet can carry unchanged.
    if !PASSWORD_LENGTH.contains(&password.len()) || !password.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err("A password is 8 to 23 letters, digits and symbols, without spaces.");
    }
    Ok(())
}

pub fn valid_email(email: &str) -> Result<(), &'static str> {
    let well_formed = email.split_once('@').is_some_and(|(name, domain)| {
        !name.is_empty()
            && !domain.contains('@')
            && domain.split('.').count() >= 2
            && domain.split('.').all(|label| !label.is_empty() && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'))
            && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b".!#$%&'*+/=?^_`{|}~-".contains(&byte))
    });
    if !well_formed || email.len() > EMAIL_MAX {
        return Err("Please enter a valid e-mail address of at most 39 characters.");
    }
    Ok(())
}
