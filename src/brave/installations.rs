use crate::browsers::{InstalledVia, Variant};
use crate::{
    browsers::Installation, util::{flatpak_base, local_data_base, snap_base},
};
use std::path::PathBuf;

// FIXME all of these execution folders

pub fn installations() -> [Option<Installation>; 4] {
    [
        Installation::brave()
            .data_folder(local_data_base().join("BraveSoftware").join("Brave-Browser"))
            .installation_folder(PathBuf::new())
            .call(),
        Installation::brave()
            .variant(Variant::Nightly)
            .data_folder(local_data_base().join("BraveSoftware").join("Brave-Browser-Nightly"))
            .installation_folder(PathBuf::new())
            .call(),
        Installation::brave()
            .installed_via(InstalledVia::Snap)
            .data_folder(snap_base()
                .join("brave")
                .join("509")
                .join(".config")
                .join("BraveSoftware")
                .join("Brave-Browser"))
            .installation_folder(PathBuf::new())
            .call(),
        Installation::brave()
            .installed_via(InstalledVia::Flatpak)
            .data_folder(flatpak_base()
                .join("com.brave.Browser")
                .join("config")
                .join("BraveSoftware")
                .join("Brave-Browser"))
            .installation_folder(PathBuf::new())
            .call()
    ]
}
