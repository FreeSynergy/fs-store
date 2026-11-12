//! Pattern: none (one style sheet constant: the style sheet of the storefront (theme variables only; the icon widgets bring their own sheet)).

pub(in crate::view) const STYLES: &str = r"
.fs-store-top { padding: var(--space-2) var(--space-4); border-bottom-width: 1px; border-bottom-color: var(--border); background: var(--surface); }
.fs-store-main { padding: var(--space-3) var(--space-4) var(--space-4) var(--space-4); }
.fs-store-filters { padding: var(--space-2) 0; }
.fs-store-body { overflow: scroll; }
.fs-store-nav { padding: var(--space-3) var(--space-2); border-right-width: 1px; border-right-color: var(--border); background: var(--surface); }
.fs-store-nav-item { padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); cursor: pointer; }
.fs-store-nav-item:hover { background: var(--accent-subtle); }
.fs-store-nav-current { background: var(--selection); color: var(--accent); font-weight: 600; }
.fs-store-nav-off { opacity: 0.55; cursor: default; }
.fs-store-chip { padding: var(--space-1) var(--space-3); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); cursor: pointer; background: var(--surface); }
.fs-store-chip:hover { background: var(--accent-subtle); }
.fs-store-chip-on { background: var(--selection); border-color: var(--accent); color: var(--accent); font-weight: 600; }
.fs-store-chip-off { opacity: 0.45; cursor: default; }
.fs-store-icon-btn { border-radius: var(--radius-default); cursor: pointer; }
.fs-store-icon-btn:hover { background: var(--accent-subtle); }
.fs-store-icon-btn-on { background: var(--selection); }
.fs-store-icon-btn-off { opacity: 0.4; cursor: default; }
.fs-store-badge { padding: 2px 8px; border-width: 1px; border-radius: 3px; font-size: var(--text-xs); font-weight: 600; }
.fs-store-badge-ok { border-color: var(--success); color: var(--success); }
.fs-store-badge-warn { border-color: var(--warning); color: var(--warning); }
.fs-store-badge-bad { border-color: var(--error); color: var(--error); }
.fs-store-badge-info { border-color: var(--accent); color: var(--accent); }
.fs-store-badge-quiet { border-color: var(--border); color: var(--text-secondary); }
.fs-store-heading { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-store-fact { padding: 2px 0; }
.fs-store-link { color: var(--accent); cursor: pointer; }
.fs-store-locked { padding: var(--space-3); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); opacity: 0.7; }
.fs-store-thead { padding: var(--space-2) var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-store-trow { padding: var(--space-2) var(--space-3); border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-store-trow:hover { background: var(--accent-subtle); }
.fs-store-table { border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--surface); }
.fs-store-preview { padding: var(--space-4); border-width: 2px; border-color: var(--accent); border-radius: var(--radius-default); background: var(--surface-elevated); }
.fs-store-footer { padding: var(--space-1) var(--space-4); border-top-width: 1px; border-top-color: var(--border); background: var(--surface); }
";
