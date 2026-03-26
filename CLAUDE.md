# CLAUDE.md – fs-store

## What is this?

FreeSynergy Store — the package management library and UI for FreeSynergy.
Reads the Store catalog, manages install records, and provides a Dioxus GUI
and a CLI for browsing and managing packages.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Quality Gates (before every commit)

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo test
```

Every lib.rs / main.rs must have:
```rust
#![deny(clippy::all, clippy::pedantic, warnings)]
```

## Architecture

Workspace with 3 crates:

- `crates/fs-store/` — core library
  - `StoreReader` — fetches catalogs from `StoreSource` (local or HTTP)
  - `Inventory` — central hub: namespaces + install records + settings
  - `Package` trait — base interface for all package types
  - `StoreSettings` — unified settings struct
- `crates/fs-store-app/` — Dioxus GUI (Provider Pattern via `StoreContext`)
- `crates/fs-store-cli/` — CLI (`fs-store-cli list/info/installed`)

## Dependencies

- `reqwest` (HTTP, rustls)
- `tokio` (async runtime)
- `serde` + `toml` (catalog parsing)
- `dioxus` (GUI)
- `tracing` (logging)
