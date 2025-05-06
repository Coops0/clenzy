use crate::util::{add_to_archive, timestamp, DEFAULT_FIREFOX_SKIP};
use crate::ARGS;
use color_eyre::eyre::{Context, ContextCompat};
use fs::File;
use ini::Ini;
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};
use tracing::{info_span, instrument, warn};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug)]
pub struct Profile<'a> {
    pub name: &'a str,
    pub path: PathBuf,
}

impl Display for Profile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[instrument(skip(path, install_user_js))]
pub fn debloat<I>(path: PathBuf, install_user_js: I) -> color_eyre::Result<()>
where
    I: Fn(&Profile) -> color_eyre::Result<()>,
{
    let profiles_str =
        fs::read_to_string(path.join("profiles.ini")).wrap_err("Failed to read profiles.ini")?;
    let profiles_doc =
        Ini::load_from_str(&profiles_str).wrap_err("Failed to parse profiles.ini")?;
    drop(profiles_str);

    let profiles: (Vec<_>, Vec<_>) = profiles_doc
        .iter()
        .filter_map(|(_, prop)| {
            Some((
                prop.get("Name")?,
                prop.get("Path")?,
                prop.get("Default").and_then(|d| d.parse::<u8>().ok()).unwrap_or_default() == 1,
            ))
        })
        .partition(|(_, _, is_default)| *is_default);
    let defaults = profiles.0.len();

    // Make sure defaults are first
    let mut profiles = [profiles.0, profiles.1]
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

    let profiles = if ARGS.get().unwrap().autoconfirm {
        profiles
    } else if profiles.len() == 1 {
        inquire::MultiSelect::new("Which profiles to debloat?", profiles)
            .with_default(&(0..defaults).collect::<Vec<_>>())
            .prompt()
            .wrap_err("Failed to select profiles")?
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        vec![profiles.remove(0)]
    };

    if profiles.is_empty() {
        warn!("No profiles selected");
        return Ok(());
    }

    for profile in profiles {
        let span = info_span!("Debloating profile", profile = %profile);
        let _enter = span.enter();

        if let Err(why) = backup_profile(&profile) {
            warn!(err = ?why, "Failed to backup profile");
            continue;
        }

        if let Err(why) = install_user_js(&profile) {
            warn!(err = ?why, "Failed to install user.js");
        }
    }

    Ok(())
}

#[instrument(skip(profile), fields(profile = %profile))]
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

    let entries = fs::read_dir(&profile.path)?.collect::<Vec<_>>();
    let mut zip =
        ZipWriter::new(File::create(&backup_path).wrap_err("Failed to create backup zip file")?);

    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let pb = indicatif::ProgressBar::new(0);
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner:.green} {msg} [{wide_bar}] {pos}/{len}")?
            .progress_chars("█░░"),
    );
    pb.set_message(format!("Backing up profile {profile}"));

    for entry in entries {
        if let Err(why) = add_to_archive(
            &mut zip,
            entry,
            &profile.path,
            &options,
            // skip these unnecessary huge dirs
            DEFAULT_FIREFOX_SKIP,
            &pb,
        ) {
            warn!(err = ?why, "Failed to add entry to archive");
        }
    }

    pb.finish_and_clear();

    zip.finish().wrap_err("Failed to finish zip file").map(|_| ())
}

pub fn util_confirm_if_exists(profile: &Profile, path: &Path) -> bool {
    if path.exists()
        && !ARGS.get().unwrap().autoconfirm
        && !inquire::prompt_confirmation(format!(
            "user.js already exists for profile {profile}. Do you want to overwrite it? (y/n)"
        ))
        .unwrap_or_default()
    {
        return false;
    }

    true
}
