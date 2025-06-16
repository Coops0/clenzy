mod installations;
pub mod resource;

use crate::{browser::Browser, firefox_common, installation::Installation, ARGS};
use installations::installations;
use tracing::{debug, warn};

pub struct Zen;

impl Browser for Zen {
    fn name() -> &'static str {
        "Zen"
    }

    fn installations() -> Vec<Installation> {
        installations()
    }

    fn fetch_resources() -> Option<fn() -> color_eyre::Result<&'static str>> {
        Some(resource::get_better_zen_user_js)
    }

    fn debloat(installation: &Installation) -> color_eyre::Result<()> {
        debloat(installation)
    }
}

pub fn debloat(installation: &Installation) -> color_eyre::Result<()> {
    // Not all of these will be used but some are
    let mut custom_overrides = vec![include_str!("../../snippets/firefox_common/betterfox_extra")];
    if ARGS.get().unwrap().search_suggestions {
        custom_overrides.push(include_str!("../../snippets/firefox_common/search_suggestions"));
    }

    let mut found_profile = false;
    for data_folder in &installation.data_folders {
        match firefox_common::debloat::<Zen>(
            data_folder,
            resource::get_better_zen_user_js()?,
            &custom_overrides.join("\n")
        ) {
            Err(why) => {
                warn!(err = ?why, "Failed to debloat data folder: {}", data_folder.display());
            }
            Ok(profiles) => {
                found_profile |= !profiles.is_empty();
                debug!(data_folder = %data_folder.display(), "Successfully debloated data folder");
            }
        }
    }

    if !found_profile {
        warn!("Failed to find any valid profiles in Zen installation");
    }

    Ok(())
}
