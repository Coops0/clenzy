use crate::engines::firefox_like::util_confirm_if_exists;
use crate::{
    engines,
    engines::firefox_like::Profile,
    util::{fetch_text_with_pb, roaming_data_base},
};
use color_eyre::eyre::{Context, ContextCompat};
use std::{fs, path::PathBuf, sync::OnceLock};
use tracing::{instrument, warn};

static BETTER_FOX_USER_JS: OnceLock<String> = OnceLock::new();
fn get_better_fox_user_js() -> color_eyre::Result<&'static str> {
    if BETTER_FOX_USER_JS.get().is_none() {
        let s = fetch_text_with_pb(
            "Betterfox User.js",
            "https://raw.githubusercontent.com/yokoffing/Betterfox/main/user.js",
        )?;
        BETTER_FOX_USER_JS.set(s).unwrap()
    }

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

#[instrument(skip(path))]
pub fn debloat(path: PathBuf) -> color_eyre::Result<()> {
    engines::firefox_like::debloat(path, user_js_profile)
}

#[instrument]
fn user_js_profile(profile: &Profile) -> color_eyre::Result<()> {
    let custom_overrides = &[
        include_str!("../snippets/betterfox_user_config"),
        // These should be optional eventually
        include_str!("../snippets/firefox_user_js_extra"),
    ]
    .join("\n");

    let user_js_path = profile.path.join("user.js");
    if util_confirm_if_exists(profile, &user_js_path) {
        return Ok(());
    }

    let configured_user_js = {
        let user_js = get_better_fox_user_js()?;
        let mut lines = user_js.lines().collect::<Vec<_>>();
        let start_my_overrides_pos = lines
            .iter()
            .rposition(|l| l.trim().starts_with("* START: MY OVERRIDE"))
            .context("Failed to find start of 'my overrides'")?;

        // Skip comments and a blank space
        let start_my_overrides_pos = start_my_overrides_pos + 4;

        lines.insert(start_my_overrides_pos, custom_overrides);
        Ok::<String, color_eyre::eyre::Error>(lines.join::<&str>("\n"))
    }?;

    fs::write(&user_js_path, configured_user_js).wrap_err("Failed to write user.js")
}
