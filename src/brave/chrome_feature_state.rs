use crate::{brave::{
    resources, resources::{DISABLED_FEATURES, REMOVE_ENABLED_FEATURES}
}, s, util::timestamp};
use color_eyre::eyre::{ContextCompat, WrapErr};
use resources::replace_symbols;
use serde_json::{json, Map, Value};
use std::{fs, path::Path};
use tracing::{debug, warn};
use crate::util::args;
use crate::util::logging::success;

pub fn chrome_feature_state(root: &Path) -> color_eyre::Result<()> {
    let path = root.join("ChromeFeatureState");
    if !path.exists() {
        debug!(path = %path.display(), "ChromeFeatureState does not exist, creating it");
    }

    if args().backup && path.exists() {
        let backup = root.join(format!("ChromeFeatureState-{}", timestamp())).with_extension("bak");
        // This is less important to have a backup of, so warn but continue
        match fs::copy(&path, &backup) {
            Ok(_) => {
                success("Backed up Brave feature state file");
                debug!("Backup dir: {}", backup.display());
            }
            Err(why) => {
                warn!(err = ?why, path = %path.display(), "Failed to backup Brave feature state file, continuing anyway");
            }
        }
    }

    let prefs_str = fs::read_to_string(&path).unwrap_or_default();
    let mut prefs_parsed =
        serde_json::from_str::<Value>(&prefs_str).unwrap_or_else(|_| Value::Object(Map::new()));

    let prefs = prefs_parsed.as_object_mut().wrap_err("failed to parse preferences as an object")?;

    // Both features are seperated by commas
    let mut disable_features = prefs
        .get("enable-features")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split(',')
        .collect::<Vec<_>>();

    let before = disable_features.len();

    for feature in DISABLED_FEATURES.iter() {
        if !disable_features.contains(feature)
            && !disable_features.contains(&replace_symbols(feature).as_str())
        {
            disable_features.push(feature);
        }
    }

    debug!("Disabled additional {} features", disable_features.len() - before);
    prefs.insert(s!("disable-features"), json!(disable_features.join(",")));

    let mut enabled_features = prefs
        .get("enable-features")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split(',')
        .collect::<Vec<_>>();
    let before = enabled_features.len();

    enabled_features.retain(|x| {
        for feature in REMOVE_ENABLED_FEATURES.iter() {
            if x == feature || *x == replace_symbols(feature) {
                return false;
            }
        }

        true
    });

    debug!("Removed {} enabled features", before - enabled_features.len());
    prefs.insert(s!("enable-features"), json!(&enabled_features.join(",")));

    // Just get rid of all of these, most are telemetry or ads.
    // These are IMMEDIATELY restored anyway
    prefs.insert(s!("force-fieldtrial-params"), json!(""));
    prefs.insert(s!("force-fieldtrials"), json!(""));

    let prefs_str = serde_json::to_string(&prefs)?;
    fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))?;

    debug!("Wrote new chrome preferences");
    Ok(())
}
