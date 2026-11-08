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
`fs:kind` (program, frontend, component, theme; default program), `fs:format` (tar.gz, rpm, flatpak, appimage; default
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

Known gap: the release archive has one top directory (`fs-tasks-server-0.6.1-rc.2-linux-x86_64/`); `TarGz` keeps it, so
the program sits one level down in `current/<name>/`. Stripping a single common top directory is the next change.

## Provisional decisions
- Versions are `major.minor.patch` with an optional pre-release `-alpha|beta|rc[.N]` (`~` accepted as in `AppStream`/rpm); a pre-release is older than its release. Anything else is skipped with a reason.
- Only one trusted fingerprint for now; sources may later bring their own keys, never replacing the built-in one.
- Texts a person reads (`words.rs`) come with the first frontend; the core only has English developer errors.
- Name of the program "Launcher" and crate names are provisional.

## State
Core traits and value types; rPGP adapter (`fs-store-pgp`, built-in release key and the revoked key as files, tested with
generated test keys); TOML source list (`fs-store-sources`); `words.rs`. 27 tests; `fs-store-catalog` reads AppStream (4 tests); `fs-store-engine` has `TarGz` (install to a root dir, atomic symlink switch, old version kept, escaping archive entries refused) and `ProcRunningCheck` (5 tests). `service::Installer` runs the whole chain (catalog signature, release file, sha256, release signature, `run_update`) over a `Fetch` trait; 4 tests with a fake verifier, so far not run with real keys. `net`: `HttpsFetch` (https only, 512 MiB cap) and `clone_catalog` (`gix`, depth 1, tested against a local repo). `fs-store-cli` (binary `fs-store`: `available`, `install`, `installed`; `FS_STORE_ROOT` overrides `~/.local/share/fs-store`) is the thin test front end; `sources`/`update` subcommands and the real-key run are still missing. Self-update (step 3, done): the store is a package like any other; the switch of `current/<name>` is a symlink swap, so the running old executable is never overwritten, and `ProcRunningCheck` ignores the calling process (tested with a real running program). The new version starts at the next launch. Still open in step 3 (rest): `gix` fetch, HTTPS download, rpm via dnf/pkexec, self-update (atomic swap of the own executable). Contract moved to `fs-interfaces` (`schema/store.capnp`, no tag yet, waits for a review by a second agent); the server `fs-store-server` is not built. Not built (older list): `gix`, real
installers, self-replacement of the executable, GUI/CLI, the capnp file, a `Command` trait (comes with remove),
key expiry checks and subkey rules in `fs-store-pgp` (provisional: a valid signature by the primary key or a public
subkey of a non-revoked trusted key counts), translation of `words.rs`, atomic write of the source file.
