use crate::{
    util::{flatpak_base, local_app_bases, local_snap_base, roaming_data_base}, zen::Zen
};
use std::path::PathBuf;
use crate::browser::installation::{Installation, InstalledVia};

fn local() -> Vec<PathBuf> {
    let mut ret = Vec::with_capacity(3);

    let roaming_base = roaming_data_base();
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        if let Some(rb) = roaming_base {
            ret.push(rb.join("zen"));
        }

        if let Some(ld) = dirs::data_local_dir() {
            ret.push(ld.join("zen"));
        }
    } else if let Some(rb) = roaming_base {
        ret.push(rb.join(".zen"));
    }

    ret
}

fn local_apps() -> Vec<PathBuf> {
    if cfg!(target_os = "windows") {
        return dirs::data_local_dir()
            .map(|f| f.join("Zen Browser"))
            .map_or_else(Vec::new, |f| vec![f]);
    }

    let bases = local_app_bases();
    if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Zen Browser.app/Contents")).collect()
    } else {
        Vec::new()
    }
}

fn snap() -> Option<PathBuf> {
    Some(local_snap_base()?.join("0xgingi-zen-browser/common/.zen"))
}

fn snap_app() -> PathBuf {
    PathBuf::from("/snap/0xgingi-zen-browser/current/usr/lib/zen")
}

fn flatpak() -> Option<PathBuf> {
    Some(flatpak_base()?.join("app.zen_browser.zen/.zen"))
}

fn flatpak_app() -> PathBuf {
    PathBuf::from("/var/lib/flatpak/app/app.zen_browser.zen/current/active/files/zen")
}

pub fn installations() -> Vec<Installation> {
    let mut ret = Vec::with_capacity(3);
    ret.push(
        Installation::builder::<Zen>().data_folders(local()).app_folders(local_apps()).build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder::<Zen>()
                .installed_via(InstalledVia::Snap)
                .data_folder(snap())
                .app_folder(Some(snap_app()))
                .build()
        );

        ret.push(
            Installation::builder::<Zen>()
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .app_folder(Some(flatpak_app()))
                .build()
        );
    }

    ret
}
