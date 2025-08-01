// Vertical tabs and UI cleanup
user_pref("browser.toolbars.bookmarks.showOtherBookmarks", false);
user_pref("browser.uiCustomization.horizontalTabsBackup", "{\"placements\":{\"widget-overflow-fixed-list\":[],\"unified-extensions-area\":[],\"nav-bar\":[\"sidebar-button\",\"back-button\",\"forward-button\",\"stop-reload-button\",\"customizableui-special-spring1\",\"vertical-spacer\",\"urlbar-container\",\"customizableui-special-spring2\",\"save-to-pocket-button\",\"downloads-button\",\"developer-button\",\"fxa-toolbar-menu-button\",\"unified-extensions-button\"],\"TabsToolbar\":[\"firefox-view-button\",\"tabbrowser-tabs\",\"new-tab-button\",\"alltabs-button\"],\"vertical-tabs\":[],\"PersonalToolbar\":[\"personal-bookmarks\"]},\"seen\":[\"save-to-pocket-button\",\"developer-button\",\"profiler-button\"],\"dirtyAreaCache\":[\"nav-bar\",\"vertical-tabs\",\"PersonalToolbar\"],\"currentVersion\":22,\"newElementCount\":2}");
user_pref("browser.uiCustomization.horizontalTabstrip", "[\"tabbrowser-tabs\",\"new-tab-button\"]");
user_pref("browser.uiCustomization.navBarWhenVerticalTabs", "[\"sidebar-button\",\"back-button\",\"forward-button\",\"stop-reload-button\",\"customizableui-special-spring1\",\"vertical-spacer\",\"urlbar-container\",\"customizableui-special-spring2\",\"downloads-button\",\"developer-button\",\"unified-extensions-button\",\"ublock0_raymondhill_net-browser-action\",\"reset-pbm-toolbar-button\"]");
user_pref("browser.uiCustomization.state", "{\"placements\":{\"widget-overflow-fixed-list\":[\"profiler-button\"],\"unified-extensions-area\":[\"_d634138d-c276-4fc8-924b-40a0ea21d284_-browser-action\",\"search_kagi_com-browser-action\",\"_5caff8cc-3d2e-4110-a88a-003cc85b3858_-browser-action\"],\"nav-bar\":[\"sidebar-button\",\"back-button\",\"forward-button\",\"stop-reload-button\",\"customizableui-special-spring1\",\"vertical-spacer\",\"urlbar-container\",\"customizableui-special-spring2\",\"downloads-button\",\"developer-button\",\"unified-extensions-button\",\"ublock0_raymondhill_net-browser-action\",\"reset-pbm-toolbar-button\"],\"vertical-tabs\":[\"tabbrowser-tabs\"],\"PersonalToolbar\":[\"personal-bookmarks\"]},\"seen\":[\"save-to-pocket-button\",\"developer-button\",\"profiler-button\",\"_d634138d-c276-4fc8-924b-40a0ea21d284_-browser-action\",\"ublock0_raymondhill_net-browser-action\",\"search_kagi_com-browser-action\",\"reset-pbm-toolbar-button\",\"_5caff8cc-3d2e-4110-a88a-003cc85b3858_-browser-action\"],\"dirtyAreaCache\":[\"nav-bar\",\"vertical-tabs\",\"PersonalToolbar\",\"TabsToolbar\",\"unified-extensions-area\",\"widget-overflow-fixed-list\"],\"currentVersion\":22,\"newElementCount\":4}");

user_pref("browser.urlbar.suggest.engines", false);

user_pref("browser.bookmarks.restore_default_bookmarks", false);
user_prefs("browser.bookmarks.showMobileBookmarks", false);
// Disable "what's new" page on start
user_prefs("browser.startup.homepage_override.mstone", "ignore");

// I am only seeing weather on Ubuntu Firefox beta?
user_pref("browser.newpagetab.activity-stream.showWeather", false);

user_pref("sidebar.main.tools", "");