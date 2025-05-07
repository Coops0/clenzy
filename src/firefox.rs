use crate::{
    firefox_common, util::{fetch_text, get_or_insert_obj, roaming_data_base, snap_base}, ARGS
};
use color_eyre::eyre::{bail, ContextCompat};
use serde_json::{json, Value};
use std::{
    fs, path::{Path, PathBuf}, sync::OnceLock
};
use tracing::{debug, info_span, instrument, warn};

static BETTER_FOX_USER_JS: OnceLock<String> = OnceLock::new();
fn get_better_fox_user_js() -> color_eyre::Result<&'static str> {
    if BETTER_FOX_USER_JS.get().is_none() {
        let s = fetch_text(
            "Betterfox User.js",
            "https://raw.githubusercontent.com/yokoffing/Betterfox/main/user.js"
        )?;
        BETTER_FOX_USER_JS.set(s).unwrap()
    }

    Ok(BETTER_FOX_USER_JS.get().unwrap())
}

pub fn firefox_folder() -> Option<PathBuf> {
    let path = if cfg!(target_os = "macos") {
        roaming_data_base()?.join("Firefox")
    } else if cfg!(target_os = "windows") {
        roaming_data_base()?.join("Mozilla").join("Firefox")
    } else {
        roaming_data_base()?.join(".mozilla").join("firefox")
    };

    if path.exists() { Some(path) } else { None }
}

pub fn firefox_snap_folder() -> Option<PathBuf> {
    let path = snap_base()?.join("firefox").join("common").join(".mozilla").join("firefox");
    if path.exists() { Some(path) } else { None }
}

#[instrument]
pub fn debloat(path: PathBuf) -> color_eyre::Result<()> {
    let mut custom_overrides = vec![
        include_str!("../snippets/betterfox_user_config"),
        // These should be optional eventually
        include_str!("../snippets/firefox_user_js_extra"),
    ];

    if ARGS.get().unwrap().vertical_tabs {
        custom_overrides.push(include_str!("../snippets/firefox_user_js_vert_tabs"));
    }

    let profiles =
        firefox_common::debloat(path, get_better_fox_user_js, &custom_overrides.join("\n"))?;
    if !ARGS.get().unwrap().vertical_tabs {
        return Ok(());
    }

    for profile in profiles {
        let span = info_span!("Updating xulstore", %profile);
        let _enter = span.enter();

        match xulstore(&profile.path) {
            Ok(_) => debug!("updated xulstore"),
            Err(why) => warn!(err = %why, "Failed to update")
        }
    }

    Ok(())
}

#[instrument]
fn xulstore(root: &Path) -> color_eyre::Result<()> {
    let path = root.join("xulstore.json");
    if !path.exists() {
        warn!(path = %path.display(), "xulstore.json does not exist");
        return Ok(());
    }

    let xulstore_str = fs::read_to_string(&root);
    let Value::Object(mut xulstore) = serde_json::from_str::<Value>(&xulstore_str?)? else {
        bail!("Failed to parse xulstore as JSON");
    };

    let browser_content =
        get_or_insert_obj(&mut xulstore, "chrome://browser/content/browser.xhtml")
            .context("Failed to cast browser content")?;

    if let Some(vertical_tabs) = get_or_insert_obj(browser_content, "vertical-tabs") {
        vertical_tabs.insert(String::from("collapsed"), json!(false));
        debug!("collapsed vertical tabs");
    }

    if let Some(tabs_toolbar) = get_or_insert_obj(browser_content, "TabsToolbar") {
        tabs_toolbar.insert(String::from("collapsed"), json!(true));
        debug!("collapsed tabs toolbar");
    }

    fs::write(&path, serde_json::to_string_pretty(&xulstore)?)
        .map_err(color_eyre::eyre::Error::from)
}
