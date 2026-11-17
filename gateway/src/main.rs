//! roweb gateway: the one native process a browser client needs beside rAthena.
//!
//! - `GET /assets/<client path>`: a file out of the client's GRF archives, read on demand.
//! - `GET /image/<client path>`: a picture out of the GRF, as a PNG the browser can show (see `images`).
//! - `GET /bgm/<file>`: the client's loose BGM folder.
//! - `GET /ws/<port>`: a WebSocket bridged to a TCP connection to the game server (see `bridge`).
//! - `POST /api/register`, `/api/recover`, `/api/reset`: accounts and lost passwords (see `accounts`).
//! - everything else: the built web client (static files).

use roweb_gateway::{accounts, bridge, grf, images, mapcache, tables};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use tower_http::services::ServeDir;

struct Config {
    client_dir: PathBuf,
    web_dir: PathBuf,
    listen: SocketAddr,
    server_host: String,
    server_ports: Vec<u16>,
    accounts: accounts::Config,
    /// rAthena's item database folder, for the items' names.
    item_database: PathBuf,
    /// Whether what is sent is compressed for browsers that take it (`--compress on|off`). Off where something
    /// in front of the gateway (a proxy, a CDN) does that itself.
    compress: bool,
    /// Whether the client's own files, asked for by their build (`?v=…`), may be kept by the browser for good
    /// (`--cache-client on|off`). Off, they are asked for anew each time, as the page always is.
    cache_client: bool,
}

struct App {
    grf: grf::Grf,
    /// The client's tables for the browser, as JSON.
    tables: String,
    items: tables::Items,
    /// What the client says of each skill, by skill ID.
    skill_texts: std::collections::HashMap<u32, Vec<String>>,
    /// The skills' names and descriptions in Russian, from the translation files.
    skills_russian: tables::Translated,
}

fn config() -> Result<(Config, Option<String>), String> {
    let mut config = Config {
        client_dir: "../Feb-Client-CLEAN".into(),
        web_dir: "client/dist".into(),
        listen: "127.0.0.1:8080".parse().unwrap(),
        server_host: "127.0.0.1".into(),
        server_ports: vec![6900, 6121, 5121],
        item_database: "../rathena/db/re".into(),
        compress: true,
        cache_client: true,
        // The defaults are those of server/docker-compose.yml: its database, and its mailbox for development.
        accounts: accounts::Config {
            database_url: "mysql://ragnarok:ragnarok@127.0.0.1:3306/ragnarok".into(),
            smtp_url: "smtp://127.0.0.1:1025".into(),
            mail_from: "Ragnarok 2026 <no-reply@localhost>".into(),
            public_url: "http://localhost:8080".into(),
        },
    };
    let mut command = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--client-dir" => config.client_dir = value()?.into(),
            "--web-dir" => config.web_dir = value()?.into(),
            "--listen" => config.listen = value()?.parse().map_err(|error| format!("--listen: {error}"))?,
            "--server-host" => config.server_host = value()?,
            "--server-ports" => {
                config.server_ports =
                    value()?.split(',').map(|port| port.trim().parse()).collect::<Result<_, _>>().map_err(|error| format!("--server-ports: {error}"))?;
            }
            "--item-database" => config.item_database = value()?.into(),
            "--compress" => config.compress = switch(&arg, &value()?)?,
            "--cache-client" => config.cache_client = switch(&arg, &value()?)?,
            "--database-url" => config.accounts.database_url = value()?,
            "--smtp-url" => config.accounts.smtp_url = value()?,
            "--mail-from" => config.accounts.mail_from = value()?,
            "--public-url" => config.accounts.public_url = value()?,
            // Debug commands: `stats`, `ls <substring>`.
            "stats" => command = Some(String::new()),
            "ls" => command = Some(value()?),
            // `lua <file>,<file>`: runs datainfo files of the client and lists the tables they leave.
            "lua" => command = Some(format!("lua:{}", value()?)),
            // `dump <file>,<file> <global>`: runs those files and prints that table of theirs as JSON.
            "dump" => command = Some(format!("dump:{}:{}", value()?, value()?)),
            // `mapcache <file>`: writes the server's map cache from the client's maps (for rAthena's db/import).
            "mapcache" => command = Some(format!("mapcache:{}", value()?)),
            // `items <id>,<id>`: prints what the item table says of those items.
            "items" => command = Some(format!("items:{}", value()?)),
            // `tables`: prints what the browser client is given of the client's tables.
            "tables" => command = Some("tables:".to_owned()),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    Ok((config, command))
}

/// The value of an option that is either on or off.
fn switch(option: &str, value: &str) -> Result<bool, String> {
    match value {
        "on" => Ok(true),
        "off" => Ok(false),
        other => Err(format!("{option}: on or off, not {other}")),
    }
}

