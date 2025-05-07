use crate::{
    archive::add_to_archive, util::{select_profiles, timestamp, validate_profile_dir}, ARGS
};
use color_eyre::eyre::{Context, ContextCompat};
use fs::File;
use ini::Ini;
use inquire::error::InquireResult;
use std::{
    fmt::Display, fs, path::{Path, PathBuf}, sync::LazyLock
};
use tracing::{debug, info, info_span, instrument, warn};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug)]
pub struct FirefoxProfile<'a> {
    pub name: &'a str,
    pub path: PathBuf
}

impl Display for FirefoxProfile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[instrument(skip_all)]
pub fn debloat<'a, F>(
    path: PathBuf,
    fetch_user_js: F,
    additional_snippets: &str
) -> color_eyre::Result<()>
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
    let profiles = [profiles.0, profiles.1]
        .concat()
        .into_iter()
        .map(|(name, profile_path, _)| FirefoxProfile { name, path: path.join(profile_path) })
        .filter(|profile| validate_profile_dir(&profile.path))
        .collect::<Vec<_>>();

    debug!("found {} valid profiles", profiles.len());

    if profiles.is_empty() {
        warn!("No FireFox profiles found in profiles.ini");
        return Ok(());
    }

    let profiles = select_profiles(profiles, &(0..defaults).collect::<Vec<_>>());
    if profiles.is_empty() {
        return Ok(());
    }

    for profile in profiles {
        let span = info_span!("Debloating profile", profile = %profile);
        let _enter = span.enter();

        if let Err(why) = backup_profile(&profile) {
            warn!(err = ?why, "Failed to backup profile");
            continue;
        }

        if let Err(why) = install_user_js(&profile, &fetch_user_js, additional_snippets) {
            warn!(err = ?why, "Failed to install user.js");
            continue;
        }

        debug!("Finished debloating profile");
    }

    Ok(())
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
            // skip these unnecessary huge dirs
            &DEFAULT_FIREFOX_SKIP
        ) {
            warn!(err = ?why, "Failed to add entry to archive");
        }
    }

    debug!("finished creating backup zip file");
    info!("Backup created for user profile");
    zip.finish().wrap_err("Failed to finish zip file").map(|_| ())
}

#[instrument(skip_all)]
fn install_user_js<'a, F>(
    profile: &FirefoxProfile,
    fetch_user_js: F,
    additional_snippets: &str
) -> color_eyre::Result<()>
where
    F: Fn() -> color_eyre::Result<&'a str>
{
    let user_js_path = profile.path.join("user.js");
    if prompt_should_skip_overwrite_user_js(profile, &user_js_path)? {
        debug!(path = %user_js_path.display(), "Skipping user.js overwrite");
        return Ok(());
    }

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

    fs::write(&user_js_path, configured_user_js).wrap_err("Failed to write user.js")
}

fn prompt_should_skip_overwrite_user_js(
    profile: &FirefoxProfile,
    path: &Path
) -> InquireResult<bool> {
    if !path.exists() || ARGS.get().unwrap().auto_confirm {
        return Ok(false);
    };

    inquire::Confirm::new(&format!(
        "user.js already exists for profile {profile}. Do you want to overwrite it? (y/n)"
    ))
    .prompt()
}
