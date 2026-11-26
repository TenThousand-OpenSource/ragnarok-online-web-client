//! The bank as rules (the client's `UIBank_NewWnd`): what the sum typed into its window comes to as a button
//! adds to it, what a deposit or a withdrawal of it is, and the two lines the client says when it will not
//! send one: a line of the chat and a short one in the window itself. The lines are numbers of the client's
//! message table (`msgstringtable.txt`).

/// The most zeny a character carries and the bank holds as the client counts it (`0x7FFFFFFF`).
pub const MOST: i64 = 0x7FFF_FFFF;

/// The line of the message table for the word the button "Max" writes into the field, and by which the window
/// knows that all of it is meant (`MSI_BANK_1_MINI`).
pub const MAX_WORD: usize = 2778;

/// What the client says when it does not send a deposit or a withdrawal: the line it puts into the chat, in
/// red, and the short one it shows in the window, at the foot (`UIBank_NewWnd::m_status`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Told {
    pub chat: usize,
    pub status: usize,
}

/// Which way the zeny goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Deposit,
    Withdraw,
}

/// Whether a text is a number as the window takes one: figures and nothing else, at least one
/// (`UIBank_NewWnd::IsNumeric`).
pub fn numeric(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|letter| letter.is_ascii_digit())
}

/// What stands in the field after a button added `step` to it (`UIBank_NewWnd::AdjustAmount`: "+ 1z", "- 1z",
/// "+ 100,000", "+ 1,000,000", "+ 10,000,000"): what was typed counts when it is a number, anything else is
/// nothing; the sum stays between nothing and the most there can be.
pub fn stepped(typed: &str, step: i64) -> i64 {
    let had = if numeric(typed) { typed.parse::<i64>().unwrap_or(MOST) } else { 0 };
    had.saturating_add(step).clamp(0, MOST)
}

/// The sum the field asks for, with `max` the word of the button "Max" (`UIBank_NewWnd::TransactionAmount`):
/// that word is everything that can go that way; otherwise what was typed must be a number from 1 to the
/// most there can be.
fn asked(typed: &str, max: &str, side: Side, zeny: i64, balance: i64) -> Result<i64, Told> {
    if typed == max {
        let over = (zeny + balance - MOST).max(0);
        return match side {
            Side::Deposit if balance >= MOST => Err(Told { chat: 2787, status: 2783 }),
            Side::Deposit if zeny == 0 => Err(Told { chat: 2770, status: 2780 }),
            Side::Deposit if zeny - over < 0 => Err(Told { chat: 2465, status: 2780 }),
            Side::Deposit => Ok(zeny - over),
            Side::Withdraw if balance == 0 => Err(Told { chat: 2770, status: 2780 }),
            Side::Withdraw if zeny + balance - over > MOST => Ok(MOST - zeny),
            Side::Withdraw => Ok(balance - over),
        };
    }
    if typed.is_empty() {
        return Err(Told { chat: 2779, status: 2781 });
    }
    if !numeric(typed) {
        return Err(Told { chat: 2488, status: 2782 });
    }
    match typed.parse::<i64>() {
        Ok(sum) if sum > MOST => Err(Told { chat: 2768, status: 2783 }),
        Ok(sum) if sum < 1 => Err(Told { chat: 2769, status: 2784 }),
        Ok(sum) => Ok(sum),
        // More figures than a number of 64 bits holds: more than there can be.
        Err(_) => Err(Told { chat: 2768, status: 2783 }),
    }
}

/// The sum a press of Deposit or Withdraw sends, or what the client says instead (`UIBank_NewWnd::SendMsg`,
/// `BANK_BUTTON_DEPOSIT` and `BANK_BUTTON_WITHDRAW`, after `TransactionAmount`): `None` is the sum of the word
/// "Max" coming to nothing, for which nothing is sent and nothing said.
pub fn sum(typed: &str, max: &str, side: Side, zeny: i64, balance: i64) -> Result<Option<i64>, Told> {
    let sum = asked(typed, max, side, zeny, balance)?;
    if sum == 0 {
        return Ok(None);
    }
    match side {
        Side::Deposit if zeny < sum => Err(Told { chat: 2456, status: 2785 }),
        Side::Deposit if balance + sum > MOST => Err(Told { chat: 2787, status: 2783 }),
        Side::Withdraw if balance < sum => Err(Told { chat: 2455, status: 2786 }),
        Side::Withdraw if zeny + sum > MOST => Err(Told { chat: 2459, status: 2787 }),
        _ => Ok(Some(sum)),
    }
}

/// The line the client puts into the chat when the server refuses a deposit, by the reason it gives
/// (`CGameMode::ReceiveBankDepositAck`); none for the reason 0, which is the deposit made.
pub fn deposit_refused(reason: i16) -> Option<usize> {
    match reason {
        0 => None,
        2 => Some(2456),
        3 => Some(2465),
        4 => Some(2489),
        _ => Some(2454),
    }
}

