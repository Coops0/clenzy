use crate::{
    browsers::{Browser, Installation, InstalledVia}, util::{flatpak_base, roaming_data_base, snap_base}
};
use std::path::PathBuf;

// FIXME all the execution folders

fn local() -> Option<PathBuf> {
    Some(roaming_data_base()?.join("Mozilla").join("Firefox"))   
}

fn local_exec() -> Option<PathBuf> {
    let base = roaming_data_base()?;
    if cfg!(target_os = "macos") {
        Some(base.join("Firefox"))
    } else if cfg!(target_os = "windows") {
        Some(base.join("Mozilla").join("Firefox"))
    } else {
        Some(base.join(".mozilla").join("firefox"))
    }
}

fn snap() -> Option<PathBuf> {
    Some(snap_base()?.join("firefox").join("common").join(".mozilla").join("firefox"))
}

fn flatpak() -> Option<PathBuf> {
    Some(flatpak_base()?.join("org.mozilla.firefox").join(".mozilla").join("firefox"))
}

pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(3);

    ret.push(
        Installation::builder(Browser::Firefox)
            .data_folder(local())
            .installation_folder(local_exec())
            .build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder(Browser::Firefox)
                .installed_via(InstalledVia::Snap)
                .data_folder(snap())
                .build()
        );

        ret.push(
            Installation::builder(Browser::Firefox)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .build()
        );
    }

    ret
}
