// mod_prelude.rs — common imports for all package submodules.
pub(crate) use crate::package::{Package, PackageData, PackageHelp};
pub(crate) use crate::release::PackageRelease;

/// Macro that implements the boilerplate `Package` trait methods by delegating
/// to `self.data`. Add `impl_package_data!();` inside every `impl Package for T`
/// block that wraps a `PackageData`.
macro_rules! impl_package_data {
    () => {
        fn id(&self) -> &str {
            &self.data.id
        }
        fn name(&self) -> &str {
            &self.data.name
        }
        fn summary(&self) -> &str {
            &self.data.summary
        }
        fn description(&self) -> &str {
            &self.data.description
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
        fn license(&self) -> &str {
            &self.data.license
        }
        fn homepage(&self) -> Option<&str> {
            self.data.homepage.as_deref()
        }
        fn screenshots(&self) -> &[String] {
            &self.data.screenshots
        }
        fn changelog_url(&self) -> Option<&str> {
            self.data.changelog_url.as_deref()
        }
        fn secondary_icon_path(&self) -> Option<&str> {
            self.data.secondary_icon_path.as_deref()
        }
        fn overlap_factor(&self) -> f32 {
            self.data.overlap_factor.clamp(0.0, 1.0)
        }
        fn caption(&self) -> Option<&str> {
            self.data.caption.as_deref()
        }
    };
}
pub(crate) use impl_package_data;
