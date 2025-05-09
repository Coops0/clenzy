use std::sync::LazyLock;
use crate::ARGS;

// Yeah. They literally write the string '\u003C' in the file.
pub fn replace_symbols(line: &str) -> String {
    line.replace(">", "\\u003C")
}

pub static DISABLED_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    let mut lines = include_str!("../../snippets/disabled_brave_features")
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if !ARGS.get().unwrap().search_suggestions {
        lines.extend(
            include_str!("../../snippets/disabled_brave_prefetch")
                .lines()
                .filter(|line| !line.is_empty())
        );
    }

    lines
});
pub static REMOVE_ENABLED_FEATURES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    let mut lines = include_str!("../../snippets/remove_enabled_features")
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if !ARGS.get().unwrap().search_suggestions {
        lines.extend(
            include_str!("../../snippets/remove_enabled_brave_prefetch")
                .lines()
                .filter(|line| !line.is_empty())
        );
    }

    lines
});