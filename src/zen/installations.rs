use crate::{
    browsers::{Browser, Installation, InstalledVia}, util::{flatpak_base, roaming_data_base, snap_base}
};
use std::path::PathBuf;

// FIXME all the execution folders

fn data_folder() -> Option<PathBuf> {
    let base = roaming_data_base()?;
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        Some(base.join("zen"))
    } else {
        Some(base.join(".zen"))
    }
}

pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(3);
    ret.push(
        Installation::builder(Browser::Zen)
            .data_folder(data_folder())
            .build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder(Browser::Zen)
                .installed_via(InstalledVia::Snap)
                .data_folder(snap_base().map(|p| p.join("0xgingi-zen-browser").join("common").join(".zen")))
                .build()
        );

        ret.push(
            Installation::builder(Browser::Zen)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak_base().map(|p| p.join("app.zen_browser.zen").join(".zen")))
                .build()
        );
    }

    ret
}
