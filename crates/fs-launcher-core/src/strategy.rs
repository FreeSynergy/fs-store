//! Pattern: Strategy (one implementation per way of installing).

use crate::package::{Format, Version};
use crate::signature::VerifiedPackage;

/// Why an installer failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallError(pub String);

/// One way of installing: rpm through dnf, Flatpak, AppImage, tar.gz.
pub trait InstallStrategy {
    /// The format this strategy handles.
    fn format(&self) -> Format;
    /// Installs a verified package.
    ///
    /// # Errors
    /// `InstallError` when the installation fails.
    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError>;
    /// Puts back `previous` (or removes the package when `None`).
    ///
    /// # Errors
    /// `InstallError` when the undo fails.
    fn restore(&mut self, name: &str, previous: Option<Version>) -> Result<(), InstallError>;
}