/// What of the gateway's answers is worth compressing: not what is small, and not what is compressed already
/// (PNG and JPEG pictures, music). Maps, tables, the client itself and the client's BMP pictures are.
fn worth_compressing() -> impl tower_http::compression::Predicate {
    use tower_http::compression::predicate::{NotForContentType, Predicate, SizeAbove};
    SizeAbove::new(1024)
        .and(NotForContentType::const_new("image/png"))
        .and(NotForContentType::const_new("image/jpeg"))
        .and(NotForContentType::const_new("audio/"))
        .and(NotForContentType::GRPC)
        .and(NotForContentType::SSE)
}

#[tokio::main]
async fn main() {
    let (config, command) = config().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    let grf = grf::Grf::open_client_dir(&config.client_dir).unwrap_or_else(|error| {
        eprintln!("{}: {error}", config.client_dir.display());
        std::process::exit(1);
    });

    if let Some(filter) = command {
        if let Some(path) = filter.strip_prefix("mapcache:") {
            let written = std::fs::File::create(path).and_then(|file| mapcache::write(&grf, std::io::BufWriter::new(file)));
            match written {
                Ok(count) => println!("{path}: {count} maps"),
                Err(error) => {
                    eprintln!("{path}: {error}");
                    std::process::exit(1);
                }
            }
        } else if filter == "tables:" {
            println!("{}", tables::with_guide(tables::with_maps(tables::for_client(&grf), &config.client_dir), &grf, &config.item_database, &config.client_dir));
        } else if let Some(ids) = filter.strip_prefix("items:") {
            println!("{}", tables::Items::load(&grf).with_server_names(&config.item_database).with_english(&config.client_dir).json(ids.split(',').filter_map(|id| id.parse().ok())));
        } else if let Some((files, global)) = filter.strip_prefix("dump:").and_then(|dump| dump.rsplit_once(':')) {
            let lua = tables::run(&grf, &files.split(',').collect::<Vec<_>>());
            println!("{}", tables::json_of(&lua.globals().get::<mlua::Value>(global).unwrap_or(mlua::Value::Nil)));
        } else if let Some(files) = filter.strip_prefix("lua:") {
            let lua = tables::run(&grf, &files.split(',').collect::<Vec<_>>());
            tables::describe(&lua).iter().for_each(|line| println!("{line}"));
        } else if filter.is_empty() {
            println!("{} files", grf.len());
            for (flags, count) in grf.flag_counts() {
                println!("flags {flags:#04x}: {count}");
            }
        } else {
            let mut paths: Vec<_> = grf.paths().filter(|path| path.contains(&filter.to_lowercase())).collect();
            paths.sort();
            paths.iter().for_each(|path| println!("{path}"));
        }
        return;
    }

    // The first port is the login server's.
    let login_guard = Arc::new(bridge::LoginGuard::new(config.server_ports.first().copied().unwrap_or(6900)));
    let items = tables::Items::load(&grf).with_server_names(&config.item_database).with_english(&config.client_dir).with_russian(std::path::Path::new("translations/ru"));
    eprintln!("tables: {} items", items.len());
    let accounts = accounts::router(config.accounts, accounts::Limits::default(), Some(login_guard.clone())).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    // The client itself. The page is asked for anew each time: a browser keeping an older build would run it
    // against a newer page. The files the page names by their build (`?v=…`) never change under that name, so
    // they may be kept, where the configuration lets them.
    let cache_client = config.cache_client;
    let client = Router::new().fallback_service(ServeDir::new(&config.web_dir)).layer(axum::middleware::from_fn(move |request: axum::extract::Request, next: axum::middleware::Next| async move {
        let by_build = cache_client && request.uri().query().is_some_and(|query| query.starts_with("v="));
        let mut response = next.run(request).await;
        let keep = if by_build && response.status().is_success() { "public, max-age=31536000, immutable" } else { "no-cache" };
        response.headers_mut().insert(header::CACHE_CONTROL, axum::http::HeaderValue::from_static(keep));
        response
    }));
    let router = Router::new()
        .route("/assets/{*path}", get(asset))
        .route("/image/{*path}", get(image))
        .route("/tables.json", get(client_tables))
        .route("/tables/items/{ids}", get(client_items))
        .route("/tables/skills/{ids}", get(client_skills))
        .route("/tables/ru/items/{ids}", get(client_items_russian))
        .route("/tables/ru/skills/{ids}", get(client_skills_russian))
        .route("/tables/ru/skill-names.json", get(client_skill_names_russian))
        .route("/tables/items.ko/{ids}", get(client_items_korean))
        .with_state(Arc::new(App { tables: tables::with_guide(tables::with_maps(tables::for_client(&grf), &config.client_dir), &grf, &config.item_database, &config.client_dir).to_string(), items, skill_texts: tables::skill_texts(&grf), skills_russian: tables::translated(std::path::Path::new("translations/ru"), "skills"), grf }))
        .merge(bridge::router(bridge::Bridge { host: config.server_host, ports: config.server_ports, login_guard: Some(login_guard.clone()), connections_per_minute: 60 }))
        .merge(accounts)
        .nest_service("/bgm", ServeDir::new(config.client_dir.join("BGM")))
        .merge(client);
    // Compressed as it is sent, and quickly: the maps are megabytes of mostly the same bytes.
    let router = if config.compress {
        router.layer(tower_http::compression::CompressionLayer::new().quality(tower_http::CompressionLevel::Fastest).compress_when(worth_compressing()))
    } else {
        router
    };

    let listener = tokio::net::TcpListener::bind(config.listen).await.unwrap_or_else(|error| {
        eprintln!("{}: {error}", config.listen);
        std::process::exit(1);
    });
    eprintln!("gateway: http://{}", config.listen);
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
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
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

async fn image(State(app): State<Arc<App>>, Path(path): Path<String>) -> Response {
    let converted = tokio::task::spawn_blocking(move || app.grf.read(&path).map(|bytes| images::to_png(&path, &bytes))).await.unwrap();
    match converted {
        Ok(Ok(png)) => ([(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "public, max-age=86400")], png).into_response(),
        Ok(Err(error)) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, error).into_response(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

async fn client_tables(State(app): State<Arc<App>>) -> Response {
    // Asked for anew every time: the tables change with the gateway, and a stale copy would hide what was added.
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], app.tables.clone()).into_response()
}

/// `GET /tables/items.ko/<id>,<id>,…`: the Korean names and descriptions of those items (the client's own table),
/// kept apart from the English shown.
async fn client_items_korean(State(app): State<Arc<App>>, Path(ids): Path<String>) -> Response {
    let ids = ids.split(',').take(400).filter_map(|id| id.parse().ok());
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], app.items.korean_json(ids).to_string()).into_response()
}

