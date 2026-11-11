# fs-store (FreeSynergy.Store) – design

## Purpose
The store installs and updates FreeSynergy packages and itself. A package is a program, a frontend, a component or a
theme (ADR 0036, 0021). Packages come from a list of Git sources. Every download is checked against an OpenPGP
signature (ADR 0051), a failed update is rolled back, and a running program is never replaced. The launcher updates
itself automatically, by the same path as any other package.

Ideas from the function analysis `temp/research/craft-launcher.md` are used as ideas only, nothing is copied (ADR 0013).

## Pattern map
| Part | Pattern | Why |
|------|---------|-----|
| `source` – the list of Git sources | Repository (`SourceRepository`) | Storage of the list is swappable; the rule "reference source can be disabled, never removed" sits behind the trait. |
| `package` – name, kind, version, format | Value object / Newtype | Valid on construction, compared by value. |
| `signature` – may this package be installed | Specification (`SignaturePolicy`) | A pure predicate over the verification outcome, combinable later (e.g. with a trust level). |
| `signature` – checking bytes against a signature | Strategy (`SignatureVerifier`) | The OpenPGP implementation is swappable; tests use a fake. |
| `signature` – `VerifiedPackage` | Capability object | Only the policy can hand it out; holding it is the right to install. An unsigned package cannot even be passed to an installer. |
| `fs-store-pgp` – rPGP behind `SignatureVerifier` | Adapter | Wraps the foreign `pgp` crate in our trait, so the core stays free of dependencies. rPGP (MIT OR Apache-2.0), not sequoia (LGPL). |
| `fs-store-sources` – TOML file | Repository (file-backed) | The file holds only the user's sources and a `disabled` list; the reference source is never an entry, so deleting it from the file cannot remove it. |
| `words` – texts for people | none | Sentences as constants with named placeholders (coding rules 2a); no behaviour besides one pure fill function. |
| `strategy` – how a package is installed | Strategy (`InstallStrategy`) | rpm via dnf, Flatpak, AppImage, tar.gz: one trait, one implementation per way. |
| `state` – course of one installation | State (enum plus one transition fn) | `Failed` has exactly one way out: `RolledBack`. |
| `update` – update and roll back | State plus `InstallStrategy::restore` | There is one operation so far (install/update with undo); a `Command` trait with `apply`/`invert` comes when remove is added and there are two or more commands. |
| `update` – running check | Strategy (`RunningCheck`) | The OS check is outside the core. |

## Engine and frontend (ADR 0021)
`fs-store-core` has no input or output: no network, no files, no GUI. An engine crate will implement the traits
(`gix`, a real OpenPGP library, dnf/Flatpak calls). Frontends (Blinc GUI, CLI, web page) are packages of their own and
talk to the engine through the contract below. They are interchangeable.

## Rules fixed in code
- The built-in reference source: `reference = true`. `set_enabled(false)` works, `remove` fails with `ReferenceSource`.
- Trusted key fingerprint, built in: `16A4A3EE563AEF51FCF0C55DED7B57C4A1904342`. A checksum from the same release is not a signature.
- Outcome other than `Valid` with the trusted fingerprint: no `VerifiedPackage`, so no install.
- Update order: running check, install (previous version remembered), on error `Failed` then `invert` then `RolledBack`.

## Catalog format (decision, step 2)
Standard: **AppStream collection metadata, XML** (freedesktop). Reasons: it is the format `dnf`/GNOME Software/Flathub
already use, it has `<releases>`, `<artifacts>` with `<location>` and `<checksum>`, and it allows extensions through
`<custom><value key="fs:...">`; the DEP-11 YAML variant is the same data for Debian and adds nothing for us. A source
is a Git repo with `catalog.xml` and `catalog.xml.asc` (detached OpenPGP signature over the catalog). Extension keys:
`fs:kind` (program, frontend, component, theme; default program), `fs:format` (tar.gz, rpm, deb, exe, flatpak, appimage; default
tar.gz), `fs:signature` (URL of the detached signature of the release file), `fs:facets` (ADR 0061, comma separated
facet ids of a program; each facet is its own component with a logo, installing it installs the program). A component
without `fs:signature`, without sha256, or with a version that is not `major.minor.patch` is **reported as skipped,
never offered and never dropped silently** (`fs-store-catalog`, `Catalog::skipped`). The facet list format is
provisional until a first program ships facets.

