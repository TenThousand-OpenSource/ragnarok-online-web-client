//! The cash shop: the banners of the strip under its title.

use super::*;

/// What the client gives the file to call (`AddCashshopBanner` of `LuaScript.cpp`, registered as
/// `add_cashshop_banner`), written in Lua: a banner's picture and the address its page is at are kept, in
/// the order they are added.
const GIVEN: &str = "BANNERS = {} function add_cashshop_banner(image, url) BANNERS[#BANNERS + 1] = { image, url } end";

/// The banners a state has after the file's `set_cashshop_banner` was called in it, as
/// `UICashShopWnd::SendMsg` calls it when the window is made: `[[picture, address], …]`.
fn told(lua: &Lua) -> serde_json::Value {
    if let Err(error) = exec(lua, b"set_cashshop_banner()", Some("set_cashshop_banner")) {
        eprintln!("tables: tb_cashshop_banner.lub: {error}");
    }
    json_of(&lua.globals().get::<Value>("BANNERS").unwrap_or(Value::Nil))
}

/// `{"banners": [["01_Cashshop_Banner.bmp", "https://…"], …]}`, out of `datainfo\tb_cashshop_banner.lub`:
/// each a picture of the folder `cashshop` of the interface's pictures, and the address opened when it is
/// pressed.
pub fn cashshop(grf: &Grf) -> serde_json::Value {
    let lua = state();
    let given = exec(&lua, GIVEN.as_bytes(), Some("cashshop"));
    debug_assert!(given.is_ok());
    run_in(&lua, grf, &["datainfo\\tb_cashshop_banner"]);
    serde_json::json!({ "banners": told(&lua) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_s_function_adds_its_banners_in_order() {
        let lua = state();
        exec(&lua, GIVEN.as_bytes(), None).expect("what is given runs");
        let source = br#"
            tb_cashshop_banner = { { "01.bmp", "https://a" }, { "02.bmp", "" } }
            function set_cashshop_banner()
                for _, banner in ipairs(tb_cashshop_banner) do add_cashshop_banner(banner[1], banner[2]) end
                return true
            end
        "#;
        exec(&lua, source, Some("test")).expect("the file runs");
        assert_eq!(told(&lua), serde_json::json!([["01.bmp", "https://a"], ["02.bmp", ""]]));
    }

    #[test]
    fn a_file_that_is_not_there_gives_no_banners() {
        let lua = state();
        exec(&lua, GIVEN.as_bytes(), None).expect("what is given runs");
        assert_eq!(told(&lua).as_array().map(Vec::len).unwrap_or(0), 0);
    }
}
