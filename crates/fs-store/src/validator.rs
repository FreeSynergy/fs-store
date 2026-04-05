// validator.rs — PackageCatalogValidator: checks catalog completeness.
//
// Design Pattern: Template Method
//   `CatalogValidator` defines the validation algorithm in `validate()`.
//   Each `CatalogCheck` trait object implements one check.
//   Subclasses (concrete checks) run inside the template method.
//
// Usage:
//   let issues = CatalogValidator::new().validate(&pkg);
//   let complete = issues.is_empty();

use crate::package::Package;

// ── CatalogIssue ──────────────────────────────────────────────────────────────

/// One problem found by the [`CatalogValidator`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogIssue {
    /// The `description` field is missing or too short (< 20 chars).
    MissingDescription,
    /// No icon path declared.
    MissingIcon,
    /// No tags declared.
    MissingTags,
    /// License field is empty.
    MissingLicense,
    /// No `[package.origin].website` / homepage URL.
    MissingHomepage,
    /// No screenshots declared (relevant for GUI packages).
    MissingScreenshots,
}

impl CatalogIssue {
    /// Short FTL key for this issue (shown in Store UI).
    #[must_use]
    pub fn ftl_key(&self) -> &'static str {
        match self {
            Self::MissingDescription => "store-issue-missing-description",
            Self::MissingIcon => "store-issue-missing-icon",
            Self::MissingTags => "store-issue-missing-tags",
            Self::MissingLicense => "store-issue-missing-license",
            Self::MissingHomepage => "store-issue-missing-homepage",
            Self::MissingScreenshots => "store-issue-missing-screenshots",
        }
    }
}

// ── CatalogCheck ──────────────────────────────────────────────────────────────

/// One atomic validation check.
///
/// Implement this to add a new rule.  The template method `CatalogValidator::validate`
/// calls every registered check in order.
trait CatalogCheck: Send + Sync {
    fn run(&self, pkg: &dyn Package) -> Option<CatalogIssue>;
}

// ── Concrete checks ───────────────────────────────────────────────────────────

struct DescriptionCheck;
impl CatalogCheck for DescriptionCheck {
    fn run(&self, pkg: &dyn Package) -> Option<CatalogIssue> {
        if pkg.description().trim().len() < 20 {
            Some(CatalogIssue::MissingDescription)
        } else {
            None
        }
    }
}

struct IconCheck;
impl CatalogCheck for IconCheck {
    fn run(&self, pkg: &dyn Package) -> Option<CatalogIssue> {
        if pkg.icon_path().is_none() {
            Some(CatalogIssue::MissingIcon)
        } else {
            None
        }
    }
}

struct TagsCheck;
impl CatalogCheck for TagsCheck {
    fn run(&self, pkg: &dyn Package) -> Option<CatalogIssue> {
        if pkg.tags().is_empty() {
            Some(CatalogIssue::MissingTags)
        } else {
            None
        }
    }
}

struct LicenseCheck;
impl CatalogCheck for LicenseCheck {
    fn run(&self, pkg: &dyn Package) -> Option<CatalogIssue> {
        if pkg.license().trim().is_empty() {
            Some(CatalogIssue::MissingLicense)
        } else {
            None
        }
    }
}

struct HomepageCheck;
impl CatalogCheck for HomepageCheck {
    fn run(&self, pkg: &dyn Package) -> Option<CatalogIssue> {
        if pkg.homepage().is_none() {
            Some(CatalogIssue::MissingHomepage)
        } else {
            None
        }
    }
}

// ── CatalogValidator ──────────────────────────────────────────────────────────

/// Validates a [`Package`] against the completeness rules.
///
/// # Template Method
///
/// `validate()` runs all registered `CatalogCheck`s in order and collects
/// every issue found.  Add new rules by implementing `CatalogCheck` and
/// pushing an instance in `new()`.
///
/// # Example
///
/// ```
/// use fs_store::validator::CatalogValidator;
///
/// // pkg: &dyn Package
/// // let issues = CatalogValidator::new().validate(pkg);
/// // assert!(issues.is_empty(), "package is complete");
/// ```
pub struct CatalogValidator {
    checks: Vec<Box<dyn CatalogCheck>>,
}