## Contract `store.capnp` (in `fs-interfaces` v0.48.0, draft; the sketch below is the older first form)
```
interface Launcher {
  sources        @0 () -> (list :List(Source));
  addSource      @1 (url :Text) -> (id :Text);
  setEnabled     @2 (id :Text, enabled :Bool) -> ();
  removeSource   @3 (id :Text) -> ();        # error referenceSource for the built-in one
  available      @4 () -> (list :List(Package));
  installed      @5 () -> (list :List(Package));
  install        @6 (name :Text) -> (job :Job);
  update         @7 (name :Text) -> (job :Job);   # self-update is update("fs-store")
  watch          @8 (job :Job, sink :Sink) -> ();
}
struct Source  { id @0 :Text; url @1 :Text; enabled @2 :Bool; reference @3 :Bool; }
struct Package { name @0 :Text; kind @1 :Kind; version @2 :Text; format @3 :Format; source @4 :Text; }
enum Kind   { program @0; frontend @1; component @2; theme @3; }
enum Format { rpm @0; flatpak @1; appImage @2; tarGz @3; }
enum Phase  { pending @0; verified @1; installing @2; installed @3; failed @4; rolledBack @5; }
```
Risk marks (ADR 0021): `install`, `update`, `removeSource` are write operations set by the engine.

## Probe against a real release (step 5, partly done 2026-11-08)
Draft `v0.6.1-rc.2` of `fs-tasks` (made by `fs-release`, fetched with `gh release download`, because drafts are not
public): the built-in `PgpVerifier` accepts `fs-tasks-server-0.6.1-rc.2-linux-x86_64.tar.gz` with its `.asc` as `Valid`
for `16A4…4342`, and rejects it after one flipped byte (`crates/fs-store-pgp/tests/real_release.rs`, runs when
`FS_PROBE_FILE`/`FS_PROBE_SIG` are set). **Done 2026-11-08:** the whole chain through a local `file://` Git catalog: `fs-store install file:///tmp/probe/cat fs-tasks-server --trust-key testkey.pub.asc` cloned it with `gix`, verified the catalog (signed by a throw-away test key made with gpg), checked sha256 and the real release signature, installed `0.6.1-rc.2` and `fs-store installed` lists it. Without `--trust-key` the same catalog is refused (`CatalogSignature`). `--trust-key <file>` trusts an extra public key for that one run only; the built-in key always stays trusted. The old note follows.

Old note: installing it through a catalog. Blockers: (1) fixed 2026-11-08: `Version` now has a pre-release part; (2) a local
catalog needs a signature by the release key, which only the user holds. Options for (2): the user signs a test catalog,
or a test-only trusted key for local sources.

A single top directory named after the package (`fs-tasks-server-0.6.1-rc.2-linux-x86_64/`) is stripped on install, so the program sits directly in `current/<name>/`; other single directories such as `bin/` are content and stay.

## Review of 2026-11-08 (second agent) and what was done
Fixed: symlink/hard-link tar entries refused (could write outside the root); `./` prefixed archives accepted; unpacking
goes to a sibling directory and is renamed, an installed version is never touched again (reinstall cannot destroy the
working install); the verified bytes go to the unpacker directly (no predictable staging file to swap); package names
must be plain file names (`PackageName`); a catalog offering an older version than the installed one is refused
(`Downgrade`, replay); the CLI verifies the catalog signature before showing or believing anything and tries the next
source when one fails; the root is made absolute; unpacked size and entry count are capped; catalog: a bad old release no longer hides
a good one, artifacts are chosen by platform, duplicate ids are reported. Contract: source per call, `JobFailure`,
`rollbackFailed`, `uninstall`, `refresh`, signer and version on `Job`.
**Not fixed yet (named limits):** the catalog has no serial/timestamp, so a replayed catalog is only caught when an
older version than the installed one is offered (a freeze, i.e. the same old catalog forever, is not detected); trust is
one flat list, not per source or per name space (a test key given with `--trust-key` is valid for every name in that run);
the OpenPGP check does not look at key or signature expiry, subkey flags or subkey revocation; `ProcRunningCheck` does
not see programs started through an interpreter; the real error behind `RolledBack` is not carried up; a symlink in a
package archive is refused, which will need a rule (relative and inside the package) when a package needs one.

## Packaging and gate (step 7, prepared 2026-11-08)
`.github/workflows/check.yml` (push gate of `fs-release`) and `release.yml` (binaries `fs-store` = CLI and `fs-store-app` = GUI, app id
`net.freesynergy.Store`). Workspace version `0.1.0-rc.1`, so the first tag is `v0.1.0-rc.1`, a pre-release; the draft is signed only
once the repo has the signing secrets. `check_rust_rules.py` is green locally (0 errors); `lib.rs` files hold only modules and re-exports.
rpm (done in code, tested with a fake commander only): `Rpm` writes the verified bytes to a private file (0700 dir, 0600, `O_EXCL|O_NOFOLLOW`) and runs `pkexec dnf install -y <file>`; root is asked by pkexec, never by us. dnf is one transaction, so `restore` has nothing to undo; a downgrade is the person's decision. Limit: the running check does not see programs installed under `/usr`. Not yet run for real.
Not yet tried: the remote run of both workflows, build dependencies of Blinc on the CI runner (`apt-packages` may be needed), rpm via dnf.

