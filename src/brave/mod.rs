mod chrome_feature_state;
mod local_state;
mod installations;
mod preferences;
mod profiles;
mod resources;

use crate::browser_profile::BrowserProfile;
use tracing::{debug, debug_span, instrument, warn};
use crate::browsers::Installation;

pub use installations::installations;

#[instrument(level = "debug")]
pub fn debloat(installation: &Installation) -> color_eyre::Result<()> {
    let data_folder =
        if cfg!(target_os = "windows") { installation.data_folder.join("User Data") } else { installation.data_folder.clone() };

    let local_state = local_state::get_local_state(&data_folder)?;

    let profiles = match profiles::try_to_get_profiles(&local_state, &data_folder) {
        Ok(profiles) => {
            debug!(len = %profiles.len(), "Found profiles");
            profiles
        }
        Err(why) => {
            warn!(err = ?why, "Failed to get profiles, falling back to default");
            vec![BrowserProfile::new(String::from("Default"), data_folder.join("Default"))]
        }
    };

    match local_state::update_local_state(local_state, &data_folder) {
        Ok(()) => debug!("Updated local state"),
        Err(why) => warn!(err = ?why, "Failed to update local state")
    }

    match chrome_feature_state::chrome_feature_state(&data_folder) {
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
