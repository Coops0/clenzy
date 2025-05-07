use crate::util::{fetch_text_with_pb, roaming_data_base};
use std::{path::PathBuf, sync::OnceLock};
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
        roaming_data_base()?.join(".mozilla").join("firefox")
    };

    if path.exists() { Some(path) } else { None }
}

#[instrument(skip(path))]
pub fn debloat(path: PathBuf) -> color_eyre::Result<()> {
    let custom_overrides = &[
        include_str!("../snippets/betterfox_user_config"),
        // These should be optional eventually
        include_str!("../snippets/firefox_user_js_extra"),
    ]
    .join("\n");

    crate::firefox_common::debloat(path, get_better_fox_user_js, custom_overrides)
}
