//! What the gateway answers out of the client's files: a file of its archives as it is or as a PNG, and the
//! tables for the browser, whole (`/tables.json`) or asked for by the numbers of what is wanted.

use std::collections::HashMap;
use std::path::{Path as FilePath, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use roweb_gateway::{grf, images, tables};

/// What those answers are made of, read once as the gateway starts.
pub struct App {
    grf: grf::Grf,
    /// The client's tables for the browser, as JSON.
    tables: String,
    items: tables::Items,
    /// What the client says of each skill, by skill ID.
    skill_texts: HashMap<u32, Vec<String>>,
    /// The skills' names and descriptions in Russian, from the translation files.
    skills_russian: tables::Translated,
    /// The client's folder: the scripts of a homunculus and a mercenary are read from beside its archives.
    client_dir: PathBuf,
}

impl App {
    /// Reads the tables out of the archives, the client's folder and the server's database; the items are
    /// given read (the gateway says how many it has before anything else starts).
    pub fn new(grf: grf::Grf, items: tables::Items, client_dir: &FilePath, item_database: &FilePath) -> Self {
        // What the tables and the skills' texts both read is read once (`tables::Sources`).
        let read = tables::Sources::read(&grf, client_dir);
        let (tables, skill_texts) = (tables::for_browser(&read, client_dir, item_database).to_string(), tables::skill_texts(&read));
        drop(read);
        Self {
            tables,
            items,
            skill_texts,
            skills_russian: tables::translated(FilePath::new("translations/ru"), "skills"),
            client_dir: client_dir.to_owned(),
            grf,
        }
    }
}

/// The routes of this file.
pub fn router(app: App) -> Router {
    Router::new()
        .route("/assets/{*path}", get(asset))
        .route("/image/{*path}", get(image))
        .route("/tables.json", get(client_tables))
        .route("/tables/items/{ids}", get(client_items))
        .route("/tables/skills/{ids}", get(client_skills))
        .route("/tables/ru/items/{ids}", get(client_items_russian))
        .route("/tables/ru/skills/{ids}", get(client_skills_russian))
        .route("/tables/ru/skill-names.json", get(client_skill_names_russian))
        .route("/tables/items.ko/{ids}", get(client_items_korean))
        .route("/tables/ai.json", get(client_ai))
        .route("/tables/ai-user.json", get(client_ai_user))
        .with_state(Arc::new(app))
}

/// An answer that is JSON. Asked for anew every time: the tables change with the gateway and the scripts with
/// the player, and a stale copy would hide what was added.
fn json(body: String) -> Response {
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], body).into_response()
}

/// The numbers of a path's `<id>,<id>,…`, of at most `most` of its entries; what is no number is passed by.
fn ids(list: &str, most: usize) -> impl Iterator<Item = u32> + '_ {
    list.split(',').take(most).filter_map(|id| id.parse().ok())
}

/// The answer for a file of the archives that could not be read: not there, or why not.
fn unread(error: std::io::Error) -> Response {
    if error.kind() == std::io::ErrorKind::NotFound {
        StatusCode::NOT_FOUND.into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
    }
}

async fn asset(State(app): State<Arc<App>>, Path(path): Path<String>) -> Response {
    // Pictures the browser can show by itself say what they are.
    let lower = path.to_ascii_lowercase();
    let kind = if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".wav") {
        "audio/wav"
    } else if lower.ends_with(".png") {
        "image/png"
    } else {
        "application/octet-stream"
    };
    let read = tokio::task::spawn_blocking(move || app.grf.read(&path)).await.unwrap();
    match read {
        // Game data never changes under a path while the gateway runs, so the browser may keep it.
        Ok(bytes) => ([(header::CONTENT_TYPE, kind), (header::CACHE_CONTROL, "public, max-age=86400")], bytes).into_response(),
        Err(error) => unread(error),
    }
}

async fn image(State(app): State<Arc<App>>, Path(path): Path<String>) -> Response {
    let converted = tokio::task::spawn_blocking(move || app.grf.read(&path).map(|bytes| images::to_png(&path, &bytes))).await.unwrap();
    match converted {
        Ok(Ok(png)) => ([(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "public, max-age=86400")], png).into_response(),
        Ok(Err(error)) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, error).into_response(),
        Err(error) => unread(error),
    }
}

