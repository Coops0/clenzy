mod archive;
mod brave;
mod browser_profile;
mod browsers;
mod firefox;
mod firefox_common;
mod logging;
mod util;
mod zen;

use crate::{
    browsers::Installation, logging::{setup_logging, success}, util::{check_and_fetch_resources, check_if_running}
};
use clap::{ArgAction, Parser};
use inquire::MultiSelect;
use std::{env, sync::OnceLock};
use sysinfo::System;
use tracing::{debug_span, info, warn};

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

    let installations = brave::installations()
        .into_iter()
        .chain(firefox::installations())
        .chain(zen::installations());

    let installations: Vec<Installation> = installations.flatten().collect();

    if installations.is_empty() {
        no_browsers_msg();
        return Ok(());
    }

    // Fetches Firefox and Zen user.js scripts immediately
    check_and_fetch_resources(&installations);

    let browsers_len = installations.len();
    let installations = if args.auto_confirm {
        installations
    } else {
        MultiSelect::new("Select browsers to debloat", installations)
            .with_all_selected_by_default()
            .with_page_size(browsers_len)
            .prompt()?
    };

    if installations.is_empty() {
        return Ok(());
    }

    let mut system = System::new();

    for installation in installations {
        let span = debug_span!("debloat", browser = %installation.browser);
        let _enter = span.enter();

        check_if_running(&mut system, installation.browser);

        match installation.debloat() {
            Ok(()) => success("Finished debloating browser"),
            Err(why) => warn!(err = %why, "Failed to debloat {}", installation.browser)
        }
    }

    success("Done");
    Ok(())
}

fn no_browsers_msg() {
    info!("No supported browsers found on your computer.");
    // let supported = browsers.iter().map(|b| b.name).collect::<Vec<_>>().join(", ");
    // info!("The list of supported browsers is: {supported}.");

    if cfg!(not(any(target_os = "windows", target_os = "macos"))) {
        warn!("You may have an unsupported OS ({}).", env::consts::OS);
    }

    info!(
        "If you have any of these installed, please open an issue at https://github.com/Coops0/clenzy/issues"
    );
}
