use crate::{browser_profile::BrowserProfile, util::validate_profile_dir};
use color_eyre::eyre::WrapErr;
use ini::Ini;
use std::{fs, path::Path};
use tracing::debug;

// Returns the number of default profiles and a vector of all profiles
pub fn get_profiles(path: &Path) -> color_eyre::Result<(usize, Vec<BrowserProfile>)> {
    let profiles_str =
        fs::read_to_string(path.join("profiles.ini")).wrap_err("Failed to read profiles.ini")?;
    let profiles_doc =
        Ini::load_from_str(&profiles_str).wrap_err("Failed to parse profiles.ini")?;
    drop(profiles_str);

    debug!(len = %profiles_doc.len(), "Profiles ini read");

    let mut profiles = profiles_doc
        .iter()
        .filter_map(|(_, prop)| {
            Some((
                prop.get("Name")?,
                prop.get("Path")?,
                prop.get("Default").and_then(|d| d.parse::<u8>().ok()).unwrap_or_default() == 1
            ))
        })
        .collect::<Vec<_>>();

    let before_dedup = profiles.len();
    profiles.dedup_by(|(_, first_path, _), (_, second_path, _)| first_path == second_path);
    debug!(before = %before_dedup, after = %profiles.len(), "Deduplicated profiles");

    // Split into default and non-default profiles
    let profiles: (Vec<_>, Vec<_>) =
        profiles.into_iter().partition(|(_, _, is_default)| *is_default);
    let defaults = profiles.0.len();

    debug!(len = %defaults, "Default profiles");

    // Make sure defaults are first
    let profiles = <[_; 2]>::from(profiles)
        .concat()
        .into_iter()
        .map(|(name, profile_path, _)| {
            BrowserProfile::new(String::from(name), path.join(profile_path))
        })
        .filter(|profile| validate_profile_dir(&profile.path))
        .collect::<Vec<_>>();

    debug!("Found {} valid profiles", profiles.len() + defaults);
    Ok((defaults, profiles))
}
