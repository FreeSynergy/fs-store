# fs-store

Package management library and UI for FreeSynergy — reads the Store catalog,
manages install records, and provides a Dioxus GUI and CLI.

## Build

```sh
cargo build --release
cargo test
```

## Workspace

| Crate | Description |
|---|---|
| `crates/fs-store` | Core library (catalog, inventory, packages) |
| `crates/fs-store-app` | Dioxus GUI (`fs-store`) |
| `crates/fs-store-cli` | CLI (`fs-store-cli list/info/installed`) |

## Architecture

- `StoreReader` — fetches TOML catalogs from a `StoreSource` (local or HTTP)
- `Inventory` — central hub: namespace map + install records + settings
- `Package` trait — base interface for all 9 package types
- `StoreSettings` — unified, extensible settings object

## Store Source

By default points to `https://raw.githubusercontent.com/FreeSynergy/Store/main`.
Override via `FS_STORE_URL` environment variable or `--local` flag.