/// The same for a withdrawal (`CGameMode::ReceiveBankWithdrawAck`).
pub fn withdrawal_refused(reason: i16) -> Option<usize> {
    match reason {
        0 => None,
        1 => Some(2455),
        3 => Some(2489),
        _ => Some(2454),
    }
}

/// The same for the balance asked for as the window is to open (`CGameMode::ReceiveBankCheck`).
pub fn check_refused(reason: i16) -> Option<usize> {
    match reason {
        0 => None,
        2 => Some(3031),
        _ => Some(2454),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_button_adds_to_the_number_typed_and_to_nothing_else() {
        assert_eq!(stepped("", 100_000), 100_000);
        assert_eq!(stepped("250", 1), 251);
        assert_eq!(stepped("Max", 1_000_000), 1_000_000);
        assert_eq!(stepped("12a", 1), 1);
        assert_eq!(stepped("0", -1), 0);
        assert_eq!(stepped("5", -1), 4);
        assert_eq!(stepped("2147483647", 10_000_000), MOST);
        assert_eq!(stepped("99999999999999999999", 1), MOST);
    }

    #[test]
    fn a_sum_typed_is_sent_when_there_is_that_much() {
        assert_eq!(sum("500", "Max", Side::Deposit, 1000, 0), Ok(Some(500)));
        assert_eq!(sum("500", "Max", Side::Withdraw, 0, 500), Ok(Some(500)));
        assert_eq!(sum("1001", "Max", Side::Deposit, 1000, 0), Err(Told { chat: 2456, status: 2785 }));
        assert_eq!(sum("501", "Max", Side::Withdraw, 0, 500), Err(Told { chat: 2455, status: 2786 }));
        assert_eq!(sum("10", "Max", Side::Deposit, 10, MOST - 5), Err(Told { chat: 2787, status: 2783 }));
        assert_eq!(sum("10", "Max", Side::Withdraw, MOST - 5, 10), Err(Told { chat: 2459, status: 2787 }));
    }

    #[test]
    fn what_is_no_sum_is_told() {
        assert_eq!(sum("", "Max", Side::Deposit, 1000, 0), Err(Told { chat: 2779, status: 2781 }));
        assert_eq!(sum("1 000", "Max", Side::Deposit, 1000, 0), Err(Told { chat: 2488, status: 2782 }));
        assert_eq!(sum("-5", "Max", Side::Withdraw, 1000, 0), Err(Told { chat: 2488, status: 2782 }));
        assert_eq!(sum("2147483648", "Max", Side::Deposit, 1000, 0), Err(Told { chat: 2768, status: 2783 }));
        assert_eq!(sum("99999999999999999999", "Max", Side::Deposit, 1000, 0), Err(Told { chat: 2768, status: 2783 }));
        assert_eq!(sum("0", "Max", Side::Deposit, 1000, 0), Err(Told { chat: 2769, status: 2784 }));
    }

    #[test]
    fn the_word_of_the_button_max_is_all_that_can_go() {
        assert_eq!(sum("Max", "Max", Side::Deposit, 1000, 50), Ok(Some(1000)));
        assert_eq!(sum("Max", "Max", Side::Withdraw, 1000, 50), Ok(Some(50)));
        // Only as much as the other side still takes.
        assert_eq!(sum("Max", "Max", Side::Deposit, 1000, MOST - 100), Ok(Some(100)));
        assert_eq!(sum("Max", "Max", Side::Withdraw, MOST - 100, 1000), Ok(Some(100)));
        assert_eq!(sum("Max", "Max", Side::Deposit, 0, 50), Err(Told { chat: 2770, status: 2780 }));
        assert_eq!(sum("Max", "Max", Side::Withdraw, 1000, 0), Err(Told { chat: 2770, status: 2780 }));
        assert_eq!(sum("Max", "Max", Side::Deposit, 1000, MOST), Err(Told { chat: 2787, status: 2783 }));
        // A full purse takes nothing more: nothing is sent, and nothing said.
        assert_eq!(sum("Max", "Max", Side::Withdraw, MOST, 1000), Ok(None));
    }

    #[test]
    fn the_server_s_reasons_have_their_lines() {
        assert_eq!(deposit_refused(0), None);
        assert_eq!(deposit_refused(2), Some(2456));
        assert_eq!(deposit_refused(3), Some(2465));
        assert_eq!(deposit_refused(1), Some(2454));
        assert_eq!(withdrawal_refused(0), None);
        assert_eq!(withdrawal_refused(1), Some(2455));
        assert_eq!(withdrawal_refused(2), Some(2454));
        assert_eq!(check_refused(0), None);
        assert_eq!(check_refused(2), Some(3031));
        assert_eq!(check_refused(1), Some(2454));
    }
}
