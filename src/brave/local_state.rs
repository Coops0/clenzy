use crate::{brave::resources::REMOVE_ENABLED_LAB_FEATURES, s, util::get_or_insert_obj, ARGS};
use color_eyre::eyre::{bail, ContextCompat, WrapErr};
use serde_json::{json, Map, Value};
use std::{fs, path::Path};
use tracing::debug;
use crate::util::args;

pub fn get_local_state(root: &Path) -> color_eyre::Result<Map<String, Value>> {
    let local_state_path = root.join("Local State");
    let local_state_str =
        fs::read_to_string(&local_state_path).wrap_err("Failed to read Local State")?;

    let Value::Object(local_state) =
        serde_json::from_str::<Value>(&local_state_str).wrap_err("Failed to parse Local State")?
    else {
        bail!("Failed to cast Local State to object");
    };

    Ok(local_state)
}

pub fn update_local_state(
    mut local_state: Map<String, Value>,
    root: &Path
) -> color_eyre::Result<()> {
    let brave = local_state
        .get_mut("brave")
        .and_then(Value::as_object_mut)
        .context("Failed to get brave object")?;

    if let Some(ai_chat) = get_or_insert_obj(brave, "ai_chat") {
        ai_chat.insert(s!("p3a_last_premium_status"), json!(false));
    }

    if let Some(brave_ads) = get_or_insert_obj(brave, "brave_ads") {
        brave_ads.insert(s!("enabled_last_profile"), json!(false));
    }

    if let Some(brave_search_conversion) = get_or_insert_obj(brave, "brave_search_conversion") {
        if let Some(action_statuses) = get_or_insert_obj(brave_search_conversion, "action_statuses")
            && let Some(banner_d) = get_or_insert_obj(action_statuses, "banner_d")
        {
            banner_d.insert(s!("shown"), json!(true));
        }

        brave_search_conversion.insert(s!("already_churned"), json!(true));
        brave_search_conversion.insert(s!("default_changed"), json!(true));
    }

    brave.insert(
        s!("enable_search_suggestions_by_default"),
        json!(args().search_suggestions)
    );

    if let Some(p3a) = get_or_insert_obj(brave, "p3a") {
        p3a.insert(s!("enabled"), json!(false));
        p3a.insert(s!("notice_acknowledged"), json!(true));
    }

    if let Some(referral) = get_or_insert_obj(brave, "referral") {
        // FIXME This is my default but I need to check if I can disable both of these
        referral.insert(s!("initialization"), json!(true));
        referral.insert(s!("promo_code"), json!("BRV001"));
    }

    let browser = local_state
        .get_mut("browser")
        .and_then(Value::as_object_mut)
        .context("Failed to get browser object")?;

    if let Some(enabled_lab_features) =
        browser.get_mut("enabled_lab_features").and_then(Value::as_array_mut)
    {
        let before = enabled_lab_features.len();
        enabled_lab_features.retain(|feature| {
            let Some(s) = feature.as_str() else {
                return true;
            };

            // Return true to keep if the feature is not in the blacklist
            !REMOVE_ENABLED_LAB_FEATURES.contains(&s)
        });
        debug!(before = %before, after = %enabled_lab_features.len(), "Removed {} enabled lab features", before - enabled_lab_features.len());
    }

    browser.insert(s!("default_browser_infobar_declined_count"), json!(9999));

    fs::write(root.join("Local State"), serde_json::to_string(&local_state)?)
        .wrap_err("Failed to write Local State")
}
