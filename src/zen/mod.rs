use crate::firefox_common;
use std::path::Path;
use tracing::instrument;
pub mod paths;
pub mod resource;

#[instrument]
pub fn debloat(path: &Path) -> color_eyre::Result<()> {
    // Not all of these will be used but some are
    let custom_overrides = include_str!("../../snippets/betterfox_user_config");
    let _ =
        firefox_common::debloat(path, "Zen", resource::get_better_zen_user_js, custom_overrides)?;
    Ok(())
}
