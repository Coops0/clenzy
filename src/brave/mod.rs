mod chrome_feature_state;
mod local_state;
mod installations;
mod preferences;
mod profiles;
mod resources;

use std::path::Path;
use crate::browser_profile::BrowserProfile;
use tracing::{debug, debug_span, warn};
use crate::browsers::Installation;

pub use installations::installations;

#[allow(clippy::unnecessary_wraps)]
pub fn debloat(installation: &Installation) -> color_eyre::Result<()> {
    for data_folder in &installation.data_folders {
        if let Err(why) = debloat_data_folder(data_folder) {
            warn!(err = ?why, "Failed to debloat data folder: {}", data_folder.display());
        } else {
            debug!(data_folder = %data_folder.display(), "Successfully debloated data folder");
        }
    }

    Ok(())
}


fn debloat_data_folder(data_folder: &Path) -> color_eyre::Result<()> {
    let local_state = local_state::get_local_state(data_folder)?;

    let profiles = match profiles::try_to_get_profiles(data_folder, &local_state) {
        Ok(profiles) => {
            debug!(len = %profiles.len(), "Found profiles");
            profiles
        }
        Err(why) => {
            warn!(err = ?why, "Failed to get profiles, falling back to default");
            vec![BrowserProfile::new(String::from("Default"), data_folder.join("Default"))]
        }
    };

    match local_state::update_local_state(local_state, data_folder) {
        Ok(()) => debug!("Updated brave's local state"),
        Err(why) => warn!(err = ?why, "Failed to brave's update local state")
    }

    match chrome_feature_state::chrome_feature_state(data_folder) {
        Ok(()) => debug!("Updated brave's ChromeFeatureState"),
        Err(why) => warn!(err = ?why, "Failed to update brave's ChromeFeatureState")
    }

    for profile in profiles {
        let span = debug_span!("Debloating brave profile", profile = %profile.name);
        let _enter = span.enter();

        match preferences::preferences(&profile.path) {
            Ok(()) => debug!("Finished debloating brave profile {profile}"),
            Err(why) => warn!(err = ?why, "Failed to debloat preferences for profile {profile}")
        }
    }

    Ok(())
}