use crate::util::config_base;
use std::path::PathBuf;
use std::sync::OnceLock;

pub fn firefox_folder() -> Option<PathBuf> {
    // todo fix these paths
    let path = config_base()?.join("Mozilla").join("firefox");
    if path.exists() { Some(path) } else { None }
}

pub fn firefox_nightly_folder() -> Option<PathBuf> {
    let path = config_base()?.join("Mozilla").join("firefox-nightly");
    if path.exists() { Some(path) } else { None }
}

static BETTER_FOX_USER_JS: OnceLock<String> = OnceLock::new();
fn get_better_fox_user_js() -> color_eyre::Result<&'static str> {
    if BETTER_FOX_USER_JS.get().is_none() {
        let s = ureq::get("https://raw.githubusercontent.com/yokoffing/Betterfox/main/user.js")
            .call()?
            .into_body()
            .read_to_string()?;
        BETTER_FOX_USER_JS.set(s).unwrap();
    }

    Ok(BETTER_FOX_USER_JS.get().unwrap())
}

pub fn debloat(path: &PathBuf) -> color_eyre::Result<()> {
    // todo find root profile
    Ok(())
}
