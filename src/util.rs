// pub static CONFIG_DIRS: LazyLock<HashSet<PathBuf>> = LazyLock::new(|| {
//     [dirs::config_dir(), dirs::config_local_dir(), dirs::data_dir(), dirs::data_local_dir()]
//         .into_iter()
//         .flatten()
//         .collect::<HashSet<PathBuf>>()
// });

use chrono::{Datelike, Timelike};
use color_eyre::eyre::{bail, Context};
use serde_json::{Map, Value};
use std::{
    fs, fs::{DirEntry, File}, io, io::{Read, Write}, path::{Path, PathBuf}
};
use tracing::{debug, instrument, warn};
use zip::{write::SimpleFileOptions, ZipWriter};

pub static mut BROWSER_FOUND: bool = false;

#[macro_export]
macro_rules! try_browser {
    ($browser:expr, $path:path, $debloat:path) => {
        if let Some(p) = $path() {
            if inquire::prompt_confirmation(format!(
                "Found {} at {}, continue?",
                $browser,
                p.display()
            ))
            .unwrap_or_default()
            {
                unsafe {
                    util::BROWSER_FOUND = true;
                }

                $debloat(p)?;
                tracing::info!("Debloated {}", $browser);
            }
        }
    };
}

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

pub fn any_browser_found() -> bool {
    unsafe { BROWSER_FOUND }
}

pub fn roaming_data_base() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        dirs::data_dir()
    } else if cfg!(target_os = "windows") {
        return dirs::data_dir();
    } else {
        return dirs::config_dir();
    }
}

pub fn local_data_base() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        dirs::data_local_dir()
    } else if cfg!(target_os = "windows") {
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

#[instrument(skip(zip, options))]
pub fn add_to_archive(
    zip: &mut ZipWriter<File>,
    entry: io::Result<DirEntry>,
    prefix: &PathBuf,
    options: &SimpleFileOptions,
    skip: &[&str]
) -> color_eyre::Result<()> {
    let entry = entry?;
    let name = entry.file_name().into_string().unwrap_or_default();
    if name.is_empty() {
        bail!("Entry name is empty");
    }

    if skip.iter().any(|s| name.contains(s)) {
        debug!(name, "Skipping entry");
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

#[instrument(skip(zip, options))]
fn add_dir_to_archive(
    zip: &mut ZipWriter<File>,
    abs_path: &PathBuf,
    path: &Path,
    prefix: &PathBuf,
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

#[instrument(skip(zip, options))]
fn add_file_to_archive(
    zip: &mut ZipWriter<File>,
    abs_path: &PathBuf,
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
