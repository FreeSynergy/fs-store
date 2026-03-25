// category.rs — PackageCategory trait and all concrete category types.
//
// A category is NOT an enum. It is an object (trait) so that third-party
// store sources can introduce new categories without modifying core code.
// This follows the Open/Closed Principle.

// ── PackageCategory trait ─────────────────────────────────────────────────────

/// Describes the namespace/category a package belongs to.
///
/// Each concrete package type returns a `&'static dyn PackageCategory` from
/// its [`Package::category`] implementation. Because concrete category types
/// are zero-sized, a `static` instance costs nothing at runtime.
///
/// [`Package::category`]: crate::package::Package::category
pub trait PackageCategory: Send + Sync {
    /// Slug used in catalog paths and bus namespaces, e.g. `"containers"`.
    fn id(&self) -> &str;

    /// Human-readable name, e.g. `"Container Services"`.
    fn name(&self) -> &str;

    /// Store-relative prefix for this namespace, e.g. `"packages/containers/"`.
    fn namespace_path(&self) -> &str;

    /// Optional Store-relative path to an SVG icon for this category.
    fn icon_path(&self) -> Option<&str> {
        None
    }
}

// ── Concrete categories ───────────────────────────────────────────────────────

macro_rules! category {
    (
        $type:ident,
        id = $id:literal,
        name = $name:literal,
        path = $path:literal
        $(, icon = $icon:literal)?
    ) => {
        #[doc = concat!("Category for `", $id, "` packages.")]
        #[derive(Debug)]
        pub struct $type;

        impl PackageCategory for $type {
            fn id(&self) -> &str {
                $id
            }
            fn name(&self) -> &str {
                $name
            }
            fn namespace_path(&self) -> &str {
                $path
            }
            $(
                fn icon_path(&self) -> Option<&str> {
                    Some($icon)
                }
            )?
        }
    };
}

category!(
    AppCategory,
    id = "apps",
    name = "Applications",
    path = "packages/apps/"
);
category!(
    ContainerCategory,
    id = "containers",
    name = "Container Services",
    path = "packages/containers/"
);
category!(
    ThemeCategory,
    id = "themes",
    name = "Themes",
    path = "packages/themes/"
);
category!(
    WidgetCategory,
    id = "widgets",
    name = "Widgets",
    path = "packages/widgets/"
);
category!(
    TaskCategory,
    id = "tasks",
    name = "Tasks",
    path = "packages/tasks/"
);
category!(
    LanguageCategory,
    id = "languages",
    name = "Language Packs",
    path = "packages/languages/"
);
category!(
    IconSetCategory,
    id = "icons",
    name = "Icon Sets",
    path = "packages/icons/"
);
category!(
    BundleCategory,
    id = "bundles",
    name = "Bundles",
    path = "bundles/"
);
category!(
    ExternalCategory,
    id = "external",
    name = "External Services",
    path = "packages/external/"
);

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_category_fields() {
        let cat = ContainerCategory;
        assert_eq!(cat.id(), "containers");
        assert_eq!(cat.namespace_path(), "packages/containers/");
    }

    #[test]
    fn bundle_category_path() {
        assert_eq!(BundleCategory.namespace_path(), "bundles/");
    }
}
