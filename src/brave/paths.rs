use crate::util::{flatpak_base, local_data_base, snap_base};
use std::path::PathBuf;

pub fn brave_folder() -> Option<PathBuf> {
    let path = local_data_base()?.join("BraveSoftware").join("Brave-Browser");
    path.exists().then_some(path)
}

pub fn brave_nightly_folder() -> Option<PathBuf> {
    let path = local_data_base()?.join("BraveSoftware").join("Brave-Browser-Nightly");
    path.exists().then_some(path)
}

pub fn brave_snap_folder() -> Option<PathBuf> {
    let path = snap_base()?
        .join("brave")
        .join("509")
        .join(".config")
        .join("BraveSoftware")
        .join("Brave-Browser");
    path.exists().then_some(path)
}

pub fn brave_flatpak_folder() -> Option<PathBuf> {
    let path = flatpak_base()?
        .join("com.brave.Browser")
        .join("config")
        .join("BraveSoftware")
        .join("Brave-Browser");
    path.exists().then_some(path)
}
