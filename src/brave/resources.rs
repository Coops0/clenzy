use std::sync::LazyLock;
use crate::ARGS;

// Yeah. They literally write the string '\u003C' in the file.
pub fn replace_symbols(line: &str) -> String {
    line.replace('>', "\\u003C")
}

pub static DISABLED_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    let mut lines = include_str!("../../snippets/brave/disabled_features")
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if !ARGS.get().unwrap().search_suggestions {
        lines.extend(
            include_str!("../../snippets/brave/disabled_prefetch_features")
                .lines()
                .filter(|line| !line.is_empty())
        );
    }

    lines
});

pub static REMOVE_ENABLED_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    let mut lines = include_str!("../../snippets/brave/remove_features")
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if !ARGS.get().unwrap().search_suggestions {
        lines.extend(
            include_str!("../../snippets/brave/remove_prefetch_features")
                .lines()
                .filter(|line| !line.is_empty())
        );
    }

    lines
});

pub static REMOVE_ENABLED_LAB_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    include_str!("../../snippets/brave/remove_lab_experiments_features")
        .lines()
        .filter(|line| !line.is_empty())
        .collect()
});