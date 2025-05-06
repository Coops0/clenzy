use crate::util::{add_to_archive, roaming_data_base, timestamp, DEFAULT_FIREFOX_SKIP};
use color_eyre::eyre::{Context, ContextCompat};
use fs::File;
use indicatif::ProgressStyle;
use std::{
    fmt::Display, fs, io::Read, path::{Path, PathBuf}, sync::OnceLock
};
use tracing::{info_span, instrument, warn};
use ureq::http::header::CONTENT_LENGTH;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

static BETTER_FOX_USER_JS: OnceLock<String> = OnceLock::new();
fn get_better_fox_user_js() -> color_eyre::Result<&'static str> {
    if let Some(s) = BETTER_FOX_USER_JS.get() {
        return Ok(s);
    }

    let bar = indicatif::ProgressBar::no_length()
        .with_style(ProgressStyle::default_spinner().template("{spinner} {msg:.cyan}")?)
        .with_message("Downloading Betterfox user.js");

    let res = ureq::get("https://raw.githubusercontent.com/yokoffing/Betterfox/main/user.js")
        .call()
        .wrap_err("Failed to download Betterfox user.js")?;

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
    reader.read_to_string(&mut s).wrap_err("Failed to read Betterfox user.js")?;
    BETTER_FOX_USER_JS.set(s).unwrap();

    Ok(BETTER_FOX_USER_JS.get().unwrap())
}

pub fn firefox_folder() -> Option<PathBuf> {
    let path = if cfg!(target_os = "macos") {
        roaming_data_base()?.join("Firefox")
    } else if cfg!(target_os = "windows") {
        roaming_data_base()?.join("Mozilla").join("Firefox")
    } else {
        roaming_data_base()?.join("firefox")
    };

    if path.exists() { Some(path) } else { None }
}

#[derive(Debug)]
struct Profile<'a> {
    name: &'a str,
    path: PathBuf
}

impl Display for Profile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[instrument]
pub fn debloat(path: PathBuf) -> color_eyre::Result<()> {
    let profiles_str =
        fs::read_to_string(path.join("profiles.ini")).wrap_err("Failed to read profiles.ini")?;
    let profiles_doc: toml::Value =
        toml::from_str(&profiles_str).wrap_err("Failed to parse profiles.ini")?;

    let profiles: (Vec<_>, Vec<_>) = profiles_doc
        .as_table()
        .context("Failed to parse profiles.ini")?
        .iter()
        .filter_map(|(_, v)| v.as_table())
        .filter_map(|t| {
            Some((
                t.get_key_value("Name")?.1.as_str()?,
                t.get_key_value("Path")?.1.as_str()?,
                t.get_key_value("Default").and_then(|v| v.1.as_bool()).unwrap_or_default()
            ))
        })
        .partition(|(_, _, is_default)| *is_default);
    let defaults = profiles.0.len();

    // Make sure defaults are first
    let profiles = [profiles.0, profiles.1]
        .concat()
        .into_iter()
        .map(|(name, profile_path, _)| Profile { name, path: path.join(profile_path) })
        .filter(|profile| {
            if !profile.path.exists() {
                warn!(path = %profile.path.display(), "Profile does not exist");
                return false;
            }

            let children = match fs::read_dir(&profile.path) {
                Ok(c) => c.count(),
                Err(why) => {
                    warn!(path = %profile.path.display(), err = %why, "Failed to read profile directory");
                    return false;
                }
            };

            // If no files or only times.json
            if children < 2 {
                warn!(path = %profile.path.display(), "Profile is empty");
                return false;
            }

            true
        })
        .collect::<Vec<_>>();

    if profiles.is_empty() {
        warn!("No FireFox profiles found in profiles.ini");
        return Ok(());
    }

    let profiles = inquire::MultiSelect::new(
        &format!("Which profile{} to debloat?", if profiles.len() > 1 { "s" } else { "" }),
        profiles
    )
    .with_default(&(0..defaults).collect::<Vec<_>>())
    .prompt()
    .wrap_err("Failed to select profiles")?
    .into_iter()
    .collect::<Vec<_>>();

    if profiles.is_empty() {
        warn!("No FireFox profiles selected");
        return Ok(());
    }

    for profile in profiles {
        let span = info_span!("Debloating profile", profile = %profile);
        let _enter = span.enter();

        if let Err(why) = backup_profile(&profile) {
            warn!(err = ?why, "Failed to backup profile");
            continue;
        }

        if let Err(why) = user_js_profile(&profile) {
            warn!(err = ?why, "Failed to install user.js");
        }
    }

    Ok(())
}

#[instrument]
fn backup_profile(profile: &Profile) -> color_eyre::Result<()> {
    // Canonicalize to convert to an absolute path just in case, so we can get parent dir
    let profiles_path = fs::canonicalize(&profile.path)
        .map_err(color_eyre::eyre::Error::from)
        .and_then(|p| p.parent().map(Path::to_path_buf).context("Parent was None"))
        .unwrap_or_else(|why| {
            warn!(path = %profile.path.display(), err = %why, "Failed to get parent directory, falling back to profile path");
            profile.path.clone()
        });

    let backup_path =
        profiles_path.join(format!("{profile}-backup-{}", timestamp())).with_extension("zip");

    let entries = fs::read_dir(&profile.path)?;
    let mut zip =
        ZipWriter::new(File::create(&backup_path).wrap_err("Failed to create backup zip file")?);

    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in entries {
        if let Err(why) = add_to_archive(
            &mut zip,
            entry,
            &profile.path,
            &options,
            // skip these unnecessary huge dirs
            DEFAULT_FIREFOX_SKIP
        ) {
            warn!(err = ?why, "Failed to add entry to archive");
        }
    }

    zip.finish().wrap_err("Failed to finish zip file").map(|_| ())
}

#[instrument]
fn user_js_profile(profile: &Profile) -> color_eyre::Result<()> {
    let user_js_path = profile.path.join("user.js");
    if user_js_path.exists()
        && !inquire::prompt_confirmation(format!(
            "user.js already exists for profile {profile}. Do you want to overwrite it?"
        ))
        .unwrap_or_default()
    {
        return Ok(());
    }

    let user_js = get_better_fox_user_js()?;
    let configured_user_js = {
        let mut lines = user_js.lines().collect::<Vec<_>>();
        let start_my_overrides_pos = lines
            .iter()
            .rposition(|l| l.trim().starts_with("* START: MY OVERRIDE"))
            .context("Failed to find start of 'my overrides'")?;

        // Skip comments and a blank space
        let start_my_overrides_pos = start_my_overrides_pos + 4;
        lines.insert(start_my_overrides_pos, include_str!("../snippets/betterfox_user_config"));
        Ok::<String, color_eyre::eyre::Error>(lines.join::<&str>("\n"))
    }?;

    fs::write(&user_js_path, configured_user_js).wrap_err("Failed to write user.js")
}
