use crate::util::get_or_insert_obj;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Help;
use serde_json::{json, Map, Value};
use std::path::PathBuf;
use tracing::{debug, info, instrument};

fn base() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return dirs::config_dir();
    #[cfg(target_os = "windows")]
    return dirs::data_dir();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return dirs::config_local_dir();
}

pub fn brave_folder() -> Option<PathBuf> {
    let path = base()?.join("BraveSoftware").join("Brave-Browser");
    if path.exists() { Some(path) } else { None }
}

pub fn brave_nightly_folder() -> Option<PathBuf> {
    let path = base()?.join("BraveSoftware").join("Brave-Browser-Nightly");
    if path.exists() { Some(path) } else { None }
}

#[instrument]
pub fn debloat(path: &PathBuf) -> color_eyre::Result<()> {
    let default = path.join("Default");
    preferences(&default)?;

    Ok(())
}

#[instrument]
fn preferences(root: &PathBuf) -> color_eyre::Result<()> {
    let path = root.join("Preferences");
    let backup = path.with_extension("bak");

    std::fs::copy(&path, &backup)?;
    info!("backed up brave preferences to {}", backup.display());

    let prefs_str = std::fs::read_to_string(&path);
    let mut prefs = serde_json::from_str::<Value>(&prefs_str?)?;

    let prefs_map = prefs.as_object_mut().context("failed to parse preferences as an object")?;
    let brave = prefs_map
        .get_mut("brave")
        .map(Value::as_object_mut)
        .flatten()
        .context("failed to get brave object")?;

    if let Some(ai_chat) = get_or_insert_obj(brave, "ai_chat") {
        ai_chat.insert(String::from("autocomplete_provider_enabled"), json!(false));
        ai_chat.insert(String::from("context_menu_enabled"), json!(false));
        ai_chat.insert(String::from("show_toolbar_button"), json!(false));
        ai_chat.insert(String::from("storage_enabled"), json!(false));
        ai_chat.insert(String::from("tab_organization_enabled"), json!(false));
        debug!("disabled brave AI chat");
    }

    if let Some(brave_ads) = get_or_insert_obj(brave, "brave_ads") {
        brave_ads
            .insert(String::from("should_allow_ads_subdivision_targeting"), json!(false));
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

    if let Some(rewards) = get_or_insert_obj(brave, "rewards") {
        rewards
            .insert(String::from("show_brave_rewards_button_in_location_bar"), json!(false));
        debug!("hid brave rewards button");
    }

    if let Some(tabs) = get_or_insert_obj(brave, "tabs") {
        tabs.insert(String::from("vertical_tabs_collapsed"), json!(false));
        tabs.insert(String::from("vertical_tabs_enabled"), json!(true));
        tabs.insert(String::from("vertical_tabs_expanded_width"), json!(114));
        tabs.insert(String::from("vertical_tabs_floating_enabled"), json!(true));
        tabs.insert(String::from("vertical_tabs_show_title_on_window"), json!(false));
        debug!("enabled vertical tabs");
    }

    // todo we could force kagi default search here

    brave.insert(String::from("webtorrent_enabled"), json!(false));
    brave.insert(String::from("enable_do_not_track"), json!(true));
    info!("disabled webtorrent and enabled do not track");

    if let Some(in_product_help) = get_or_insert_obj(brave, "in_product_help") {
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

    if let Some(ntp) = get_or_insert_obj(brave, "ntp") {
        ntp.insert(String::from("shortcust_visible"), json!(false));
        ntp.insert(String::from("use_most_visited_tiles"), json!(false));
        debug!("hid ntp widgets");
    }

    if let Some(privacy_sandbox) = get_or_insert_obj(brave, "privacy_sandbox") {
        privacy_sandbox.insert(String::from("first_party_sets_enabled"), json!(false));
        if let Some(m1) = get_or_insert_obj(privacy_sandbox, "m1") {
            m1.insert(String::from("ad_measurement_enabled"), json!(false));
            m1.insert(String::from("fledge_enabled"), json!(false));
            m1.insert(String::from("topics_enabled"), json!(false));
        }
        debug!("disabled ad measurement, fledge, and topics");
    }

    let prefs_str = serde_json::to_string(&prefs)?;
    std::fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))
}

const DISABLED_FEATURES: [&str; 40] = [
    "AIChat",
    "AIChatContextMenuRewriteInPlace",
    "AIChatFirst",
    "AIPromptAPIForWebPlatform",
    "AIPromptAPIMultimodalInput",
    "AIRewriter",
    "AIRewriterAPI",
    "AISummarizationAPI",
    "AIWriterAPI",
    "AiSettingsPageEnterpriseDisabledUi",
    "AllowedToFallbackToCustomNotificationAd",
    "BraveAdblockDefault1pBlocking<Default1pBlockingStudy",
    "BraveCleanupSessionCookiesOnSessionRestore<BraveCleanupSessionCookiesOnSessionRestore",
    "BraveEnableAutoTranslate<BraveAutoTranslateStudy",
    "BraveNTPSuperReferralWallpaperName",
    "BraveNewsCardPeek",
    "BraveNewsFeedUpdate",
    "BraveRewardsAllowSelfCustodyProviders",
    "BraveRewardsAllowUnsupportedWalletProviders",
    "BraveRewardsAnimatedBackground",
    "BraveRewardsGemini",
    "BraveRewardsNewRewardsUI",
    "BraveRewardsPlatformCreatorDetection",
    "BraveRewardsVerboseLogging",
    "BraveShowStrictFingerprintingMode<BraveAggressiveModeRetirementExperiment",
    "BraveWalletAnkrBalances",
    "BraveWalletBitcoin",
    "BraveWalletTransactionSimulations",
    "BraveWalletZCash",
    "ClampPlatformVersionClientHint<ClampPlatformVersionClientHint",
    "CosmeticFilterSyncLoad",
    "CryptoWalletsForNewInstallsFeature",
    "CustomNotificationAds",
    "CustomSiteDistillerScripts",
    "EnableDiscountInfoApi",
    "NativeBraveWallet",
    "OpenAIChatFromBraveSearch",
    "PageContentRefine",
    "PageContextEnabledInitially",
    "PrivacyGuideAiSettings",
];

#[instrument]
fn chrome_feature_state(root: &PathBuf) -> color_eyre::Result<()> {
    let path = root.join("ChromeFeatureState");
    let backup = path.with_extension("bak");

    let _ = std::fs::copy(&path, &backup);
    info!("backed up brave chrome feature state to {}", backup.display());

    let prefs_str = std::fs::read_to_string(&path).unwrap_or_default();
    let mut prefs_parsed =
        serde_json::from_str::<Value>(&prefs_str).unwrap_or_else(|_| Value::Object(Map::new()));

    let prefs = prefs_parsed.as_object_mut().context("failed to parse preferences as an object")?;

    let disable_features = prefs
        .entry("disable-features")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .context("failed to get disable-features array")?;

    for feature in DISABLED_FEATURES {
        let v = json!(feature);
        if !disable_features.contains(&v) {
            disable_features.push(v);
            debug!("added {} to disable-features", feature);
        }
    }

    let prefs_str = serde_json::to_string(&prefs)?;
    std::fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))?;

    info!("disabled chrome features");
    Ok(())
}
