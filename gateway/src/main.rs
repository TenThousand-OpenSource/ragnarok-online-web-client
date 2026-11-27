//! roweb gateway: the one native process a browser client needs beside rAthena.
//!
//! - `GET /assets/<client path>`: a file out of the client's GRF archives, read on demand.
//! - `GET /image/<client path>`: a picture out of the GRF, as a PNG the browser can show (see `images`).
//! - `GET /tables.json`, `/tables/…`: the client's tables for the browser (see `tables`); these three are the
//!   routes of `routes.rs`.
//! - `GET /bgm/<file>`: the client's loose BGM folder.
//! - `GET /ws/<port>`: a WebSocket bridged to a TCP connection to the game server (see `bridge`).
//! - `POST /api/register`, `/api/recover`, `/api/reset`: accounts and lost passwords (see `accounts`).
//! - `POST /party/<request>`: the Adventurer's Agency, handed on to the game's web service (see `agency`).
//! - everything else: the built web client (static files).
//!
//! Reached from outside (a tunnel, a proxy), it asks for an invitation word and counts visitors by their own
//! addresses (see `public`).
//!
//! Here: the start-up, and what is done instead of serving when the command line asks for it (`told`). The
//! options and the commands of the command line are in `config.rs`.

use roweb_gateway::{accounts, agency, bridge, grf, mapcache, public, tables};

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::http::header;
use tower_http::services::ServeDir;

mod config;
mod routes;

use config::{Command, Config};

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

/// Does what the gateway was asked on its command line instead of serving (`config::Command`).
fn told(grf: &grf::Grf, config: &Config, command: Command) {
    let run = |files: &str| tables::run(grf, &files.split(',').collect::<Vec<_>>());
    match command {
        Command::MapCache(path) => {
            let written = std::fs::File::create(&path).and_then(|file| mapcache::write(grf, std::io::BufWriter::new(file)));
            match written {
                Ok(count) => println!("{path}: {count} maps"),
                Err(error) => {
                    eprintln!("{path}: {error}");
                    std::process::exit(1);
                }
            }
        }
        Command::Tables => println!("{}", tables::for_browser(&tables::Sources::read(grf, &config.client_dir), &config.client_dir, &config.item_database, &config.attendance)),
        Command::ChatTabs => println!("{}", tables::chat_tabs(grf)),
        Command::Items(ids) => {
            let items = tables::Items::load(grf).with_server_names(&config.item_database).with_english(&config.client_dir);
            println!("{}", items.json(ids.split(',').filter_map(|id| id.parse().ok())));
        }
        Command::Dump { files, global } => println!("{}", tables::json_of(&run(&files).globals().get::<mlua::Value>(global).unwrap_or(mlua::Value::Nil))),
        Command::Lua(files) => tables::describe(&run(&files)).iter().for_each(|line| println!("{line}")),
        Command::Stats => {
            println!("{} files", grf.len());
            for (flags, count) in grf.flag_counts() {
                println!("flags {flags:#04x}: {count}");
            }
        }
        Command::List(words) => {
            let mut paths: Vec<_> = grf.paths().filter(|path| path.contains(&words.to_lowercase())).collect();
            paths.sort();
            paths.iter().for_each(|path| println!("{path}"));
        }
    }
}

/// The client itself, out of its folder. The page is asked for anew each time: a browser keeping an older build
/// would run it against a newer page. The files the page names by their build (`?v=…`) never change under that
/// name, so they may be kept, where the configuration lets them.
fn client(web_dir: &std::path::Path, cache_client: bool) -> Router {
    Router::new().fallback_service(
        ServeDir::new(web_dir)
    ).layer(axum::middleware::from_fn(move |request: axum::extract::Request, next: axum::middleware::Next| async move {
        let by_build = cache_client && request.uri().query().is_some_and(|query| query.starts_with("v="));
        let mut response = next.run(request).await;
        let keep = if by_build && response.status().is_success() { "public, max-age=31536000, immutable" } else { "no-cache" };
        response.headers_mut().insert(header::CACHE_CONTROL, axum::http::HeaderValue::from_static(keep));
        response
    }))
}

/// What stands before everything the gateway serves: who a visitor from outside is and whether they were
/// invited, and the compressing of what is sent.
fn guarded(router: Router, invite: Option<&str>, behind_proxy: bool, compress: bool) -> Router {
    // What is asked of a visitor from outside comes before everything else: first who they are, then whether
    // they were invited.
    let router = match invite {
        Some(word) => router.layer(axum::middleware::from_fn_with_state(Arc::new(public::Invite::new(word, behind_proxy)), public::invited)),
        None => router,
    };
    let router = if behind_proxy { router.layer(axum::middleware::from_fn(public::behind_proxy)) } else { router };
    // Compressed as it is sent, and quickly: the maps are megabytes of mostly the same bytes.
    if compress {
        router.layer(tower_http::compression::CompressionLayer::new().quality(tower_http::CompressionLevel::Fastest).compress_when(worth_compressing()))
    } else {
        router
    }
}

#[tokio::main]
async fn main() {
    let (config, command) = config::config().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    let grf = grf::Grf::open_client_dir(&config.client_dir).unwrap_or_else(|error| {
        eprintln!("{}: {error}", config.client_dir.display());
        std::process::exit(1);
    });

    if let Some(command) = command {
        return told(&grf, &config, command);
    }

    // The first port is the login server's.
    let login_guard = Arc::new(bridge::LoginGuard::new(config.server_ports.first().copied().unwrap_or(6900)));
    let items = tables::Items::load(&grf).with_server_names(&config.item_database).with_english(&config.client_dir).with_russian(
        std::path::Path::new("translations/ru")
    );
    eprintln!("tables: {} items", items.len());
    let accounts = accounts::router(config.accounts, accounts::Limits::default(), Some(login_guard.clone())).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    let client = client(&config.web_dir, config.cache_client);
    let router = routes::router(routes::App::new(grf, items, &config.client_dir, &config.item_database, &config.attendance).with_navigation(&config.navigation_dir))
        .merge(bridge::router(bridge::Bridge {
            host: config.server_host,
            ports: config.server_ports,
            login_guard: Some(login_guard.clone()),
            connections_per_minute: 60,
            lag: config.lag
        }))
        .merge(accounts)
        .merge(agency::router(config.web_server))
        .nest_service("/bgm", ServeDir::new(config.client_dir.join("BGM")))
        .merge(client);
    let router = guarded(router, config.invite.as_deref(), config.behind_proxy, config.compress);

    let listener = tokio::net::TcpListener::bind(config.listen).await.unwrap_or_else(|error| {
        eprintln!("{}: {error}", config.listen);
        std::process::exit(1);
    });
    eprintln!("gateway: http://{}", config.listen);
    if !config.lag.fixed.is_zero() || !config.lag.jitter.is_zero() {
        eprintln!(
            "gateway: DEBUG DELAY on everything to and from the game server: {} ms and up to {} ms more, each way",
            config.lag.fixed.as_millis(),
            config.lag.jitter.as_millis()
        );
    }
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
