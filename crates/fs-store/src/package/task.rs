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
    fn id(&self) -> &str {
        &self.data.id
    }
    fn name(&self) -> &str {
        &self.data.name
    }
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: TaskCategory = TaskCategory;
        &CAT
    }
    fn summary(&self) -> &str {
        &self.data.summary
    }
    fn icon_path(&self) -> Option<&str> {
        self.data.icon_path.as_deref()
    }
    fn tags(&self) -> &[String] {
        &self.data.tags
    }
    fn releases(&self) -> &[PackageRelease] {
        &self.data.releases
    }
    fn help(&self) -> &PackageHelp {
        &self.data.help
    }
}
