use color_eyre::eyre::{bail, ContextCompat, WrapErr};
use serde_json::{json, Map, Value};
use std::{fs, path::Path};
use tracing::instrument;

#[instrument]
pub fn get_local_state(root: &Path) -> color_eyre::Result<Map<String, Value>> {
    let local_state_path = root.join("Local State");
    let local_state_str =
        fs::read_to_string(&local_state_path).wrap_err("Failed to read Local State")?;

    let Value::Object(local_state) =
        serde_json::from_str::<Value>(&local_state_str).wrap_err("Failed to parse Local State")?
    else {
        bail!("Failed to cast Local State to object");
    };

    Ok(local_state)
}

#[instrument(skip(local_state))]
pub fn update_local_state(
    mut local_state: Map<String, Value>,
    root: &Path
) -> color_eyre::Result<()> {
    let browser = local_state
        .get_mut("browser")
        .and_then(Value::as_object_mut)
        .context("Failed to get browser object")?;

    browser.insert(String::from("default_browser_infobar_declined_count"), json!(9999));

    fs::write(root.join("Local State"), serde_json::to_string(&local_state)?)
        .wrap_err("Failed to write Local State")
}
