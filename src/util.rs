use crate::ARGS;
use color_eyre::eyre::Context;
use serde_json::{Map, Value};
use std::{
    fmt::Display, fs, path::{Path, PathBuf}
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tracing::{debug, instrument, warn};

#[instrument(skip(map))]
pub fn get_or_insert_obj<'a>(
    map: &'a mut Map<String, Value>,
    key: &str
) -> Option<&'a mut Map<String, Value>> {
    let ret = map
        .entry(key.to_string())
        .or_insert_with(|| {
            debug!("Inserting");
            Value::Object(serde_json::Map::new())
        })
        .as_object_mut();

    if ret.is_none() {
        debug!("Failed to cast to object");
    }

    ret
}

pub fn roaming_data_base() -> Option<PathBuf> {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        dirs::data_dir()
    } else {
        dirs::home_dir()
    }
}

pub fn local_data_base() -> Option<PathBuf> {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        dirs::data_local_dir()
    } else {
        dirs::config_local_dir()
    }
}

// 202501192003
pub fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d%H%M").to_string()
}

#[instrument]
pub fn fetch_text(name: &str, url: &str) -> color_eyre::Result<String> {
    ureq::get(url)
        .call()
        .wrap_err_with(|| format!("Failed to request {name}"))?
        .into_body()
        .read_to_string()
        .wrap_err_with(|| format!("Failed to read {name} to string"))
}

pub fn validate_profile_dir(profile: &Path) -> bool {
    if !profile.exists() {
        warn!(path = %profile.display(), "Profile does not exist");
        return false;
    }

    let children = match fs::read_dir(profile) {
        Ok(c) => c.count(),
        Err(why) => {
            warn!(path = %profile.display(), err = %why, "Failed to read profile directory");
            return false;
        }
    };

    // If no files or only times.json (on Firefox)
    if children < 2 {
        return false;
    }

    true
}

pub fn select_profiles<P: Display>(mut profiles: Vec<P>, selected: &[usize]) -> Vec<P> {
    if ARGS.get().unwrap().auto_confirm {
        profiles
    } else if profiles.len() == 1 {
        vec![profiles.remove(0)]
    } else {
        inquire::MultiSelect::new("Which profiles to debloat?", profiles)
            .with_default(selected)
            .prompt()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>()
    }
}

#[instrument(skip(system))]
pub fn get_matching_running_processes(system: &mut System, name: &str) -> String {
    let lower_name = name.to_lowercase();
    system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::default()));
    let processes = system.processes();

    debug!("found {} processes total", processes.len());

    let running_instances = processes
        .values()
        .filter_map(|p| {
            let name = p.name().to_str()?;
            if name.to_lowercase().contains(&lower_name) { Some(name) } else { None }
        })
        .collect::<Vec<_>>();

    running_instances.join(", ")
}
