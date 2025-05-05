- clap
	- auto: do not ask for confirmations, just go ahead
	- have the favorites/links/bookmarks auto setting by default disabled
	- verbose
- need to redo windows paths, linux paths need to setup vm somewhere
- need to check if program is running
- firefox beta, nightly, & developer version
	- https://github.com/yokoffing/Betterfox/blob/main/install.py#L48
	- /Users/cooper/Library/Application Support/Firefox/Profiles/tx5kh1d1.default
	- i think profiles.ini has profile data
	- NEED to find config to disable a lot of the default show icons and stuff
		- **can all of this be done thru user.js??**
		- hide sidebar shit
		- disable all telemetry
		- disable new tab bullshit
		- change to vertical tabs
		- bookmarks toolbar only on new tabs
		- disable container tabs (if beta+)
		- there will be some weird json encoded shit
- for backups, create with timestamp
- remove kagi snippet from brave
- centralized links/bookmarks handler
	- also update from safari
- for firefox user profiles
	- first get all the profile names
	- prioritze the last used 
	- interactively ask which profile to debloat
- customize betterfox config
	- // PREF: revert back to Standard ETP
		user_pref("browser.contentblocking.category", "standard");
	- // PREF: allow websites to ask you for your location
		user_pref("permissions.default.geo", 0);
	- // PREF: restore Top Sites on New Tab page
		user_pref("browser.newtabpage.activity-stream.feeds.topsites", true);
	- // PREF: remove default Top Sites (Facebook, Twitter, etc.)
		// This does not block you from adding your own.
		user_pref("browser.newtabpage.activity-stream.default.sites", "");
	- // PREF: remove sponsored content on New Tab page
		user_pref("browser.newtabpage.activity-stream.showSponsoredTopSites", false); // Sponsored shortcuts 
		user_pref("browser.newtabpage.activity-stream.feeds.section.topstories", false); // Recommended by Pocket
		user_pref("browser.newtabpage.activity-stream.showSponsored", false); // Sponsored Stories
	- // PREF: restore search engine suggestions
		user_pref("browser.search.suggest.enabled", true);
	- // PREF: disable unified search button
		user_pref("browser.urlbar.scotchBonnet.enableOverride", false);
	- // PREF: disable login manager
		user_pref("signon.rememberSignons", false);
	- // PREF: disable address and credit card manager
		user_pref("extensions.formautofill.addresses.enabled", false);
		user_pref("extensions.formautofill.creditCards.enabled", false);
	- if windows:
		// PREF: improve font rendering by using DirectWrite everywhere like Chrome [WINDOWS]
		user_pref("gfx.font_rendering.cleartype_params.rendering_mode", 5);
		user_pref("gfx.font_rendering.cleartype_params.cleartype_level", 100);
		user_pref("gfx.font_rendering.directwrite.use_gdi_table_loading", false);
- tell to install extensions at the end
- better error on failing to fetch betterfox
- log before fetching betterfox
- betterfox indicatif [impl example](https://github.com/rust-secure-code/cargo-supply-chain/pull/55)
- move big blocks (e.x. kagi search, links, better fox snippets) to snippets directory or smth
- better zen: https://github.com/yokoffing/Betterfox/blob/main/zen/user.js
- need to test brave and all of them to make sure no breakage
- did brave config have weird stringified json that i didnt keep track of???

### FUTURE:
- auto install extensions? (ublock (auto configure too???), kagi, 1pass)
	- option to configure these too
- option to set custom dns over http config (once done with mcps blocking nextdns)
- custom search engine (kagi) with backup duckduckgo in incognito
- using only brave default profile isn't the best solution
- disable setting vertical tabs wherever i can
- interactive ui for the clap settings
