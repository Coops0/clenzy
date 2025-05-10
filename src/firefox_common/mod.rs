use crate::{browser_profile::BrowserProfile, browsers::Installation, util::select_profiles, ARGS};
use std::path::Path;
use tracing::{debug, debug_span, instrument, warn};

mod backup;
mod profiles;
mod user_js;

#[instrument(skip(fetch_user_js, additional_snippets), level = "debug")]
pub fn debloat<'a, F>(
    installation: &Installation,
    fetch_user_js: F,
    additional_snippets: &str
) -> color_eyre::Result<Vec<BrowserProfile>>
where
    F: Fn() -> color_eyre::Result<&'a str>
{
    let (defaults, profiles) = profiles::get_profiles(&installation.data_folder)?;
    debug!("Found {} valid profiles", profiles.len());

    if profiles.is_empty() {
        warn!("No {} profiles found in profiles.ini", installation.browser);
        return Ok(Vec::new());
    }

    let profiles = select_profiles(profiles, &(0..defaults).collect::<Vec<_>>(), installation.browser);
    if profiles.is_empty() {
        return Ok(Vec::new());
    }

    for profile in &profiles {
        let span = debug_span!("Debloating profile", profile = %profile);
        let _enter = span.enter();

        if ARGS.get().unwrap().backup {
            if let Err(why) = backup::backup_profile(profile) {
                warn!(err = ?why, "Failed to backup profile {profile}");
                continue;
            }
        }

        if let Err(why) = user_js::install_user_js(profile, &fetch_user_js, additional_snippets) {
            warn!(err = ?why, "Failed to install user.js for profile {profile}");
            continue;
        }

        debug!("Finished debloating profile");
    }

    Ok(profiles)
}
