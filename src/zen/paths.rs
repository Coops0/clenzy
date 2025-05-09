use crate::util::{flatpak_base, roaming_data_base, snap_base};
use std::path::PathBuf;

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

pub fn zen_flatpak_folder() -> Option<PathBuf> {
    let path = flatpak_base()?.join("app.zen_browser").join(".zen");
    path.exists().then_some(path)
}
