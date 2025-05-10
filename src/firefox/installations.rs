use crate::{
    browsers::{Browser, Installation, InstalledVia}, util::{flatpak_base, roaming_data_base, snap_base}
};
use std::path::PathBuf;

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

pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(3);

    ret.push(
        Installation::builder(Browser::Firefox)
            .data_folder(roaming_data_base().join("Mozilla").join("Firefox"))
            .installation_folder(local_exec())
            .build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder(Browser::Firefox)
                .installed_via(InstalledVia::Snap)
                .data_folder(
                    snap_base().join("firefox").join("common").join(".mozilla").join("firefox")
                )
                .build()
        );

        ret.push(
            Installation::builder(Browser::Firefox)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(
                    flatpak_base().join("org.mozilla.firefox").join(".mozilla").join("firefox")
                )
                .build()
        );
    }

    ret
}
