use crate::util::{flatpak_base, roaming_data_base, snap_base};
use std::path::PathBuf;

pub fn firefox_folder() -> Option<PathBuf> {
    let path = if cfg!(target_os = "macos") {
        roaming_data_base()?.join("Firefox")
    } else if cfg!(target_os = "windows") {
        roaming_data_base()?.join("Mozilla").join("Firefox")
    } else {
        roaming_data_base()?.join(".mozilla").join("firefox")
    };

    path.exists().then_some(path)
}

pub fn firefox_snap_folder() -> Option<PathBuf> {
    let path = snap_base()?.join("firefox").join("common").join(".mozilla").join("firefox");
    path.exists().then_some(path)
}

pub fn firefox_flatpak_folder() -> Option<PathBuf> {
    let path = flatpak_base()?.join("org.mozilla.firefox").join(".mozilla").join("firefox");
    path.exists().then_some(path)
}
