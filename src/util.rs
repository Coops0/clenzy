// pub static CONFIG_DIRS: LazyLock<HashSet<PathBuf>> = LazyLock::new(|| {
//     [dirs::config_dir(), dirs::config_local_dir(), dirs::data_dir(), dirs::data_local_dir()]
//         .into_iter()
//         .flatten()
//         .collect::<HashSet<PathBuf>>()
// });

use chrono::{Datelike, Timelike};
use color_eyre::eyre::{bail, Context};
use indicatif::ProgressStyle;
use serde_json::{Map, Value};
use std::{
    fs,
    fs::{DirEntry, File},
    io,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tracing::{debug, instrument, warn};
use ureq::http::header::CONTENT_LENGTH;
use zip::{write::SimpleFileOptions, ZipWriter};

#[instrument(skip(map))]
pub fn get_or_insert_obj<'a>(
    map: &'a mut Map<String, Value>,
    key: &str,
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
    "suggest.sqlite-wal",
];

#[instrument(skip_all)]
pub fn add_to_archive(
    zip: &mut ZipWriter<File>,
    entry: io::Result<DirEntry>,
    prefix: &Path,
    options: &SimpleFileOptions,
    skip: &[&str],
    pb: &indicatif::ProgressBar,
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
        add_dir_to_archive(zip, &abs_path, path, prefix, options, skip, pb)
    } else {
        add_file_to_archive(zip, &abs_path, path, options)
    };

    pb.inc(1);

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
    skip: &[&str],
    pb: &indicatif::ProgressBar,
) -> color_eyre::Result<()> {
    zip.add_directory(path.display().to_string(), *options)?;
    let entries = fs::read_dir(abs_path)?.collect::<Vec<_>>();
    pb.inc_length(entries.len() as u64);
    
    for entry in entries {
        if let Err(why) = add_to_archive(zip, entry, prefix, options, skip, pb) {
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
    options: &SimpleFileOptions,
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
pub fn fetch_text_with_pb(name: &str, url: &str) -> color_eyre::Result<String> {
    let bar = indicatif::ProgressBar::no_length()
        .with_style(ProgressStyle::default_spinner().template("{spinner} {msg:.cyan}")?)
        .with_message(format!("Downloading {name}"));

    let res = ureq::get(url).call().wrap_err_with(|| format!("Failed to download {name}"))?;

    if let Some(length) =
        res.headers().get(CONTENT_LENGTH).and_then(|l| l.to_str().ok()).and_then(|l| l.parse().ok())
    {
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:27}] {bytes:>9}/{total_bytes:9}  {bytes_per_sec} {elapsed:>4}/{eta:4} - {msg:.cyan}")?
                .progress_chars("=> "));
        bar.set_length(length);
    }

    let mut reader = bar.wrap_read(res.into_body().into_reader());
    let mut s = String::new();
    reader.read_to_string(&mut s).wrap_err_with(|| format!("Failed to read {name} to string"))?;

    Ok(s)
}
