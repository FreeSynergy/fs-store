# fs-store

Installs and updates FreeSynergy packages and itself. Design: see DESIGN.md; changes: CHANGELOG.md.

## The window and its frontends

`fs-store` (without arguments) opens the window. Its **frontend** is chosen in the window: menu **View** (German: **Ansicht**), entries
"Frontend: ...". The choice is remembered (the setting is still called `face` in `settings.toml`, value `"storefront"` or `"compact"`, in the
data folder of the store, `FS_STORE_ROOT` or `~/.local/share/fs-store`; the name stays so that saved settings keep working).
Tiles, details and list are **views** of a frontend, not frontends.

| Frontend | What it is |
|---|---|
| Storefront (default) | Craft look: tiles or a table, navigation at the left, start page, package page |
| Compact | a list at the left, the page of the package at the right (platform look) |

`--variant b|c` wears a frontend for one run only (it wins over the saved choice and is not saved). `--version` / `-V` prints the version.
The look (Craft or platform theme) is chosen once at the start: a frontend chosen in the menu changes the layout at once, its own look
follows at the next start.

### Reload, messages, install question, navigation (0.0.16)

- **Reload** is the round-arrow button (`view-refresh`, tooltip "Reload catalog" / "Katalog neu laden") at the right of the top bar. A source that cannot be read is told in a message and what was shown stays.
- **Messages** at the top go away by themselves (a hint after 6 s, an error after 20 s), each has a close button; menu **Store > Messages** (German: **Meldungen**) lists every message of the session.
- **Install** is one click on "Install", then the question (what is added, size, source, rights), then **Install now**. Nothing asks again; for a system install (rpm/deb) the question says that the system asks for your password next.
- **Start** is the overview (updates, new, not-supported entries) and has no number. **Programs**, **Frontends**, **Components**, **Themes**, ... list ALL packages of their kind; the number is what the list shows.

## Commands and the install form

The commands of a program and its facets (`~/.local/bin/<name>`) follow the form that is active: `/usr/bin` for an rpm/deb,
`current/<name>/` for a package the store unpacked. `fs-store verify [name]` also checks that no command starts another version than
`installed.toml` says (exit code not zero, with the repair); `fs-store relink [name]` makes the commands again for the active form.

License: MIT OR Apache-2.0.
