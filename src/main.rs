mod archive;
mod brave;
mod browser_profile;
mod firefox;
mod firefox_common;
mod logging;
mod util;
mod zen;

use crate::{
    logging::{setup_logging, success}, util::{check_and_fetch_resources, check_if_running}
};
use brave::paths;
use clap::{ArgAction, Parser};
use inquire::MultiSelect;
use std::{
    env, fmt::Display, path::{Path, PathBuf}, sync::OnceLock
};
use sysinfo::System;
use tracing::{info, info_span, warn};

#[derive(Parser, Default)]
#[command(version)]
pub struct Args {
    /// Print extra debug information (max 3 levels with -vvv)
    #[clap(short, long, action = ArgAction::Count, default_value_t = 0)]
    pub verbose: u8,

    /// Assume yes to all prompts
    #[clap(long = "auto-confirm", short = 'Y', default_value_t = false)]
    pub auto_confirm: bool,

    /// Disable setting browsers to use vertical tabs
    #[clap(long = "no-vertical-tabs", short = 'V', action = ArgAction::SetFalse, default_value_t = true)]
    pub vertical_tabs: bool,

    /// Disable the creation of backups
    #[clap(long = "no-backup", short = 'B', action = ArgAction::SetFalse, default_value_t = true)]
    pub backup: bool,

    /// Enable search suggestions and prefetching. Every word in the URL bar you type will be sent to your search provider.
    #[clap(long = "search-suggestions", short = 'S', default_value_t = false)]
    pub search_suggestions: bool
}

pub static ARGS: OnceLock<Args> = OnceLock::new();

fn main() -> color_eyre::Result<()> {
    unsafe {
        // There's no better way to enable backtraces programmatically
        env::set_var("RUST_BACKTRACE", "1");
    }

    let args = ARGS.get_or_init(Args::parse);

    setup_logging(args)?;

    let browsers: [BrowserTuple; 10] = [
        ("Brave", paths::brave_folder(), brave::debloat),
        ("Brave Nightly", paths::brave_nightly_folder(), brave::debloat),
        ("Brave (Snap)", paths::brave_snap_folder(), brave::debloat),
        ("Brave (Flatpak", paths::brave_flatpak_folder(), brave::debloat),
        ("Firefox", firefox::paths::firefox_folder(), firefox::debloat),
        ("Firefox (Snap)", firefox::paths::firefox_snap_folder(), firefox::debloat),
        ("Firefox (Flatpak)", firefox::paths::firefox_flatpak_folder(), firefox::debloat),
        ("Zen", zen::paths::zen_folder(), zen::debloat),
        ("Zen (Unofficial Snap)", zen::paths::zen_snap_folder(), zen::debloat),
        ("Zen (Flatpak)", zen::paths::zen_flatpak_folder(), zen::debloat)
    ];

    let browsers = browsers
        .into_iter()
        .filter_map(|(name, path, debloat)| Some((name, path?, debloat)))
        .map(|(name, path, debloat)| Browser { name, folder: path, debloat })
        .collect::<Vec<_>>();

    if browsers.is_empty() {
        no_browsers_msg(&browsers);
        return Ok(());
    }

    // Fetches Firefox and Zen user.js scripts immediately
    check_and_fetch_resources(&browsers);

    let browsers_len = browsers.len();
    let browsers = if args.auto_confirm {
        browsers
    } else {
        MultiSelect::new("Select browsers to debloat", browsers)
            .with_all_selected_by_default()
            .with_page_size(browsers_len)
            .prompt()?
    };

    if browsers.is_empty() {
        return Ok(());
    }

    let mut system = System::new();

    for browser in browsers {
        let span = info_span!("debloat", browser = %browser.name);
        let _enter = span.enter();

        check_if_running(&mut system, browser.name);

        match (browser.debloat)(&browser.folder) {
            Ok(()) => success("Finished debloating browser"),
            Err(why) => warn!(err = %why, "Failed to debloat {}", browser.name)
        }
    }

    success("Done");
    Ok(())
}

type BrowserTuple = (&'static str, Option<PathBuf>, fn(&Path) -> color_eyre::Result<()>);

pub struct Browser {
    pub name: &'static str,
    folder: PathBuf,
    debloat: fn(&Path) -> color_eyre::Result<()>
}

impl Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn no_browsers_msg(browsers: &[Browser]) {
    info!("No supported browsers found on your computer.");
    let supported = browsers.iter().map(|b| b.name).collect::<Vec<_>>().join(", ");
    info!("The list of supported browsers is: {supported}.");

    if cfg!(not(any(target_os = "windows", target_os = "macos"))) {
        warn!("You may have an unsupported OS ({}).", env::consts::OS);
    }

    info!(
        "If you have any of these installed, please open an issue at https://github.com/Coops0/clenzy/issues"
    );
}
