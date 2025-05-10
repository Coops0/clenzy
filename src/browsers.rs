use crate::{brave, firefox, zen};
use std::fmt::Write;
use std::{fmt::Display, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Browser {
    Brave,
    Firefox,
    Zen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstalledVia {
    Local,
    Snap,
    Flatpak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Nightly
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Installation {
    pub browser: Browser,
    pub installed_via: InstalledVia,
    pub data_folder: PathBuf,
    pub installation_folders: Vec<PathBuf>,
    pub variant: Option<Variant>,
}

#[bon::bon]
impl Installation {
    #[builder(start_fn = new, finish_fn = build)]
    pub fn new_if_exists(
        #[builder(start_fn)]
        browser: Browser,
        installed_via: Option<InstalledVia>,
        data_folder: PathBuf,
        installation_folders: Vec<PathBuf>,
        variant: Option<Variant>
    ) -> Option<Self>{
        if !data_folder.exists() { return None }
        let installation_folders = installation_folders.into_iter().filter(|p| p.exists()).collect();
        let installed_via = installed_via.unwrap_or(InstalledVia::Local);

        Some(Self { browser, installed_via, data_folder, installation_folders, variant })
    }

    pub fn debloat(&self) -> color_eyre::Result<()> {
        (self.browser.debloat_fn())(self)
    }
}

impl Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Brave => write!(f, "Brave"),
            Self::Firefox => write!(f, "Firefox"),
            Self::Zen => write!(f, "Zen")
        }
    }
}

impl Display for InstalledVia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Snap => write!(f, "Snap"),
            Self::Flatpak => write!(f, "Flatpak")
        }
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nightly => write!(f, "Nightly")
        }
    }
}

impl InstalledVia {
    pub const fn display_discrete(self) -> &'static str {
        match self {
            Self::Local => "",
            Self::Snap => "(Snap)",
            Self::Flatpak => "(Flatpak)"
        }
    }
}

impl Browser {
    fn debloat_fn(self) -> fn(&Installation) -> color_eyre::Result<()> {
        match self {
            Self::Brave => brave::debloat,
            Self::Firefox => firefox::debloat,
            Self::Zen => zen::debloat
        }
    }
}

impl Display for Installation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.browser.to_string();
        if let Some(variant) = self.variant {
            write!(name, " ({variant})")?;
        }
        write!(f, "{} {}", name, self.installed_via.display_discrete())
    }
}