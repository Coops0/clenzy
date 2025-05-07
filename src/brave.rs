use crate::util::{
    get_or_insert_obj, local_data_base, select_profiles, timestamp, validate_profile_dir,
};
use color_eyre::eyre::{bail, Context, ContextCompat};
use serde_json::{json, Map, Value};
use std::fmt::Display;
use std::path::Path;
use std::{fs, path::PathBuf, sync::LazyLock};
use tracing::{debug, info, info_span, instrument, warn};
use crate::ARGS;

pub fn brave_folder() -> Option<PathBuf> {
    let path = local_data_base()?.join("BraveSoftware").join("Brave-Browser");
    if path.exists() { Some(path) } else { None }
}

pub fn brave_nightly_folder() -> Option<PathBuf> {
    let path = local_data_base()?.join("BraveSoftware").join("Brave-Browser-Nightly");
    if path.exists() { Some(path) } else { None }
}

#[instrument]
pub fn debloat(mut path: PathBuf) -> color_eyre::Result<()> {
    if cfg!(target_os = "windows") {
        path = path.join("User Data")
    }

    let profiles = match try_to_get_profiles(&path) {
        Ok(profiles) => {
            debug!(profiles = %profiles.len(), "Found profiles");
            profiles
        }
        Err(why) => {
            warn!(err = ?why, "Failed to get profiles, falling back to default");
            vec![Profile { name: String::from("Default"), path: path.join("Default") }]
        }
    };

    for profile in profiles {
        let span = info_span!("Debloating profile", profile = %profile.name);
        let _enter = span.enter();

        if let Err(why) = preferences(&profile.path) {
            warn!(err = ?why, "Failed to debloat preferences");
            continue;
        }

        if let Err(why) = chrome_feature_state(&profile.path) {
            warn!(err = ?why, "Failed to debloat chrome feature state");
            continue;
        }

        debug!("Finished debloating profile");
    }

    Ok(())
}

