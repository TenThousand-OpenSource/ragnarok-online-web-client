//! The windows of a deal between two players: the deal itself with what each side has put in (the client's
//! `CUIExchangeUI`), the question whether to deal (`UIExchangeAcceptWnd`), and how many of a thing go in.

use super::*;
use crate::exchange::zeny_colour;
use crate::press::exchange::Press;

/// One of the window's three buttons, 42 by 20: its pictures as the client names them (`name.bmp`, `name_b.bmp`
/// under the pointer, `name_a.bmp` pressed), or the one that shows it cannot be pressed.
fn deal_button(x: i32, name: &str, off: Option<&str>, press: Press) -> String {
    match off {
        Some(off) => art(&format!("{off}.bmp"), x, 354, 42, 20),
        None => pictured_by(Convention::BA, x, 354, (42, 20), name, press, ""),
    }
}

impl App {
    /// The deal's window, 560 by 377 (`CUIExchangeUI::OnCreate`): the title and the other's name in its bar;
    /// over the picture of its ground the player's side at 4,21 and the other's at 283,21, ten rows of 30
    /// each; under each side its zeny; at the bottom OK, the screenshot's box, Trade and Cancel.
    pub(in crate::web) fn exchange_window(&self) -> String {
        let deal = &self.exchange;
        let mut window = title_bar(560, "");
        window.push_str(&text(12, 2, BLACK, &format!("{} : ", Self::words(206, "Trade"))));
        // The client writes the other's level after the name, and after that a code of the character in
        // brackets, whose making was not found: the code is left out.
        window.push_str(&text(62, 2, BLACK, &if deal.level > 0 { format!("{} Lv{}", deal.partner, deal.level) } else { deal.partner.clone() }));
        window.push_str(&art("basic_interface/exchange_bg2.bmp", 0, 17, 560, 360));
        window.push_str(&self.exchange_side(4, &deal.mine, deal.closed.0));
        window.push_str(&self.exchange_side(283, &deal.theirs, deal.closed.1));
        // The player's zeny is typed until the side is closed, and written after (`UM_MYDRAGCLOSED`).
        if deal.closed.0 {
            window.push_str(&Self::exchange_zeny(70, deal.zeny.0));
        } else {
            window.push_str(&numeric_field("exchange-zeny", (140, 330, 100, 16), "0", 13, &format!("background:{FIELD_GROUND};border:1px solid #c6c6c6;box-sizing:border-box")));
        }
        window.push_str(&Self::exchange_zeny(350, deal.zeny.1));
        window.push_str(&deal_button(3, "btn_ok", (deal.locking || deal.closed.0).then_some("btn_ok_dis"), Press::Lock));
        window.push_str(&format!(
            r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat" {}></div>"#,
            at(105, 359, 10, 10),
            picture(if deal.screenshot { "checkbox_1.bmp" } else { "checkbox_0.bmp" }),
            pressed(Press::Screenshot)
        ));
        window.push_str(&text(123, 356, "#ff0000", &Self::words(1831, "Screenshot Trade")));
        window.push_str(&deal_button(260, "btn_exchange", (deal.closed != (true, true) || deal.trading).then_some("btn_exchange_dis"), Press::Trade));
        window.push_str(&deal_button(510, "btn_cancel", deal.cancelling.then_some("btn_cancel"), Press::Cancel));
        window
    }

    /// One side of the deal: ten rows (`ExchangeUI::CUIItem`, 272 by 30), each with the cell of the inventory 9
    /// in, the thing's picture 13,1, how many there are 25,15 in the smaller text, and its name from 45. Once
    /// the side is closed its rows lose the cell and stand on grey in place of white (`CUIItem::Paint` for a
    /// frozen row: the skin's colour 10,6, which is 210, 210, 210 in the client's `colorchip.bmp`).
    fn exchange_side(&self, x: i32, items: &[Item], closed: bool) -> String {
        let mut side = String::new();
        for row in 0..10 {
            let y = 21 + row * 30;
            if closed {
                side.push_str(&fill(x, y, 272, 30, "#d2d2d2"));
            } else {
                side.push_str(&art("basic_interface/itemwin_mid.bmp", x + 9, y, 32, 32));
            }
            let Some(item) = items.get(row as usize) else { continue };
            side.push_str(&format!(r#"<div class="a" style="{}" {}>{}</div>"#, at(x + 13, y + 1, 24, 24), item_marks(item.id, item.identified), Self::icon_of(item)));
            side.push_str(&text_in(SMALL, x + 25, y + 15, BLACK, &item.amount.to_string()));
            side.push_str(&format!(
                r#"<div class="a t" style="{};line-height:30px;overflow:hidden;white-space:nowrap">{}</div>"#,
                at(x + 45, y, 215, 30),
                escape(&Self::item_name(item))
            ));
        }
        side
    }

    /// A side's zeny, ending 150 right of `x`, 330 down, as the client's three texts (`CUIExchangeUI::OnCreate`,
    /// `UpdateZeny`), in Arial 16: one in the colour of its size a pixel to the left, one in that colour in
    /// place, and over both, added last, a dark one a pixel lower. So the number reads dark with an edge of
    /// its colour. (The colour is the first of the two `PriceColor` gives; the second, a ground behind the text
    /// for some sizes, is not drawn.)
    fn exchange_zeny(x: i32, zeny: u32) -> String {
        let words = grouped(i64::from(zeny));
        let font = Font(1, 16, false);
        let colour = zeny_colour(zeny);
        [(x + 149, 330, colour), (x + 150, 330, colour), (x + 150, 331, "#323232")].into_iter().map(|(right, y, colour)| text_ending_in(font, right, 560, y, colour, &words)).collect()
    }

    /// The question whether to deal, in the client's message box (`UIExchangeAcceptWnd`, 280 by 120): who asks
    /// and their level, OK 189,96 and Cancel 234,96. (The client writes a code of the character before the
    /// level, whose making was not found: it is left out.)
    pub(in crate::web) fn exchange_question(&self) -> String {
        let Some((name, level)) = &self.exchange.asked else { return String::new() };
        message_box(Asked::Offer, &format!("({}){}<br>Lv.{level}", escape(name), escape(&Self::words(93, " requests a deal."))), Press::Accept, Press::Refuse)
    }

    /// How many of a thing go into the deal: the window that asks how many to drop, asking this (the client's
    /// `UIItemDropCntWnd` in its fifth state), with all of them ready to be typed over.
    pub(in crate::web) fn exchange_count_window(&self) -> String {
        self.count_box(&windows::EXCHANGE_COUNT, self.exchange.counting, Form::Offer, Press::CountOk)
    }
}
