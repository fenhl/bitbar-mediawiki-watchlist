This is a BitBar plugin (supporting both [SwiftBar](https://swiftbar.app/) and [xbar](https://xbarapp.com/)) that displays unread [watchlist](https://www.mediawiki.org/wiki/Manual:Watchlist) entries on [MediaWiki](https://www.mediawiki.org/wiki/MediaWiki) wikis.

# Installing

1. [Install Rust](https://www.rust-lang.org/en-US/install.html).
2. Clone this repository.
3. In the repository, run `cargo build --release`.
4. Symlink the file `target/release/bitbar-mediawiki-watchlist` into your SwiftBar/xbar plugin directory.
5. At `~/.config/bitbar/plugins/mediawiki-watchlist.json`, create a [JSON](https://json.org/) file containing the following fields:
    * `"wikis"`, an array of objects, one for each wiki whose watchlist you want to display, each with the following fields:
        * `"displayName"`: a name for the wiki that will be displayed in the menu.
        * `"apiUrl"`: The wiki's “api.php” URL, which can be found on the `Special:Version` page of the wiki, in section “Entry Point URLs”.
        * `"indexUrl"`: The wiki's “index.php” URL, which can be found on the `Special:Version` page of the wiki, in section “Entry Point URLs”.
        * `"username"`: Your username on the wiki.
        * `"watchlistToken"`: Your watchlist token, which can be found on the “Watchlist” tab of the `Special:Preferences` page.
    * Optionally, `"openAll"`, containing the full path to the `target/release/open-all` file in the cloned repo. If given, clicking on a wiki's display name will open all diff pages for that wiki.
6. Refresh SwiftBar/xbar by opening a menu and pressing <kbd>⌘</kbd><kbd>R</kbd>.
