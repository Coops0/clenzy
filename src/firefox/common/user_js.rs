use crate::{
    browser::profile::BrowserProfile, util::{UnwrapOrExit, args}
};
use color_eyre::eyre::{ContextCompat, WrapErr};
use std::{fs, path::Path};
use tracing::debug;

pub fn install_user_js(
    profile: &BrowserProfile,
    user_js: &str,
    additional_snippets: &str
) -> color_eyre::Result<()> {
    let user_js_path = profile.path.join("user.js");

    let configured_user_js = {
        let mut lines = user_js.lines().collect::<Vec<_>>();
        let start_my_overrides_pos = lines
            .iter()
            .rposition(|l| l.trim().starts_with("* START: MY OVERRIDE"))
            .wrap_err("Failed to find start of 'my overrides'")?;

        // Skip comments and a blank space
        let start_my_overrides_pos = start_my_overrides_pos + 6;

        if !additional_snippets.is_empty() {
            lines.insert(start_my_overrides_pos, additional_snippets);
        }

        debug!(
            "Added {} additional lines to user.js (originally {})",
            additional_snippets.lines().count(),
            lines.len()
        );
        Ok::<String, color_eyre::eyre::Error>(lines.join::<&str>("\n"))
    }?;

    // Checks if user.js exists and content differs from configured_user_js
    if !should_write_user_js(profile, &user_js_path, &configured_user_js) {
        debug!(path = %user_js_path.display(), "Not overwriting user.js");
        return Ok(());
    }

    fs::write(&user_js_path, configured_user_js).wrap_err("Failed to write user.js")
}

fn should_write_user_js(profile: &BrowserProfile, path: &Path, user_js_str: &str) -> bool {
    if !path.exists() || args().auto_confirm {
        return true;
    }

    let existing_fs_user_js = fs::read_to_string(path).unwrap_or_default();
    let mut efuj_lines = existing_fs_user_js.lines();
    let ujs_lines = user_js_str.lines();

    // Check whether all our lines are present in the existing user.js
    let mismatches = ujs_lines
        .filter(|preset_line| {
            let preset_line = preset_line.trim();

            // Line isn't empty or comment, make sure this line is valid to check for
            !preset_line.is_empty() && !preset_line.starts_with("//")
            // No matching line
            && !efuj_lines.any(|line| line.trim().eq_ignore_ascii_case(preset_line))
        })
        .count();

    if mismatches > 5 {
        debug!(path = %path.display(), "user.js already exists and is the same");
        return false;
    }

    if existing_fs_user_js.is_empty() {
        debug!(path = %path.display(), "user.js already exists but is empty");
        return true;
    }

    inquire::Confirm::new(&format!(
        "user.js already exists for profile {profile}. Do you want to overwrite it? (y/n)"
    ))
    .prompt()
    .unwrap_or_exit()
}
