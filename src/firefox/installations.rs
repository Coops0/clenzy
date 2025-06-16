use crate::{
    installation::{Installation, InstalledVia}, util::{flatpak_base, local_app_bases, local_snap_base, roaming_data_base}
};
use std::path::PathBuf;
use crate::firefox::Firefox;

fn local() -> Vec<PathBuf> {
    let mut ret = Vec::with_capacity(3);

    let roaming_base = roaming_data_base();
    if cfg!(target_os = "macos") {
        if let Some(rb) = roaming_base {
            ret.push(rb.join("Firefox"));
        }

        if let Some(lb) = dirs::data_local_dir() {
            ret.push(lb.join("Mozilla").join("Firefox"));
        }
    } else if cfg!(target_os = "windows") {
        if let Some(rb) = roaming_base {
            ret.push(rb.join("Mozilla").join("Firefox"));
        }
    } else if let Some(rb) = roaming_base {
        ret.push(rb.join(".mozilla").join("firefox"));
    }

    ret
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
    } else {
        let mut bases = bases.map(|f| f.join("firefox")).collect::<Vec<_>>();
        if let Some(home) = dirs::home_dir() {
            bases.push(home.join("firefox"));
        }

        bases
    }
}

fn snap() -> Option<PathBuf> {
    Some(local_snap_base()?.join("firefox").join("common").join(".mozilla").join("firefox"))
}

fn snap_app() -> PathBuf {
    PathBuf::from("/")
        .join("snap")
        .join("firefox")
        .join("current")
        .join("usr")
        .join("lib")
        .join("firefox")
}

fn flatpak() -> Option<PathBuf> {
    Some(flatpak_base()?.join("org.mozilla.firefox").join(".mozilla").join("firefox"))
}

fn flatpak_app() -> PathBuf {
    PathBuf::from("/")
        .join("var")
        .join("lib")
        .join("flatpak")
        .join("app")
        .join("org.mozilla.firefox")
        .join("current")
        .join("active")
        .join("files")
        .join("lib")
        .join("firefox")
}

pub fn installations() -> Vec<Installation> {
    let mut ret = Vec::with_capacity(3);

    ret.push(
        Installation::builder::<Firefox>()
            .data_folders(local())
            .app_folders(local_apps())
            .build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder::<Firefox>()
                .installed_via(InstalledVia::Snap)
                .data_folder(snap())
                .app_folder(Some(snap_app()))
                .build()
        );

        ret.push(
            Installation::builder::<Firefox>()
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .app_folder(Some(flatpak_app()))
                .build()
        );
    }

    ret
}
