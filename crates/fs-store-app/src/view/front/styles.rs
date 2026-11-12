//! Pattern: none (one style sheet constant: what the storefront adds to the Craft sheet of `fs-components-styles`; colours and sizes
//! are theme variables, the corners stay square).

pub(in crate::view) const STYLES: &str = r"
.fs-store-main { padding: 24px 24px 32px 24px; overflow: scroll; }
.fs-store-body { overflow: scroll; }
.fs-store-empty { padding: 24px; color: var(--text-secondary); }
.fs-store-problem { padding: 10px 14px; border-width: 1px; border-color: var(--error); background: var(--error-bg); color: var(--text-primary); }
.fs-store-job { display: flex; flex-direction: column; gap: 6px; padding: 14px; border-width: 1px; border-color: var(--border); background: var(--background); }
.fs-more { border-radius: 0; }
.fs-more-open { background: var(--accent-subtle); }
";