impl CatalogValidator {
    /// Build the default validator with all standard checks.
    #[must_use]
    pub fn new() -> Self {
        Self {
            checks: vec![
                Box::new(DescriptionCheck),
                Box::new(IconCheck),
                Box::new(TagsCheck),
                Box::new(LicenseCheck),
                Box::new(HomepageCheck),
            ],
        }
    }

    /// Run all checks against `pkg`.  Returns every [`CatalogIssue`] found.
    ///
    /// An empty `Vec` means the package is complete.
    pub fn validate(&self, pkg: &dyn Package) -> Vec<CatalogIssue> {
        self.checks
            .iter()
            .filter_map(|check| check.run(pkg))
            .collect()
    }

    /// `true` when the package passes all checks.
    pub fn is_complete(&self, pkg: &dyn Package) -> bool {
        self.validate(pkg).is_empty()
    }
}

impl Default for CatalogValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        category::PackageCategory,
        package::{PackageData, PackageHelp},
        release::PackageRelease,
    };

    struct TestCategory;
    #[allow(clippy::unnecessary_literal_bound)]
    impl PackageCategory for TestCategory {
        fn id(&self) -> &str {
            "test"
        }
        fn name(&self) -> &str {
            "Test"
        }
        fn namespace_path(&self) -> &str {
            "packages/test/"
        }
    }
    static TEST_CAT: TestCategory = TestCategory;

    struct TestPkg {
        data: PackageData,
    }

    impl Package for TestPkg {
        fn id(&self) -> &str {
            &self.data.id
        }
        fn name(&self) -> &str {
            &self.data.name
        }
        fn category(&self) -> &'static dyn PackageCategory {
            &TEST_CAT
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
    }

    fn complete_pkg() -> TestPkg {
        TestPkg {
            data: PackageData {
                id: "test-pkg".into(),
                name: "Test Package".into(),
                summary: "A test package.".into(),
                description: "This is a sufficiently long description for the test package.".into(),
                icon_path: Some("icon.svg".into()),
                tags: vec!["test".into()],
                releases: vec![],
                help: PackageHelp::default(),
                license: "MIT".into(),
                homepage: Some("https://example.com".into()),
                screenshots: vec![],
                changelog_url: None,
            },
        }
    }

    #[test]
    fn complete_package_has_no_issues() {
        let pkg = complete_pkg();
        let issues = CatalogValidator::new().validate(&pkg);
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn missing_description_detected() {
        let mut pkg = complete_pkg();
        pkg.data.description = "Short".into();
        let issues = CatalogValidator::new().validate(&pkg);
        assert!(issues.contains(&CatalogIssue::MissingDescription));
    }

    #[test]
    fn missing_icon_detected() {
        let mut pkg = complete_pkg();
        pkg.data.icon_path = None;
        let issues = CatalogValidator::new().validate(&pkg);
        assert!(issues.contains(&CatalogIssue::MissingIcon));
    }

    #[test]
    fn missing_tags_detected() {
        let mut pkg = complete_pkg();
        pkg.data.tags = vec![];
        let issues = CatalogValidator::new().validate(&pkg);
        assert!(issues.contains(&CatalogIssue::MissingTags));
    }

    #[test]
    fn missing_license_detected() {
        let mut pkg = complete_pkg();
        pkg.data.license = String::new();
        let issues = CatalogValidator::new().validate(&pkg);
        assert!(issues.contains(&CatalogIssue::MissingLicense));
    }

    #[test]
    fn missing_homepage_detected() {
        let mut pkg = complete_pkg();
        pkg.data.homepage = None;
        let issues = CatalogValidator::new().validate(&pkg);
        assert!(issues.contains(&CatalogIssue::MissingHomepage));
    }

    #[test]
    fn is_complete_returns_false_when_issues_exist() {
        let mut pkg = complete_pkg();
        pkg.data.icon_path = None;
        assert!(!CatalogValidator::new().is_complete(&pkg));
    }

    #[test]
    fn is_complete_returns_true_for_valid_package() {
        let pkg = complete_pkg();
        assert!(CatalogValidator::new().is_complete(&pkg));
    }
}
