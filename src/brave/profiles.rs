use std::path::Path;
use crate::{
    browser_profile::BrowserProfile, util::{select_profiles, validate_profile_dir}
};
use color_eyre::eyre::{bail, ContextCompat};
use serde_json::{Map, Value};
use tracing::debug;
use crate::brave::Brave;

pub fn try_to_get_profiles(
    data_folder: &Path,
    local_state: &Map<String, Value>
) -> color_eyre::Result<Vec<BrowserProfile>> {
    let profile = local_state
        .get("profile")
        .and_then(Value::as_object)
        .context("Failed to get profile object")?;

    let mut info_cache = profile
        .get("info_cache")
        .and_then(Value::as_object)
        .inspect(|ic| debug!(len = %ic.iter().len(), "Initial info_cache object"))
        .context("Failed to get info_cache object")?
        .into_iter()
        .filter_map(|(n, o)| Some((n, o.as_object()?)))
        .filter_map(|(n, o)| {
            let name = o.get("name").and_then(Value::as_str)?.to_owned();
            let path = data_folder.join(n);
            Some(BrowserProfile::new(name, path))
        })
        .filter(|profile| validate_profile_dir(&profile.path))
        .collect::<Vec<_>>();

    let before_dedup = info_cache.len();
    info_cache.dedup_by(|first, second| first.path == second.path);
    debug!(before = %before_dedup, after = %info_cache.len(), "Deduplicated profiles");

    let profiles_order = profile
        .get("profile_order")
        .and_then(Value::as_array)
        .map(|orders| orders.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut profiles = Vec::with_capacity(info_cache.len());
    // Try to keep the order of profiles
    for profile_name in profiles_order {
        if let Some(position) = info_cache.iter().position(|p| p.name == profile_name) {
            let profile = info_cache.remove(position);
            profiles.push(profile);
        } else {
            debug!(profile = %profile_name, "Profile not found in info_cache");
        }
    }

    // Add any remaining profiles that were not in the order array
    profiles.extend(info_cache);

    // We're raising an error because above we're falling back to using default
    // only if this function returns Err
    if profiles.is_empty() {
        bail!("No profiles found");
    }

    // Have preselected any last active profiles.
    // If there are none, then just select all.
    let selected = profile.get("last_active_profiles").and_then(Value::as_array).map_or_else(
        || (0..profiles.len()).collect(),
        |a| {
            a.iter()
                .filter_map(Value::as_str)
                .filter_map(|profile_name| {
                    profiles.iter().position(|prof| prof.name == profile_name)
                })
                .collect::<Vec<_>>()
        }
    );

    let profiles = select_profiles::<_, Brave>(profiles, &selected);
    if profiles.is_empty() {
        // If they explicitly select no profiles, then don't fallback to default
        return Ok(Vec::new());
    }

    Ok(profiles)
}
