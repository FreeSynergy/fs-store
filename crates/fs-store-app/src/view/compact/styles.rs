//! Pattern: none (one style sheet constant: variant C over theme variables only, so any style can wear it; a Breeze style is a request to fs-components).

pub(in crate::view) const STYLES: &str = r"
.fs-c-root { background: var(--background); }
.fs-c-list { background: var(--surface); border-right-width: 1px; border-right-color: var(--border); }
.fs-c-search { padding: 8px; }
.fs-c-chips { padding: 0 8px 8px 8px; }
.fs-c-chip { padding: 3px 10px; border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); cursor: pointer; }
.fs-c-chip:hover { background: var(--accent-subtle); }
.fs-c-chip-on { background: var(--accent); border-color: var(--accent); }
.fs-c-chip-on-text { color: var(--text-inverse); }
.fs-c-rows { overflow: scroll; }
.fs-c-row { padding: 6px 10px; cursor: pointer; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-c-row:hover { background: var(--accent-subtle); }
.fs-c-row-on { background: var(--accent); }
.fs-c-row-on:hover { background: var(--accent); }
.fs-c-on-text { color: var(--text-inverse); }
.fs-c-sub { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-c-count { padding: 4px 10px; border-top-width: 1px; border-top-color: var(--border); color: var(--text-secondary); font-size: var(--text-xs); }
.fs-c-detail { padding: 16px 20px; overflow: scroll; }
.fs-c-head { padding-bottom: 12px; border-bottom-width: 1px; border-bottom-color: var(--border); }
.fs-c-card { padding: 12px; border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--surface); }
.fs-c-h { font-weight: 600; font-size: var(--text-base); }
.fs-c-ok { color: var(--success); }
.fs-c-warn { color: var(--warning); }
.fs-c-bad { color: var(--error); }
.fs-c-info { color: var(--accent); }
.fs-c-warning { padding: 6px 8px; border-left-width: 3px; border-left-color: var(--warning); background: var(--warning-bg); }
.fs-c-track { height: 6px; background: var(--border); border-radius: var(--radius-default); }
";
