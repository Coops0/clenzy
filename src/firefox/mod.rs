mod installations;
mod policies;
pub mod resource;
mod xulstore;

use std::path::Path;
use crate::{browsers::Installation, firefox_common, ARGS};
pub use installations::installations;
use tracing::{debug, debug_span, warn};
use crate::browser_profile::BrowserProfile;

pub fn debloat(installation: &Installation) -> color_eyre::Result<()> {
    let mut custom_overrides = vec![
        include_str!("../../snippets/firefox_common/betterfox_extra"),
        include_str!("../../snippets/firefox/extra"),
    ];

    if ARGS.get().unwrap().vertical_tabs {
        custom_overrides.push(include_str!("../../snippets/firefox/vert_tabs"));
    }

    if ARGS.get().unwrap().search_suggestions {
        custom_overrides.push(include_str!("../../snippets/firefox_common/search_suggestions"));
    }

    let mut found_profile = false;
    for data_folder in &installation.data_folders {
        match debloat_profile(installation, data_folder, &custom_overrides[..]) {
            Err(why) => warn!(err = ?why, "Failed to debloat data folder: {}", data_folder.display()),
            Ok(profiles) => {
                found_profile |= !profiles.is_empty();
                debug!(data_folder = %data_folder.display(), "Successfully debloated data folder");
            }
        }
    }

    if !found_profile {
        warn!("Failed to find any valid profiles in Firefox installation");
    }

    Ok(())
}

fn debloat_profile(installation: &Installation, data_folder: &Path, custom_overrides: &[&str]) -> color_eyre::Result<Vec<BrowserProfile>> {
    let profiles = firefox_common::debloat(
        installation,
        data_folder,
        resource::get_better_fox_user_js()?,
        &custom_overrides.join("\n")
    )?;

    if !ARGS.get().unwrap().vertical_tabs {
        return Ok(profiles);
    }

    for profile in &profiles {
        let span = debug_span!("Updating xulstore", %profile);
        let _enter = span.enter();

        match xulstore::xulstore(&profile.path) {
            Ok(()) => debug!("Updated xulstore.json for {profile}"),
            Err(why) => warn!(err = %why, "Failed to update xulstore.json for {profile}"),
        }
    }

    if !ARGS.get().unwrap().create_policies {
        return Ok(profiles);
    }

    if installation.app_folders.is_empty() {
        warn!("No app folders found for Firefox, skipping creating policies");
        return Ok(profiles);
    }

    for folder in &installation.app_folders {
        if let Err(why) = policies::create_policies_file(folder) {
            warn!(err = %why, "Failed to create policies file in {}", folder.display());
        }
    }

    Ok(profiles)
}
