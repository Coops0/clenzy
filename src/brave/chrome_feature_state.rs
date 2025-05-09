use crate::{logging::success, util::timestamp, ARGS};
use color_eyre::eyre::{ContextCompat, WrapErr};
use serde_json::{json, Map, Value};
use std::{fs, path::Path, sync::LazyLock};
use tracing::{debug, instrument, warn};

static DISABLED_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    include_str!("../../snippets/disabled_brave_features")
        .lines()
        .filter(|line| !line.is_empty())
        .collect()
});

#[instrument]
pub fn chrome_feature_state(root: &Path) -> color_eyre::Result<()> {
    let path = root.join("ChromeFeatureState");
    if !path.exists() {
        debug!(path = %path.display(), "ChromeFeatureState does not exist, creating it");
    }

    if ARGS.get().unwrap().backup && path.exists() {
        let backup = root.join(format!("ChromeFeatureState-{}", timestamp())).with_extension("bak");
        // This is less important to have a backup of, so warn but continue
        match fs::copy(&path, &backup) {
            Ok(_) => {
                success("Backed up Brave feature state file");
                debug!("backup dir: {}", backup.display());
            }
            Err(why) => {
                warn!(err = ?why, path = %path.display(), "Failed to backup Brave feature state file, continuing anyway");
            }
        }
    }

    let prefs_str = fs::read_to_string(&path).unwrap_or_default();
    let mut prefs_parsed =
        serde_json::from_str::<Value>(&prefs_str).unwrap_or_else(|_| Value::Object(Map::new()));

    let prefs = prefs_parsed.as_object_mut().context("failed to parse preferences as an object")?;

    let disable_features = prefs
        .entry("disable-features")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .context("failed to get disable-features array")?;
    let before = disable_features.len();

    for feature in DISABLED_FEATURES.iter() {
        let v = json!(feature);
        if !disable_features.contains(&v) {
            disable_features.push(v);
            debug!("added {} to disabled features", feature);
        }
    }

    debug!("disabled additional {} features", disable_features.len() - before);

    // In case we are creating the file, we need to supplement with the other properties
    let _ = prefs.entry("enable-features").or_insert_with(|| Value::Array(Vec::new()));
    let _ = prefs.entry("force-fieldtrial-params").or_insert_with(|| Value::String(String::new()));
    let _ = prefs
        .entry("force-fieldtrial")
        // ? this is the default in mine
        .or_insert_with(|| Value::String(String::from("*SeedFileTrial/Control_V7")));

    let prefs_str = serde_json::to_string(&prefs)?;
    fs::write(&path, prefs_str)
        .wrap_err_with(|| format!("failed to write preferences to {}", path.display()))?;

    debug!("wrote new chrome preferences");
    Ok(())
}
