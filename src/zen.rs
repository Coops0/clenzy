use crate::{
    firefox_common, util::{fetch_text, roaming_data_base, snap_base}
};
use std::{path::PathBuf, sync::OnceLock};
use tracing::instrument;

static BETTER_ZEN_USER_JS: OnceLock<String> = OnceLock::new();
fn get_better_zen_user_js() -> color_eyre::Result<&'static str> {
    if BETTER_ZEN_USER_JS.get().is_none() {
        let s = fetch_text(
            "Better Zen user.js",
            "https://raw.githubusercontent.com/yokoffing/Betterfox/refs/heads/main/zen/user.js"
        )?;
        BETTER_ZEN_USER_JS.set(s).unwrap();
    }

    Ok(BETTER_ZEN_USER_JS.get().unwrap())
}

pub fn zen_folder() -> Option<PathBuf> {
    let base = roaming_data_base()?;
    let path = if cfg!(any(target_os = "macos", target_os = "windows")) {
        base.join("zen")
    } else {
        base.join(".zen")
    };

    path.exists().then_some(path)
}

pub fn zen_snap_folder() -> Option<PathBuf> {
    // This is unofficial
    let path = snap_base()?.join("0xgingi-zen-browser").join("common").join(".zen");
    path.exists().then_some(path)
}

#[instrument]
pub fn debloat(path: PathBuf) -> color_eyre::Result<()> {
    // Not all of these will be used but some are
    let custom_overrides = include_str!("../snippets/betterfox_user_config");
    let _ = firefox_common::debloat(path, get_better_zen_user_js, custom_overrides)?;
    Ok(())
}
