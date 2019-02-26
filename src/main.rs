#![warn(trivial_casts)]
#![deny(unused, unused_qualifications)]
#![forbid(unused_import_braces)]

use std::{
    collections::BTreeMap,
    env::{
        self,
        current_exe
    },
    fmt,
    fs::File,
    io,
    process::Command,
    thread,
    time::Duration
};
use bitbar::{
    ContentItem,
    Menu,
    MenuItem
};
use itertools::Itertools;
use notify_rust::Notification;
use serde_derive::Deserialize;

/// A modified version of https://commons.wikimedia.org/wiki/File:Mediawiki_logo_sunflower.svg
///
/// Modifications: monocolored, resized, added a 0-alpha circle to make it parse as a sunflower at icon size.
///
/// original CC-BY-SA Isarra and Anthere, see link for details
const TOURNESOL: &str = "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";

#[derive(Deserialize)]
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

#[derive(Debug)]
enum Error {
    ConfigFormat(serde_json::Error),
    Fmt(fmt::Error),
    Io(io::Error),
    MissingConfig,
    OpenAll(OpenAllError),
    UrlParse(url::ParseError),
    WatchlistFormat(Result<serde_json::Error, serde_json::Value>),
    Wikibase(wikibase::WikibaseError)
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Error {
        Error::Fmt(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<OpenAllError> for Error {
    fn from(e: OpenAllError) -> Error {
        Error::OpenAll(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Error {
        Error::UrlParse(e)
    }
}

impl From<wikibase::WikibaseError> for Error {
    fn from(e: wikibase::WikibaseError) -> Error {
        Error::Wikibase(e)
    }
}

#[derive(Clone, Deserialize)]
struct WatchlistItem {
    old_revid: u64,
    //revid: u64,
    pageid: u64,
    title: String
}

fn get_watchlist(wikibase_config: &mut wikibase::Configuration, wiki_config: &ConfigWiki) -> Result<BTreeMap<u64, WatchlistItem>, Error> {
    wikibase_config.set_api_url(&wiki_config.api_url[..]);
    let mut json = wikibase::requests::wikibase_request(
        &format!("{}?action=query&format=json&list=watchlist&wlallrev=1&wldir=newer&wllimit=max&wlshow=unread&wlowner={}&wltoken={}", wiki_config.api_url, wiki_config.username, wiki_config.watchlist_token),
        &wikibase_config
    )?;
    let watchlist = serde_json::from_value::<Vec<WatchlistItem>>(
        json.pointer_mut("/query/watchlist")
            .map(serde_json::Value::take)
            .ok_or_else(|| Error::WatchlistFormat(Err(json.clone())))?
    ).map_err(|e| Error::WatchlistFormat(Ok(e)))?;
    let mut filtered_watchlist = BTreeMap::default();
    for watchlist_item in watchlist {
        // only show the oldest unread event of each page
        if filtered_watchlist.entry(watchlist_item.pageid).or_insert_with(|| watchlist_item.clone()).old_revid > watchlist_item.old_revid {
            filtered_watchlist.insert(watchlist_item.pageid, watchlist_item);
        }
    }
    Ok(filtered_watchlist)
}

fn bitbar() -> Result<Menu, Error> {
    let config = Config::new()?;
    let mut wikibase_config = wikibase::Configuration::new(concat!("bitbar-mediawiki-watchlist/", env!("CARGO_PKG_VERSION")))?;
    let watchlists = config.wikis.iter().map(|wiki_config| get_watchlist(&mut wikibase_config, wiki_config)).collect::<Result<Vec<_>, Error>>()?;
    let mut items = Vec::default();
    let total = watchlists.iter().map(BTreeMap::len).sum::<usize>();
    if total > 0 {
        items.push(ContentItem::new(total).template_image(TOURNESOL).into());
        for (watchlist, wiki_config) in watchlists.into_iter().zip(config.wikis) {
            if !watchlist.is_empty() {
                items.push(MenuItem::Sep);
                if let Ok(ref exe_path) = current_exe() {
                    items.push(ContentItem::new(&wiki_config.display_name)
                        .command(&[&exe_path.display().to_string(), "open-all", &wiki_config.display_name])
                        .refresh()
                        .into()
                    );
                } else {
                    items.push(MenuItem::new(wiki_config.display_name));
                }
                for (_, watchlist_item) in watchlist {
                    items.push(ContentItem::new(watchlist_item.title)
                        .href(format!("{}?{}&diff=next&oldid={}", wiki_config.index_url, watchlist_item.pageid, watchlist_item.old_revid).parse()?) //TODO use Url::query_pairs_mut
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

fn open_all(args: env::Args) -> Result<(), Error> {
    let (display_name,) = args.collect_tuple().ok_or(OpenAllError::MissingDisplayName)?;
    let mut wikibase_config = wikibase::Configuration::new(concat!("bitbar-mediawiki-watchlist/", env!("CARGO_PKG_VERSION")))?;
    let (wiki_config,) = Config::new()?.wikis.into_iter().filter(|wiki_config| wiki_config.display_name == display_name).collect_tuple().ok_or(OpenAllError::UnknownWiki(display_name.to_string()))?;
    let watchlist = get_watchlist(&mut wikibase_config, &wiki_config)?;
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

fn main() {
    let mut args = env::args();
    let _ = args.next(); // ignore executable name
    if let Some(arg) = args.next() {
        match &arg[..] {
            "open-all" => { open_all(args).notify("error in open-all cmd"); }
            subcmd => { notify("unknown subcommand", subcmd); }
        }
    } else {
        match bitbar() {
            Ok(menu) => { print!("{}", menu); }
            Err(e) => {
                println!("?|templateImage={}", TOURNESOL);
                println!("---");
                match e {
                    Error::ConfigFormat(e) => { println!("error in config file: {}", e); }
                    Error::MissingConfig => { println!("missing or invalid configuration file"); } //TODO better error message
                    Error::WatchlistFormat(Ok(e)) => { println!("received incorrectly formatted watchlist: {}", e); }
                    Error::WatchlistFormat(Err(json)) => { println!("did not receive watchlist, received {}", json); }
                    Error::Wikibase(wikibase::WikibaseError::Configuration(msg)) => { println!("wikibase configuration error: {}", msg); }
                    Error::Wikibase(wikibase::WikibaseError::Request(msg)) => { println!("wikibase request error: {}", msg); }
                    Error::Wikibase(wikibase::WikibaseError::Serialization(msg)) => { println!("wikibase serialization error: {}", msg); }
                    Error::Wikibase(wikibase::WikibaseError::Validation(msg)) => { println!("wikibase validation error: {}", msg); }
                    e => { println!("{:?}", e); } //TODO handle separately
                }
            }
        }
    }
}
