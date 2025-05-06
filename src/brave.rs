use crate::util::{get_or_insert_obj, roaming_data_base, timestamp};
use color_eyre::eyre::{Context, ContextCompat};
use serde_json::{json, Map, Value};
use std::{fs, path::PathBuf, sync::LazyLock};
use tracing::{debug, info, instrument};

pub fn brave_folder() -> Option<PathBuf> {
    let path = roaming_data_base()?.join("BraveSoftware").join("Brave-Browser");
    if path.exists() { Some(path) } else { None }
}

pub fn brave_nightly_folder() -> Option<PathBuf> {
    let path = roaming_data_base()?.join("BraveSoftware").join("Brave-Browser-Nightly");
    if path.exists() { Some(path) } else { None }
}

#[instrument]
pub fn debloat(mut path: PathBuf) -> color_eyre::Result<()> {
    if cfg!(target_os = "windows") {
        path = path.join("User Data")
    }

    let default = path.join("Default");
    preferences(&default)?;
    chrome_feature_state(&default)?;

    Ok(())
}

#[instrument]
fn preferences(root: &PathBuf) -> color_eyre::Result<()> {
    let path = root.join("Preferences");
    let backup = root.join(format!("Preferences-{}", timestamp())).with_extension("bak");

    fs::copy(&path, &backup)?;
    info!("backed up brave preferences to {}", backup.display());

    let prefs_str = fs::read_to_string(&path);
    let mut prefs = serde_json::from_str::<Value>(&prefs_str?)?;

    let prefs_map = prefs.as_object_mut().context("failed to parse preferences as an object")?;

    if let Some(bookmark_bar) = get_or_insert_obj(prefs_map, "bookmark_bar") {
        bookmark_bar.insert(String::from("show_on_all_tabs"), json!(false));
        bookmark_bar.insert(String::from("show_tab_groups"), json!(false));
        debug!("disabled bookmark bar on all tabs and tab groups");
    }

    let brave = prefs_map
        .get_mut("brave")
        .and_then(Value::as_object_mut)
        .context("failed to get brave object")?;

    if let Some(ai_chat) = get_or_insert_obj(brave, "ai_chat") {
        ai_chat.insert(String::from("autocomplete_provider_enabled"), json!(false));
        ai_chat.insert(String::from("context_menu_enabled"), json!(false));
        ai_chat.insert(String::from("show_toolbar_button"), json!(false));
        ai_chat.insert(String::from("storage_enabled"), json!(false));
        ai_chat.insert(String::from("tab_organization_enabled"), json!(false));
        debug!("disabled brave AI chat");
    }

    brave.insert(String::from("always_show_bookmark_bar_on_ntp"), json!(true));
    // People will want this disabled by default probably
    brave.insert(String::from("autocomplete_enabled"), json!(true));
    debug!("enabled bookmark bar and autocomplete");

    if let Some(brave_ads) = get_or_insert_obj(brave, "brave_ads") {
        brave_ads.insert(String::from("should_allow_ads_subdivision_targeting"), json!(false));
        debug!("disabled brave ads");
    }

    if let Some(brave_search_conversation) = get_or_insert_obj(brave, "brave_search_conversion") {
        brave_search_conversation.insert(String::from("dismissed"), json!(false));
        debug!("dismissed brave search conversation");
    }

    if let Some(brave_vpn) = get_or_insert_obj(brave, "brave_vpn") {
        brave_vpn.insert(String::from("show_button"), json!(false));
        debug!("hid brave VPN button");
    }

    brave.insert(String::from("enable_closing_last_tab"), json!(true));
    brave.insert(String::from("enable_window_closing_confirm"), json!(true));
    brave.insert(String::from("location_bar_is_wide"), json!(true));
    debug!("enabled closing last tab and wide location bar");

    if let Some(new_tab_page) = get_or_insert_obj(brave, "new_tab_page") {
        new_tab_page.insert(String::from("hide_all_widgets"), json!(true));
        new_tab_page.insert(String::from("show_background_image"), json!(true));
        new_tab_page.insert(String::from("show_branded_background_image"), json!(false));
        new_tab_page.insert(String::from("show_brave_news"), json!(false));
        new_tab_page.insert(String::from("show_brave_vpn"), json!(false));
        new_tab_page.insert(String::from("show_clock"), json!(true));
        new_tab_page.insert(String::from("show_rewards"), json!(false));
        new_tab_page.insert(String::from("show_stats"), json!(false));
        new_tab_page.insert(String::from("show_together"), json!(false));
        new_tab_page.insert(String::from("shows_options"), json!(1));
        debug!("hid new tab page widgets");
    }

    brave.insert(String::from("other_search_engines_enabled"), json!(true));
    debug!("enabled other search engines");

    if let Some(rewards) = get_or_insert_obj(brave, "rewards") {
        rewards.insert(String::from("show_brave_rewards_button_in_location_bar"), json!(false));
        debug!("hid brave rewards button");
    }

    if let Some(shields) = get_or_insert_obj(brave, "shields") {
        shields.insert(String::from("advanced_view_enabled"), json!(true));
        shields.insert(String::from("stats_badge_visible"), json!(false));
        debug!("enabled shields advanced view and hid stats badge");
    }

    brave.insert(String::from("show_fullscreen_reminder"), json!(false));
    brave.insert(String::from("show_side_panel_button"), json!(false));
    debug!("hid fullscreen reminder and side panel button");

    if let Some(sidebar) = get_or_insert_obj(brave, "sidebar") {
        sidebar.insert(String::from("hidden_built_in_items"), json!([7]));
        sidebar.insert(String::from("item_added_feedback_bubble_shown_count"), json!(1));
        sidebar.insert(
            String::from("sidebar_items"),
            json!([
                { "built_in_item_type": 1, "type": 0 },
                { "built_in_item_type": 2, "type": 0 },
                { "built_in_item_type": 3, "type": 0 },
                { "built_in_item_type": 4, "type": 0 }
            ])
        );
        sidebar.insert(String::from("sidebar_show_option"), json!(3));
        debug!("hid sidebar items");
    }

    if let Some(tabs) = get_or_insert_obj(brave, "tabs") {
        tabs.insert(String::from("vertical_tabs_collapsed"), json!(false));
        tabs.insert(String::from("vertical_tabs_enabled"), json!(true));
        tabs.insert(String::from("vertical_tabs_expanded_width"), json!(114));
        tabs.insert(String::from("vertical_tabs_floating_enabled"), json!(true));
        tabs.insert(String::from("vertical_tabs_show_title_on_window"), json!(false));
        debug!("enabled vertical tabs");
    }

    brave.insert(String::from("tabs_search_show"), json!(false));
    brave.insert(String::from("webtorrent_enabled"), json!(false));
    info!("disabled webtorrent and hid tabs search");

    if let Some(today) = get_or_insert_obj(brave, "today") {
        today.insert(String::from("should_show_toolbar_button"), json!(false));
        debug!("hid today toolbar button");
    }

    if let Some(browser) = get_or_insert_obj(brave, "browser") {
        browser.insert(String::from("has_seen_welcome_page"), json!(true));
        debug!("marked welcome page as seen");
    }

    if let Some(custom_links) = get_or_insert_obj(prefs_map, "custom_links") {
        custom_links.insert(String::from("initialized"), json!(true));
        custom_links.insert(
            String::from("list"),
            json!([
                 { "isMostVisited": true, "title": "Kagi", "url": "https://kagi.com/" },
                 { "isMostVisited": true, "title": "Claude", "url": "http://claude.com/" },
                 { "isMostVisited": true, "title": "YouTube", "url": "http://youtube.com/" },
                 { "isMostVisited": true, "title": "Gemini", "url": "http://gemini.google.com/" },
                 { "isMostVisited": true, "title": "ChatGPT", "url": "http://chatgpt.com/" },
                 { "isMostVisited": true, "title": "Instagram", "url": "http://instagram.com/" },
                 { "isMostVisited": true, "title": "NextDNS", "url": "http://my.nextdns.com/" }
            ])
        );
    }

    prefs_map.insert(String::from("enable_do_not_track"), json!(true));
    debug!("enabled do not track");

    if let Some(in_product_help) = get_or_insert_obj(prefs_map, "in_product_help") {
        if let Some(new_badge) = get_or_insert_obj(in_product_help, "new_badge") {
            if let Some(compose_nudge) = get_or_insert_obj(new_badge, "ComposeNudge") {
                compose_nudge.insert(String::from("show_count"), json!(0));
            }

            if let Some(compose_proactive_nudge) =
                get_or_insert_obj(new_badge, "ComposeProactiveNudge")
            {
                compose_proactive_nudge.insert(String::from("show_count"), json!(0));
            }
        }

        if let Some(snoozed_feature) = get_or_insert_obj(in_product_help, "snoozed_feature") {
            if let Some(iph_discard_ring) = get_or_insert_obj(snoozed_feature, "IPH_DiscardRing") {
                iph_discard_ring.insert(String::from("is_dismissed"), json!(true));
            }
        }
        debug!("disabled in product help");
    }

    if let Some(ntp) = get_or_insert_obj(prefs_map, "ntp") {
        ntp.insert(String::from("shortcust_visible"), json!(false));
        ntp.insert(String::from("use_most_visited_tiles"), json!(false));
        debug!("hid ntp widgets");
    }

    if let Some(omnibox) = get_or_insert_obj(prefs_map, "omnibox") {
        omnibox.insert(String::from("prevent_url_elisions"), json!(true));
        omnibox.insert(String::from("shown_count_history_scope_promo"), json!(false));
        debug!("disabled omnibox elisions and history scope promo");
    }

    if let Some(search) = get_or_insert_obj(prefs_map, "search") {
        search.insert(String::from("suggest_enabled"), json!(true));
        debug!("enabled search suggestions");
    }

    if let Some(privacy_sandbox) = get_or_insert_obj(prefs_map, "privacy_sandbox") {
        privacy_sandbox.insert(String::from("first_party_sets_enabled"), json!(false));
        if let Some(m1) = get_or_insert_obj(privacy_sandbox, "m1") {
            m1.insert(String::from("ad_measurement_enabled"), json!(false));
            m1.insert(String::from("fledge_enabled"), json!(false));
            m1.insert(String::from("topics_enabled"), json!(false));
        }
        debug!("disabled ad measurement, fledge, and topics");
    }

    let prefs_str = serde_json::to_string(&prefs)?;
    fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))
}

static DISABLED_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    include_str!("../snippets/disabled_features").lines().filter(|line| !line.is_empty()).collect()
});

#[instrument]
fn chrome_feature_state(root: &PathBuf) -> color_eyre::Result<()> {
    let path = root.join("ChromeFeatureState");
    let backup = root.join(format!("ChromeFeatureState-{}", timestamp())).with_extension("bak");

    let _ = fs::copy(&path, &backup);
    info!("backed up brave chrome feature state to {}", backup.display());

    let prefs_str = fs::read_to_string(&path).unwrap_or_default();
    let mut prefs_parsed =
        serde_json::from_str::<Value>(&prefs_str).unwrap_or_else(|_| Value::Object(Map::new()));

    let prefs = prefs_parsed.as_object_mut().context("failed to parse preferences as an object")?;

    let disable_features = prefs
        .entry("disable-features")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .context("failed to get disable-features array")?;

    for feature in DISABLED_FEATURES.iter() {
        let v = json!(feature);
        if !disable_features.contains(&v) {
            disable_features.push(v);
            debug!("added {} to disable-features", feature);
        }
    }

    let prefs_str = serde_json::to_string(&prefs)?;
    fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))?;

    info!("disabled chrome features");
    Ok(())
}
