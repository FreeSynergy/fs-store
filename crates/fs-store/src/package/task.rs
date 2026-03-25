// task.rs — TaskPackage: automation task templates.

use serde::{Deserialize, Serialize};

use crate::category::{PackageCategory, TaskCategory};
use crate::package::mod_prelude::*;

/// An automation task template package.
///
/// Tasks describe event-triggered pipelines (e.g. "on git.commit → notify chat").
/// They are community-shareable, similar to themes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// Bus capability namespaces this task listens to, e.g. `["git.commit"]`.
    #[serde(default)]
    pub listens: Vec<String>,

    /// Bus capability namespaces this task emits, e.g. `["chat.notify"]`.
    #[serde(default)]
    pub emits: Vec<String>,
}

impl Package for TaskPackage {
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: TaskCategory = TaskCategory;
        &CAT
    }
}
