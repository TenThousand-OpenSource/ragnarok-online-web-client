//! A picture of the screen, given to the player as a file (the client's `MM_SCREENSHOT`: Print Screen, and a
//! deal with its box "Screenshot Trade" ticked).
//!
//! The client copies its one drawing surface. Here the world is a canvas and the windows are HTML over it, so
//! the picture is put together: the world's canvas is drawn into a new one; then a copy of the page's windows
//! (with their pictures read in as data, the canvases in them turned into pictures, what is typed kept, and
//! the page's own style sheet beside them) is drawn over it as an SVG picture that holds HTML; then the
//! client's stamp, the date and time, in the bottom left corner. What does not come out: the game's mouse
//! pointer; a font the page fetches (the browser's own stands in); and in a browser that will not read back
//! a canvas such a picture was drawn on, the windows are left out and the picture is the world alone.

use super::*;

#[wasm_bindgen(inline_js = r#"
async function asData(url, kept) {
  if (!url || url.startsWith('data:')) return url;
  if (!kept.has(url)) {
    kept.set(url, fetch(url).then((answer) => answer.blob()).then((blob) => new Promise((done) => {
      const reader = new FileReader();
      reader.onload = () => done(reader.result);
      reader.onerror = () => done('');
      reader.readAsDataURL(blob);
    })).catch(() => ''));
  }
  return kept.get(url);
}

async function copied(part, kept) {
  const copy = part.cloneNode(true);
  const from = [part, ...part.querySelectorAll('*')];
  const to = [copy, ...copy.querySelectorAll('*')];
  for (let at = 0; at < from.length; at++) {
    const [was, is] = [from[at], to[at]];
    const style = getComputedStyle(was);
    if (style.display === 'none') continue;
    const back = style.backgroundImage;
    if (back && back.includes('url(')) {
      const urls = [...back.matchAll(/url\("?([^")]+)"?\)/g)].map((found) => found[1]);
      let inlined = back;
      for (const url of urls) inlined = inlined.replace(url, await asData(url, kept));
      is.style.backgroundImage = inlined;
    }
    if (was.tagName === 'IMG' && was.src) is.src = await asData(was.src, kept);
    if (was.tagName === 'CANVAS') {
      const picture = document.createElement('img');
      try { picture.src = was.toDataURL(); } catch (error) { picture.src = ''; }
      picture.setAttribute('style', (was.getAttribute('style') || '') + `;width:${was.offsetWidth}px;height:${was.offsetHeight}px`);
      picture.className = was.className;
      is.replaceWith(picture);
    }
    if (was.tagName === 'INPUT') is.setAttribute('value', was.value);
  }
  return copy;
}

function sheet() {
  let rules = '';
  for (const styles of document.styleSheets) {
    try { for (const rule of styles.cssRules) rules += rule.cssText + '\n'; } catch (error) { /* another site's sheet */ }
  }
  return rules;
}

function drawn(url) {
  return new Promise((done, failed) => {
    const picture = new Image();
    picture.onload = () => done(picture);
    picture.onerror = () => failed(new Error('the windows could not be drawn'));
    picture.src = url;
  });
}

function blobOf(canvas) {
  return new Promise((done, failed) => {
    try { canvas.toBlob((blob) => (blob ? done(blob) : failed(new Error('no picture'))), 'image/png'); } catch (error) { failed(error); }
  });
}

function world(game, stamp) {
  const canvas = document.createElement('canvas');
  canvas.width = game.width;
  canvas.height = game.height;
  canvas.getContext('2d').drawImage(game, 0, 0);
  return canvas;
}

function stamped(canvas, stamp, scale) {
  const pen = canvas.getContext('2d');
  pen.font = `${12 * scale}px Arial, sans-serif`;
  pen.textBaseline = 'top';
  const y = canvas.height - 16 * scale;
  pen.fillStyle = '#000';
  for (const [x, down] of [[5, 0], [3, 0], [4, 1], [4, -1]]) pen.fillText(stamp, x * scale, y + down * scale);
  pen.fillStyle = '#fff';
  pen.fillText(stamp, 4 * scale, y);
}

