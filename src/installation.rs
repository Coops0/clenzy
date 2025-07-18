use crate::browser::Browser;
use std::{fmt::Display, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstalledVia {
    Local,
    Snap,
    Flatpak
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Nightly,
    Beta
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Installation {
    // TODO see which we can make private & remove clippy allow
    pub browser_name: &'static str,
    pub debloat_fn: fn(&Installation) -> color_eyre::Result<()>,
    pub installed_via: InstalledVia,
    pub data_folders: Vec<PathBuf>,
    pub app_folders: Vec<PathBuf>,
    pub variant: Option<Variant>
}

impl Installation {
    pub fn builder<B: Browser>() -> InstallationBuilder {
        InstallationBuilder::new::<B>()
    }

    pub fn debloat(&self) -> color_eyre::Result<()> {
        (self.debloat_fn)(self)
    }

    pub const fn is_valid(&self) -> bool {
        !self.data_folders.is_empty()
    }
}

pub struct InstallationBuilder {
    browser_name: &'static str,
    debloat_fn: fn(&Installation) -> color_eyre::Result<()>,
    installed_via: Option<InstalledVia>,
    data_folders: Vec<PathBuf>,
    app_folders: Vec<PathBuf>,
    variant: Option<Variant>
}

impl InstallationBuilder {
    fn new<B: Browser>() -> Self {
        Self {
            browser_name: B::name(),
            debloat_fn: B::debloat,
            installed_via: None,
            data_folders: Vec::new(),
            app_folders: Vec::new(),
            variant: None
        }
    }

    #[inline]
    pub const fn installed_via(mut self, installed_via: InstalledVia) -> Self {
        self.installed_via = Some(installed_via);
        self
    }

    #[rustfmt::skip]
    #[inline]
    pub fn data_folder(mut self, data_folder: Option<PathBuf>) -> Self {
        if let Some(df) = data_folder && df.exists() {
            self.data_folders.push(df);
        }
        self
    }

    #[inline]
    pub fn data_folders(mut self, data_folders: Vec<PathBuf>) -> Self {
        let extras = data_folders.into_iter().filter(|f| f.exists());

        self.data_folders.extend(extras);
        self
    }


    #[rustfmt::skip]
    #[inline]
    pub fn app_folder(mut self, app_folder: Option<PathBuf>) -> Self {
        if let Some(extra) = app_folder && extra.exists() {
            self.app_folders.push(extra);
        }
        self
    }

    #[inline]
    pub fn app_folders(mut self, app_folders: Vec<PathBuf>) -> Self {
        let extras = app_folders.into_iter().filter(|f| f.exists());

        self.app_folders.extend(extras);
        self
    }

    #[inline]
    pub const fn variant(mut self, variant: Variant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn build(self) -> Installation {
        Installation {
            browser_name: self.browser_name,
            debloat_fn: self.debloat_fn,
            installed_via: self.installed_via.unwrap_or(InstalledVia::Local),
            data_folders: self.data_folders,
            app_folders: self.app_folders,
            variant: self.variant
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
            Self::Nightly => write!(f, "Nightly"),
            Self::Beta => write!(f, "Beta"),
        }
    }
}

impl Display for Installation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.browser_name)?;
        if let Some(variant) = self.variant {
            write!(f, "/{variant}")?;
        }

        if self.installed_via != InstalledVia::Local {
            write!(f, " ({})", self.installed_via)?;
        }

        Ok(())
    }
}
