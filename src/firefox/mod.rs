pub mod paths;
pub mod resource;
mod xulstore;

use crate::{firefox_common, logging::success, ARGS};
use std::path::Path;
use tracing::{info_span, instrument, warn};

#[instrument]
pub fn debloat(path: &Path) -> color_eyre::Result<()> {
    let mut custom_overrides = vec![
        include_str!("../../snippets/betterfox_user_config"),
        // These should be optional eventually
        include_str!("../../snippets/firefox_user_js_extra"),
    ];

    if ARGS.get().unwrap().vertical_tabs {
        custom_overrides.push(include_str!("../../snippets/firefox_user_js_vert_tabs"));
    }

    let profiles = firefox_common::debloat(
        path,
        "Firefox",
        resource::get_better_fox_user_js,
        &custom_overrides.join("\n")
    )?;
    if !ARGS.get().unwrap().vertical_tabs {
        return Ok(());
    }

    for profile in profiles {
        let span = info_span!("Updating xulstore", %profile);
        let _enter = span.enter();

        match xulstore::xulstore(&profile.path) {
            Ok(()) => success(&format!("Updated xulstore.json for {profile}")),
            Err(why) => warn!(err = %why, "Failed to update")
        }
    }

    Ok(())
}
