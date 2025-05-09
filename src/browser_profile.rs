use std::{fmt::Display, path::PathBuf};

#[derive(Clone, Debug)]
pub struct BrowserProfile {
    pub name: String,
    pub path: PathBuf
}

impl BrowserProfile {
    pub const fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }
}

impl Display for BrowserProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
