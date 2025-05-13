use crate::{
    browsers::{Browser, Installation, InstalledVia}, util::{flatpak_base, local_app_bases, local_snap_base, roaming_data_base}
};
use std::path::PathBuf;

fn local() -> Option<PathBuf> {
    let base = roaming_data_base()?;
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        Some(base.join("zen"))
    } else {
        Some(base.join(".zen"))
    }
}

fn local_apps() -> Vec<PathBuf> {
    if cfg!(target_os = "windows") {
        return dirs::data_local_dir()
            .map(|f| f.join("Zen Browser"))
            .map_or_else(Vec::new, |f| vec![f]);
    }

    let bases = local_app_bases();
    if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Zen Browser.app").join("Contents")).collect()
    } else {
        Vec::new()
    }
}

fn snap() -> Option<PathBuf> {
    Some(local_snap_base()?.join("0xgingi-zen-browser").join("common").join(".zen"))
}

fn snap_app() -> PathBuf {
    PathBuf::from("/")
        .join("snap")
        .join("0xgingi-zen-browser")
        .join("current")
        .join("usr")
        .join("lib")
        .join("zen")
}

fn flatpak() -> Option<PathBuf> {
    Some(flatpak_base()?.join("app.zen_browser.zen").join(".zen"))
}

fn flatpak_app() -> PathBuf {
    PathBuf::from("/")
        .join("var")
        .join("lib")
        .join("flatpak")
        .join("app")
        .join("app.zen_browser.zen")
        .join("current")
        .join("active")
        .join("files")
        .join("zen")
}

pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(3);
    ret.push(
        Installation::builder(Browser::Zen).data_folder(local()).app_folders(local_apps()).build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder(Browser::Zen)
                .installed_via(InstalledVia::Snap)
                .data_folder(snap())
                .app_folder(Some(snap_app()))
                .build()
        );

        ret.push(
            Installation::builder(Browser::Zen)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .app_folder(Some(flatpak_app()))
                .build()
        );
    }

    ret
}
