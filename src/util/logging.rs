use crate::Args;
use owo_colors::{colors::Green, OwoColorize};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter
};

pub fn success(message: &str) {
    info!("{} {}", "✓".fg::<Green>().bold(), message);
}

pub fn setup_logging(args: &Args) -> color_eyre::Result<()> {
    let filter =
        EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env()?;

    let mut verbosity = args.verbose;
    if cfg!(debug_assertions) {
        verbosity = verbosity.max(2);
    }

    if verbosity > 3 {
        warn!("Verbosity level {verbosity} is too high, defaulting to max of 3");
        verbosity = verbosity.min(3);
    }

    if verbosity == 0 {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .compact()
            .with_target(false)
            .with_level(true)
            .with_file(false)
            .with_line_number(false)
            .with_thread_names(false)
            .with_thread_ids(false)
            .with_span_events(FmtSpan::NONE)
            .event_format(
                fmt::format()
                    .without_time()
                    .with_ansi(true)
                    .with_target(false)
                    .with_level(true)
                    .with_file(false)
                    .with_line_number(false)
                    .with_thread_names(false)
                    .with_thread_ids(false)
                    .with_source_location(false)
                    .compact()
            );
        tracing_subscriber::registry().with(filter).with(fmt_layer).init();
        return Ok(());
    }

    let filter = match verbosity {
        1 => filter.add_directive("clenzy=debug".parse()?),
        2 => filter.add_directive("clenzy=trace".parse()?),
        _ => EnvFilter::builder().with_default_directive(LevelFilter::TRACE.into()).from_env()?
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(args.verbose >= 1)
        .with_thread_ids(args.verbose >= 3)
        .with_thread_names(args.verbose >= 3)
        .with_file(args.verbose >= 1)
        .with_line_number(args.verbose >= 2)
        .with_level(true);

    tracing_subscriber::registry().with(filter).with(fmt_layer).init();
    Ok(())
}