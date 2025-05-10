mod chrome_feature_state;
mod local_state;
pub mod paths;
mod preferences;
mod profiles;
mod resources;

use crate::browser_profile::BrowserProfile;
use std::path::Path;
use tracing::{debug, debug_span, instrument, warn};

#[instrument(level = "debug")]
pub fn debloat(path: &Path) -> color_eyre::Result<()> {
    let path =
        if cfg!(target_os = "windows") { path.join("User Data") } else { path.to_path_buf() };

    let local_state = local_state::get_local_state(&path)?;

    let profiles = match profiles::try_to_get_profiles(&local_state, &path) {
        Ok(profiles) => {
            debug!(len = %profiles.len(), "Found profiles");
            profiles
        }
        Err(why) => {
            warn!(err = ?why, "Failed to get profiles, falling back to default");
            vec![BrowserProfile::new(String::from("Default"), path.join("Default"))]
        }
    };

    match local_state::update_local_state(local_state, &path) {
        Ok(()) => debug!("Updated local state"),
        Err(why) => warn!(err = ?why, "Failed to update local state")
    }

    match chrome_feature_state::chrome_feature_state(&path) {
        Ok(()) => debug!("Updated ChromeFeatureState"),
        Err(why) => warn!(err = ?why, "Failed to update ChromeFeatureState")
    }

    for profile in profiles {
        let span = debug_span!("Debloating profile", profile = %profile.name);
        let _enter = span.enter();

        match preferences::preferences(&profile.path) {
            Ok(()) => debug!("Finished debloating profile"),
            Err(why) => warn!(err = ?why, "Failed to debloat preferences")
        }
    }

    Ok(())
}
