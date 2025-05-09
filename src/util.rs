use crate::{firefox, zen, ARGS};
use color_eyre::eyre::Context;
use serde_json::{Map, Value};
use std::{
    fmt::Display, fs, io::{stdin, Read}, path::{Path, PathBuf}, process
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tracing::{debug, debug_span, info, instrument, warn};
use crate::browsers::{Browser, Installation};

#[instrument(skip(map), level = "debug")]
pub fn get_or_insert_obj<'a>(
    map: &'a mut Map<String, Value>,
    key: &str
) -> Option<&'a mut Map<String, Value>> {
    let ret = map
        .entry(key.to_owned())
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

pub fn roaming_data_base() -> PathBuf {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        dirs::data_dir().unwrap_or_default()
    } else {
        dirs::home_dir().unwrap_or_default()
    }
}

pub fn local_data_base() -> PathBuf {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        dirs::data_local_dir().unwrap_or_default()
    } else {
        dirs::config_local_dir().unwrap_or_default()
    }
}

pub fn snap_base() -> PathBuf {
    if cfg!(target_os = "linux") {
        dirs::home_dir().unwrap_or_default().join("snap")
    } else {
        PathBuf::default()
    }
}

pub fn flatpak_base() -> PathBuf {
    if cfg!(target_os = "linux") {
        // Is this the only location?
        dirs::home_dir().unwrap_or_default().join(".var").join("app")
    } else {
        PathBuf::new()
    }
}

// 202501192003
pub fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d%H%M").to_string()
}

#[instrument(level = "debug")]
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
        debug!(path = %profile.display(), "Profile does not exist");
        return false;
    }

    let children = match fs::read_dir(profile) {
        Ok(c) => c,
        Err(why) => {
            debug!(path = %profile.display(), err = %why, "Failed to read profile directory");
            return false;
        }
    };

    let children = children
        .into_iter()
        .filter_map(Result::ok)
        .filter(|c| c.file_type().map(|f| f.is_file() || f.is_dir()).unwrap_or(false))
        .count();

    // If no files or only times.json (on Firefox)
    if children <= 3 {
        return false;
    }

    true
}

pub fn select_profiles<P: Display>(mut profiles: Vec<P>, selected: &[usize], browser: Browser) -> Vec<P> {
    if ARGS.get().unwrap().auto_confirm {
        profiles
    } else if profiles.len() == 1 {
        vec![profiles.remove(0)]
    } else {
        inquire::MultiSelect::new(&format!("Which profiles to debloat for {browser}?"), profiles)
            .with_default(selected)
            .prompt()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>()
    }
}

#[instrument(skip(system), level = "debug")]
fn get_matching_running_processes(system: &mut System, name: &str) -> String {
    let lower_name = name.to_lowercase();
    system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::default()));
    let processes = system.processes();

    debug!("Found {} processes total", processes.len());

    let running_instances = processes
        .values()
        .filter_map(|p| {
            let name = p.name().to_str()?;
            name.to_lowercase().contains(&lower_name).then_some(name)
        })
        .collect::<Vec<_>>();

    running_instances.join(", ")
}

#[instrument(skip(system), level = "debug")]
pub fn check_if_running(system: &mut System, browser: Browser) {
    if ARGS.get().unwrap().auto_confirm {
        return;
    }

    let processes = get_matching_running_processes(system, format!("{browser}").as_str());
    if processes.is_empty() {
        return;
    }

    warn!("Please close all instances before debloating ({processes})");
    info!("Press any key to continue");
    let _ = stdin().read_exact(&mut [0_u8]);

    let processes = get_matching_running_processes(system, format!("{browser}").as_str());
    if processes.is_empty() {
        return;
    }

    warn!("Some processes are still running ({processes})");

    // We don't need to check for auto confirm since it's checked at the start of the function
    let should_continue =
        inquire::prompt_confirmation("Continue anyway? (y/n)").expect("User killed program");

    if !should_continue {
        process::exit(0);
    }
}

pub fn check_and_fetch_resources(browsers: &[Installation]) {
    todo!();
    // if browsers.iter().any(|b| b.name.contains("Firefox")) {
    //     start_fetch_resource("Betterfox User.js", firefox::resource::get_better_fox_user_js);
    // }
    // if browsers.iter().any(|b| b.name.contains("Zen")) {
    //     start_fetch_resource("Better Zen user.js", zen::resource::get_better_zen_user_js);
    // }
}

fn start_fetch_resource<F, O>(name: &'static str, f: F)
where
    F: Fn() -> color_eyre::Result<O> + Send + 'static
{
    std::thread::spawn(move || {
        let span = debug_span!("fetching resource", name);
        let _enter = span.enter();

        match f() {
            Ok(_) => debug!("Fetched resource"),
            Err(why) => warn!(err = ?why, "Failed to fetch resource")
        }
    });
}

#[macro_export]
macro_rules! s {
    ($s:expr) => {
        String::from($s)
    };
}