export async function picture_of_screen(name, stamp) {
  const game = document.getElementById('game');
  const scale = game.width / Math.max(1, game.clientWidth);
  let canvas = world(game);
  let whole = true;
  try {
    const kept = new Map();
    const holder = document.createElement('div');
    holder.setAttribute('xmlns', 'http://www.w3.org/1999/xhtml');
    holder.setAttribute('style', `position:relative;width:${game.clientWidth}px;height:${game.clientHeight}px;overflow:hidden`);
    const styles = document.createElement('style');
    styles.textContent = sheet();
    holder.appendChild(styles);
    for (const id of ['bars', 'pvp-rank', 'unit-name', 'mapsign', 'ui']) {
      const part = document.getElementById(id);
      if (part) holder.appendChild(await copied(part, kept));
    }
    const html = new XMLSerializer().serializeToString(holder);
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${game.clientWidth}" height="${game.clientHeight}"><foreignObject width="100%" height="100%">${html}</foreignObject></svg>`;
    const windows = await drawn('data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svg));
    canvas.getContext('2d').drawImage(windows, 0, 0, canvas.width, canvas.height);
    stamped(canvas, stamp, scale);
    var blob = await blobOf(canvas);
  } catch (error) {
    whole = false;
    canvas = world(game);
    stamped(canvas, stamp, scale);
    blob = await blobOf(canvas);
  }
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = name;
  document.body.appendChild(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(link.href), 60000);
  return whole;
}
"#)]
extern "C" {
    /// Makes the picture and hands it to the browser to save under that name; tells whether the windows are
    /// in it.
    #[wasm_bindgen(catch)]
    async fn picture_of_screen(name: &str, stamp: &str) -> Result<JsValue, JsValue>;
}

/// Where the browser keeps how many pictures were made: the next one's number (the client looks for the first
/// number no file has yet).
const KEPT: &str = "roweb.pictures";

impl App {
    /// Makes a picture of the screen (`MM_SCREENSHOT`); `caption` is what it is of, when something asked for
    /// it (a deal: line 1832 of the client's message table with the partner's name).
    pub(in crate::web::app) fn screenshot(&mut self, caption: &str) {
        let Some(game) = &self.game else { return };
        let number = stored(KEPT).and_then(|number| number.parse::<u32>().ok()).unwrap_or(0);
        store(KEPT, Some(&(number + 1).to_string()));
        let name = crate::screenshot::file_name(caption, &game.name, number);
        let now = js_sys::Date::new_0();
        let stamp = crate::screenshot::stamp((now.get_full_year(), now.get_month() + 1, now.get_date()), now.get_day(), (now.get_hours(), now.get_minutes(), now.get_seconds()));
        wasm_bindgen_futures::spawn_local(async move {
            let made = picture_of_screen(&name, &stamp).await;
            dispatch(Action::Pictured(match made {
                Ok(whole) if whole.as_bool() == Some(false) => Ok(format!("{name} ({})", crate::i18n::tr("without the windows"))),
                Ok(_) => Ok(name),
                Err(error) => Err(js_error(error)),
            }));
        });
    }

    /// The picture was made: the chat says so as the client does ("<file> is Saved.", in green); or that it
    /// could not be made, in this client's words.
    pub(in crate::web::app) fn screenshot_saved(&mut self, saved: Result<String, String>) {
        // A deal that waited for its picture goes on, whether or not the picture could be made.
        if std::mem::take(&mut self.exchange.picturing) {
            self.send_on_map(&crate::net::exchange::trade());
        }
        match saved {
            Ok(name) => self.chat_line(format!("{name} is Saved."), "#00ff00"),
            Err(why) => self.chat_line(format!("{}: {why}", crate::i18n::tr("The picture of the screen could not be made")), RED),
        }
    }
}
