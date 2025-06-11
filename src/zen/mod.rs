mod installations;
pub mod resource;

use crate::{firefox_common, ARGS};
use tracing::{debug, instrument, warn};
use crate::browsers::Installation;

pub use installations::installations;


#[instrument(level = "debug")]
pub fn debloat(installation: &Installation) -> color_eyre::Result<()> {
    // Not all of these will be used but some are
    let mut custom_overrides = vec![include_str!("../../snippets/firefox_common/betterfox_extra")];
    if ARGS.get().unwrap().search_suggestions {
        custom_overrides.push(include_str!("../../snippets/firefox_common/search_suggestions"));
    }

    for data_folder in &installation.data_folders {
        if let Err(why) = firefox_common::debloat(installation, data_folder, resource::get_better_zen_user_js()?, &custom_overrides.join("\n")) {
            warn!(err = ?why, "Failed to debloat data folder: {}", data_folder.display());
        } else {
            debug!(data_folder = %data_folder.display(), "Successfully debloated data folder");
        }
    }

    Ok(())
}
