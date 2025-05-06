mod brave;
mod engines;
mod firefox;
mod util;
mod zen;

use clap::Parser;
use inquire::MultiSelect;
use std::{
    env, fmt::Display, io::{stdin, Read}, path::PathBuf, sync::OnceLock
};
use sysinfo::System;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Default)]
pub struct Args {
    #[clap(short, long, default_value_t = false)]
    pub verbose: bool,
    
    // Assume yes to all prompts
    #[clap[short = 'Y', default_value_t = false]]
    pub autoconfirm: bool
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

    if cfg!(debug_assertions) || args.verbose {
        filter = filter.add_directive("browser_debloat=debug".parse()?);
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

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

    let browsers = if args.autoconfirm {
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

    let system = System::new();

    for browser in browsers {
        let processes = system.processes();
        let running_instances = processes
            .values()
            .filter_map(|p| {
                let name = p.name().to_str()?;
                if name.to_lowercase().contains(&browser.name.to_lowercase()) {
                    None
                } else {
                    Some(name)
                }
            })
            .collect::<Vec<_>>();

        if !running_instances.is_empty() {
            warn!(browser = %browser.name, instances = running_instances.join(", "), "Please close all instances of before debloating");
            if stdin().read_exact(&mut [0_u8]).is_err() {
                continue;
            }
        }

        match (browser.debloat)(browser.folder) {
            Ok(_) => info!("Debloated {}", browser.name),
            Err(why) => warn!(err = ?why, "Failed to debloat {}", browser.name)
        }
    }

    info!("done");

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