## Lesson of v0.1.0-rc.1 (2026-11-08)
The release workflow signs with the **subkey** `372A…C097`, but the public key embedded in `fs-store-pgp` was the older file
without that subkey, so our own verifier rejected the store's own release (`Invalid`) while `gpg` accepted it. Found by the
real-file probe (`FS_PROBE_FILE`), not by the generated-key tests. Fixed in `0.1.0-rc.2` by embedding the current
`fs-release/release-signing-key.asc`. rc.1 is public and cannot install anything signed by the subkey; use rc.2.
When `fs-release` rotates the key or subkey, the file in `crates/fs-store-pgp/keys/` must follow (a test against a
real signed release belongs in the release checklist).

## One binary (ADR 0063, 2026-11-08; replaces the two-binary naming of the same day)
`fs-store` is the only binary (crate `fs-store`, `crates/fs-store`). No arguments: the window (`fs-store-app::gui::run`).
Subcommands as in git: `fs-store sources|available|install|update|installed ...` (`fs-store-cli::run`), `fs-store serve`
(server; answers "not built yet" until `fs-store-server` exists). The other crates are libraries. Pattern: Facade over
Command. Cargo feature `gui` (default): `cargo build --no-default-features -p fs-store` builds without graphics (then
the bare command prints a hint). rpm, menu entry "FreeSynergy.Store" (`packaging/net.freesynergy.Store.desktop`) and the
catalog list only `fs-store`.

## Window resize crash (found by the user, 2026-11-08; solved by Q0, 2026-11-11)
Enlarging the window aborted with a wgpu validation error (copy into a swapchain texture one frame behind). Cause was in
`blinc_app` 0.6.0; the fix lives in our Blinc fork (tag `fs-blinc-2`). Until Q0 the store carried a `[patch.crates-io]` for it,
which is gone: see "One Blinc" below.

## One Blinc (Q0, 2026-11-11)
The store has no `blinc*` dependency and no `[patch]`. Blinc comes only through `fs-components` (git tag `v0.0.5`, which
pins our fork once); the window code writes `use fs_components::blinc::layout::...` (re-export of layout, cn, core, app,
animation, theme, noto_emoji). All `fs-components*` dependencies stand on the same tag. `.fs-rules.toml` has
`blinc = "components-only"`, so `fs-release`'s `check_blinc` fails the repo on a `blinc*` line, a `[patch]` or two tags.
Blinc macros (`flow!`, derive) are not used by the store. Moving to a new fs-components tag: `scripts/bump-components.sh` of fs-release.

## Server and window, update, `--demo` (Q0, 2026-11-11)
- `fs-store serve` answers as `run_until_signal_as(socket, ServerIdentity::new("fs-store", version), ...)`: a client of
  another version can replace it (`connect_matching`), and it answers `Control.stop`. The window needs no handshake: it talks
  to the engine in the process, not to a server (a Cap'n Proto `StoreBackend` may change that; then it uses `connect_matching`
  with `CommandStarter::new(current_exe, ["serve"])` and shows `HandshakeError` with "Try again").
- `--facet NAME` of the window is read by `fs_interfaces::rpc::facet_from_args`/`parse_facet`; the store has one view, so a valid
  facet changes nothing and a bad one is an error.
- **End server after an update (job step).** `fs-<name> serve` of a program is ended in order by `end_server`: it asks
  `Control.stop` over the socket (`--socket` of its command line, else the default `<name>.sock`). The process to wait for is the
  OWNER OF THE SOCKET (peer credentials, `SO_PEERCRED`), not a process found by its command line; a server older than `Control`
  answers `Unsupported` and then that owner gets SIGTERM through `rustix` (never this process, never PID 1 or lower, no external
  `kill`, never SIGKILL). Only a server whose socket cannot be reached at all is signalled by the process id of the `/proc`
  scan, and only while its command line still is the one found. Known gap: a process id can be reused between check and signal (a
  `pidfd` would close it; same follow-up as in `fs-interfaces`). `fs-interfaces` v0.0.17 keeps `server_pid`/`terminate` private
  (`pub(super)`), so the store does the same with `tokio`'s `peer_cred` and `rustix` itself; they should become public there.
  This happens before the files are replaced (a running program is never replaced) and again as the last job step after a good
  install (`ending.rs`): a server that runs now is asked `Control.version`; one that already says the new version (the window
  started it meanwhile) is left alone, any other (also one that does not answer) is ended. Each server gets `END_WAIT` =
  `FINISH_TIMEOUT` of `fs-interfaces` + 5 s (35 s), so a busy server that finishes its work is not taken for "did not end".
  A failed install, or a server that does not end, starts the servers that did end again (old version); after a good install
  the new server is NOT started here, the program's own window starts it (`connect_matching`). A server that cannot be ended is
  never silent: `InstallFailure::ServerNotEnded` -> `JobError::ServerNotEnded`; the job is installed, the register is updated, and the
  page says "installed, but the old background server could not be ended: <why>". Tests with fake servers:
  `fs-store-engine/tests/end_server.rs` and the unit tests of `ending.rs` (new server running already, old server, stubborn server).
  **Windows:** `servers.rs` and `end.rs` have `cfg(unix)` where it uses Unix API and answers "nothing found" / `Unsupported` otherwise; the
  replacement follows in the win-port (the rest of the engine, links, tar.gz, facets, is Unix-only as before).