struct Profile {
    name: String,
    path: PathBuf,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[instrument(skip_all)]
fn try_to_get_profiles(root: &Path) -> color_eyre::Result<Vec<Profile>> {
    let local_state_path = root.join("Local State");
    let Value::Object(local_state) =
        serde_json::from_str::<Value>(&fs::read_to_string(local_state_path)?)
            .wrap_err("Failed to read Local State")?
    else {
        bail!("Failed to parse Local State as JSON");
    };

    let profile = local_state
        .get("profile")
        .and_then(Value::as_object)
        .context("Failed to get profile object")?;

    let mut info_cache = profile
        .get("info_cache")
        .and_then(Value::as_object)
        .inspect(|ic| debug!(len = %ic.iter().len(), "initial info_cache object"))
        .context("Failed to get info_cache object")?
        .into_iter()
        .filter_map(|(n, o)| Some((n, o.as_object()?)))
        .filter_map(|(n, o)| {
            let name = o.get("name").and_then(Value::as_str)?.to_owned();
            let path = root.join(n);
            Some(Profile { name, path })
        })
        .filter(|profile| validate_profile_dir(&profile.path))
        .collect::<Vec<_>>();

    let profiles_order = profile
        .get("profile_order")
        .and_then(Value::as_array)
        .map(|orders| {
            orders.iter().filter_map(Value::as_str).collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut profiles = Vec::with_capacity(info_cache.len());
    // Try to keep the order of profiles
    for profile_name in profiles_order {
        if let Some(position) = info_cache.iter().position(|p| p.name == profile_name) {
            let profile = info_cache.remove(position);
            profiles.push(profile);
        } else {
            warn!(profile = %profile_name, "Profile not found in info_cache");
        }
    }

    // Add any remaining profiles that were not in the order array
    profiles.extend(info_cache);

    // We're only making this an error because above we're falling back to using default
    // only if this function returns Err
    if profiles.is_empty() {
        bail!("No profiles found");
    }

    // Have preselected any last active profiles.
    // If there are none, then just select all.
    let selected = profile
        .get("last_active_profiles")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .filter_map(|profile_name| {
                    profiles.iter().position(|prof| prof.name == profile_name)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| (0..profiles.len()).collect());

    let profiles = select_profiles(profiles, &selected);
    if profiles.is_empty() {
        // If they explicitly select no profiles, they don't fallback to default
        return Ok(Vec::new());
    }

    Ok(profiles)
}

macro_rules! s {
    ($s:expr) => {
        String::from($s)
    };
}

#[instrument(skip_all)]
fn preferences(root: &Path) -> color_eyre::Result<()> {
    let path = root.join("Preferences");
    let backup = root.join(format!("Preferences-{}", timestamp())).with_extension("bak");

    fs::copy(&path, &backup)?;
    info!("Backed up brave preferences to {}", backup.display());
    debug!("backup dir: {}", backup.display());

    let prefs_str = fs::read_to_string(&path);
    let Value::Object(mut prefs) = serde_json::from_str::<Value>(&prefs_str?)? else {
        bail!("Failed to parse preferences as JSON");
    };

    if let Some(bookmark_bar) = get_or_insert_obj(&mut prefs, "bookmark_bar") {
        bookmark_bar.insert(s!("show_on_all_tabs"), json!(false));
        bookmark_bar.insert(s!("show_tab_groups"), json!(false));
        debug!("disabled bookmark bar on all tabs and tab groups");
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
        debug!("disabled brave AI chat");
    }

    brave.insert(s!("always_show_bookmark_bar_on_ntp"), json!(true));
    // People will want this disabled by default probably
    brave.insert(s!("autocomplete_enabled"), json!(true));
    debug!("enabled bookmark bar and autocomplete");

    if let Some(brave_ads) = get_or_insert_obj(brave, "brave_ads") {
        brave_ads.insert(s!("should_allow_ads_subdivision_targeting"), json!(false));
        debug!("disabled brave ads");
    }

    if let Some(brave_search_conversation) = get_or_insert_obj(brave, "brave_search_conversion") {
        brave_search_conversation.insert(s!("dismissed"), json!(false));
        debug!("dismissed brave search conversation");
    }

    if let Some(brave_vpn) = get_or_insert_obj(brave, "brave_vpn") {
        brave_vpn.insert(s!("show_button"), json!(false));
        debug!("hid brave VPN button");
    }

    brave.insert(s!("enable_closing_last_tab"), json!(true));
    brave.insert(s!("enable_window_closing_confirm"), json!(true));
    brave.insert(s!("location_bar_is_wide"), json!(true));
    debug!("enabled closing last tab and wide location bar");

    if let Some(new_tab_page) = get_or_insert_obj(brave, "new_tab_page") {
        new_tab_page.insert(s!("hide_all_widgets"), json!(true));
        new_tab_page.insert(s!("show_background_image"), json!(true));
        new_tab_page.insert(s!("show_branded_background_image"), json!(false));
        new_tab_page.insert(s!("show_brave_news"), json!(false));
        new_tab_page.insert(s!("show_brave_vpn"), json!(false));
        new_tab_page.insert(s!("show_clock"), json!(true));
        new_tab_page.insert(s!("show_rewards"), json!(false));
        new_tab_page.insert(s!("show_stats"), json!(false));
        new_tab_page.insert(s!("show_together"), json!(false));
        new_tab_page.insert(s!("shows_options"), json!(1));
        debug!("hid new tab page widgets");
    }

    brave.insert(s!("other_search_engines_enabled"), json!(true));
    debug!("enabled other search engines");

    if let Some(rewards) = get_or_insert_obj(brave, "rewards") {
        rewards.insert(s!("show_brave_rewards_button_in_location_bar"), json!(false));
        debug!("hid brave rewards button");
    }

    if let Some(shields) = get_or_insert_obj(brave, "shields") {
        shields.insert(s!("advanced_view_enabled"), json!(true));
        shields.insert(s!("stats_badge_visible"), json!(false));
        debug!("enabled shields advanced view and hid stats badge");
    }

    brave.insert(s!("show_fullscreen_reminder"), json!(false));
    brave.insert(s!("show_side_panel_button"), json!(false));
    debug!("hid fullscreen reminder and side panel button");

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
            ]),
        );
        sidebar.insert(s!("sidebar_show_option"), json!(3));
        debug!("hid sidebar items");
    }

