This is a BitBar plugin (supporting both [SwiftBar](https://swiftbar.app/) and [xbar](https://xbarapp.com/)) that displays unread [watchlist](https://www.mediawiki.org/wiki/Manual:Watchlist) entries on [MediaWiki](https://www.mediawiki.org/wiki/MediaWiki) wikis.

# Installing

1. Install [SwiftBar](https://swiftbar.app/) or [xbar](https://xbarapp.com/).
    * If you're unsure which to install, I recommend SwiftBar, as this plugin has been tested with it.
    * If you have [Homebrew](https://brew.sh/), you can also install with `brew install --cask swiftbar` or `brew install --cask xbar`.
2. [Install Rust](https://www.rust-lang.org/tools/install).
    * If you have Homebrew, you can also install with `brew install rust`.
3. Install the plugin:
    ```sh
    cargo install --git=https://github.com/fenhl/bitbar-mediawiki-watchlist --branch=main
    ```
4. Create a symlink to `~/.cargo/bin/bitbar-mediawiki-watchlist` in your SwiftBar/xbar plugin folder. Call it something like `bitbar-mediawiki-watchlist.30s.o`, where `30s` is the rate at which the plugin will check for notifications.
5. At `~/.config/bitbar/plugins/mediawiki-watchlist.json`, create a [JSON](https://json.org/) file containing the following fields:
    * `"wikis"`, an array of objects, one for each wiki whose watchlist you want to display, each with the following fields:
        * `"displayName"`: a name for the wiki that will be displayed in the menu.
        * `"apiUrl"`: The wiki's “api.php” URL, which can be found on the `Special:Version` page of the wiki, in section “Entry Point URLs”.
        * `"indexUrl"`: The wiki's “index.php” URL, which can be found on the `Special:Version` page of the wiki, in section “Entry Point URLs”.
        * `"username"`: Your username on the wiki.
        * `"watchlistToken"`: Your watchlist token, which can be found on the “Watchlist” tab of the `Special:Preferences` page.
6. If you're using SwiftBar, the plugin should now appear in your menu bar. If it doesn't appear automatically, or if you're using xbar, refresh by opening a menu and pressing <kbd>⌘</kbd><kbd>R</kbd>.
