mod installations;
mod policies;
pub mod resource;
mod xulstore;
pub mod common;

use crate::browser::Browser;
use installations::installations;
use std::path::Path;
use tracing::{debug, debug_span, warn};
use crate::browser::installation::Installation;
use crate::browser::profile::BrowserProfile;
use crate::util::args;

pub struct Firefox;

impl Browser for Firefox {
    fn name() -> &'static str {
        "Firefox"
    }

    fn installations() -> Vec<Installation> {
        installations()
    }

    fn fetch_resources() -> Option<fn() -> color_eyre::Result<&'static str>> {
        Some(resource::get_better_fox_user_js)
    }

    fn debloat(installation: &Installation) -> color_eyre::Result<()> {
        debloat(installation);
        Ok(())
    }
}

#[allow(clippy::unnecessary_wraps)]
pub fn debloat(installation: &Installation) {
    let mut custom_overrides = vec![
        include_str!("../../snippets/firefox_common/betterfox_extra.js"),
        include_str!("../../snippets/firefox/extra.js"),
    ];

    if args().vertical_tabs {
        custom_overrides.push(include_str!("../../snippets/firefox/vert_tabs.js"));
    }

    if args().search_suggestions {
        custom_overrides.push(include_str!("../../snippets/firefox_common/search_suggestions.js"));
    }

    let mut found_profile = false;
    for data_folder in &installation.data_folders {
        match debloat_profile(installation, data_folder, &custom_overrides[..]) {
            Err(why) => {
                warn!(err = ?why, "Failed to debloat data folder: {}", data_folder.display());
            }
            Ok(profiles) => {
                found_profile |= !profiles.is_empty();
                debug!(data_folder = %data_folder.display(), "Successfully debloated data folder");
            }
        }
    }

    if !found_profile {
        warn!("Failed to find any valid profiles in Firefox installation");
    }
}

fn debloat_profile(
    installation: &Installation,
    data_folder: &Path,
    custom_overrides: &[&str]
) -> color_eyre::Result<Vec<BrowserProfile>> {
    let profiles = common::debloat::<Firefox>(
        data_folder,
        resource::get_better_fox_user_js()?,
        &custom_overrides.join("\n")
    )?;

    if !args().vertical_tabs {
        return Ok(profiles);
    }

    for profile in &profiles {
        let span = debug_span!("Updating xulstore", %profile);
        let _enter = span.enter();

        match xulstore::xulstore(&profile.path) {
            Ok(()) => debug!("Updated xulstore.json for {profile}"),
            Err(why) => warn!(err = %why, "Failed to update xulstore.json for {profile}")
        }
    }

    if !args().create_policies {
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
