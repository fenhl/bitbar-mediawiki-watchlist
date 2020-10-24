#![deny(rust_2018_idioms, unused, unused_import_braces, unused_qualifications, warnings)]

use {
    std::{
        collections::BTreeMap,
        convert::Infallible as Never,
        env::{
            self,
            current_exe,
        },
        fmt,
        fs::File,
        io,
        process::Command,
        thread,
        time::Duration,
    },
    bitbar::{
        ContentItem,
        Menu,
        MenuItem,
    },
    derive_more::From,
    futures::prelude::*,
    itertools::Itertools,
    maplit::{
        convert_args,
        hashmap,
    },
    notify_rust::Notification,
    serde::Deserialize,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigWiki {
    display_name: String,
    api_url: String,
    index_url: String,
    username: String,
    watchlist_token: String
}

#[derive(Deserialize)]
struct Config {
    wikis: Vec<ConfigWiki>
}

impl Config {
    fn new() -> Result<Config, Error> {
        let dirs = xdg_basedir::get_config_home().into_iter().chain(xdg_basedir::get_config_dirs());
        let file = dirs.filter_map(|cfg_dir| File::open(cfg_dir.join("bitbar/plugins/mediawiki-watchlist.json")).ok())
            .next().ok_or(Error::MissingConfig)?;
        Ok(serde_json::from_reader(file).map_err(Error::ConfigFormat)?)
    }
}

#[derive(Debug)]
enum OpenAllError {
    MissingDisplayName,
    UnknownWiki(String)
}

#[derive(Debug, From)]
enum Error {
    #[from(ignore)]
    ConfigFormat(serde_json::Error),
    Fmt(fmt::Error),
    Io(io::Error),
    MissingConfig,
    OpenAll(OpenAllError),
    Other(Box<dyn std::error::Error>),
    UrlParse(url::ParseError),
    #[from(ignore)]
    WatchlistFormatInner(ConfigWiki, serde_json::Error),
    WatchlistFormatOuter(ConfigWiki, serde_json::Value),
}

impl From<Never> for Error {
    fn from(never: Never) -> Error {
        match never {}
    }
}

#[derive(Clone, Deserialize)]
struct WatchlistItem {
    old_revid: u64,
    //revid: u64,
    pageid: u64,
    title: String
}

trait ResultNeverExt<T> {
    fn never_unwrap(self) -> T;
}

impl<T> ResultNeverExt<T> for Result<T, Never> {
    fn never_unwrap(self) -> T {
        match self {
            Ok(inner) => inner,
            Err(never) => match never {}
        }
    }
}

async fn get_watchlist(wiki_config: &ConfigWiki) -> Result<BTreeMap<u64, WatchlistItem>, Error> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static(concat!("bitbar-mediawiki-watchlist/", env!("CARGO_PKG_VERSION"))));
    let client_builder = reqwest::Client::builder().default_headers(headers);
    let api = mediawiki::api::Api::new_from_builder(&wiki_config.api_url, client_builder).await?;
    let mut json = api.get_query_api_json_all(&convert_args!(hashmap!(
        "action" => "query",
        "list" => "watchlist",
        "wlallrev" => "1",
        "wldir" => "newer",
        "wllimit" => "max",
        "wlshow" => "unread",
        "wlowner" => &wiki_config.username[..],
        "wltoken" => &wiki_config.watchlist_token[..]
    ))).await?;
    let watchlist = serde_json::from_value::<Vec<WatchlistItem>>(
        json.pointer_mut("/query/watchlist")
            .map(serde_json::Value::take)
            .ok_or_else(|| Error::WatchlistFormatOuter(wiki_config.clone(), json.clone()))?
    ).map_err(|e| Error::WatchlistFormatInner(wiki_config.clone(), e))?;
    let mut filtered_watchlist = BTreeMap::default();
    for watchlist_item in watchlist {
        // only show the oldest unread event of each page
        if filtered_watchlist.entry(watchlist_item.pageid).or_insert_with(|| watchlist_item.clone()).old_revid > watchlist_item.old_revid {
            filtered_watchlist.insert(watchlist_item.pageid, watchlist_item);
        }
    }
    Ok(filtered_watchlist)
}