- **`--demo` / `--view sources` (for the typing test only).** `fs-store --demo` runs the window over sealed in-memory parts
  (`demo::parts`): `DemoBackend` (one reference source, three programs; a job installs nothing and says so), `DemoUpdates`
  (settings and findings in memory; its check refuses, so no `fs-store background`, no catalog over the network, no `updates.toml`,
  no notification), no check at the start, `DemoLauncher` (starts no process, opens no address), and no menu entry is written.
  Proof: `fs-store-app/tests/demo.rs`. Why: `fs-release/actions/typing-test` starts the binary on
  a virtual screen and types into a text field; the real engine would read the person's folders, clone catalogs and install. `--view
  sources` opens on the page with the field "Git address of a source". Workflow `typing-test.yml` calls the action with
  `--demo --view sources --size 900x640` and `click-in-window: 300,190`; on GitHub the time is only reported.

## Facets as commands (ADR 0063 addendum, 2026-11-08; format provisional, built 2026-11-08 for the links and menu entries)
A facet gets its own command as a symbolic link to the program (`fs-contacts` -> `fs-people`); the program reads `argv[0]`
(like busybox) and opens that facet, `fs-contacts admin` is the facet's admin. For the store this means, on installing a
program: for every facet in its catalog entry, create the link in `~/.local/bin` and a menu entry (`.desktop`) with the facet's
icon ("FreeSynergy.People", "FreeSynergy.Contacts"); on removing: take both away again, and only links that point into the
package. Catalog format: the program's `fs:facets` (comma separated command names) names the links; a facet's display name
and icon come from its own component (`<name>`, `<icon type="remote">`), whose `fs:facet-of` names the program. Facet names are
reserved project-wide: the store refuses a link whose name already exists and is not its own. Built: `fs-store-engine::link_facets/unlink_facets`; links point to the program (`<package>` or `<package>-app`), menu entries `net.freesynergy.<command>.desktop` (marked `X-FreeSynergy-Managed=true`; only such files and links into the package are ever replaced or removed), the facets made are kept in `installed.toml` so `remove` can take them away; GUI installs do the same as the commands. No icons yet; a facet's own name and icon from its catalog component are not read yet (the name is derived from the command).

## TUF (ADR 0062) – roles and keys, proposal for the user to confirm (2026-11-08)
Library: `tough` 0.24 (MIT OR Apache-2.0, builds here); client = `RepositoryLoader`, publishing = its editor. TUF keys are
raw Ed25519 (or RSA/ECDSA) keys, **not OpenPGP keys**; gpg cannot sign TUF's canonical JSON. So the OpenPGP key `16A4…4342`
cannot literally be the TUF root key. Proposal that keeps the chain of trust on `16A4…4342`:
- `root`: 2 of 3 offline Ed25519 TUF keys of the user, expiry 1 year. The first `root.json` is **built into the store** (like the
  OpenPGP key) and `root.json` is published with a detached OpenPGP signature by `16A4…4342`; a new root is accepted only
  when TUF's rotation rules hold **and** that signature is valid. So both the TUF keys and the project key must agree.
- `targets`: a CI key (Ed25519, secret in `fs-catalog`), expiry 3 months; lists `catalog.xml` and the package files with hashes.
- `snapshot` and `timestamp`: another CI key, expiry 7 days / 1 day (renewed by a scheduled CI run), so a mirror cannot
  keep an old state alive. A mirror that serves expired metadata is refused (the freeze attack named in the review).
- The OpenPGP signatures on catalog and packages stay (ADR 0051): TUF protects against rollback, freeze and mix-and-match;
  the OpenPGP signature protects the content, and works without TUF for a plain download.
The user must make the root keys; the CI keys are made in `fs-catalog` by a workflow. Not built yet.

## Register, automatic updates, commands on the PATH (2026-11-08)
`installed.toml` (in the store root) records per package: version, source (Git address) and `auto_update` (default on).
`fs-store update` / `update --all` updates every package with automatic updates on, each from its own source; one failing
package does not stop the others. `fs-store auto <name> on|off` switches one package. `fs-store timer enable|disable|status`
installs a systemd **user** timer (daily, 5 min after boot, random delay) that runs `fs-store update --all`; the store updates
itself the same way (package `fs-store` in the catalog). After an install the executables at the top of the package are linked
into `~/.local/bin` (or `FS_STORE_BIN`) through `current/<name>/`; foreign files of the same name are never replaced; `remove`
takes the links away. Rollback is the one of `run_update`. **Not yet:** TUF checks in the update path (catalog signature and
the no-downgrade rule apply), programs asking the store for updates through `store.capnp` (needs the server), a per-program
reminder in the programs. `fs-store check` lists newer versions without installing (for a start-up hint). Known gap: `fs-catalog` is signed by CI in a second commit, so for a few seconds after a push `catalog.xml` and `catalog.xml.asc` do not match and clients refuse the catalog (safe, but noisy); fix: sign before pushing, or publish the pair in one commit. The reference source is now `https://github.com/FreeSynergy/fs-catalog`.