/// `GET /tables/ru/items/<id>,<id>,…`: those items in Russian (see `Items::russian_json`).
async fn client_items_russian(State(app): State<Arc<App>>, Path(ids): Path<String>) -> Response {
    let ids = ids.split(',').take(400).filter_map(|id| id.parse().ok());
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], app.items.russian_json(ids).to_string()).into_response()
}

/// `GET /tables/ru/skills/<id>,…`: the Russian descriptions of those skills; a skill with none gets the client's.
async fn client_skills_russian(State(app): State<Arc<App>>, Path(ids): Path<String>) -> Response {
    let found: serde_json::Map<String, serde_json::Value> = ids
        .split(',')
        .take(100)
        .filter_map(|id| id.parse::<u32>().ok())
        .filter_map(|id| app.skills_russian.get(&id).map(|(_, lines)| lines).filter(|lines| !lines.is_empty()).or_else(|| app.skill_texts.get(&id)).map(|lines| (id.to_string(), serde_json::json!(lines))))
        .collect();
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], serde_json::Value::from(found).to_string()).into_response()
}

/// `GET /tables/ru/skill-names.json`: what the translated skills are called in Russian, by ID.
async fn client_skill_names_russian(State(app): State<Arc<App>>) -> Response {
    let names: serde_json::Map<String, serde_json::Value> = app.skills_russian.iter().filter(|(_, (name, _))| !name.is_empty()).map(|(id, (name, _))| (id.to_string(), name.as_str().into())).collect();
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], serde_json::Value::from(names).to_string()).into_response()
}

/// `GET /tables/skills/<id>,<id>,…`: the client's descriptions of those skills, line by line.
async fn client_skills(State(app): State<Arc<App>>, Path(ids): Path<String>) -> Response {
    let found: serde_json::Map<String, serde_json::Value> =
        ids.split(',').take(100).filter_map(|id| id.parse::<u32>().ok()).filter_map(|id| app.skill_texts.get(&id).map(|lines| (id.to_string(), serde_json::json!(lines)))).collect();
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], serde_json::Value::from(found).to_string()).into_response()
}

/// `GET /tables/items/<id>,<id>,…`: the names and pictures of those items.
async fn client_items(State(app): State<Arc<App>>, Path(ids): Path<String>) -> Response {
    // At most a few hundred at once: an inventory holds a hundred.
    let ids = ids.split(',').take(400).filter_map(|id| id.parse().ok());
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], app.items.json(ids).to_string()).into_response()
}