async fn bitbar() -> Result<Menu, Error> {
    let config = Config::new()?;
    let watchlists = stream::iter(&config.wikis).then(get_watchlist).try_collect::<Vec<_>>().await?;
    let mut items = Vec::default();
    let total = watchlists.iter().map(BTreeMap::len).sum::<usize>();
    if total > 0 {
        items.push(ContentItem::new(total).template_image(&include_bytes!("../assets/tournesol.png")[..])?.into());
        for (watchlist, wiki_config) in watchlists.into_iter().zip(config.wikis) {
            if !watchlist.is_empty() {
                items.push(MenuItem::Sep);
                if let Ok(exe_path) = current_exe() {
                    items.push(ContentItem::new(&wiki_config.display_name)
                        .command((exe_path.display(), "open-all", wiki_config.display_name))
                        .refresh()
                        .into()
                    );
                } else {
                    items.push(MenuItem::new(wiki_config.display_name));
                }
                for (_, watchlist_item) in watchlist {
                    items.push(ContentItem::new(watchlist_item.title)
                        .href(format!("{}?{}&diff=next&oldid={}", wiki_config.index_url, watchlist_item.pageid, watchlist_item.old_revid))? //TODO use Url::query_pairs_mut
                        .into()
                    );
                }
            }
        }
    }
    Ok(Menu(items))
}

fn notify(summary: impl fmt::Display, body: impl fmt::Display) -> ! {
    //let _ = notify_rust::set_application(&notify_rust::get_bundle_identifier_or_default("BitBar")); //TODO uncomment when https://github.com/h4llow3En/mac-notification-sys/issues/8 is fixed
    let _ = Notification::default()
        .summary(&summary.to_string())
        .sound_name("Funk")
        .body(&body.to_string())
        .show();
    panic!("{}: {}", summary, body);
}

trait ResultExt {
    type Ok;

    fn notify(self, summary: impl fmt::Display) -> Self::Ok;
}

impl<T, E: fmt::Debug> ResultExt for Result<T, E> {
    type Ok = T;

    fn notify(self, summary: impl fmt::Display) -> T {
        match self {
            Ok(t) => t,
            Err(e) => { notify(summary, format!("{:?}", e)); }
        }
    }
}

async fn open_all(args: env::Args) -> Result<(), Error> {
    let (display_name,) = args.collect_tuple().ok_or(OpenAllError::MissingDisplayName)?;
    let (wiki_config,) = Config::new()?.wikis.into_iter().filter(|wiki_config| wiki_config.display_name == display_name).collect_tuple().ok_or(OpenAllError::UnknownWiki(display_name.to_string()))?;
    let watchlist = get_watchlist(&mut &wiki_config).await?;
    let processes = watchlist.into_iter().map(|(_, watchlist_item)|
            Command::new("open")
                .arg(format!("{}?pageid={}&diff=next&oldid={}", wiki_config.index_url, watchlist_item.pageid, watchlist_item.old_revid))
                .spawn()
        )
        .collect::<Result<Vec<_>, _>>()?;
    for mut process in processes {
        process.wait()?;
    }
    thread::sleep(Duration::from_secs(2)); // wait for 2 seconds to allow for marking pages as visited before letting BitBar refresh the plugin
    Ok(())
}

#[wheel::main]
async fn main() -> Result<(), Never> {
    let mut args = env::args();
    let _ = args.next(); // ignore executable name
    if let Some(arg) = args.next() {
        match &arg[..] {
            "open-all" => { open_all(args).await.notify("error in open-all cmd"); }
            subcmd => { notify("unknown subcommand", subcmd); }
        }
    } else {
        match bitbar().await {
            Ok(menu) => { print!("{}", menu); }
            Err(e) => {
                let mut error_menu = vec![
                    ContentItem::new("?").template_image(&include_bytes!("../assets/tournesol.png")[..]).never_unwrap().into(),
                    MenuItem::Sep
                ];
                match e {
                    Error::ConfigFormat(e) => { error_menu.push(MenuItem::new(format!("error in config file: {}", e))); }
                    Error::Fmt(e) => { error_menu.push(MenuItem::new(format!("formatting error: {}", e))); }
                    Error::Io(e) => { error_menu.push(MenuItem::new(format!("I/O error: {}", e))); }
                    Error::MissingConfig => { error_menu.push(MenuItem::new("missing or invalid configuration file")); } //TODO better error message
                    Error::OpenAll(_) => unreachable!(),
                    Error::Other(e) => {
                        error_menu.push(MenuItem::new(&e));
                        error_menu.push(MenuItem::new(format!("{:?}", e)));
                    }
                    Error::UrlParse(e) => { error_menu.push(MenuItem::new(format!("error parsing URL: {}", e))); }
                    Error::WatchlistFormatInner(config, e) => {
                        error_menu.push(MenuItem::new(format!("received incorrectly formatted watchlist for {}", config.display_name)));
                        error_menu.push(MenuItem::new(e));
                    }
                    Error::WatchlistFormatOuter(config, json) => {
                        error_menu.push(MenuItem::new(format!("did not receive watchlist for {}, received:", config.display_name)));
                        error_menu.push(MenuItem::new(json));
                    }
                }
                print!("{}", Menu(error_menu));
            }
        }
    }
    Ok(())
}
