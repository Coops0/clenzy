use crate::{
    browsers::{Browser, Installation, InstalledVia, Variant}, util::{flatpak_base, local_data_base, snap_base}
};
use std::path::PathBuf;

// FIXME all of these execution folders

fn local() -> Option<PathBuf> {
    Some(local_data_base()?.join("BraveSoftware").join("Brave-Browser"))
}

fn local_nightly() -> Option<PathBuf> {
    Some(local_data_base()?.join("BraveSoftware").join("Brave-Browser-Nightly"))
}

fn snap() -> Option<PathBuf> {
    Some(
        snap_base()?
            .join("brave")
            .join("509")
            .join(".config")
            .join("BraveSoftware")
            .join("Brave-Browser")
    )
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
pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(4);
    ret.push(Installation::builder(Browser::Brave).data_folder(local()).build());

    ret.push(
        Installation::builder(Browser::Brave)
            .variant(Variant::Nightly)
            .data_folder(local_nightly())
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
