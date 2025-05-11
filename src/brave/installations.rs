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
    } else if cfg!(target_os = "linux") {
        todo!();
    } else {
        Vec::new()
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
    } else if cfg!(target_os = "linux") {
        // https://brave.com/linux/
        todo!();
    } else {
        Vec::new()
    }
}

fn snap() -> Option<PathBuf> {
    Some(
        local_snap_base()?
            .join("brave")
            .join("current") // This is a symlink TODO (make sure this works)
            .join(".config")
            .join("BraveSoftware")
            .join("Brave-Browser")
    )
}

// /snap/brave/current/opt/brave.com/brave

fn flatpak() -> Option<PathBuf> {
    Some(
        flatpak_base()?
            .join("com.brave.Browser")
            .join("config")
            .join("BraveSoftware")
            .join("Brave-Browser")
    )
}
// /var/lib/flatpak/app/com.brave.Browser/current/active/files/brave

pub fn installations() -> Vec<Option<Installation>> {
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
                .build()
        );

        ret.push(
            Installation::builder(Browser::Brave)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .build()
        );
    }

    ret
}
