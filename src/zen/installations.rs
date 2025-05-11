use crate::{
    browsers::{Browser, Installation, InstalledVia}, util::{flatpak_base, roaming_data_base, local_snap_base}
};
use std::path::PathBuf;
use crate::util::local_app_bases;
// FIXME all the execution folders

fn local() -> Option<PathBuf> {
    let base = roaming_data_base()?;
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        Some(base.join("zen"))
    } else if cfg!(target_os = "linux") {
        Some(base.join(".zen"))
    } else {
        None
    }
}

fn local_apps() -> Vec<PathBuf> {
    if cfg!(target_os = "windows") {
        return match dirs::data_local_dir().map(|f| f.join("Zen Browser")) {
            Some(f) => vec![f],
            None => Vec::new()
        };
    }
    
    let bases = local_app_bases();
    if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Zen Browser.app").join("Contents")).collect()
    } else if cfg!(target_os = "linux") {
        todo!();
    } else {
        Vec::new()
    }
}

fn snap() -> Option<PathBuf> {
   Some(local_snap_base()?.join("0xgingi-zen-browser").join("common").join(".zen")) 
}

// /snap/0xgingi-zen-browser/current/usr/lib/zen

fn flatpak() -> Option<PathBuf> {
    Some(flatpak_base()?.join("app.zen_browser.zen").join(".zen"))
}

// /var/lib/flatpak/app/app.zen_browser.zen/current/active/files/zen

pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(3);
    ret.push(
        Installation::builder(Browser::Zen)
            .data_folder(local())
            .app_folders(local_apps())
            .build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder(Browser::Zen)
                .installed_via(InstalledVia::Snap)
                .data_folder(snap())
                .build()
        );

        ret.push(
            Installation::builder(Browser::Zen)
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .build()
        );
    }

    ret
}
