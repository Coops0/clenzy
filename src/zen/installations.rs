use crate::{
    browsers::{Browser, Installation, InstalledVia}, util::{flatpak_base, roaming_data_base, snap_base}
};
use std::path::PathBuf;

fn data_folder() -> PathBuf {
    let base = roaming_data_base();
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        base.join("zen")
    } else {
        base.join(".zen")
    }
}

pub fn installations() -> [Option<Installation>; 3] {
    [
        Installation::new(Browser::Zen)
            .data_folder(data_folder())
            .installation_folder(PathBuf::new())
            .build(),
        Installation::new(Browser::Zen)
            .installed_via(InstalledVia::Snap)
            .data_folder(snap_base().join("0xgingi-zen-browser").join("common").join(".zen"))
            .installation_folder(PathBuf::new())
            .build(),
        Installation::new(Browser::Zen)
            .installed_via(InstalledVia::Flatpak)
            .data_folder(flatpak_base().join("app.zen_browser.zen").join(".zen"))
            .installation_folder(PathBuf::new())
            .build()
    ]
}