    if ARGS.get().unwrap().vertical_tabs && let Some(tabs) = get_or_insert_obj(brave, "tabs") {
        tabs.insert(s!("vertical_tabs_collapsed"), json!(false));
        tabs.insert(s!("vertical_tabs_enabled"), json!(true));
        tabs.insert(s!("vertical_tabs_expanded_width"), json!(114));
        tabs.insert(s!("vertical_tabs_floating_enabled"), json!(true));
        tabs.insert(s!("vertical_tabs_show_title_on_window"), json!(false));
        debug!("enabled vertical tabs");
    }

    if let Some(wallet) = get_or_insert_obj(brave, "wallet") {
        wallet.insert(s!("show_wallet_icon_on_toolbar"), json!(false));
        debug!("hid wallet button");
    }

    brave.insert(s!("tabs_search_show"), json!(false));
    brave.insert(s!("webtorrent_enabled"), json!(false));
    debug!("disabled webtorrent and hid tabs search");

    if let Some(today) = get_or_insert_obj(brave, "today") {
        today.insert(s!("should_show_toolbar_button"), json!(false));
        debug!("hid today toolbar button");
    }

    if let Some(browser) = get_or_insert_obj(brave, "browser") {
        browser.insert(s!("has_seen_welcome_page"), json!(true));
        debug!("marked welcome page as seen");
    }

    // -- END BRAVE SECTION --

    if let Some(custom_links) = get_or_insert_obj(&mut prefs, "custom_links") {
        custom_links.insert(s!("initialized"), json!(true));
    }

    prefs.insert(s!("enable_do_not_track"), json!(true));
    debug!("enabled do not track");

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

        if let Some(snoozed_feature) = get_or_insert_obj(in_product_help, "snoozed_feature") {
            if let Some(iph_discard_ring) = get_or_insert_obj(snoozed_feature, "IPH_DiscardRing") {
                iph_discard_ring.insert(s!("is_dismissed"), json!(true));
            }
        }
        debug!("disabled in product help");
    }

    if let Some(ntp) = get_or_insert_obj(&mut prefs, "ntp") {
        ntp.insert(s!("shortcust_visible"), json!(false));
        ntp.insert(s!("use_most_visited_tiles"), json!(false));
        debug!("hid ntp widgets");
    }

    if let Some(omnibox) = get_or_insert_obj(&mut prefs, "omnibox") {
        // show entire URL always
        omnibox.insert(s!("prevent_url_elisions"), json!(true));
        omnibox.insert(s!("shown_count_history_scope_promo"), json!(false));
        debug!("enabled showing full url and history scope promo");
    }

    if let Some(search) = get_or_insert_obj(&mut prefs, "search") {
        search.insert(s!("suggest_enabled"), json!(true));
        debug!("enabled search suggestions");
    }

    if let Some(privacy_sandbox) = get_or_insert_obj(&mut prefs, "privacy_sandbox") {
        privacy_sandbox.insert(s!("first_party_sets_enabled"), json!(false));
        if let Some(m1) = get_or_insert_obj(privacy_sandbox, "m1") {
            m1.insert(s!("ad_measurement_enabled"), json!(false));
            m1.insert(s!("fledge_enabled"), json!(false));
            m1.insert(s!("topics_enabled"), json!(false));
        }
        debug!("disabled ad measurement, fledge, and topics");
    }

    let prefs_str = serde_json::to_string(&prefs)?;
    fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))
}

static DISABLED_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    include_str!("../snippets/disabled_brave_features")
        .lines()
        .filter(|line| !line.is_empty())
        .collect()
});

#[instrument(skip_all)]
fn chrome_feature_state(root: &Path) -> color_eyre::Result<()> {
    let path = root.join("ChromeFeatureState");
    let backup = root.join(format!("ChromeFeatureState-{}", timestamp())).with_extension("bak");

    let _ = fs::copy(&path, &backup);
    info!("Backed up brave chrome feature state");
    debug!("backup dir: {}", backup.display());

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

    debug!("disabled chrome features");
    Ok(())
}
