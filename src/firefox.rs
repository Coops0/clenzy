use crate::util::data_base;
use color_eyre::eyre::{bail, Context, ContextCompat};
use indicatif::ProgressStyle;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::instrument;
use ureq::http::header::CONTENT_LENGTH;

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
    let path = data_base()?.join("firefox");
    if path.exists() { Some(path) } else { None }
}

#[derive(Debug)]
struct Profile<'a> {
    name: &'a str,
    path: &'a str,
}

#[instrument]
pub fn debloat(path: &PathBuf) -> color_eyre::Result<()> {
    let profiles_str = std::fs::read_to_string(path.join("profiles.ini"))
        .wrap_err("Failed to read profiles.ini")?;
    let mut profiles_doc: toml::Value =
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
                t.get_key_value("Default").and_then(|v| v.1.as_bool()).unwrap_or_default(),
            ))
        })
        .partition(|(_, _, is_default)| *is_default);
    let defaults = profiles.0.len();

    // Make sure defaults are first
    let profiles = [profiles.0, profiles.1]
        .concat()
        .into_iter()
        .map(|(name, path, _)| (name, path))
        .collect::<HashMap<_, _>>();

    if profiles.is_empty() {
        bail!("No profiles found in profiles.ini");
    }

    let profiles = inquire::MultiSelect::new(
        &format!("Which profile{} to debloat?", if profiles.len() > 1 { "s" } else { "" }),
        profiles.keys().collect::<Vec<_>>(),
    )
    .with_default(&(0..defaults).collect::<Vec<_>>())
    .prompt()
    .wrap_err("Failed to select profiles")?
    .into_iter()
    .map(|name| profiles.get_key_value(name).unwrap())
    .map(|(name, path)| Profile { name, path })
    .collect::<Vec<_>>();

    if profiles.is_empty() {
        bail!("No profiles selected");
    }

    for profile in profiles {
        user_js_profile(path, &profile).wrap_err_with(|| {
            format!("Failed to configure user.js for profile {}", profile.name)
        })?;
    }

    Ok(())
}

// TODO backup user profile dir
#[instrument]
fn user_js_profile(root: &PathBuf, profile: &Profile) -> color_eyre::Result<()> {
    let user_js_path = root.join(profile.path).join("user.js");
    if user_js_path.exists() {
        if !inquire::prompt_confirmation(format!(
            "user.js already exists for profile {}. Do you want to overwrite it?",
            profile.name
        ))
        .unwrap_or_default()
        {
            return Ok(());
        }
    }

    let user_js = get_better_fox_user_js()?;
    let configured_user_js = {
        let mut lines = user_js.lines().collect::<Vec<_>>();
        let start_my_overrides_pos = lines
            .iter()
            .rposition(|l| l.trim().starts_with(" * START: MY OVERRIDE"))
            .context("Failed to find start of 'my overrides'")?;

        // Skip comments and a blank space
        let start_my_overrides_pos = start_my_overrides_pos + 4;
        lines.insert(start_my_overrides_pos, include_str!("../snippets/betterfox_user_config"));
        // todo windows \r\n
        Ok::<String, color_eyre::eyre::Error>(lines.join::<&str>("\n"))
    }?;

    std::fs::write(&user_js_path, configured_user_js)
        .wrap_err_with(|| format!("Failed to write user.js for profile {}", profile.name))
}
