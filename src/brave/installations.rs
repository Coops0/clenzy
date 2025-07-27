use crate::{
    brave::Brave, util::{flatpak_base, local_app_bases, local_data_base, local_snap_base}
};
use std::path::PathBuf;
use crate::browser::installation::{Installation, InstalledVia, Variant};

fn local() -> Option<PathBuf> {
    let mut ret = local_data_base()?.join("BraveSoftware/Brave-Browser");
    if cfg!(target_os = "windows") {
        ret = ret.join("User Data");
    }

    Some(ret)
}

fn local_apps() -> Vec<PathBuf> {
    let bases = local_app_bases();
    if cfg!(target_os = "windows") {
        bases.map(|f| f.join("BraveSoftware/Brave-Browser/Application")).collect()
    } else if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Brave Browser.app/Contents")).collect()
    } else {
        bases.map(|f| f.join("brave.com/brave/brave-browser")).collect()
    }
}

fn local_nightly() -> Option<PathBuf> {
    let mut ret = local_data_base()?.join("BraveSoftware/Brave-Browser-Nightly");
    if cfg!(target_os = "windows") {
        ret = ret.join("User Data");
    }

    Some(ret)
}

fn local_beta() -> Option<PathBuf> {
    let mut ret = local_data_base()?.join("BraveSoftware/Brave-Browser-Beta");
    if cfg!(target_os = "windows") {
        ret = ret.join("User Data");
    }

    Some(ret)
}

fn local_nightly_apps() -> Vec<PathBuf> {
    let bases = local_app_bases();
    if cfg!(target_os = "windows") {
        bases.map(|f| f.join("BraveSoftware/Brave-Browser-Nightly/Application")).collect()
    } else if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Brave Browser Nightly.app/Contents")).collect()
    } else {
        // https://brave.com/linux/
        // FIXME
        Vec::new()
    }
}

fn local_beta_apps() -> Vec<PathBuf> {
    let bases = local_app_bases();
    if cfg!(target_os = "windows") {
        bases.map(|f| f.join("BraveSoftware/Brave-Browser-Beta/Application")).collect()
    } else if cfg!(target_os = "macos") {
        bases.map(|f| f.join("Brave Browser Beta.app/Contents")).collect()
    } else {
        // FIXME
        Vec::new()
    }
}

fn snap() -> Option<PathBuf> {
    Some(local_snap_base()?.join("brave/current/.config/BraveSoftware/Brave-Browser"))
}

fn snap_app() -> PathBuf {
    PathBuf::from("/snap/brave/current/opt/brave.com/brave")
}

fn flatpak() -> Option<PathBuf> {
    Some(flatpak_base()?.join("com.brave.Browser/config/BraveSoftware/Brave-Browser"))
}

fn flatpak_app() -> PathBuf {
    PathBuf::from("/var/lib/flatpak/app/com.brave.Browser/current/active/files/brave")
}

pub fn installations() -> Vec<Installation> {
    let mut ret = Vec::with_capacity(4);
    ret.push(
        Installation::builder::<Brave>().data_folder(local()).app_folders(local_apps()).build()
    );

    ret.push(
        Installation::builder::<Brave>()
            .variant(Variant::Beta)
            .data_folder(local_beta())
            .app_folders(local_beta_apps())
            .build()
    );

    ret.push(
        Installation::builder::<Brave>()
            .variant(Variant::Nightly)
            .data_folder(local_nightly())
            .app_folders(local_nightly_apps())
            .build()
    );

    if cfg!(target_os = "linux") {
        ret.push(
            Installation::builder::<Brave>()
                .installed_via(InstalledVia::Snap)
                .data_folder(snap())
                .app_folder(Some(snap_app()))
                .build()
        );

        ret.push(
            Installation::builder::<Brave>()
                .installed_via(InstalledVia::Flatpak)
                .data_folder(flatpak())
                .app_folder(Some(flatpak_app()))
                .build()
        );
    }

    ret
}
