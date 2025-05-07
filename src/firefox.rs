use crate::{
    firefox_common, util::{fetch_text, roaming_data_base}, ARGS
};
use std::{path::PathBuf, sync::OnceLock};
use tracing::{instrument, warn};

static BETTER_FOX_USER_JS: OnceLock<String> = OnceLock::new();
fn get_better_fox_user_js() -> color_eyre::Result<&'static str> {
    if BETTER_FOX_USER_JS.get().is_none() {
        let s = fetch_text(
            "Betterfox User.js",
            "https://raw.githubusercontent.com/yokoffing/Betterfox/main/user.js"
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
        roaming_data_base()?.join(".mozilla").join("firefox")
    };

    if path.exists() { Some(path) } else { None }
}

#[instrument]
pub fn debloat(path: PathBuf) -> color_eyre::Result<()> {
    let mut custom_overrides = vec![
        include_str!("../snippets/betterfox_user_config"),
        // These should be optional eventually
        include_str!("../snippets/firefox_user_js_extra"),
    ];

    if ARGS.get().unwrap().vertical_tabs {
        custom_overrides.push(include_str!("../snippets/firefox_user_js_vert_tabs"));
    }

    firefox_common::debloat(path, get_better_fox_user_js, &custom_overrides.join("\n"))
}
