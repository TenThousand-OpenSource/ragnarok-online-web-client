//! The scripts that move a homunculus and a mercenary: the Lua files of the client's `AI` folder (`AI.lua`,
//! `AI_M.lua`, `Const.lua`, `Util.lua`, and the player's own under `USER_AI`), which lie beside the client,
//! not in its archives. The browser runs them as the Windows client does (`client/src/ai.rs`).

use std::path::Path;

/// No script is this large, and no folder of them: a larger file is left out, and reading stops at the whole.
const LARGEST: u64 = 1 << 20;
const WHOLE: usize = 8 << 20;

/// The Lua files of the client's `AI` folder as JSON: `{"ai/ai.lua": "…", "ai/user_ai/ai.lua": "…"}`, each
/// path in small letters with `/` between, each text with a character for each byte of the file (the files
/// are Korean in the client's own encoding, and the script runs on their bytes). Without `user`, only the
/// files right in the folder: the client's own scripts.
pub fn scripts(client_dir: &Path, user: bool) -> String {
    let mut files = serde_json::Map::new();
    let mut read = 0;
    gather(&client_dir.join("AI"), "ai", user, &mut files, &mut read);
    serde_json::Value::Object(files).to_string()
}

fn gather(folder: &Path, under: &str, deeper: bool, files: &mut serde_json::Map<String, serde_json::Value>, read: &mut usize) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        let Ok(kind) = entry.file_type() else { continue };
        if kind.is_dir() && deeper {
            gather(&entry.path(), &format!("{under}/{name}"), deeper, files, read);
        } else if kind.is_file() && name.ends_with(".lua") && entry.metadata().is_ok_and(|about| about.len() <= LARGEST) {
            let Ok(bytes) = std::fs::read(entry.path()) else { continue };
            *read += bytes.len();
            if *read > WHOLE {
                return;
            }
            files.insert(format!("{under}/{name}"), serde_json::Value::String(bytes.iter().map(|byte| char::from(*byte)).collect()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scripts;

    #[test]
    fn gives_the_scripts_of_the_folder_byte_for_byte() {
        let folder = std::env::temp_dir().join(format!("roweb-ai-{}", std::process::id()));
        std::fs::create_dir_all(folder.join("AI/USER_AI")).unwrap();
        std::fs::write(folder.join("AI/AI.lua"), b"-- \xc8\xa3\nfunction AI(id) end").unwrap();
        std::fs::write(folder.join("AI/notes.htm"), b"not a script").unwrap();
        std::fs::write(folder.join("AI/USER_AI/Mine.lua"), b"x = 1").unwrap();
        let own: serde_json::Value = serde_json::from_str(&scripts(&folder, false)).unwrap();
        let all: serde_json::Value = serde_json::from_str(&scripts(&folder, true)).unwrap();
        std::fs::remove_dir_all(&folder).unwrap();
        assert_eq!(own.as_object().unwrap().len(), 1);
        let text: Vec<u8> = own["ai/ai.lua"].as_str().unwrap().chars().map(|byte| byte as u8).collect();
        assert_eq!(text, b"-- \xc8\xa3\nfunction AI(id) end");
        assert_eq!((all.as_object().unwrap().len(), all["ai/user_ai/mine.lua"].as_str()), (2, Some("x = 1")));
        // A client without the folder has no scripts.
        assert_eq!(scripts(&folder, true), "{}");
    }
}