## Freshness until TUF is built (2026-11-08)
The catalog root carries `fs-serial` (Unix time of generation, only goes up) and `fs-expires` (21 days later). `fs-catalog`'s
workflow regenerates and signs every Monday and on a change, and commits `catalog.xml` and `catalog.xml.asc` in one commit
(so the signing gap named above is closed). The store checks after the signature: past `fs-expires` -> refused (freeze, a mirror
serving an old copy); lower than the highest serial seen for that source (`serials.toml`) -> refused (rollback); a catalog
without them is accepted. This covers rollback and freeze for the catalog but not mix-and-match or key compromise; those are what
TUF adds (`tough`, roles above). The same check runs in the commands and in the window.

## Install mode and commands (2026-11-08, user test)
Rule (user): the store installs the way it was installed itself. `preferred_format()` is `Rpm` when the running executable belongs
to an installed rpm (`rpm -qf`), else `TarGz`. With `Rpm` and rpm artifacts in the catalog (`fs-format="rpm"` + `fs-signature` on
each artifact, all rpms of the release), every rpm is checked (sha256 + signature) and `pkexec dnf install -y <all>` runs once; with
no rpm offered it falls back to the `tar.gz`. Commands (ADR 0063): only `fs-<name>` (and the facets) are put on the PATH, not
`-app`/`-server`; `fs-<name>` links to `<name>` or `<name>-app` (in the package or in `/usr/bin`), and the menu entry "FreeSynergy.<Name>"
runs it. The store makes its own menu entry at the first start of the window. Open (named): the shipped `fs-tasks-app` stores in memory by
default (`--store`): a program from the store must never lose data by default (Thread G/R); the server is not started by anything yet; icons.

## The package of the system (Thread D, 2026-11-10)
Rule (user, 2026-11-09): the store knows the system and offers the matching package: rpm, deb or exe; what it cannot do says
`Unsupported` out loud. Pattern: Strategy selection (`System` picks the format; `Rpm` and `Deb` are `InstallStrategy`
implementations over the same `Commander`, sharing the file handling in `native.rs` as a Template method).
- **System:** `detect_system()` reads `ID`/`ID_LIKE` of `/etc/os-release` (debian, ubuntu -> `Deb`; fedora, rhel, centos, rocky, almalinux ->
  `Rpm`, the dnf families; anything else `Other`, also openSUSE: it has rpm but `zypper`, no installer for it); on Windows (`cfg!(windows)`) it is `Windows`.
- **Format:** `format_for(system, owned)`: `Rpm`/`Deb` when the store itself belongs to an installed package of that system
  (`rpm -qf` or `dpkg -S`), else `TarGz` (a store unpacked by hand brings tar.gz, as before); `Windows` -> `Exe`.
- **Catalog:** artifacts next to the main file with `fs-format="rpm"`, `"deb"` or `"exe"`, a `platform` that fits this machine
  (`<arch>-...<os>...`) and `fs-signature`. They never become the main file. `CatalogEntry` has `rpms`, `debs`, `exes`.
- **Install:** all deb of a batch go into ONE `pkexec apt-get install -y --no-remove <files>` (like rpm and dnf; `--no-remove` because apt, unlike dnf, would otherwise remove conflicting packages without asking). Checked by hand in a
  `debian:13` container: apt installs a 0600 file from a 0700 directory of another user, `dpkg-query` reports `ii |1.2.3`,
  `apt-get remove` takes it away. `native_installed_version` asks rpm or dpkg by system; `remove_native` removes accordingly.
- **Fallback:** a deb system whose release has no deb gets the `tar.gz` (it works). A release with a different main
  format than tar.gz/rpm/deb (flatpak, appimage) stays `Unsupported(format)` as before.
