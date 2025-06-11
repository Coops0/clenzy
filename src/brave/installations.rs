use crate::{
    browsers::{Browser, Installation, InstalledVia, Variant}, util::{flatpak_base, local_app_bases, local_data_base, local_snap_base}
};
use std::path::PathBuf;

fn local() -> Option<PathBuf> {
    let mut ret = local_data_base()?.join("BraveSoftware").join("Brave-Browser");
    if cfg!(target_os = "windows") {
        ret = ret.join("User Data");
    }

    Some(ret)
}

fn local_apps() -> Vec<PathBuf> {
    let bases = local_app_bases();
    if cfg!(target_os = "windows") {
        bases.map(|f| f.join("BraveSoftware").join("Brave-Browser").join("Application")).collect()
    } else if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Brave Browser.app").join("Contents")).collect()
    } else {
        bases.map(|f| f.join("brave.com").join("brave").join("brave-browser")).collect()
    } 
}

fn local_nightly() -> Option<PathBuf> {
    let mut ret = local_data_base()?.join("BraveSoftware").join("Brave-Browser-Nightly");
    if cfg!(target_os = "windows") {
        ret = ret.join("User Data");
    }

    Some(ret)
}

fn local_nightly_apps() -> Vec<PathBuf> {
    let bases = local_app_bases();
    if cfg!(target_os = "windows") {
        bases
            .map(|f| f.join("BraveSoftware").join("Brave-Browser-Nightly").join("Application"))
            .collect()
    } else if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Brave Browser Nightly.app").join("Contents")).collect()
    } else {
        // https://brave.com/linux/
        // FIXME
        Vec::new()
    }
}

fn snap() -> Option<PathBuf> {
    Some(
        local_snap_base()?
            .join("brave")
            .join("current")
            .join(".config")
            .join("BraveSoftware")
            .join("Brave-Browser")
    )
}

fn snap_app() -> PathBuf {
    PathBuf::from("/")
        .join("snap")
        .join("brave")
        .join("current")
        .join("opt")
        .join("brave.com")
        .join("brave")
}

fn flatpak() -> Option<PathBuf> {
    Some(
        flatpak_base()?
            .join("com.brave.Browser")
            .join("config")
            .join("BraveSoftware")
            .join("Brave-Browser")
    )
}

fn flatpak_app() -> PathBuf {
    PathBuf::from("/")
        .join("var")
        .join("lib")
        .join("flatpak")
        .join("app")
        .join("com.brave.Browser")
        .join("current")
        .join("active")
        .join("files")
        .join("brave")
}

pub fn installations() -> Vec<Installation> {
    let mut ret = Vec::with_capacity(4);
    ret.push(
        Installation::builder(Browser::Brave)
            .data_folder(local())
            .app_folders(local_apps())
            .build()
    );

    ret.push(
        Installation::builder(Browser::Brave)
            .variant(Variant::Nightly)
            .data_folder(local_nightly())
            .app_folders(local_nightly_apps())
            .build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder(Browser::Brave)
                .installed_via(InstalledVia::Snap)
                .data_folder(snap())
                .app_folder(Some(snap_app()))
                .build()
        );

        ret.push(
            Installation::builder(Browser::Brave)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .app_folder(Some(flatpak_app()))
                .build()
        );
    }

    ret
}
