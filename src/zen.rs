use crate::engines::firefox_like::util_confirm_if_exists;
use crate::{engines, engines::firefox_like::Profile, util::{fetch_text_with_pb, roaming_data_base}};
use color_eyre::eyre::Context;
use std::{fs, path::PathBuf, sync::OnceLock};
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
    if util_confirm_if_exists(profile, &user_js_path) {
        return Ok(());
    }
    
    let user_js = get_better_zen_user_js()?;
    fs::write(&user_js_path, user_js).wrap_err("Failed to write user.js")
}
