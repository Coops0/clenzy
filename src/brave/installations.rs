use crate::{
    browsers::{Browser, Installation, InstalledVia, Variant}, util::{flatpak_base, local_data_base, snap_base}
};
use std::path::PathBuf;

// FIXME all of these execution folders

pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(4);
    ret.push(
        Installation::builder(Browser::Brave)
            .data_folder(local_data_base().join("BraveSoftware").join("Brave-Browser"))
            .build()
    );
    
    ret.push(
        Installation::builder(Browser::Brave)
            .variant(Variant::Nightly)
            .data_folder(local_data_base().join("BraveSoftware").join("Brave-Browser-Nightly"))
            .build()
    );
   
    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder(Browser::Brave)
                .installed_via(InstalledVia::Snap)
                .data_folder(
                    snap_base()
                        .join("brave")
                        .join("509")
                        .join(".config")
                        .join("BraveSoftware")
                        .join("Brave-Browser")
                )
                .build()
        );
        
        ret.push(
            Installation::builder(Browser::Brave)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(
                    flatpak_base()
                        .join("com.brave.Browser")
                        .join("config")
                        .join("BraveSoftware")
                        .join("Brave-Browser")
                )
                .installation_folder(PathBuf::new())
                .build()
        );
    }
    
    ret
}
