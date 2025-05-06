use crate::{
    engines, engines::firefox_like::Profile, util::{fetch_text_with_pb, roaming_data_base}
};
use std::{fs, path::PathBuf, sync::OnceLock};
use color_eyre::eyre::Context;
use tracing::instrument;

static BETTER_ZEN_USER_JS: OnceLock<String> = OnceLock::new();
fn get_better_zen_user_js() -> color_eyre::Result<&'static str> {
    if BETTER_ZEN_USER_JS.get().is_none() {
        let s = fetch_text_with_pb(
            "Better Zen user.js",
            "https://raw.githubusercontent.com/yokoffing/Betterfox/refs/heads/main/zen/user.js"
        )?;
        BETTER_ZEN_USER_JS.set(s).unwrap()
    }

    Ok(BETTER_ZEN_USER_JS.get().unwrap())
}

pub fn zen_folder() -> Option<PathBuf> {
    let path = roaming_data_base()?.join("zen");
    if path.exists() { Some(path) } else { None }
}

#[instrument(skip(path))]
pub fn debloat(path: PathBuf) -> color_eyre::Result<()> {
    engines::firefox_like::debloat(path, user_js_profile)
}

#[instrument]
fn user_js_profile(profile: &Profile) -> color_eyre::Result<()> {
    let user_js_path = profile.path.join("user.js");
    if user_js_path.exists()
        && !inquire::prompt_confirmation(format!(
            "user.js already exists for profile {profile}. Do you want to overwrite it? (y/n)"
        ))
        .unwrap_or_default()
    {
        return Ok(());
    }
    
    let user_js = get_better_zen_user_js()?;
    fs::write(&user_js_path, user_js).wrap_err("Failed to write user.js")
}