- **Unsupported (explicit):** on Windows `plan` answers `Unsupported(Exe)` ("the Windows setup.exe cannot be installed by the store
  yet"): there is no installer for it, and `fs-store` itself does not build for Windows (`std::os::unix` in the installers). The
  catalog and the choice are tested; running the setup.exe is not built. Open: Windows store, `Msi`/zip as alternatives.
- A main file (`fs:format`) that is a deb is `Unsupported(Deb)` for a store that is not a deb store. A release with only exe files has no main file and is `NotOffered` on Windows (open with the Windows store).
- **Contract:** `store.capnp` `PackageFormat` got `deb @4` and `exe @5` (fs-interfaces, tag v0.0.3).
- **v0.0.9 (R5 findings of v0.0.8):** (1) no exe/zip for the store is intended and now written down: the engine is Unix-only (symlinks, `/proc`, file modes), so `windows-targets` stays empty and a Windows store would be `Unsupported`; (2) the deb binary differed from tar.gz/rpm because `cargo deb` stripped an already stripped file (same BuildID, 256 bytes fewer): fs-release now calls `cargo deb --no-strip`; (3) `fs-catalog/scripts/gen_catalog.py` now writes rpm, deb and exe artifacts (x86_64 and aarch64).
- **Not tested here:** `pkexec` with a real person's password (as for rpm); Ubuntu apt (only Debian 13 by hand).

## Look for updates, tell the person (2026-11-08, user wish after the rc.9 test)
`settings.toml`: `check` = off | start | daily (default) | hourly, `install` = ask (default) | auto, `notify` = on (default).
`fs-store settings [check .. | install .. | notify ..]` shows and sets them; `fs-store background` is the check the timer runs
(`timer enable` follows `check`: hourly -> 1h, daily -> 1d): it fetches the sources (signature and freshness checked),
compares with the register, writes `updates.toml` (updates, programs new since the last check, the names offered; no "new" flood on the
very first check), tells the person (`notify-send`, and the launcher badge as Unity `LauncherEntry` count over `gdbus`; both best effort, switched off by
`notify off`) and installs by itself only with `install auto`. The window reads `updates.toml` for "Updates (n)" and the "New" marks.
**Measured (release build, 13 MB package, 2026-11-08):** catalog 0.65 s, download 2.8 s, sha256 0.01 s, signature 0.02 s, unpack 0.16 s
(`FS_STORE_TIMING=1` prints the steps). So a slow rpm update is `pkexec`/`dnf`; dnf now gets `--setopt=metadata_expire=never`
(a local rpm needs no repository refresh; dry run 1.5 s -> 0.65 s with cache). The time spent typing the password in the pkexec dialog is not ours.

## `fs-store-server`: programs ask the store for updates (2026-11-08, first slice)
Programs never download by themselves; they ask the store ("is there an update for me?") over `store.capnp` on a Unix
socket (`$XDG_RUNTIME_DIR/freesynergy/store.sock`), started by `fs-store serve [--socket PATH]` (feature-independent). Crate
`crates/fs-store-server` only translates; the shared server side (socket, peer check, problems) is `fs-interfaces` with feature `rpc`.

| Pattern | Where |
|---|---|
| Facade | `lib.rs` declares modules and re-exports only |
| Strategy | trait `StoreView` (sources, available, installed): the server asks it, never reads files itself |
| Adapter | `LocalStore` puts `sources.toml`, `installed.toml`, the signed catalogs and `rpm -q` behind `StoreView`; `wire.rs` puts values onto the wire |
| Template method | `catalogs.rs`: clone, verify signature, parse, check freshness; a failing source answers as a skipped entry |
| Value object | `Capabilities` built once; `capabilities`, `supportReport` and every refusal answer from it |
| Null object | the writes answer the provider-rule `Unsupported` |
| Command | `serve_command` is the subcommand `fs-store serve` |

Answers in this slice: `capabilities`, `supportReport`, `sources`, `available` (signed catalogs of enabled sources, as
`fs-store available`), `installed` (register `installed.toml`; the version `rpm -q` reports wins, then the format is `rpm`; kind
is `program` and format `tarGz` otherwise, as the register does not hold them). **Unsupported, with a sentence:** `addSource`,
`setEnabled`, `removeSource`, `refresh`, `install`, `update`, `uninstall` (no write acts without a person; the transport does not
know the caller yet, ADR 0054), and `job`, `subscribe` (the server starts no job). No change feed yet. Later: writes once the
caller is known (acts-in-name), jobs and `subscribe` with them.

## TUF client (2026-11-08, first slice)
`fs-store-tuf::read_target(root, metadata_dir, targets_dir, name)` reads a file through `tough` 0.24: signatures of the chain
root -> timestamp -> snapshot -> targets, versions, expiry, and the target's length and hash are checked before a byte is returned
(5 tests against the TUF reference-implementation repository: a listed file is read; an unlisted, a changed file, damaged metadata and an unrelated root
are refused). In the store: when `FS_STORE_TUF_ROOT=<root.json>` is set, a source must carry `tuf/metadata` and `tuf/targets` and its `catalog.xml`
must be the exact file the metadata signs; else the source is refused. **Not done:** the project's own root (the user makes the root keys and signs
`root.json` with `16A4…4342` as described above), a TUF layout in `fs-catalog` (CI keys for targets/snapshot/timestamp, weekly re-signing of timestamp),
TUF for the release files themselves (today: OpenPGP signature plus the sha256 in the signed catalog), the built-in root.

