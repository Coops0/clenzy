use crate::ARGS;
use chrono::{Datelike, Timelike};
use color_eyre::eyre::{bail, Context};
use serde_json::{Map, Value};
use std::{
    fmt::Display, fs, fs::{DirEntry, File}, io, io::{Read, Write}, path::{Path, PathBuf}
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tracing::{debug, instrument, warn};
use zip::{write::SimpleFileOptions, ZipWriter};

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
    if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        dirs::data_dir()
    } else {
        dirs::home_dir()
    }
}

pub fn local_data_base() -> Option<PathBuf> {
    if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        dirs::data_local_dir()
    } else {
        dirs::config_local_dir()
    }
}

// 202501192003
pub fn timestamp() -> String {
    let now = chrono::Local::now();
    format!(
        "{:04}{:02}{:02}{:02}{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute()
    )
}

pub const DEFAULT_FIREFOX_SKIP: &[&str] = &[
    "storage",
    "security_state",
    "minidumps",
    "gmp-widevinecdm",
    "gmp-gmpopenh264",
    "extensions",
    "crashes",
    "sessionstore-logs",
    "saved-telemetry-pings",
    "domain_to_categories.sqlite",
    "favicons.sqlite",
    "suggest.sqlite-wal",
    "places.suggest.sqlite-wal",
    "favicons.sqlite-wal",
    "suggest.sqlite-wal"
];

#[instrument(skip_all)]
pub fn add_to_archive(
    zip: &mut ZipWriter<File>,
    entry: io::Result<DirEntry>,
    prefix: &Path,
    options: &SimpleFileOptions,
    skip: &[&str]
) -> color_eyre::Result<()> {
    let entry = entry?;
    let name = entry.file_name().into_string().unwrap_or_default();
    if name.is_empty() {
        bail!("Entry name is empty");
    }

    if skip.iter().any(|s| name.contains(s)) {
        return Ok(());
    }

    let abs_path = entry.path();
    let path = abs_path.strip_prefix(prefix).unwrap_or(&abs_path);

    let r = if entry.file_type()?.is_dir() {
        add_dir_to_archive(zip, &abs_path, path, prefix, options, skip)
    } else {
        add_file_to_archive(zip, &abs_path, path, options)
    };

    if let Err(why) = r {
        warn!(err = ?why, "Failed to add entry to archive");
    }

    Ok(())
}

#[instrument(skip(zip, abs_path, options, skip))]
fn add_dir_to_archive(
    zip: &mut ZipWriter<File>,
    abs_path: &Path,
    path: &Path,
    prefix: &Path,
    options: &SimpleFileOptions,
    skip: &[&str]
) -> color_eyre::Result<()> {
    zip.add_directory(path.display().to_string(), *options)?;
    let entries = fs::read_dir(abs_path)?;

    for entry in entries {
        if let Err(why) = add_to_archive(zip, entry, prefix, options, skip) {
            warn!(err = ?why, "Failed to add entry to archive (nested)");
        }
    }
    Ok(())
}

#[instrument(skip(zip, options, path))]
fn add_file_to_archive(
    zip: &mut ZipWriter<File>,
    abs_path: &Path,
    path: &Path,
    options: &SimpleFileOptions
) -> color_eyre::Result<()> {
    zip.start_file(path.display(), *options)?;

    let mut file = File::open(abs_path).wrap_err("Failed to open file")?;
    let mut buffer = [0; 4096];

    loop {
        let b = file.read(&mut buffer);
        if matches!(b, Ok(0) | Err(_)) {
            break;
        }

        zip.write_all(&buffer)?;
    }

    zip.flush()?;
    Ok(())
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

    // If no files or only times.json (on firefix)
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
pub fn warn_if_process_is_running(system: &mut System, name: &str) -> bool {
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

    let detected = !running_instances.is_empty();
    if detected {
        warn!(
            instances = running_instances.join(", "),
            "Please close all instances before debloating"
        );
    }

    detected
}
