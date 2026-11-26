//! Where recordings are kept, the browser having no `Replay` folder beside the client: in the browser's own
//! database (IndexedDB), each under its file's name; and the two ways between it and files of the player's
//! machine, saving one and taking one in. This is the project's own; the Windows client reads and writes
//! `Replay\*.rrf` (`ReplayFileList::Refresh`, `Open`).

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
function store(mode) {
  return new Promise((done, failed) => {
    const opening = indexedDB.open('roweb-replays', 1);
    opening.onupgradeneeded = () => opening.result.createObjectStore('files');
    opening.onerror = () => failed(opening.error);
    opening.onsuccess = () => done(opening.result.transaction('files', mode).objectStore('files'));
  });
}
function asked(request) {
  return new Promise((done, failed) => { request.onsuccess = () => done(request.result); request.onerror = () => failed(request.error); });
}
export async function replay_files() {
  const files = await store('readonly');
  const names = await asked(files.getAllKeys());
  const bytes = await asked(files.getAll());
  return names.map((name, place) => [String(name), new Uint8Array(bytes[place])]);
}
export async function replay_keep(name, bytes) {
  await asked((await store('readwrite')).put(bytes, name));
}
export async function replay_drop(name) {
  await asked((await store('readwrite')).delete(name));
}
export function replay_save(name, bytes) {
  const link = document.createElement('a');
  link.href = URL.createObjectURL(new Blob([bytes], { type: 'application/octet-stream' }));
  link.download = name;
  document.body.appendChild(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(link.href), 60000);
}
export function replay_pick() {
  return new Promise((done) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.rrf';
    input.onchange = async () => {
      const file = input.files && input.files[0];
      done(file ? [file.name, new Uint8Array(await file.arrayBuffer())] : null);
    };
    input.oncancel = () => done(null);
    input.click();
  });
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn replay_files() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn replay_keep(name: &str, bytes: &[u8]) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    async fn replay_drop(name: &str) -> Result<(), JsValue>;
    fn replay_save(name: &str, bytes: &[u8]);
    async fn replay_pick() -> JsValue;
}

/// A name with its bytes out of what the page's side hands over.
fn named(pair: &JsValue) -> Option<(String, Vec<u8>)> {
    let pair = js_sys::Array::from(pair);
    Some((pair.get(0).as_string()?, js_sys::Uint8Array::new(&pair.get(1)).to_vec()))
}

/// The recordings the browser keeps, by name.
pub(super) async fn files() -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = match replay_files().await {
        Ok(files) => js_sys::Array::from(&files).iter().filter_map(|pair| named(&pair)).collect(),
        Err(_) => Vec::new(),
    };
    files.sort_by(|one, other| one.0.cmp(&other.0));
    files
}

/// Keeps a recording under that name; `false` when the browser would not.
pub(super) async fn keep(name: &str, bytes: &[u8]) -> bool {
    replay_keep(name, bytes).await.is_ok()
}

/// Throws the recording of that name away.
pub(super) async fn drop_file(name: &str) {
    let _ = replay_drop(name).await;
}

/// Hands a recording to the browser to save as a file.
pub(super) fn save(name: &str, bytes: &[u8]) {
    replay_save(name, bytes);
}

/// Lets the player pick a file of their machine: its name and its bytes, or none.
pub(super) async fn pick() -> Option<(String, Vec<u8>)> {
    let picked = replay_pick().await;
    if picked.is_null() || picked.is_undefined() { None } else { named(&picked) }
}
