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
}

struct App {
    grf: grf::Grf,
    /// The client's tables for the browser, as JSON.
    tables: String,
    items: tables::Items,
}

fn config() -> Result<(Config, Option<String>), String> {
    let mut config = Config {
        client_dir: "../Feb-Client-CLEAN".into(),
        web_dir: "client/dist".into(),
        listen: "127.0.0.1:8080".parse().unwrap(),
        server_host: "127.0.0.1".into(),
        server_ports: vec![6900, 6121, 5121],
        item_database: "../rathena/db/re".into(),
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
            "--database-url" => config.accounts.database_url = value()?,
            "--smtp-url" => config.accounts.smtp_url = value()?,
            "--mail-from" => config.accounts.mail_from = value()?,
            "--public-url" => config.accounts.public_url = value()?,
            // Debug commands: `stats`, `ls <substring>`.
            "stats" => command = Some(String::new()),
            "ls" => command = Some(value()?),
            // `lua <file>,<file>`: runs datainfo files of the client and lists the tables they leave.
            "lua" => command = Some(format!("lua:{}", value()?)),
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
            println!("{}", tables::for_client(&grf));
        } else if let Some(ids) = filter.strip_prefix("items:") {
            println!("{}", tables::Items::load(&grf).with_server_names(&config.item_database).json(ids.split(',').filter_map(|id| id.parse().ok())));
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
    let items = tables::Items::load(&grf).with_server_names(&config.item_database);
    eprintln!("tables: {} items", items.len());
    let accounts = accounts::router(config.accounts, accounts::Limits::default(), Some(login_guard.clone())).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    let router = Router::new()
        .route("/assets/{*path}", get(asset))
        .route("/image/{*path}", get(image))
        .route("/tables.json", get(client_tables))
        .route("/tables/items/{ids}", get(client_items))
        .with_state(Arc::new(App { tables: tables::for_client(&grf).to_string(), items, grf }))
        .merge(bridge::router(bridge::Bridge { host: config.server_host, ports: config.server_ports, login_guard: Some(login_guard.clone()), connections_per_minute: 60 }))
        .merge(accounts)
        .nest_service("/bgm", ServeDir::new(config.client_dir.join("BGM")))
        .fallback_service(ServeDir::new(&config.web_dir));

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

/// `GET /tables/items/<id>,<id>,…`: the names and pictures of those items.
async fn client_items(State(app): State<Arc<App>>, Path(ids): Path<String>) -> Response {
    // At most a few hundred at once: an inventory holds a hundred.
    let ids = ids.split(',').take(400).filter_map(|id| id.parse().ok());
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-cache")], app.items.json(ids).to_string()).into_response()
}
