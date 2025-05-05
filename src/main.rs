mod brave;
mod firefox;
mod util;

use std::env;
use tracing::level_filters::LevelFilter;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

fn main() -> color_eyre::Result<()> {
    unsafe {
        // There's no better way to enable backtraces programmatically
        env::set_var("RUST_BACKTRACE", "1");
    }

    let mut filter =
        EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env()?;

    if cfg!(debug_assertions) {
        filter = filter.add_directive("browser_debloat=debug".parse()?);
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    try_browser!("Brave", brave::brave_folder, brave::debloat);
    try_browser!("Brave Nightly", brave::brave_nightly_folder, brave::debloat);
    try_browser!("Firefox", firefox::firefox_folder, firefox::debloat);

    if !util::any_browser_found() {
        warn!("no browsers were found");
    }

    Ok(())
}