/// `GET /tables/ai.json`: the client's own scripts for a homunculus and a mercenary (`tables/ai.rs`), read as
/// they are asked for: a player changes them while the gateway runs.
async fn client_ai(State(app): State<Arc<App>>) -> Response {
    json(tokio::task::spawn_blocking(move || tables::ai_scripts(&app.client_dir, false)).await.unwrap())
}

/// `GET /tables/ai-user.json`: those and the player's own under `USER_AI`.
async fn client_ai_user(State(app): State<Arc<App>>) -> Response {
    json(tokio::task::spawn_blocking(move || tables::ai_scripts(&app.client_dir, true)).await.unwrap())
}

/// `GET /tables.json`: everything the browser is given at once (`tables::for_browser`).
async fn client_tables(State(app): State<Arc<App>>) -> Response {
    json(app.tables.clone())
}

/// `GET /tables/items.ko/<id>,<id>,…`: the Korean names and descriptions of those items (the client's own table),
/// kept apart from the English shown.
async fn client_items_korean(State(app): State<Arc<App>>, Path(list): Path<String>) -> Response {
    json(app.items.korean_json(ids(&list, 400)).to_string())
}

/// `GET /tables/ru/items/<id>,<id>,…`: those items in Russian (see `Items::russian_json`).
async fn client_items_russian(State(app): State<Arc<App>>, Path(list): Path<String>) -> Response {
    json(app.items.russian_json(ids(&list, 400)).to_string())
}

/// `GET /tables/items/<id>,<id>,…`: the names and pictures of those items.
async fn client_items(State(app): State<Arc<App>>, Path(list): Path<String>) -> Response {
    // At most a few hundred at once: an inventory holds a hundred.
    json(app.items.json(ids(&list, 400)).to_string())
}

/// The descriptions of the skills a path names (at most a hundred of them), line by line and by ID, each as
/// `lines` gives it; a skill it gives nothing for is left out.
fn skills_described<'app>(list: &str, lines: impl Fn(u32) -> Option<&'app Vec<String>>) -> Response {
    let found: serde_json::Map<String, serde_json::Value> =
        ids(list, 100).filter_map(|id| lines(id).map(|lines| (id.to_string(), serde_json::json!(lines)))).collect();
    json(serde_json::Value::from(found).to_string())
}

/// `GET /tables/skills/<id>,<id>,…`: the client's descriptions of those skills, line by line.
async fn client_skills(State(app): State<Arc<App>>, Path(list): Path<String>) -> Response {
    skills_described(&list, |id| app.skill_texts.get(&id))
}

/// `GET /tables/ru/skills/<id>,…`: the Russian descriptions of those skills; a skill with none gets the client's.
async fn client_skills_russian(State(app): State<Arc<App>>, Path(list): Path<String>) -> Response {
    skills_described(&list, |id| app.skills_russian.get(&id).map(|(_, lines)| lines).filter(|lines| !lines.is_empty()).or_else(|| app.skill_texts.get(&id)))
}

/// `GET /tables/ru/skill-names.json`: what the translated skills are called in Russian, by ID.
async fn client_skill_names_russian(State(app): State<Arc<App>>) -> Response {
    let names: serde_json::Map<String, serde_json::Value> = app.skills_russian
        .iter()
        .filter(|(_, (name, _))| !name.is_empty())
        .map(|(id, (name, _))| (id.to_string(), name.as_str().into()))
        .collect();
    json(serde_json::Value::from(names).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_numbers_of_a_path_are_read_up_to_so_many_entries() {
        assert_eq!(ids("501,x,1101,,4001", 400).collect::<Vec<_>>(), [501, 1101, 4001]);
        // What is no number counts among the entries taken.
        assert_eq!(ids("1,x,2,3", 3).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(ids("", 3).count(), 0);
    }

    #[test]
    fn json_is_never_kept_by_the_browser() {
        let answer = json("{}".to_owned());
        assert_eq!(answer.status(), StatusCode::OK);
        assert_eq!(answer.headers()[header::CONTENT_TYPE], "application/json");
        assert_eq!(answer.headers()[header::CACHE_CONTROL], "no-cache");
    }

    #[test]
    fn a_file_that_is_not_there_is_told_from_one_that_cannot_be_read() {
        assert_eq!(unread(std::io::Error::from(std::io::ErrorKind::NotFound)).status(), StatusCode::NOT_FOUND);
        assert_eq!(unread(std::io::Error::other("broken")).status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
