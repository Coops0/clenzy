use color_eyre::eyre::{bail, Context, ContextCompat};
use std::{
    fs, fs::{DirEntry, File}, io, io::{BufReader, Read, Write}, path::Path
};
use tracing::{debug, instrument, trace, warn};
use zip::{write::SimpleFileOptions, ZipWriter};

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
    let file_type = match entry.file_type() {
        Ok(f) => f,
        Err(why) => {
            debug!(err = ?why, path = %path.display(), "Failed to get file type");
            return Ok(());
        }
    };

    let r = if file_type.is_dir() {
        add_dir_to_archive(zip, &abs_path, path, prefix, options, skip)
    } else if file_type.is_file() {
        add_file_to_archive(zip, &abs_path, path, options)
    } else {
        trace!(path = %path.display(), file_type = ?file_type, "Skipping entry of bad type");
        return Ok(());
    };

    if let Err(why) = r {
        debug!(err = ?why, path = %path.display(), "Failed to add entry to archive");
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
            debug!(err = ?why, "Failed to add entry to archive (nested)");
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

    let mut reader =
        BufReader::with_capacity(8192, File::open(abs_path).wrap_err("Failed to open file")?);
    let mut buffer = [0u8; 8192];

    loop {
        let b = reader.read(&mut buffer)?;
        if b == 0 {
            break;
        }

        let bytes = buffer
            .get(..b)
            .with_context(|| format!("failed to index into byte {b} when zipping"))?;
        zip.write_all(bytes)?;
    }

    zip.flush()?;
    Ok(())
}
