//! The cash shop's window (the client's `UICashShopWnd` with its banner, its things and its list of what is
//! bought), and the message box it asks and tells with.

use super::*;
use crate::cashshop::{self, Good, LIMITED, Line, SHOWN, TABS, Turn};
use crate::press::Step;
use crate::press::cashshop::Press;
use crate::web::app::CashShopAsk;

/// One of the client's colours of text, written as its code writes it (`0xBBGGRR`), as the page takes it.
fn colour(code: u32) -> String {
    format!("#{:02x}{:02x}{:02x}", code & 0xff, code >> 8 & 0xff, code >> 16 & 0xff)
}

/// A picture of the shop's folder.
fn shop(name: &str) -> String {
    format!("cashshop/{name}.bmp")
}

/// A line in the middle of a stretch `width` wide that begins at `x` (`DrawCenteredText` with a width).
fn centred(x: i32, y: i32, width: i32, font: Font, colour: &str, words: &str) -> String {
    [
        format!(r#"<div class="a t" style="left:{x}px;top:{y}px;width:{width}px;text-align:center;white-space:nowrap;overflow:hidden;"#),
        format!(r#"text-overflow:ellipsis;pointer-events:none;color:{colour};{}">{}</div>"#, font.style(), escape(words))
    ].concat()
}

/// One of the shop's buttons with words on it (`UIBmpTextButton`): a picture as it stands and one pressed,
/// the words in the middle. Without a press it is the one that cannot be pressed.
fn worded(x: i32, y: i32, size: (i32, i32), name: &str, words: &str, press: Option<Press>) -> String {
    let (normal, down) = (picture(&shop(&format!("{name}_normal"))), picture(&shop(&format!("{name}_press"))));
    [
        format!(r#"<button type="button" class="a b" style="{};--n:url('{normal}');--o:url('{down}');"#, at(x, y, size.0, size.1)),
        format!(r#"--p:url('{down}');{};line-height:{}px" "#, Font(0, 12, false).style(), size.1),
        format!("{}>{}</button>", press.map_or_else(|| "disabled".to_owned(), pressed), escape(words))
    ].concat()
}

/// What an item is called and the name of its pictures, for a thing of the shop (`ITEM_INFO::SetName` of
/// `GetIdDisplayName`, and `ResourcePath`).
fn about(item: u32) -> (String, Option<String>) {
    (called(item), data::item(item).map(|(_, resource)| resource))
}

/// The large picture of an item (`collection\<name>.bmp`) at a place, of that size; the right button on it
/// opens what is known of the item (`OnRBtnUp`).
fn portrait(item: u32, resource: Option<&str>, (x, y, width, height): (i32, i32, i32, i32)) -> String {
    let Some(resource) = resource.filter(|resource| !resource.is_empty()) else { return String::new() };
    format!(
        r#"<img class="a" src="{}" alt="" draggable="false" style="{}" {} onerror="this.remove()">"#,
        picture(&format!("collection/{resource}.bmp")),
        at(x, y, width, height),
        item_marks(item, true)
    )
}

/// A thing of the page, by its place on it (`UICashShopWnd_Item::Paint` in its smaller layout, 172 by 126,
/// three to a row from (8, 107), 175 apart across and 127 down): `img_shop_itemBg.bmp`; the name in the
/// middle at 6 down, cut to 168, in bold of the client's 0xA44F3B, told whole when pointed at; the large
/// picture at (5, 23); the price ending at 160, 50 down, in bold white; "Buy" (line 1779) at (83, 85).
fn thing(place: usize, good: &Good) -> String {
    let (x, y) = (place as i32 % 3 * 175 + 8, place as i32 / 3 * 127 + 107);
    let (name, resource) = about(good.item);
    [
        format!(r#"<div class="a" style="{};background:url('{}') no-repeat">"#, at(x, y, 172, 126), picture(&shop("img_shop_itemBg"))),
        format!(r#"<div class="a" style="{}" data-tip="{}"></div>"#, at(0, 0, 172, 23), escape(&name)),
        centred(2, 6, 168, Font(0, 12, true), &colour(0xA44F3B), &name),
        portrait(good.item, resource.as_deref(), (5, 23, 75, 100)),
        text_ending_in(Font(0, 12, true), 160, 172, 50, WHITE, &good.price.to_string()),
        worded(83, 85, (81, 24), "btn_add", &App::words(1779, "Buy"), Some(Press::Add(place))),
        "</div>".to_owned()
    ].concat()
}

/// A kind in the list of what is bought, in its place among the six that show (`UICashShopWnd_CartItem`, 172
/// by 54 at (3, 5) of the list and 57 apart): `img_shop_itemBg2.bmp`; the name at (48, 12), cut to 112, in
/// the client's 0x902418; the large picture at half its size at (4, 0); the price of one ending at 160, 34
/// down, with " C"; how many in the middle of 32 from 50, 33 down; the small cross at (161, 2); the plus at
/// (82, 29) and the minus at (82, 38).
fn kind(place: usize, line: &Line) -> String {
    let item = line.good.item;
    let (name, resource) = about(item);
    let step = |y: i32, name: &str, press: Press| pictured_by(Convention::NormalOverPress, 82, y, (19, 8), &format!("cashshop/{name}"), press, "");
    [
        format!(
            r#"<div class="a" style="{};background:url('{}') no-repeat">"#,
            at(3, 5 + place as i32 * 57, 172, 54),
            picture(&shop("img_shop_itemBg2"))
        ),
        portrait(item, resource.as_deref(), (4, 0, 37, 50)),
        format!(
            r#"<div class="a t" style="{};overflow:hidden;white-space:nowrap;text-overflow:ellipsis;color:{};{}" data-tip="{3}">{3}</div>"#,
            at(48, 12, 112, 12),
            colour(0x902418),
            SMALL.style(),
            escape(&name)
        ),
        text_ending(160, 172, 34, BLACK, &format!("{} C", line.good.price)),
        centred(50, 33, 32, SMALL, BLACK, &line.amount.to_string()),
        pictured(161, 2, (7, 7), [&shop("btn_close"), &shop("btn_close"), &shop("btn_close")], Press::Take(item), ""),
        step(29, "btn_plus", Press::More(item)),
        step(38, "btn_minus", Press::Less(item)),
        "</div>".to_owned()
    ].concat()
}

impl App {
    /// The strip of banners (`UICashShopWnd_Banner`, 531 by 51 at (3, 18)): the banner shown, a press on
    /// which opens its page; and, eight square and 12 apart at the strip's right end, 41 down, a small button
    /// for each banner, the shown one's lit.
    fn cashshop_banner(&self) -> String {
        let banners = data::cashshop_banners();
        let Some((image, _)) = banners.get(self.cashshop.banner) else { return String::new() };
        let mut strip = format!(
            r#"<div class="a" style="{};overflow:hidden;cursor:pointer;background:url('{}') no-repeat" {}></div>"#,
            at(3, 18, 531, 51),
            picture(&format!("cashshop/{image}")),
            pressed(Press::Banner)
        );
        for place in 0..banners.len() {
            let lit = shop(if place == self.cashshop.banner { "btn_ad_on" } else { "btn_ad_off" });
            let x = 3 + 531 + (place as i32 - banners.len() as i32) * 12 - 12;
            strip.push_str(&pictured(x, 18 + 41, (8, 8), [&lit, &lit, &shop("btn_ad_on")], Press::BannerPicked(place), ""));
        }
        strip
    }

    /// The tabs (`UICashShopWnd::OnCreate`: 56 by 31 from (8, 72), 58 apart, a list each but the eighth and
    /// the search's), the open one's lit; each tells its name when pointed at.
    fn cashshop_tabs(&self) -> String {
        TABS.iter()
            .enumerate()
            .map(|(place, tab)| {
                let lit = shop(&format!("img_shop_tap{tab}_{}", if *tab == self.cashshop.tab { "on" } else { "off" }));
                let tells = format!(r#" data-tip="{}""#, escape(&Self::words(cashshop::tab_line(*tab), "")));
                pictured(8 + place as i32 * 58, 72, (56, 31), [&lit, &lit, &shop(&format!("img_shop_tap{tab}_on"))], Press::Tab(*tab), &tells)
            })
            .collect()
    }

    /// The things of the page shown, the four arrows under them with the page and how many there are, the
    /// line to search with and "Refresh" (`UICashShopWnd::Paint`, `OnCreate`, `LayoutPage`). The larger
    /// layout of the sale for one account is not drawn: no server here tells of such a sale, and its list is
    /// empty.
    fn cashshop_page(&self) -> String {
        let shop_state = &self.cashshop;
        let pages = shop_state.pages();
        let mut page: String = if shop_state.tab == LIMITED {
            String::new()
        } else {
            let first = shop_state.page * shop_state.page_size;
            shop_state.shown().iter().skip(first).take(shop_state.page_size).enumerate().map(|(place, good)| thing(place, good)).collect()
        };
        let arrow = |x: i32, width: i32, name: &str, turn: Turn| {
            let (off, on) = (shop(&format!("{name}_off")), shop(&format!("{name}_on")));
            pictured(x, 493, (width, 12), [&off, &off, &on], Press::Page(turn), "")
        };
        page.push_str(&[arrow(217, 16, "bt_arrowL2", Turn::First), arrow(236, 11, "bt_arrowL", Turn::Back)].concat());
        page.push_str(&[arrow(290, 11, "bt_arrowR", Turn::On), arrow(304, 16, "bt_arrowR2", Turn::Last)].concat());
        let (at_page, of) = if pages == 0 { (0, 0) } else { (shop_state.page + 1, pages) };
        page.push_str(&format!(
            r#"<div class="a t" style="{};text-align:center;white-space:nowrap;color:{}"><b>{at_page}</b> / {of}</div>"#,
            at(247, 494, 43, 14),
            colour(0x6B2921)
        ));
        page.push_str(&[
            format!(r#"<form {} class="a" style="{}">"#, sent(Form::CashShopSearch), at(94, 518, 291, 14)),
            format!(
                r#"<input class="a in" id="{}" autocomplete="off" value="{}" placeholder="{}" style="{};background:none"></form>"#,
                windows::CASH_SHOP.field(),
                escape(&shop_state.sought),
                escape(&Self::words(3302, "Item Search")),
                at(0, 0, 291, 14)
            )
        ].concat());
        page.push_str(&pictured_by(Convention::NormalOverPress, 392, 516, (54, 18), "cashshop/btn_searchbar", Press::Search, ""));
        let refresh = (shop_state.tab == LIMITED).then_some(Press::Refresh);
        page.push_str(&worded(452, 516, (76, 18), "btn_refresh", &Self::words(3306, "Refresh"), refresh));
        page
    }

    /// The list of what is bought (`UICashShopWnd_Cart`, 185 by 350 at (537, 72)): `img_shop_cart_bg.bmp`,
    /// the six kinds that show, and at its right edge the client's bar, 7 wide, with its two arrows.
    fn cashshop_cart(&self) -> String {
        let (kinds, top) = (self.cashshop.cart.len(), self.cashshop.top);
        let ground = picture(&shop("img_shop_cart_bg"));
        let mut list = format!(r#"<div class="a" style="{};background:url('{ground}') no-repeat">"#, at(537, 72, 185, 350));
        list.extend(self.cashshop.cart.iter().skip(top).take(SHOWN).enumerate().map(|(place, line)| kind(place, line)));
        let arrow = |y: i32, name: &str, step: Step| pictured(178, y, (7, 7), [&shop(name), &shop(name), &shop(name)], Press::Scroll(step), "");
        list.push_str(&arrow(0, "shop_scroll_up", Step::Less));
        list.push_str(&arrow(343, "shop_scroll_down", Step::More));
        if kinds > SHOWN {
            // The bar's thumb: as much of the way between the arrows as the six are of the kinds.
            let (way, long) = (336, 336 * SHOWN as i32 / kinds as i32);
            list.push_str(&format!(
                r#"<div class="a" style="{};background:url('{}') repeat-y"></div>"#,
                at(178, 7 + way * top as i32 / kinds as i32, 7, long),
                picture(&shop("shop_bar_mid"))
            ));
        }
        list.push_str("</div>");
        list
    }

    /// What stands at the right of the window (`UICashShopWnd::Paint`): "Buy List" (line 3289) over the list,
    /// white over a black copy of itself; the cash points ending at 627, 51 down, and "Charging" (3299) at
    /// (654, 46); under the list "Free points" (3303) with the account's, "Use Free points" (3304) with the
    /// field for them and "Owned points" (3305), which the client writes nought by; the price of everything in
    /// the list in the middle at 492 down, and "Purchase" (3300) at (552, 508).
    fn cashshop_side(&self) -> String {
        let title = Self::words(3289, "Buy List");
        let row = |y: i32, line: usize, otherwise: &str, sum: &str| {
            [text(550, y, BLACK, &Self::words(line, otherwise)), text_ending(708, 723, y, BLACK, sum)].concat()
        };
        [
            centred(538, 27, 185, Font(0, 14, true), BLACK, &title),
            centred(537, 26, 185, Font(0, 14, true), WHITE, &title),
            text_ending_in(Font(0, 12, true), 627, 723, 51, WHITE, &self.cashshop.cash.to_string()),
            worded(654, 46, (64, 20), "btn_charge", &Self::words(3299, "Charging"), Some(Press::Charge)),
            self.cashshop_cart(),
            row(434, 3303, "Free points", &format!("{} C", self.cashshop.points)),
            row(450, 3304, "Use Free points", "C"),
            row(466, 3305, "Owned points", "0 C"),
            format!(
                r#"<input id="{}" class="a in" style="{};background:#ced6eb;text-align:right" type="text" inputmode="numeric" maxlength="10" value="{}">"#,
                windows::cash_shop_points(),
                at(636, 448, 62, 14),
                escape(&self.cashshop.use_points)
            ),
            centred(552, 492, 154, Font(0, 14, true), WHITE, &format!("{} C", cashshop::total(&self.cashshop.cart))),
            worded(552, 508, (154, 23), "btn_buy", &Self::words(3300, "Purchase"), Some(Press::Buy))
        ].concat()
    }

    /// The cash shop (`UICashShopWnd`, 723 by 540): `cashshop\img_shop_bg.bmp` (`img_shop_bg2.bmp` under the
    /// sale for one account) with the title bar "Cash Shop" (line 3288 of the message table) and the cross
    /// over it, the banners, the tabs, the page of things and what stands at the right.
    pub(in crate::web) fn cashshop_window(&self) -> String {
        let ground = shop(if self.cashshop.tab == LIMITED { "img_shop_bg2" } else { "img_shop_bg" });
        [
            art(&ground, 0, 0, 723, 540),
            title_bar(723, &Self::words(3288, "Cash Shop")),
            cross(723, Press::Close),
            self.cashshop_banner(),
            self.cashshop_tabs(),
            self.cashshop_page(),
            self.cashshop_side()
        ].concat()
    }

    /// What the shop asks or tells in the client's message box (`UIWindowMgr::Confirm`): something told has
    /// OK alone, a purchase to be meant has OK and Cancel.
    pub(in crate::web) fn cashshop_question(&self) -> String {
        match &self.cashshop.asks {
            Some(CashShopAsk::Told(words, error)) => told_box(if *error { "Error" } else { "Message" }, words, Press::No),
            Some(CashShopAsk::Buy(words, _)) => confirm_box(words, Press::Yes, Press::No),
            None => String::new(),
        }
    }
}
