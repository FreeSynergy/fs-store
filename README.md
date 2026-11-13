# fs-store

Installs and updates FreeSynergy packages and itself. Design: see DESIGN.md; changes: CHANGELOG.md.

## The window and its faces

`fs-store` (without arguments) opens the window. Its **face** is chosen in the window: menu **View** (German: **Ansicht**), entries
"Face: ...". The choice is remembered (`face = "storefront"` or `"compact"` in `settings.toml` in the data folder of the store,
`FS_STORE_ROOT` or `~/.local/share/fs-store`).

| Face | What it is |
|---|---|
| Storefront (default) | Craft look: tiles or a table, navigation at the left, start page, package page |
| Compact | a list at the left, the page of the package at the right (platform look) |

`--variant b|c` wears a face for one run only (it wins over the saved choice and is not saved). `--version` / `-V` prints the version.
The look (Craft or platform theme) is chosen once at the start: a face chosen in the menu changes the layout at once, its own look
follows at the next start.

## Commands and the install form

The commands of a program and its facets (`~/.local/bin/<name>`) follow the form that is active: `/usr/bin` for an rpm/deb,
`current/<name>/` for a package the store unpacked. `fs-store verify [name]` also checks that no command starts another version than
`installed.toml` says (exit code not zero, with the repair); `fs-store relink [name]` makes the commands again for the active form.

License: MIT OR Apache-2.0.
