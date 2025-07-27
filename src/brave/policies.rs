use crate::{ARGS, browser::installation::Installation, util::args};
use color_eyre::eyre::bail;

// todo put into the debloat module
pub fn create_policies(installation: &Installation) -> color_eyre::Result<()> {
    create(installation, args().backup)
}

#[cfg(target_os = "windows")]
// regedit
pub fn create_policies_windows(installation: &Installation, backup: bool) -> color_eyre::Result<()> {
    use windows_registry::*;

    // FIXME for beta/nightly?
    let mut policies_key = LOCAL_MACHINE
        // Creates or opens
        .create("Software\\Policies\\BraveSoftware\\Brave")?;

    fn backup() {}

    Ok(())
}

#[cfg(target_os = "macos")]
// plist
fn create(installation: &Installation, backup: bool) -> color_eyre::Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
// json
fn create(installation: &Installation, backup: bool) -> color_eyre::Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn create(_installation: &Installation, _backup: bool) -> color_eyre::Result<()> {
    if !cfg!(target_os = "windows") {
        bail!("Unsupported OS for Brave policies creation: {}", std::env::consts::OS);
    }
}
