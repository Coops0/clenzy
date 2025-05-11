use crate::{
    browsers::{Browser, Installation, InstalledVia}, util::{flatpak_base, local_app_bases, roaming_data_base, local_snap_base}
};
use std::path::PathBuf;

fn local() -> Option<PathBuf> {
    let base = roaming_data_base()?;
    if cfg!(target_os = "macos") {
        Some(base.join("Firefox"))
    } else if cfg!(target_os = "windows") {
        Some(base.join("Mozilla").join("Firefox"))
    } else if cfg!(target_os = "linux") {
        Some(base.join(".mozilla").join("firefox"))
    } else {
        None
    }
}

// We're not going to classify the variants as different installations, as there's
// no easy way to differentiate the profiles.
fn local_apps() -> Vec<PathBuf> {
    let bases = local_app_bases();
    let variants = ["Firefox", "Mozilla Firefox", "Firefox Developer Edition", "Firefox Nightly"];

    if cfg!(target_os = "windows") {
        let mut bases = bases.collect::<Vec<_>>();
        if let Some(local_app_data) = dirs::data_local_dir() {
            bases.push(local_app_data);
        }

        bases.into_iter().flat_map(|f| variants.iter().map(move |v| f.join(v))).collect()
    } else if cfg!(target_os = "macos") {
        bases
            .flat_map(|f| variants.iter().map(move |v| f.join(format!("{v}.app")).join("Contents")))
            .collect()
    } else if cfg!(target_os = "linux") {
        // /opt/firefox
        // ~/firefox/
        todo!();
    } else {
        Vec::new()
    }
}

fn snap() -> Option<PathBuf> {
    Some(local_snap_base()?.join("firefox").join("common").join(".mozilla").join("firefox"))
}

// /snap/firefox/current/usr/lib/firefox (symlink)

fn flatpak() -> Option<PathBuf> {
    Some(flatpak_base()?.join("org.mozilla.firefox").join(".mozilla").join("firefox"))
}

// /var/lib/flatpak/app/org.mozilla.firefox/current/active/files/lib/firefox (symlink)

pub fn installations() -> Vec<Option<Installation>> {
    let mut ret = Vec::with_capacity(3);

    ret.push(
        Installation::builder(Browser::Firefox)
            .data_folder(local())
            .app_folders(local_apps())
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
