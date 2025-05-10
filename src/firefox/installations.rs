use crate::util::{flatpak_base, roaming_data_base, snap_base};
use std::path::PathBuf;
use crate::browsers::{Installation, InstalledVia};

// FIXME all the execution folders

fn local_exec() -> PathBuf {
    if cfg!(target_os = "macos") {
        roaming_data_base().join("Firefox")
    } else if cfg!(target_os = "windows") {
        roaming_data_base().join("Mozilla").join("Firefox")
    } else {
        roaming_data_base().join(".mozilla").join("firefox")
    }
}

pub fn installations() -> [Option<Installation>; 3] {
    [
        Installation::firefox()
            .data_folder(roaming_data_base().join("Mozilla").join("Firefox"))
            .installation_folder(local_exec())
            .call(),
        Installation::firefox()
            .installed_via(InstalledVia::Snap)
            .data_folder(snap_base().join("firefox").join("common").join(".mozilla").join("firefox"))
            .installation_folder(PathBuf::new())
            .call(),
        Installation::firefox()
            .installed_via(InstalledVia::Flatpak)
            .data_folder(flatpak_base().join("org.mozilla.firefox").join(".mozilla").join("firefox"))
            .installation_folder(PathBuf::new())
            .call()
    ]
}