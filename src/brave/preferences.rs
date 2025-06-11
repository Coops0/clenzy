use crate::{logging::success, s, util::{get_or_insert_obj, timestamp}, ARGS};
use color_eyre::eyre::{bail, ContextCompat, WrapErr};
use serde_json::{json, Value};
use std::{fs, path::Path};
use tracing::{debug, instrument, trace};

#[instrument(level = "debug")]
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub fn preferences(root: &Path) -> color_eyre::Result<()> {
    let path = root.join("Preferences");

    if ARGS.get().unwrap().backup {
        let backup = root.join(format!("Preferences-{}", timestamp())).with_extension("bak");

        fs::copy(&path, &backup)?;
        success("Backed up Brave preferences file");
        debug!("Backup file path: {}", backup.display());
    }

    let prefs_str = fs::read_to_string(&path);
    let Value::Object(mut prefs) = serde_json::from_str::<Value>(&prefs_str?)? else {
        bail!("Failed to cast preferences to an object");
    };

    if let Some(bookmark_bar) = get_or_insert_obj(&mut prefs, "bookmark_bar") {
        bookmark_bar.insert(s!("show_on_all_tabs"), json!(false));
        bookmark_bar.insert(s!("show_tab_groups"), json!(false));
        trace!("disabled bookmark bar on all tabs and tab groups");
    }

    let brave = prefs
        .get_mut("brave")
        .and_then(Value::as_object_mut)
        .context("failed to get brave object")?;

    if let Some(ai_chat) = get_or_insert_obj(brave, "ai_chat") {
        ai_chat.insert(s!("autocomplete_provider_enabled"), json!(false));
        ai_chat.insert(s!("context_menu_enabled"), json!(false));
        ai_chat.insert(s!("show_toolbar_button"), json!(false));
        ai_chat.insert(s!("storage_enabled"), json!(false));
        ai_chat.insert(s!("tab_organization_enabled"), json!(false));
        trace!("disabled brave AI chat");
    }

    brave.insert(s!("always_show_bookmark_bar_on_ntp"), json!(true));
    trace!("enabled bookmark bar (on new tab page)");

    brave.insert(s!("autocomplete_enabled"), json!(ARGS.get().unwrap().search_suggestions));

    if let Some(brave_ads) = get_or_insert_obj(brave, "brave_ads") {
        brave_ads.insert(s!("should_allow_ads_subdivision_targeting"), json!(false));
        trace!("disabled brave ads");
    }

    if let Some(brave_search_conversation) = get_or_insert_obj(brave, "brave_search_conversion") {
        brave_search_conversation.insert(s!("dismissed"), json!(false));
        trace!("dismissed brave search conversation");
    }

    // This is disabled by default anyways
    if let Some(settings) = get_or_insert_obj(brave, "settings") {
        settings.insert(s!("force_google_safesearch"), json!(false));
        trace!("disabled force google safesearch");
    }

    if let Some(brave_vpn) = get_or_insert_obj(brave, "brave_vpn") {
        brave_vpn.insert(s!("show_button"), json!(false));
        trace!("hid brave VPN button");
    }

    brave.insert(s!("enable_closing_last_tab"), json!(true));
    brave.insert(s!("enable_window_closing_confirm"), json!(true));
    brave.insert(s!("location_bar_is_wide"), json!(true));
    trace!("enabled closing last tab, window closing confirm, and wide location bar");

    if let Some(new_tab_page) = get_or_insert_obj(brave, "new_tab_page") {
        new_tab_page.insert(s!("hide_all_widgets"), json!(true));
        new_tab_page.insert(s!("show_background_image"), json!(true));
        new_tab_page.insert(s!("show_branded_background_image"), json!(false));
        new_tab_page.insert(s!("show_brave_news"), json!(false));
        new_tab_page.insert(s!("show_brave_vpn"), json!(false));
        new_tab_page.insert(s!("show_clock"), json!(false));
        new_tab_page.insert(s!("show_rewards"), json!(false));
        new_tab_page.insert(s!("show_stats"), json!(false));
        new_tab_page.insert(s!("show_together"), json!(false));
        new_tab_page.insert(s!("shows_options"), json!(0));
        trace!("hid new tab page widgets");
    }

    brave.insert(s!("other_search_engines_enabled"), json!(true));
    trace!("enabled other search engines");

    if let Some(rewards) = get_or_insert_obj(brave, "rewards") {
        rewards.insert(s!("notifications"), json!("{\"displayed\":[],\"notifications\":[]}"));
        rewards.insert(s!("show_brave_rewards_button_in_location_bar"), json!(false));
        trace!("hid brave rewards button and dismissed any notifications");
    }

    if let Some(shields) = get_or_insert_obj(brave, "shields") {
        shields.insert(s!("advanced_view_enabled"), json!(true));
        shields.insert(s!("stats_badge_visible"), json!(false));
        trace!("enabled shields advanced view and hid stats badge");
    }

    brave.insert(s!("show_fullscreen_reminder"), json!(false));
    brave.insert(s!("show_side_panel_button"), json!(false));
    brave.insert(s!("show_bookmarks_button"), json!(false));
    trace!("hid fullscreen reminder, side panel button, and bookmarks button");

    if let Some(sidebar) = get_or_insert_obj(brave, "sidebar") {
        sidebar.insert(s!("hidden_built_in_items"), json!([7]));
        sidebar.insert(s!("item_added_feedback_bubble_shown_count"), json!(1));
        sidebar.insert(
            s!("sidebar_items"),
            json!([
                { "built_in_item_type": 1, "type": 0 },
                { "built_in_item_type": 2, "type": 0 },
                { "built_in_item_type": 3, "type": 0 },
                { "built_in_item_type": 4, "type": 0 }
            ])
        );
        sidebar.insert(s!("sidebar_show_option"), json!(3));
        trace!("hid sidebar items");
    }

    if ARGS.get().unwrap().vertical_tabs
        && let Some(tabs) = get_or_insert_obj(brave, "tabs")
    {
        tabs.insert(s!("vertical_tabs_collapsed"), json!(false));
        tabs.insert(s!("vertical_tabs_enabled"), json!(true));
        tabs.insert(s!("vertical_tabs_expanded_width"), json!(114));
        tabs.insert(s!("vertical_tabs_floating_enabled"), json!(true));
        tabs.insert(s!("vertical_tabs_show_title_on_window"), json!(false));
        trace!("enabled vertical tabs");
    }

    if let Some(wallet) = get_or_insert_obj(brave, "wallet") {
        wallet.insert(s!("default_solana_wallet"), json!(0));
        wallet.insert(s!("default_wallet2"), json!(0));
        wallet.insert(s!("show_wallet_icon_on_toolbar"), json!(false));
        wallet.insert(s!("should_show_wallet_suggestion_badge"), json!(false));
        trace!("hid wallet button and disabled default wallets");
    }

    brave.insert(s!("tabs_search_show"), json!(false));
    brave.insert(s!("webtorrent_enabled"), json!(false));
    trace!("disabled webtorrent and hid tabs search");

    if let Some(today) = get_or_insert_obj(brave, "today") {
        today.insert(s!("should_show_toolbar_button"), json!(false));
        trace!("hid today toolbar button");
    }

    if let Some(browser) = get_or_insert_obj(brave, "browser") {
        browser.insert(s!("has_seen_welcome_page"), json!(true));
        trace!("marked welcome page as seen");
    }

    // -- END BRAVE MAP SECTION --

    if let Some(custom_links) = get_or_insert_obj(&mut prefs, "custom_links") {
        custom_links.insert(s!("initialized"), json!(true));
    }

    prefs.insert(s!("enable_do_not_track"), json!(true));
    trace!("enabled do not track");

    if let Some(in_product_help) = get_or_insert_obj(&mut prefs, "in_product_help") {
        if let Some(new_badge) = get_or_insert_obj(in_product_help, "new_badge") {
            if let Some(compose_nudge) = get_or_insert_obj(new_badge, "ComposeNudge") {
                compose_nudge.insert(s!("show_count"), json!(0));
            }

            if let Some(compose_proactive_nudge) =
                get_or_insert_obj(new_badge, "ComposeProactiveNudge")
            {
                compose_proactive_nudge.insert(s!("show_count"), json!(0));
            }
        }

        if let Some(snoozed_feature) = get_or_insert_obj(in_product_help, "snoozed_feature")
            && let Some(iph_discard_ring) = get_or_insert_obj(snoozed_feature, "IPH_DiscardRing")
        {
            iph_discard_ring.insert(s!("is_dismissed"), json!(true));
        }

        trace!("dismissed some in product help features");
    }

    if let Some(ntp) = get_or_insert_obj(&mut prefs, "ntp") {
        ntp.insert(s!("shortcust_visible"), json!(false));
        ntp.insert(s!("use_most_visited_tiles"), json!(false));
        trace!("hid new tab page widgets");
    }

    if let Some(omnibox) = get_or_insert_obj(&mut prefs, "omnibox") {
        // show the entire URL always
        omnibox.insert(s!("prevent_url_elisions"), json!(true));
        omnibox.insert(s!("shown_count_history_scope_promo"), json!(false));
        trace!("enabled showing full url and history scope promo");
    }

    if let Some(search) = get_or_insert_obj(&mut prefs, "search") {
        search.insert(s!("suggest_enabled"), json!(ARGS.get().unwrap().search_suggestions));
    }

    if let Some(privacy_sandbox) = get_or_insert_obj(&mut prefs, "privacy_sandbox") {
        privacy_sandbox.insert(s!("first_party_sets_enabled"), json!(false));
        if let Some(m1) = get_or_insert_obj(privacy_sandbox, "m1") {
            m1.insert(s!("ad_measurement_enabled"), json!(false));
            m1.insert(s!("fledge_enabled"), json!(false));
            m1.insert(s!("topics_enabled"), json!(false));
        }

        trace!("disabled ad measurement, fledge, and topics");
    }

    let prefs_str = serde_json::to_string(&prefs)?;
    fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))
}
