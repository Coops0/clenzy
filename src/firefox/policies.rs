use crate::{browser_profile::BrowserProfile, s, util::UnwrapOrExit, ARGS};
use color_eyre::eyre::Context;
use serde_json::json;
use std::{fs, path::Path};
use tracing::debug;

pub fn create_policies_file(
    profile: &BrowserProfile,
    app_folder: &Path
) -> color_eyre::Result<()> {
    let policies = generate_policies()?;
    let folder = if cfg!(target_os = "macos") {
        // Firefox.app/Contents/Resources/distribution
        app_folder
            .join("Firefox.app")
            .join("Contents")
            .join("Resources")
            .join("distribution")
    } else if cfg!(target_os = "linux") {
        // Same for windows and linux
        app_folder.join("distribution")
    } else {
        unreachable!();
    };

    let policies_path = folder.join("policies.json");
    if !should_write_policies(profile, &policies_path, &policies) {
        debug!(path = %policies_path.display(), "Not overwriting policies.json");
        return Ok(());
    }

    let _ = fs::create_dir_all(fs::canonicalize(&folder).unwrap_or(folder));
    fs::write(&policies_path, policies).wrap_err("Failed to write policies.json")
}

fn should_write_policies(profile: &BrowserProfile, policies_path: &Path, policies: &str) -> bool {
    if !policies_path.exists() || ARGS.get().unwrap().auto_confirm {
        return true;
    }

    let existing_fs_policies = fs::read_to_string(policies_path).unwrap_or_default();
    if existing_fs_policies == policies {
        debug!(path = %policies_path.display(), "policies.json already exists and is the same");
        return false;
    }

    if policies.is_empty() {
        debug!(path = %policies_path.display(), "policies.json is empty");
        return true;
    }

    inquire::Confirm::new(&format!(
        "policies.json already exists for profile {profile}. Do you want to overwrite it? (y/n)"
    ))
    .prompt()
    .unwrap_or_exit()
}

fn generate_policies() -> serde_json::Result<String> {
    let mut obj = json!({"policies": {}});
    let policies = obj.as_object_mut().unwrap();
    // If set to false, application updates are downloaded but the user can choose when to install the update.
    policies.insert(s!("AppAutoUpdate"), json!(false));
    // Enables or disables autofill for payment methods.
    policies.insert(s!("AutofillCreditCardEnabled"), json!(false));
    // Disable the menus for reporting sites (Submit Feedback, Report Deceptive Site).
    policies.insert(s!("DisableFeedbackCommands"), json!(true));
    // Disable Firefox studies (Shield).
    policies.insert(s!("DisableFirefoxStudies"), json!(true));
    // Disable the "Forget" button.
    policies.insert(s!("DisableForgetButton"), json!(true));
    // Remove the master password functionality.
    // If this value is true, it works the same as setting PrimaryPassword to false and removes the primary password functionality.
    policies.insert(s!("DisableMasterPasswordCreation"), json!(true));
    // Turn off saving information on web forms and the search bar.
    policies.insert(s!("DisableFormHistory"), json!(true));
    // Remove Pocket in the Firefox UI. It does not remove it from the new tab page.
    policies.insert(s!("DisablePocket"), json!(true));
    // Disables the “Import data from another browser” option in the bookmarks window.
    policies.insert(s!("DisableProfileImport"), json!(true));
    // Prevent the upload of telemetry data.
    // As of Firefox 83 and Firefox ESR 78.5, local storage of telemetry data is disabled as well.
    policies.insert(s!("DisableTelemetry"), json!(true));
    // Set the initial state of the bookmarks toolbar. A user can still change how it is displayed.
    // `always` means the bookmarks toolbar is always shown.
    // `never` means the bookmarks toolbar is not shown.
    // `newtab` means the bookmarks toolbar is only shown on the new tab page.
    policies.insert(s!("DisplayBookmarksToolbar"), json!(s!("newtab")));
    // Don’t check if Firefox is the default browser at startup.
    policies.insert(s!("DontCheckDefaultBrowser"), json!(true));
    // Customize the Firefox Home page.
    let firefox_home = json!({
        "Search": true,
        "TopSites": false,
        "SponsoredTopSites": false,
        "Highlights": false,
        "Pocket": false,
        "SponsoredPocket": false,
        "Snippets": false,
        "Locked": false
    });
    policies.insert(s!("FirefoxHome"), firefox_home);
    // Customize Firefox Suggest (US only).
    let firefox_suggest = json!({
        "WebSuggestions": ARGS.get().unwrap().search_suggestions,
        "SponsoredSuggestions": false,
        "ImproveSuggest": false,
        "Locked": false
    });
    policies.insert(s!("FirefoxSuggest"), firefox_suggest);
    // Enable or disable network prediction (DNS prefetching).
    policies.insert(s!("NetworkPrediction"), json!(ARGS.get().unwrap().search_suggestions));
    // Sets the default value of signon.rememberSignons without locking it.
    policies.insert(s!("OfferToSaveLoginsDefault"), json!(false));
    // Override the first run page. If the value is an empty string (“”), the first run page is not displayed.
    policies.insert(s!("OverrideFirstRunPage"), json!(""));
    // Enable search suggestions.
    policies.insert(s!("SearchSuggestEnabled"), json!(ARGS.get().unwrap().search_suggestions));
    // Show the home button on the toolbar.
    policies.insert(s!("ShowHomeButton"), json!(false));
    // If true, don’t display the Firefox Terms of Use and Privacy Notice upon startup. You represent that you accept and have the authority to accept the Terms of Use on behalf of all individuals to whom you provide access to this browser.
    policies.insert(s!("SkipTermsOfUse"), json!(true));
    // Prevent Firefox from messaging the user in certain situations.
    let user_messaging = json!({
        "WhatsNew": false, // Remove the "What’s New" icon and menuitem. (Deprecated)
        "ExtensionRecommendations": false, // If false, don’t recommend extensions while the user is visiting web pages.
        "FeatureRecommendations": false, // If false, don’t recommend browser features.
        "UrlbarInterventions": ARGS.get().unwrap().search_suggestions, // If false, don’t offer Firefox specific suggestions in the URL bar.
        "SkipOnboarding": true, // If true, don’t show onboarding messages on the new tab page.
        "MoreFromMozilla": false, // If false, don’t show the "More from Mozilla" section in Preferences.
        "FirefoxLabs": false, // If false, don’t show the "Firefox Labs" section in Preferences.
        "Locked": false // prevents the user from changing user messaging preferences
    });
    policies.insert(s!("UserMessaging"), user_messaging);

    serde_json::to_string(&obj)
}
