use {
    std::{
        collections::BTreeMap,
        convert::Infallible as Never,
        env::current_exe,
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
    futures::prelude::*,
    itertools::Itertools,
    maplit::{
        convert_args,
        hashmap,
    },
    mediawiki::media_wiki_error::MediaWikiError,
    serde::Deserialize,
    xdg::BaseDirectories,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigWiki {
    display_name: String,
    api_url: String,
    index_url: String,
    username: String,
    watchlist_token: String,
}

#[derive(Deserialize)]
struct Config {
    wikis: Vec<ConfigWiki>,
}

impl Config {
    fn new() -> Result<Config, Error> {
        let path = BaseDirectories::new()?.find_config_file("bitbar/plugins/mediawiki-watchlist.json").ok_or(Error::MissingConfig)?;
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file).map_err(Error::ConfigFormat)?)
    }
}

#[derive(Debug, thiserror::Error)]
enum OpenAllError {
    #[error("error in open_all command: unknown wiki: {0}")]
    UnknownWiki(String),
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)] Fmt(#[from] fmt::Error),
    #[error(transparent)] Io(#[from] io::Error),
    #[error(transparent)] MediaWiki(#[from] MediaWikiError),
    #[error(transparent)] OpenAll(#[from] OpenAllError),
    #[error(transparent)] Other(#[from] Box<dyn std::error::Error>),
    #[error(transparent)] UrlParse(#[from] url::ParseError),
    #[error(transparent)] Xdg(#[from] xdg::BaseDirectoriesError),
    #[error("error in config file: {0}")]
    ConfigFormat(#[source] serde_json::Error),
    #[error("missing or invalid configuration file")]
    MissingConfig,
    #[error("received incorrectly formatter watchlist for {}: {1}", .0.display_name)]
    WatchlistFormatInner(ConfigWiki, serde_json::Error),
    #[error("did not receive watchlist for {}, received: {1}", .0.display_name)]
    WatchlistFormatOuter(ConfigWiki, serde_json::Value),
}

impl From<Never> for Error {
    fn from(never: Never) -> Error {
        match never {}
    }
}

impl From<Error> for Menu {
    fn from(e: Error) -> Menu {
        let mut error_menu = Vec::default();
        match e {
            Error::Other(e) => {
                error_menu.push(MenuItem::new(&e));
                error_menu.push(MenuItem::new(format!("{:?}", e)));
            }
            Error::WatchlistFormatInner(config, e) => {
                error_menu.push(MenuItem::new(format!("received incorrectly formatted watchlist for {}", config.display_name)));
                error_menu.push(MenuItem::new(e));
            }
            Error::WatchlistFormatOuter(config, json) => {
                error_menu.push(MenuItem::new(format!("did not receive watchlist for {}, received:", config.display_name)));
                error_menu.push(MenuItem::new(json));
            }
            _ => error_menu.push(MenuItem::new(e)),
        }
        Menu(error_menu)
    }
}

#[derive(Clone, Deserialize)]
struct WatchlistItem {
    old_revid: u64,
    //revid: u64,
    pageid: u64,
    title: String,
}

async fn get_watchlist(wiki_config: &ConfigWiki) -> Result<BTreeMap<u64, WatchlistItem>, Error> {
    let client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .use_rustls_tls();
    let mut api = mediawiki::api::Api::new_from_builder(&wiki_config.api_url, client_builder).await?;
    api.set_user_agent(concat!("bitbar-mediawiki-watchlist/", env!("CARGO_PKG_VERSION"), " (https://github.com/fenhl/bitbar-mediawiki-watchlist)")); // https://www.mediawiki.org/wiki/API:Etiquette#The_User-Agent_header
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

#[bitbar::command]
async fn open_all(display_name: String) -> Result<(), Error> {
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

#[bitbar::main(
    error_template_image = "../assets/mediawiki-small.png",
    commands(open_all),
)]
async fn main() -> Result<Menu, Error> {
    let config = Config::new()?;
    let watchlists = stream::iter(&config.wikis).then(get_watchlist).try_collect::<Vec<_>>().await?;
    let mut items = Vec::default();
    let total = watchlists.iter().map(BTreeMap::len).sum::<usize>();
    if total > 0 {
        items.push(ContentItem::new(total).template_image(&include_bytes!("../assets/mediawiki-small.png")[..])?.into());
        for (watchlist, wiki_config) in watchlists.into_iter().zip(config.wikis) {
            if !watchlist.is_empty() {
                items.push(MenuItem::Sep);
                if let Ok(exe_path) = current_exe() {
                    items.push(ContentItem::new(&wiki_config.display_name)
                        .command((exe_path.display(), "open_all", wiki_config.display_name))?
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
