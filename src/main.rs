#![warn(trivial_casts)]
#![deny(unused, unused_qualifications)]
#![forbid(unused_import_braces)]

#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate wikibase;
extern crate xdg_basedir;

use std::collections::HashMap;
use std::fmt::{self, Write};
use std::fs::File;

/// A monocolored, resized version of https://commons.wikimedia.org/wiki/File:Mediawiki_logo_sunflower.svg
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
enum Error {
    ConfigFormat(serde_json::Error),
    Fmt(fmt::Error),
    MissingConfig,
    WatchlistFormat(Option<serde_json::Error>),
    Wikibase(wikibase::WikibaseError)
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Error {
        Error::Fmt(e)
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

fn bitbar() -> Result<String, Error> {
    let config = Config::new()?;
    let mut wikibase_config = wikibase::Configuration::new(&format!("bitbar-mediawiki-watchlist/{}", env!("CARGO_PKG_VERSION")))?;
    let watchlists = config.wikis.iter().map(|wiki_config| {
        wikibase_config.set_api_url(&wiki_config.api_url[..]);
        let mut json = wikibase::requests::wikibase_request(
            &format!("{}?action=query&format=json&list=watchlist&wlallrev=1&wldir=newer&wllimit=max&wlshow=unread&wlowner={}&wltoken={}", wiki_config.api_url, wiki_config.username, wiki_config.watchlist_token),
            &wikibase_config
        )?;
        let watchlist = serde_json::from_value::<Vec<WatchlistItem>>(
            json.pointer_mut("/query/watchlist").ok_or(Error::WatchlistFormat(None))?.take()
        ).map_err(|e| Error::WatchlistFormat(Some(e)))?;
        let mut filtered_watchlist = HashMap::default();
        for watchlist_item in watchlist {
            // only show the oldest unread event of each page
            if filtered_watchlist.entry(watchlist_item.pageid).or_insert_with(|| watchlist_item.clone()).old_revid > watchlist_item.old_revid {
                filtered_watchlist.insert(watchlist_item.pageid, watchlist_item);
            }
        }
        Ok(filtered_watchlist)
    }).collect::<Result<Vec<HashMap<u64, WatchlistItem>>, Error>>()?;
    let mut text = String::default();
    let total = watchlists.iter().map(HashMap::len).sum::<usize>();
    if total > 0 {
        writeln!(&mut text, "{}|templateImage={}\n", total, TOURNESOL)?;
        for (watchlist, wiki_config) in watchlists.into_iter().zip(config.wikis) {
            if !watchlist.is_empty() {
                writeln!(&mut text, "---")?;
                writeln!(&mut text, "{}|", wiki_config.display_name)?;
                for (_, watchlist_item) in watchlist {
                    writeln!(&mut text, "{}|href={}?pageid={}&diff=next&oldid={}", watchlist_item.title, wiki_config.index_url, watchlist_item.pageid, watchlist_item.old_revid)?;
                }
            }
        }
    }
    Ok(text)
}

fn main() {
    match bitbar() {
        Ok(text) => { print!("{}", text); }
        Err(e) => {
            println!("?|templateImage={}", TOURNESOL);
            println!("---");
            match e {
                Error::ConfigFormat(e) => { println!("error in config file: {}", e); }
                Error::MissingConfig => { println!("missing or invalid configuration file"); } //TODO better error message
                Error::WatchlistFormat(Some(e)) => { println!("received incorrectly formatted watchlist: {}", e); }
                Error::WatchlistFormat(None) => { println!("did not receive watchlist"); }
                Error::Wikibase(wikibase::WikibaseError::Configuration(msg)) => { println!("wikibase configuration error: {}", msg); }
                Error::Wikibase(wikibase::WikibaseError::Request(msg)) => { println!("wikibase request error: {}", msg); }
                Error::Wikibase(wikibase::WikibaseError::Serialization(msg)) => { println!("wikibase serialization error: {}", msg); }
                Error::Wikibase(wikibase::WikibaseError::Validation(msg)) => { println!("wikibase validation error: {}", msg); }
                e => { println!("{:?}", e); } //TODO handle separately
            }
        }
    }
}
