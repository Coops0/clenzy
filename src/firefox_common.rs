use crate::{
    archive::add_to_archive, util::{select_profiles, timestamp, validate_profile_dir}, ARGS
};
use ahash::AHasher;
use color_eyre::eyre::{Context, ContextCompat};
use fs::File;
use ini::Ini;
use std::{
    fmt::Display, fs, hash::Hasher, io::{BufReader, Read}, path::{Path, PathBuf}, sync::LazyLock
};
use tracing::{debug, info, info_span, instrument, warn};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[instrument(skip(fetch_user_js, additional_snippets))]
pub fn debloat<'a, F>(
    path: PathBuf,
    fetch_user_js: F,
    additional_snippets: &str
) -> color_eyre::Result<Vec<OwnedFirefoxProfile>>
where
    F: Fn() -> color_eyre::Result<&'a str>
{
    let profiles_str =
        fs::read_to_string(path.join("profiles.ini")).wrap_err("Failed to read profiles.ini")?;
    let profiles_doc =
        Ini::load_from_str(&profiles_str).wrap_err("Failed to parse profiles.ini")?;
    drop(profiles_str);

    debug!(len = %profiles_doc.len(), "profiles ini read");

    let profiles: (Vec<_>, Vec<_>) = profiles_doc
        .iter()
        .filter_map(|(_, prop)| {
            Some((
                prop.get("Name")?,
                prop.get("Path")?,
                prop.get("Default").and_then(|d| d.parse::<u8>().ok()).unwrap_or_default() == 1
            ))
        })
        .partition(|(_, _, is_default)| *is_default);
    let defaults = profiles.0.len();
    debug!(len = %defaults, "default profiles");

    // Make sure defaults are first
    let profiles = <[_; 2]>::from(profiles)
        .concat()
        .into_iter()
        .map(|(name, profile_path, _)| FirefoxProfile { name, path: path.join(profile_path) })
        .filter(|profile| validate_profile_dir(&profile.path))
        .collect::<Vec<_>>();

    debug!("found {} valid profiles", profiles.len());

    if profiles.is_empty() {
        warn!("No FireFox profiles found in profiles.ini");
        return Ok(Vec::new());
    }

    let profiles = select_profiles(profiles, &(0..defaults).collect::<Vec<_>>());
    if profiles.is_empty() {
        return Ok(Vec::new());
    }

    for profile in &profiles {
        let span = info_span!("Debloating profile", profile = %profile);
        let _enter = span.enter();

        if ARGS.get().unwrap().backup {
            if let Err(why) = backup_profile(profile) {
                warn!(err = ?why, "Failed to backup profile");
                continue;
            }
        }

        if let Err(why) = install_user_js(profile, &fetch_user_js, additional_snippets) {
            warn!(err = ?why, "Failed to install user.js");
            continue;
        }

        debug!("Finished debloating profile");
    }

    Ok(profiles.into_iter().map(Into::into).collect::<Vec<_>>())
}

static DEFAULT_FIREFOX_SKIP: LazyLock<Vec<&str>> = LazyLock::new(|| {
    include_str!("../snippets/default_firefox_skip").lines().filter(|l| !l.is_empty()).collect()
});

#[instrument(skip(profile), fields(profile = %profile))]
fn backup_profile(profile: &FirefoxProfile) -> color_eyre::Result<()> {
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

    debug!("creating backup zip file at {}", backup_path.display());
    for entry in entries {
        if let Err(why) = add_to_archive(
            &mut zip,
            entry,
            &profile.path,
            &options,
            // skip these unnecessary huge dirs/files
            &DEFAULT_FIREFOX_SKIP
        ) {
            warn!(err = ?why, "Failed to add entry to archive");
        }
    }

    debug!("finished creating backup zip file");
    info!("Backup created for user profile");
    zip.finish().wrap_err("Failed to finish zip file").map(|_| ())
}

#[instrument(skip(fetch_user_js, additional_snippets))]
fn install_user_js<'a, F>(
    profile: &FirefoxProfile,
    fetch_user_js: F,
    additional_snippets: &str
) -> color_eyre::Result<()>
where
    F: Fn() -> color_eyre::Result<&'a str>
{
    let user_js_path = profile.path.join("user.js");

    let configured_user_js = {
        let user_js = fetch_user_js()?;
        let mut lines = user_js.lines().collect::<Vec<_>>();
        let start_my_overrides_pos = lines
            .iter()
            .rposition(|l| l.trim().starts_with("* START: MY OVERRIDE"))
            .context("Failed to find start of 'my overrides'")?;

        // Skip comments and a blank space
        let start_my_overrides_pos = start_my_overrides_pos + 4;

        if !additional_snippets.is_empty() {
            lines.insert(start_my_overrides_pos, additional_snippets);
        }

        debug!(
            "added {} additional lines to user.js (originally {})",
            additional_snippets.lines().count(),
            lines.len()
        );
        Ok::<String, color_eyre::eyre::Error>(lines.join::<&str>("\n"))
    }?;

    // Checks if user.js exists and content differs from configured_user_js.
    // Assumes any error means the file doesn't exist.
    if check_user_js_exists(profile, &user_js_path, &configured_user_js).unwrap_or_default() {
        debug!(path = %user_js_path.display(), "not overwriting user.js");
        return Ok(());
    }

    fs::write(&user_js_path, configured_user_js).wrap_err("Failed to write user.js")
}

// returns Ok(true) if exists
fn check_user_js_exists(
    profile: &FirefoxProfile,
    path: &Path,
    user_js_str: &str
) -> color_eyre::Result<bool> {
    if !path.exists() || ARGS.get().unwrap().auto_confirm {
        return Ok(false);
    }

    // Read hash of existing user.js
    let fs_hash = {
        let mut fs_hasher = AHasher::default();
        let mut reader = BufReader::with_capacity(8192, File::open(path)?);
        let mut buffer = [0u8; 8192];
        loop {
            if reader.read(&mut buffer)? == 0 {
                break;
            }

            fs_hasher.write(&buffer);
        }

        fs_hasher.finish()
    };

    let user_js_hash = {
        let mut user_js_hasher = AHasher::default();
        user_js_hasher.write(user_js_str.as_bytes());
        user_js_hasher.finish()
    };

    if fs_hash == user_js_hash {
        debug!(path = %path.display(), "user.js already exists and is the same");
        return Ok(true);
    }

    inquire::Confirm::new(&format!(
        "user.js already exists for profile {profile}. Do you want to overwrite it? (y/n)"
    ))
    .prompt()
    .map_err(color_eyre::eyre::Error::from)
}

#[derive(Debug)]
struct FirefoxProfile<'a> {
    name: &'a str,
    path: PathBuf
}

#[derive(Debug)]
pub struct OwnedFirefoxProfile {
    pub name: String,
    pub path: PathBuf
}

impl Display for FirefoxProfile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for OwnedFirefoxProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<FirefoxProfile<'_>> for OwnedFirefoxProfile {
    fn from(profile: FirefoxProfile) -> Self {
        Self { name: profile.name.to_owned(), path: profile.path }
    }
}
