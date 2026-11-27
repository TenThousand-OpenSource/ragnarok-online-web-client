//! The records of the windows read back (`CGameMode::ProcessUiMessage`): what a record of the kinds 900 to
//! 902 and 913 says was done, as presses of this client's page, texts of fields, a count and a drop. Lengths
//! and numbers of a file are not trusted: a record cut short or naming what is not there is passed by.

use super::*;

fn number(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// Reads a record of the kinds 900 to 902 and 913: what it says was done, by what stands open. None for
/// another kind, for a record cut short, and for a window or a command nothing here answers to.
pub fn deed(packet: &[u8], open: &Open) -> Option<Deed> {
    if packet.len() < 6 || u16::from_le_bytes([packet[0], packet[1]]) != CC_REPLAYPACKET {
        return None;
    }
    let (kind, body) = (u16::from_le_bytes([packet[2], packet[3]]), &packet[6..]);
    match kind {
        kind::RESPONSE => {
            let values = [number(body, 8)?, number(body, 12)?, number(body, 16)?];
            let (window, command) = (number(body, 0)?, number(body, 4)?);
            if window == window::COUNT {
                let counted = command == command::OK && values[0] == command::COUNTED;
                return counted.then_some(Deed::Counted { asked: values[2], amount: values[1] });
            }
            Some(Deed::Presses(presses_of(window, command, values, open)?))
        }
        kind::DROP => {
            let name = body.get(0x97 - 6..0x97 - 6 + 100)?;
            let digits: String = name.iter().take_while(|byte| byte.is_ascii_digit()).map(|byte| char::from(*byte)).collect();
            Some(Deed::Dropped(Dropped {
                onto: number(body, 0)?,
                from: number(body, 0x12 - 6)? as i32,
                id: number(body, 0x16 - 6)?,
                item: digits.parse().ok()?,
                amount: number(body, 0x22 - 6)? as i32,
                kind: number(body, 0x2a - 6)?,
            }))
        }
        kind::TEXTS => {
            let (window, count) = (number(body, 0)?, number(body, 4)?);
            let (mut texts, mut at) = (Vec::new(), 8);
            for _ in 0..count.min(64) {
                let length = number(body, at)? as usize;
                texts.push(String::from_utf8_lossy(body.get(at + 4..(at + 4).checked_add(length)?)?).into_owned());
                at += 4 + length;
            }
            Some(Deed::Typed { window, texts })
        }
        kind::VALUES => {
            let (window, message) = (number(body, 0)?, number(body, 20)?);
            let presses = match (window, message, number(body, 4)?, number(body, 8)?) {
                (window::QUEST_UI, command::TAB, bit @ (1 | 2 | 4 | 8), _) => {
                    vec![Press::Quests(quests::Press::Tab(bit.trailing_zeros() as usize))]
                }
                // The box is set to what the record says: pressed only if it stands otherwise.
                (window::QUEST_UI, command::FLAG, command::CHECK, shown) if (shown != 0) != open.quests_shown => {
                    vec![Press::Quests(quests::Press::Display)]
                }
                (window::QUEST_UI, command::FLAG, command::CHECK, _) => Vec::new(),
                _ => return None,
            };
            Some(Deed::Presses(presses))
        }
        _ => None,
    }
}

/// The presses a command of a window is (`ProcessUiMessage`, 900: `SendMsg(UM_COMMAND, …)` to that window).
fn presses_of(window: u32, command: u32, values: [u32; 3], open: &Open) -> Option<Vec<Press>> {
    let yes = match command {
        command::OK => Some(true),
        command::CANCEL => Some(false),
        _ => None,
    };
    let one = |press: Press| Some(vec![press]);
    match window {
        // `UISayDialogWnd::SendMsg`: the third value is the message the window answers with as it plays
        // (`m_responseMessage`); the button stands at 600 after "next" and does nothing then.
        window::SAY => match (command, values[2]) {
            (command::NOTHING, _) => None,
            (_, command::CLOSE_DIALOG) | (command::CLOSE, _) => one(talk::Press::Close.into()),
            (command::NEXT, _) => one(talk::Press::Next.into()),
            _ => None,
        },
        window::CHOOSE if yes? => Some(vec![talk::Press::Pick(values[0] as usize).into(), talk::Press::Ok.into()]),
        window::CHOOSE => one(talk::Press::Cancel.into()),
        window::NPC_NUMBER | window::NPC_WORDS if yes? => one(talk::Press::Ok.into()),
        window::CHOOSE_SELL_BUY => match command {
            command::BUY => one(shops::Press::Buy.into()),
            command::SELL => one(shops::Press::Sell.into()),
            command::CANCEL => one(shops::Press::Cancel.into()),
            _ => None,
        },
        window::PURCHASE | window::SELL => one(if yes? { shops::Press::Ok } else { shops::Press::Cancel }.into()),
        // The box is set to what the record says (`UIItemShopWnd::SendMsg`, `ID_CHECK`).
        window::SHOP_LIST if command == command::CHECK => {
            Some(if (values[0] != 0) != open.whole_stacks { vec![shops::Press::WholeStacks.into()] } else { Vec::new() })
        }
        window::EXCHANGE_ACCEPT => one(if yes? { exchange::Press::Accept } else { exchange::Press::Refuse }.into()),
        window::JOIN_PARTY if open.party_invited => one(if yes? { party::Press::Yes } else { party::Press::No }.into()),
        window::JOIN_GUILD if open.guild_invited => one(if yes? { guild::Press::Yes } else { guild::Press::No }.into()),
        window::JOIN_FRIEND => one(if yes? { friends::Press::Yes } else { friends::Press::No }.into()),
        _ => list_presses(window, yes?, values, open),
    }
}

/// A list's OK or Cancel done again: the row its window recorded is picked first (the client sets its list's
/// `selected` from the record as it plays), if it is one.
fn list_presses(window: u32, ok: bool, values: [u32; 3], open: &Open) -> Option<Vec<Press>> {
    let list = LISTS.into_iter().find(|list| of_list(*list).0 == window)?;
    let row = of_list(list).1.map(|place| values[place]).filter(|row| *row != u32::MAX).map(|row| row as usize);
    Some(match list {
        List::Identify => {
            let picked = row.filter(|_| ok).map(|row| inventory::Press::IdentifyPick(row).into());
            let answer = if ok { inventory::Press::IdentifyOk } else { inventory::Press::IdentifyCancel };
            picked.into_iter().chain([answer.into()]).collect()
        }
        List::Eggs => {
            let picked = row.filter(|_| ok).map(|row| pets::Press::EggPick(row).into());
            picked.into_iter().chain([if ok { pets::Press::EggOk } else { pets::Press::EggCancel }.into()]).collect()
        }
        // (A list of another window than the one open is not answered.)
        _ if open.list.map(|(open, _)| open) != Some(list) => return None,
        _ => {
            let picked = row.filter(|_| ok).map(|row| choosing::Press::Pick(row).into());
            picked.into_iter().chain([if ok { choosing::Press::Ok } else { choosing::Press::Cancel }.into()]).collect()
        }
    })
}
