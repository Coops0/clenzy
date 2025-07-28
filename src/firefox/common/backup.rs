use crate::util::timestamp;
use color_eyre::eyre::{ContextCompat, WrapErr};
use std::{fs, fs::File, path::Path, sync::LazyLock};
use tracing::{debug, warn};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};
use crate::browser::profile::BrowserProfile;
use crate::util::archive::add_to_archive;
use crate::util::logging::success;

static DEFAULT_FIREFOX_SKIP: LazyLock<Vec<&str>> = LazyLock::new(|| {
    include_str!("../../../snippets/firefox_common/skipped_files").lines().filter(|l| !l.is_empty()).collect()
});

pub fn backup_profile(profile: &BrowserProfile) -> color_eyre::Result<()> {
    // Canonicalize to convert to an absolute path just in case, so we can get parent dir
    let profiles_path = fs::canonicalize(&profile.path)
        .map_err(color_eyre::eyre::Error::from)
        .and_then(|p| p.parent().map(Path::to_path_buf).wrap_err("Parent was None"))
        .unwrap_or_else(|why| {
            warn!(path = %profile.path.display(), err = ?why, "Failed to get parent directory, falling back to profile path");
            profile.path.clone()
        });

    let backup_path =
        profiles_path.join(format!("{profile}-backup-{}", timestamp())).with_extension("zip");

    let entries = fs::read_dir(&profile.path)?;
    let mut zip =
        ZipWriter::new(File::create(&backup_path).wrap_err("Failed to create backup zip file")?);

    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    debug!("Creating backup zip file at {}", backup_path.display());
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

    debug!("Finished creating backup zip file");
    success(&format!("Backup created for user profile {profile}"));
    zip.finish().wrap_err("Failed to finish zip file").map(|_| ())
}
