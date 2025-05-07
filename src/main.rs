mod brave;
mod firefox;
mod firefox_common;
mod util;
mod zen;

use crate::util::warn_if_process_is_running;
use clap::{ArgAction, Parser};
use inquire::MultiSelect;
use std::{
    env, fmt::Display, io::{stdin, Read}, path::PathBuf, sync::OnceLock
};
use sysinfo::System;
use tracing::{info, info_span, level_filters::LevelFilter, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Default)]
pub struct Args {
    #[clap(short, long, default_value_t = false)]
    pub verbose: bool,

    // Assume yes to all prompts
    #[clap(short = 'Y', default_value_t = false)]
    pub auto_confirm: bool,

    // Disable enabling vertical tabs
    #[clap(long = "no-vertical-tabs", action = ArgAction::SetFalse, default_value_t = true)]
    pub vertical_tabs: bool
}

pub static ARGS: OnceLock<Args> = OnceLock::new();

fn main() -> color_eyre::Result<()> {
    unsafe {
        // There's no better way to enable backtraces programmatically
        env::set_var("RUST_BACKTRACE", "1");
    }

    let args = ARGS.get_or_init(Args::parse);

    let mut filter =
        EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env()?;

    let mut fmt_layer = tracing_subscriber::fmt::layer().without_time().pretty();
    if cfg!(debug_assertions) || args.verbose {
        filter = filter.add_directive("browser_debloat=debug".parse()?);
    } else {
        fmt_layer =
            fmt_layer.with_target(false).with_level(true).with_file(false).with_line_number(false);
    }

    tracing_subscriber::registry().with(filter).with(fmt_layer).init();

    let browsers: [BrowserTuple; 4] = [
        ("Brave", brave::brave_folder(), brave::debloat),
        ("Brave Nightly", brave::brave_nightly_folder(), brave::debloat),
        ("Firefox", firefox::firefox_folder(), firefox::debloat),
        ("Zen", zen::zen_folder(), zen::debloat)
    ];

    let browsers = browsers
        .into_iter()
        .filter_map(|(name, path, debloat)| Some((name, path?, debloat)))
        .map(|(name, path, debloat)| Browser { name, folder: path, debloat })
        .collect::<Vec<_>>();

    if browsers.is_empty() {
        warn!("No browsers found");
        return Ok(());
    }

    let browsers = if args.auto_confirm {
        browsers
    } else {
        MultiSelect::new("Select browsers to debloat", browsers)
            .with_all_selected_by_default()
            .prompt()
            .unwrap_or_default()
    };

    if browsers.is_empty() {
        return Ok(());
    }

    let mut system = System::new();

    for browser in browsers {
        let span = info_span!("debloat", browser = %browser.name);
        let _enter = span.enter();

        if !ARGS.get().unwrap().auto_confirm
            && warn_if_process_is_running(&mut system, browser.name)
        {
            let _ = stdin().read_exact(&mut [0_u8]);
        }

        match (browser.debloat)(browser.folder) {
            Ok(_) => info!("Debloated {}", browser.name),
            Err(why) => warn!(err = ?why, "Failed to debloat {}", browser.name)
        }
    }

    info!("Done");
    Ok(())
}

type BrowserTuple = (&'static str, Option<PathBuf>, fn(PathBuf) -> color_eyre::Result<()>);

struct Browser {
    name: &'static str,
    folder: PathBuf,
    debloat: fn(PathBuf) -> color_eyre::Result<()>
}

impl Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
