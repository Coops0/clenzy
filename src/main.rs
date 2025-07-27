mod brave;
mod browser;
mod firefox;
mod zen;
mod util;

use crate::util::{check_if_running, start_fetch_resource, RenderedBrowser};
use crate::{
    brave::Brave, browser::Browser, firefox::Firefox, zen::Zen
};
use clap::{ArgAction, Parser};
use inquire::MultiSelect;
use std::{env, sync::OnceLock};
use std::sync::LazyLock;
use sysinfo::System;
use tracing::{debug_span, info, warn};
use util::logging::{setup_logging, success};

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
    #[clap(long = "no-vertical-tabs", action = ArgAction::SetFalse, default_value_t = true)]
    pub vertical_tabs: bool,

    /// Disable the creation of backups
    #[clap(long = "no-backup", action = ArgAction::SetFalse, default_value_t = true)]
    pub backup: bool,

    /// Disable search suggestions and prefetching. Every word in the URL bar you type will be sent to your search provider if search suggestions are enabled.
    #[clap(long = "no-search-suggestions", action = ArgAction::SetFalse, default_value_t = true)]
    pub search_suggestions: bool,

    /// Enable creating policy files, for Firefox only right now
    #[clap(long = "create-policies", short = 'P', default_value_t = false)]
    pub create_policies: bool
}

pub static ARGS: OnceLock<Args> = OnceLock::new();
pub static BROWSERS: LazyLock<Vec<RenderedBrowser>> = LazyLock::new(|| render_browsers!(Firefox, Brave, Zen));

fn main() -> color_eyre::Result<()> {
    if cfg!(debug_assertions) {
        unsafe {
            // There's no better way to enable backtraces programmatically
            env::set_var("RUST_BACKTRACE", "1");
        }
    }

    let args = ARGS.get_or_init(Args::parse);

    setup_logging(args)?;

    let installations = BROWSERS
        .iter()
        .flat_map(|browser| &browser.installations)
        .filter(|installation| installation.is_valid())
        .collect::<Vec<_>>();

    if installations.is_empty() {
        no_browsers_msg();
        return Ok(());
    }

    for browser in &*BROWSERS {
        if let Some(fetch_resources) = browser.fetch_resources {
            start_fetch_resource(fetch_resources, browser.name);
        }
    }

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
        let span = debug_span!("debloat", browser = %installation.browser_name);
        let _enter = span.enter();

        check_if_running(&mut system, installation.browser_name);

        match installation.debloat() {
            Ok(()) => success(&format!("Finished debloating {}", installation.browser_name)),
            Err(why) => warn!(err = %why, "Failed to debloat {}", installation.browser_name)
        }
    }

    success("Done");
    Ok(())
}

fn no_browsers_msg() {
    info!("No supported browsers found on your computer.");
    let supported = BROWSERS.iter()
        .flat_map(|browser| &browser.installations)
        .map(|installation| format!("{installation}"))
        .collect::<Vec<_>>()
        .join(", ");

    info!("Supported browsers: {supported}");
    if cfg!(not(any(target_os = "windows", target_os = "macos"))) {
        warn!("You may have an unsupported OS ({}).", env::consts::OS);
    }

    info!(
        "If you DO have any of these installed, please open an issue at https://github.com/Coops0/clenzy/issues"
    );
}