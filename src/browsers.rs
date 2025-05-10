use crate::{brave, firefox, zen};
use std::{fmt::Display, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Browser {
    Brave,
    Firefox,
    Zen
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstalledVia {
    Local,
    Snap,
    Flatpak
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
    pub variant: Option<Variant>
}

impl Installation {
    pub const fn builder(browser: Browser) -> InstallationBuilder {
        InstallationBuilder::new(browser)
    }

    pub fn debloat(&self) -> color_eyre::Result<()> {
        self.browser.debloat_fn()(self)
    }
}

pub struct InstallationBuilder {
    browser: Browser,
    installed_via: Option<InstalledVia>,
    data_folder: Option<PathBuf>,
    installation_folders: Vec<PathBuf>,
    variant: Option<Variant>
}

impl InstallationBuilder {
    const fn new(browser: Browser) -> Self {
        Self {
            browser,
            installed_via: None,
            data_folder: None,
            installation_folders: Vec::new(),
            variant: None
        }
    }

    #[inline]
    pub const fn installed_via(mut self, installed_via: InstalledVia) -> Self {
        self.installed_via = Some(installed_via);
        self
    }

    #[inline]
    pub fn data_folder(mut self, data_folder: PathBuf) -> Self {
        if data_folder.exists() {
            self.data_folder = Some(data_folder);
        }
        self
    }

    #[inline]
    pub fn installation_folder(mut self, installation_folder: PathBuf) -> Self {
        if installation_folder.exists() {
            self.installation_folders.push(installation_folder);
        }
        self
    }

    #[inline]
    pub const fn variant(mut self, variant: Variant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn build(self) -> Option<Installation> {
        let data_folder = match self.data_folder {
            Some(f) if f.exists() => f,
            _ => return None
        };

        Some(Installation {
            browser: self.browser,
            installed_via: self.installed_via.unwrap_or(InstalledVia::Local),
            data_folder,
            installation_folders: self.installation_folders,
            variant: self.variant
        })
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

impl Display for Installation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.browser)?;
        if let Some(variant) = self.variant {
            write!(f, "/{variant}")?;
        }

        if self.installed_via != InstalledVia::Local {
            write!(f, " ({})", self.installed_via)?;
        }

        Ok(())
    }
}
