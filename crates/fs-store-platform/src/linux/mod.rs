//! The Linux (Unix) implementation of every part. Pattern: Facade (declarations).

mod background;
mod build;
mod links;
mod menu;
mod paths;
mod permissions;
mod processes;
mod system;

pub(crate) use build::platform;
pub use paths::LinuxPaths;