## Update hint in the programs (decision, provisional, 2026-11-08)
Crate `fs-store-client`: `update_hint("fs-tasks") -> Option<UpdateHint{program, from, to}>` (and `.sentence()`). It reads what the store's
background check wrote (`updates.toml`) and the person's `notify` setting (`settings.toml`); it never downloads and never fails a
program (a missing or unreadable file means no hint). A program calls it once at start (a plain notice with the sentence; the
store opens with the program's `fs-store` command). Why a file and not `store.capnp`: no process has to run, and the server cannot know its
callers yet (ADR 0054); the same question can later go through the contract. Depend on it by Git tag:
`fs-store-client = { git = "https://github.com/FreeSynergy/fs-store", tag = "v0.0.N" }`.

## TUF publishing (2026-11-08)
Crate `fs-store-tuf-publish` (library plus binary `fs-store-tuf-publish`, on `tough` 0.24's editor). Directory layout it works on:
`catalog.xml`, `root.json` (trusted), `targets.pk8`, `snapshot.pk8`, `timestamp.pk8` (Ed25519, PKCS#8 DER); it writes `tuf/metadata/*.json` and
`tuf/targets/catalog.xml` (consistent snapshots off). Expiries: targets 90 days, snapshot 7, timestamp 1 (`--targets-days`, `--snapshot-days`,
`--timestamp-days`). Versions only go up: each run reads `tuf/metadata/{targets,snapshot,timestamp}.json` and writes version + 1. So every run
re-signs all three roles, which is also the renewal. `fs-store-tuf-publish generate <dir> [--root-keys 3 --threshold 2 --root-days 365]`
makes `root-N.pk8`, the three role keys and a signed `root.json` (tests use the same function with throw-away keys in a temp dir; no key is ever committed).
6 tests: publish then `read_target` reads `catalog.xml`; an expired timestamp, a changed catalog and a foreign root are refused; a republish bumps versions.

Who holds which key:
- `root-*.pk8`: the user, offline (2 of 3), never on a server. `root.json` is also signed with OpenPGP `16A4...4342` (detached `root.json.asc`) and
  published; the store accepts a new root only when both agree (rotation check still to be built).
- `targets.pk8`, `snapshot.pk8`, `timestamp.pk8`: CI secrets of `fs-catalog` (`TUF_TARGETS_KEY`, `TUF_SNAPSHOT_KEY`, `TUF_TIMESTAMP_KEY`, base64 of the file).
- Re-signing: timestamp lives 1 day, so a daily scheduled run of `sign.yml` is needed (weekly is the limit for snapshot only). A mirror cannot keep an old state alive.

What the user must do once: `fs-store-tuf-publish generate /safe/place` on an offline machine; move `root-*.pk8` to offline storage; put the three
role keys into the `fs-catalog` secrets and delete them locally; `gpg --detach-sign --armor root.json`; copy `root.json` (and `root.json.asc`) to
`fs-catalog/tuf/root.json` and `root.json` to `crates/fs-store-cli/keys/tuf-root.json` (this switches the check on, see below).

Ready-to-use step for `fs-catalog`'s `sign.yml` (not committed there):
```yaml
on:
  schedule: [{cron: "17 3 * * *"}]   # daily: the timestamp expires after 1 day
  push: {branches: [main]}
jobs:
  tuf:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install --git https://github.com/FreeSynergy/fs-store --tag <tag> fs-store-tuf-publish
      - run: |
          cp tuf/root.json root.json
          printf %s "${{ secrets.TUF_TARGETS_KEY }}"   | base64 -d > targets.pk8
          printf %s "${{ secrets.TUF_SNAPSHOT_KEY }}"  | base64 -d > snapshot.pk8
          printf %s "${{ secrets.TUF_TIMESTAMP_KEY }}" | base64 -d > timestamp.pk8
          fs-store-tuf-publish publish .
          rm -f targets.pk8 snapshot.pk8 timestamp.pk8 root.json
      - run: git add tuf && git commit -m "tuf: re-sign" && git push   # with the bot identity of sign.yml
```
Store side: `crates/fs-store-cli/src/tuf.rs` uses `FS_STORE_TUF_ROOT` if set, else the built-in root `crates/fs-store-cli/keys/tuf-root.json`
(embedded by `build.rs` when the file exists at build time). **Not active until the project root exists:** the file is absent now, so a plain build
checks nothing. Once it exists every source must carry valid TUF metadata that signs its exact `catalog.xml` (a source without `tuf/metadata` is
refused, so metadata cannot be stripped).

## Install queue and servers (2026-11-08, user wish after the test)

- **Two halves of an install.** `plan` (verified catalog, which files), `download`, `check` (sum, signature) change nothing on the machine; `commit` installs. Several packages are one batch (Unit of Work): `prepare_all` runs the downloads side by side, `commit_all` installs when none is left. All rpm of a batch go into ONE `pkexec dnf install` (one password); tar.gz are unpacked one after the other. A failed download does not stop the others; its error stays on its card. A package added after the install step starts a new batch.
- **Window.** The model holds a list of jobs (one phase each); `take_commit` hands the ready jobs out once. Button "Update all (N)" shows only when updates wait; `fs-store update --all` is one batch too.
- **Running servers.** `fs-<name> serve` never blocks an update: the store ends it (`Control.stop`, SIGTERM for an old one; never SIGKILL, see "Server and window"), waits up to 15 s, installs, starts the server again with the same options only when the install failed (Q0: after a good install the program's window starts the new one). A server that does not end in time keeps the update away ("still running"). Only processes of the same person are touched.
- **Open.** Window asks the program to "save the session and restart" through the update hint (ADR 0065 point 7); the real `pkexec dnf` path of a batch and the window itself are not tried by hand yet.

## Provisional decisions
- Versions are `major.minor.patch` with an optional pre-release `-alpha|beta|rc[.N]` (`~` accepted as in `AppStream`/rpm); a pre-release is older than its release. Anything else is skipped with a reason.
- Only one trusted fingerprint for now; sources may later bring their own keys, never replacing the built-in one.
- Texts a person reads (`words.rs`) come with the first frontend; the core only has English developer errors.
- Name of the program "Launcher" and crate names are provisional.

## State
Core traits and value types; rPGP adapter (`fs-store-pgp`, built-in release key and the revoked key as files, tested with
generated test keys); TOML source list (`fs-store-sources`); `words.rs`. 27 tests; `fs-store-catalog` reads AppStream (4 tests); `fs-store-engine` has `TarGz` (install to a root dir, atomic symlink switch, old version kept, escaping archive entries refused) and `ProcRunningCheck` (5 tests). `service::Installer` runs the whole chain (catalog signature, release file, sha256, release signature, `run_update`) over a `Fetch` trait; 4 tests with a fake verifier, so far not run with real keys. `net`: `HttpsFetch` (https only, 512 MiB cap) and `clone_catalog` (`gix`, depth 1, tested against a local repo). `fs-store-cli` (binary `fs-store`: `available`, `install`, `installed`; `FS_STORE_ROOT` overrides `~/.local/share/fs-store`) is the thin test front end; subcommands `sources [add|enable|disable|remove]`, `available [source]`, `install <name> [source]`, `update <name> [source]` (only if newer), `installed`; tried by hand against the local catalog, no automated CLI test yet. Self-update (step 3, done): the store is a package like any other; the switch of `current/<name>` is a symlink swap, so the running old executable is never overwritten, and `ProcRunningCheck` ignores the calling process (tested with a real running program). The new version starts at the next launch. Still open in step 3 (rest): `gix` fetch, HTTPS download, rpm via dnf/pkexec, self-update (atomic swap of the own executable). Contract moved to `fs-interfaces` (`schema/store.capnp`, no tag yet, waits for a review by a second agent); the server `fs-store-server` is not built. Not built (older list): `gix`, real
installers, self-replacement of the executable, GUI/CLI, the capnp file, a `Command` trait (comes with remove),
key expiry checks and subkey rules in `fs-store-pgp` (provisional: a valid signature by the primary key or a public
subkey of a non-revoked trusted key counts), translation of `words.rs`, atomic write of the source file.

## Start page, job list, display names (Q3, 2026-11-10)

- Updates live on the start page (block "Updates" above the cards, with "Update all"), not in a menu entry: `StoreModel::update_rows`.
- Installing runs in the background (queue above); every job is one line of the job list (`job_lines`), nothing else is
  blocked; only an uninstall disables the buttons. Finished jobs go with "Clear finished".
- Display name: ONE function, `fs_store_core::display_name(id, title)` (`names.rs`): the catalog `<name>` wins (an empty one, or one that still starts with `fs-`, counts as missing), else
  `fs-people` becomes `FreeSynergy.People`. `StoreModel::title` is what every window and sentence uses; `fs-...` stays
  for repository, package and command only.
- ADR 0066: the new view code (job list, update block) is the smallest possible and moves to `fs-components` with
  the rest of the render code (Thread R); no new Component was built here. Missing Components to ask for: a job list
  and an update list.
- Refresh: the catalogs are read on a worker thread outside the model lock (`begin_read`, `ReadTicket::read`,
  `finish_read`). An epoch counter goes up with every change of what is installed or of the sources; a read made under an
  older epoch is thrown away and read again, so an old read never undoes a newer change. One read at a time; the button
  shows "Reading...". Source commands in the window only ask for a read (`defer_refresh`).
