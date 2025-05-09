use crate::{firefox_common, ARGS};
use std::path::Path;
use tracing::instrument;
pub mod paths;
pub mod resource;

#[instrument(level = "debug")]
pub fn debloat(path: &Path) -> color_eyre::Result<()> {
    // Not all of these will be used but some are
    let mut custom_overrides = vec![include_str!("../../snippets/betterfox_extra")];
    if ARGS.get().unwrap().search_suggestions {
        custom_overrides.push(include_str!("../../snippets/firefox_search_suggestions"));
    }

    let _ =
        firefox_common::debloat(path, "Zen", resource::get_better_zen_user_js, &custom_overrides.join("\n"))?;
    Ok(())
}
