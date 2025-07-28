use crate::{
    browser::installation::{Installation, Variant}, util::{args, logging::success}
};
use color_eyre::eyre::ContextCompat;
use std::{fs, sync::LazyLock};
use tracing::warn;

static POLICIES: LazyLock<serde_json::Map<String, serde_json::Value>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../snippets/brave/policies.json"))
        .expect("to parse policies json file")
});

pub fn create_policies(installation: &Installation) -> color_eyre::Result<()> {
    create(installation, args().backup)
}

#[cfg(target_os = "windows")]
#[allow(clippy::items_after_statements)]
// regedit
pub fn create_policies_windows(
    installation: &Installation,
    should_backup: bool
) -> color_eyre::Result<()> {
    use std::fmt::Write;
    use windows_registry::*;

    // FIXME for beta/nightly?
    let mut policies_key = LOCAL_MACHINE
        // Creates or opens
        .create("Software\\Policies\\BraveSoftware\\Brave")?;

    fn stringify(v: Vec<(String, Value)>) -> String {
        let mut backup = String::from(
            r#"Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\BraveSoftware\Brave]
"#
        );

        for (key, value) in v {
            if let Ok(n) = TryInto::<u32>::try_into(value) {
                let _ = writeln!(&mut backup, r#""{key}"=dword:{n:08}"#);
            }
        }

        backup
    }

    let original = if should_backup {
        let v = policies_key.values().unwrap().collect::<Vec<(String, Value)>>();
        Some(stringify(v))
    } else {
        None
    };

    let mut inserted_new_lines = false;
    for (key, value) in POLICIES.iter() {
        if policies_key.get_value(key).is_ok() {
            continue;
        };

        if let Some(n) = value.as_u64() {
            inserted_new_lines = true;
            policies_key.set_u32(key, n as u32)?;
        }
    }

    if !inserted_new_lines {
        return Ok(());
    }

    if let Some(stringified) = original {
        let backup_path = installation.data_folders.first().map(|f| {
            f.join(format!("policies-backup-{}.reg", chrono::Utc::now().format("%Y%m%d%H%M")))
        });
        
        if let Some(p) = backup_path {
            if let Err(why) = fs::write(&p, stringified.as_bytes()) {
                warn!(err = ?why, "Failed to write backup file: {}", p.display());
            } else {
                success(&format!("Backed up policies for {installation}"));
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(clippy::items_after_statements)]
// plist
fn create(installation: &Installation, should_backup: bool) -> color_eyre::Result<()> {
    let home = dirs::home_dir().context("Couldn't find home directory")?;

    let modifier = match installation.variant {
        Some(Variant::Beta) => ".beta",
        Some(Variant::Nightly) => ".nightly",
        _ => ""
    };

    let file_name = format!("com.brave.Browser{modifier}");

    let plist_path = home.join(format!("Library/Preferences/{file_name}.plist"));
    let plist_data = fs::read(&plist_path).ok();
    let plist = plist_data
        .as_ref()
        .and_then(|c| plist::from_bytes::<plist::Value>(c).ok())
        .and_then(plist::Value::into_dictionary)
        .unwrap_or_else(plist::Dictionary::new);

    fn backup(i: &Installation, name: &str, p: &[u8]) -> Option<()> {
        let root = i.data_folders.first()?;
        let backup_path =
            root.join(format!("{name}-{}.plist", chrono::Utc::now().format("%Y%m%d%H%M")));

        fs::write(&backup_path, p).ok()
    }

    let mut new_plist = plist.clone();
    for (key, value) in POLICIES.iter() {
        if plist.get(key).is_some() {
            continue;
        }

        let val = match value {
            // Only 0's and 1's for now
            serde_json::Value::Number(n) => plist::Value::Integer(n.as_i64().unwrap().into()),
            serde_json::Value::String(s) => plist::Value::String(s.clone()),
            _ => continue
        };

        new_plist.insert(key.clone(), val);
    }

    if let Some(d) = &plist_data
        && should_backup
        && plist != new_plist
    {
        if backup(installation, &file_name, d).is_some() {
            success(&format!("Backed up existing Brave policy file for {installation}"));
        } else {
            warn!("Failed to backup existing Brave policy file for {installation}");
        }
    }

    plist::to_file_binary(&plist_path, &new_plist).map_err(Into::into)
}

#[cfg(target_os = "linux")]
#[allow(clippy::items_after_statements)]
// json
fn create(installation: &Installation, should_backup: bool) -> color_eyre::Result<()> {
    let root = std::path::Path::new("/etc/brave/policies/managed/");
    let _ = fs::create_dir_all(root);

    let policies_path = root.join("policies.json");
    let policies_data = fs::read(&policies_path).ok();
    let existing_policies = policies_data
        .as_ref()
        .and_then(|c| serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&c).ok())
        .unwrap_or(serde_json::Map::new());

    let mut new_policies = existing_policies.clone();
    for (key, value) in POLICIES.iter() {
        if existing_policies.get(key).is_none() {
            new_policies.insert(key.clone(), value.clone());
        }
    }

    if let Some(d) = &policies_data
        && should_backup
        && new_policies != existing_policies
    {
        let target =
            root.join(format!("policies-{}.json", chrono::Utc::now().format("%Y%m%d%H%M")));
        if let Err(why) = fs::write(&target, d) {
            warn!(err = ?why, "Failed to backup existing Brave policy file for {installation}");
        } else {
            success(&format!("Backed up existing Brave policies file for {installation}"));
        }
    }

    // FIXME the docs don't say if the file needs a specific name?
    fs::write(&policies_path, serde_json::to_string_pretty(&new_policies)?).map_err(Into::into)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn create(_installation: &Installation, _backup: bool) -> color_eyre::Result<()> {
    if !cfg!(target_os = "windows") {
        color_eyre::eyre::bail!(
            "Unsupported OS for Brave policies creation: {}",
            std::env::consts::OS
        );
    }

    Ok(())
}
