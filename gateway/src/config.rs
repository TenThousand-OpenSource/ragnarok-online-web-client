//! What the gateway is started with: its options, each with what it is when not given, and the debug commands
//! of its command line.

use std::net::SocketAddr;
use std::path::PathBuf;

use roweb_gateway::{accounts, bridge};

/// The server's item database that goes with the rules it plays by: `server/.env`, written by `rules.sh`, says
/// which (`ROWEB_PRERE=no` is Renewal); without it, the classic one.
fn rules_database() -> PathBuf {
    let renewal = std::fs::read_to_string("server/.env").is_ok_and(|env| env.lines().any(|line| line.trim() == "ROWEB_PRERE=no"));
    if renewal { "../rathena/db/re" } else { "../rathena/db/pre-re" }.into()
}

pub struct Config {
    pub client_dir: PathBuf,
    pub web_dir: PathBuf,
    pub listen: SocketAddr,
    pub server_host: String,
    pub server_ports: Vec<u16>,
    pub accounts: accounts::Config,
    /// rAthena's item database folder, for the items' names.
    pub item_database: PathBuf,
    /// Whether what is sent is compressed for browsers that take it (`--compress on|off`). Off where something
    /// in front of the gateway (a proxy, a CDN) does that itself.
    pub compress: bool,
    /// Whether the client's own files, asked for by their build (`?v=…`), may be kept by the browser for good
    /// (`--cache-client on|off`). Off, they are asked for anew each time, as the page always is.
    pub cache_client: bool,
    /// The word a browser must give before anything is served to it (`--invite-file <file>`: the file holds
    /// the word, so that it is not in the list of running programs); none, and everyone is served.
    pub invite: Option<String>,
    /// Whether a proxy or a tunnel stands in front (`--behind-proxy on|off`): then a visitor's address is the
    /// one it reports, not its own.
    pub behind_proxy: bool,
    /// A debug delay on everything that goes between a browser and the game server, each way (`--lag
    /// <milliseconds>`, and `--jitter <milliseconds>` for a random part on top of it): for looking at the game
    /// as a player far away has it. None unless given. Never for the gateway others play through
    /// (`public.sh`): it delays everyone.
    pub lag: bridge::Lag,
    /// Where the game's web service is (`--web-server <host:port>`; rAthena's `web-server`, its `web_port`
    /// 8888): the Adventurer's Agency is asked of it (`agency.rs`).
    pub web_server: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_dir: "../Feb-Client-CLEAN".into(),
            web_dir: "client/dist".into(),
            listen: "127.0.0.1:8080".parse().unwrap(),
            server_host: "127.0.0.1".into(),
            server_ports: vec![6900, 6121, 5121],
            // The server is built for the classic game (see server/docker-compose.yml): its databases are these.
            item_database: rules_database(),
            compress: true,
            cache_client: true,
            invite: None,
            behind_proxy: false,
            lag: bridge::Lag::default(),
            web_server: "127.0.0.1:8888".into(),
            // The defaults are those of server/docker-compose.yml: its database, and its mailbox for development.
            accounts: accounts::Config {
                database_url: "mysql://ragnarok:ragnarok@127.0.0.1:3306/ragnarok".into(),
                smtp_url: "smtp://127.0.0.1:1025".into(),
                mail_from: "Ragnarok 2026 <no-reply@localhost>".into(),
                public_url: "http://localhost:8080".into(),
            },
        }
    }
}

/// What the gateway is asked on its command line to do instead of serving: the debug commands.
pub enum Command {
    /// `stats`: how many files the archives hold, and how many of each kind of storing.
    Stats,
    /// `ls <words>`: the files of the archives whose path has those words.
    List(String),
    /// `lua <file>,<file>`: runs datainfo files of the client and lists the tables they leave.
    Lua(String),
    /// `dump <file>,<file> <global>`: runs those files and prints that table of theirs as JSON.
    Dump { files: String, global: String },
    /// `mapcache <file>`: writes the server's map cache from the client's maps (for rAthena's db/import).
    MapCache(String),
    /// `items <id>,<id>`: prints what the item table says of those items.
    Items(String),
    /// `tables`: prints what the browser client is given of the client's tables.
    Tables,
    /// `chat-tabs`: prints the client's table of the chat window's tabs.
    ChatTabs,
}

/// The options of the command line over those of `Config::default`, and the command, if one was given (of
/// several, the last).
pub fn config() -> Result<(Config, Option<Command>), String> {
    let mut config = Config::default();
    let mut command = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--client-dir" => config.client_dir = value()?.into(),
            "--web-dir" => config.web_dir = value()?.into(),
            "--listen" => config.listen = value()?.parse().map_err(|error| format!("--listen: {error}"))?,
            "--server-host" => config.server_host = value()?,
            "--web-server" => config.web_server = value()?,
            "--server-ports" => {
                config.server_ports =
                    value()?.split(',').map(|port| port.trim().parse()).collect::<Result<_, _>>().map_err(|error| format!("--server-ports: {error}"))?;
            }
            "--item-database" => config.item_database = value()?.into(),
            "--compress" => config.compress = switch(&arg, &value()?)?,
            "--cache-client" => config.cache_client = switch(&arg, &value()?)?,
            "--behind-proxy" => config.behind_proxy = switch(&arg, &value()?)?,
            // Debug only, never on a gateway others play through: see `Config::lag`.
            "--lag" => config.lag.fixed = milliseconds(&arg, &value()?)?,
            "--jitter" => config.lag.jitter = milliseconds(&arg, &value()?)?,
            "--invite-file" => config.invite = Some(invitation(&value()?)?),
            "--database-url" => config.accounts.database_url = value()?,
            "--smtp-url" => config.accounts.smtp_url = value()?,
            "--mail-from" => config.accounts.mail_from = value()?,
            "--public-url" => config.accounts.public_url = value()?,
            "stats" => command = Some(Command::Stats),
            "ls" => command = Some(Command::List(value()?)),
            "lua" => command = Some(Command::Lua(value()?)),
            "dump" => command = Some(Command::Dump { files: value()?, global: value()? }),
            "mapcache" => command = Some(Command::MapCache(value()?)),
            "items" => command = Some(Command::Items(value()?)),
            "tables" => command = Some(Command::Tables),
            "chat-tabs" => command = Some(Command::ChatTabs),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    // The debug delay is for one's own machine: a gateway opened to others does not start with it.
    if (!config.lag.fixed.is_zero() || !config.lag.jitter.is_zero()) && (config.invite.is_some() || config.behind_proxy) {
        return Err("--lag and --jitter are debug options: not with --invite-file or --behind-proxy".to_owned());
    }
    Ok((config, command))
}

/// The invitation word a file holds (`--invite-file`).
fn invitation(file: &str) -> Result<String, String> {
    let word = std::fs::read_to_string(file).map_err(|error| format!("--invite-file {file}: {error}"))?;
    if word.trim().len() < 8 {
        return Err(format!("--invite-file {file}: the word must be at least 8 characters"));
    }
    Ok(word)
}

/// The value of an option that is a time in milliseconds.
fn milliseconds(option: &str, value: &str) -> Result<std::time::Duration, String> {
    value.parse().map(std::time::Duration::from_millis).map_err(|error| format!("{option}: {error}"))
}

/// The value of an option that is either on or off.
fn switch(option: &str, value: &str) -> Result<bool, String> {
    match value {
        "on" => Ok(true),
        "off" => Ok(false),
        other => Err(format!("{option}: on or off, not {other}")),
    }
}
