use crate::util::get_or_insert_obj;
use color_eyre::eyre::{bail, ContextCompat};
use serde_json::{json, Value};
use std::{fs, path::Path};
use tracing::{debug, instrument, warn};

#[instrument(level = "debug")]
pub fn xulstore(root: &Path) -> color_eyre::Result<()> {
    let path = root.join("xulstore.json");
    if !path.exists() {
        warn!(path = %path.display(), "xulstore.json does not exist");
        return Ok(());
    }

    let xulstore_str = fs::read_to_string(&path);
    let Value::Object(mut xulstore) = serde_json::from_str::<Value>(&xulstore_str?)? else {
        bail!("Failed to parse xulstore as JSON");
    };

    let browser_content =
        get_or_insert_obj(&mut xulstore, "chrome://browser/content/browser.xhtml")
            .context("Failed to cast browser content")?;

    if let Some(vertical_tabs) = get_or_insert_obj(browser_content, "vertical-tabs") {
        vertical_tabs.insert(String::from("collapsed"), json!(false));
        debug!("Collapsed vertical tabs");
    }

    if let Some(tabs_toolbar) = get_or_insert_obj(browser_content, "TabsToolbar") {
        tabs_toolbar.insert(String::from("collapsed"), json!(true));
        debug!("Collapsed tabs toolbar");
    }

    fs::write(&path, serde_json::to_string(&xulstore)?).map_err(color_eyre::eyre::Error::from)
}
