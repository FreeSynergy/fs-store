# fs-launcher – design

## Purpose
The launcher installs and updates FreeSynergy packages and itself. A package is a program, a frontend, a component or a
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
| `fs-launcher-pgp` – rPGP behind `SignatureVerifier` | Adapter | Wraps the foreign `pgp` crate in our trait, so the core stays free of dependencies. rPGP (MIT OR Apache-2.0), not sequoia (LGPL). |
| `strategy` – how a package is installed | Strategy (`InstallStrategy`) | rpm via dnf, Flatpak, AppImage, tar.gz: one trait, one implementation per way. |
| `state` – course of one installation | State (enum plus one transition fn) | `Failed` has exactly one way out: `RolledBack`. |
| `update` – update and roll back | State plus `InstallStrategy::restore` | There is one operation so far (install/update with undo); a `Command` trait with `apply`/`invert` comes when remove is added and there are two or more commands. |
| `update` – running check | Strategy (`RunningCheck`) | The OS check is outside the core. |

## Engine and frontend (ADR 0021)
`fs-launcher-core` has no input or output: no network, no files, no GUI. An engine crate will implement the traits
(`gix`, a real OpenPGP library, dnf/Flatpak calls). Frontends (Blinc GUI, CLI, web page) are packages of their own and
talk to the engine through the contract below. They are interchangeable.

## Rules fixed in code
- The built-in reference source: `reference = true`. `set_enabled(false)` works, `remove` fails with `ReferenceSource`.
- Trusted key fingerprint, built in: `16A4A3EE563AEF51FCF0C55DED7B57C4A1904342`. A checksum from the same release is not a signature.
- Outcome other than `Valid` with the trusted fingerprint: no `VerifiedPackage`, so no install.
- Update order: running check, install (previous version remembered), on error `Failed` then `invert` then `RolledBack`.

## Contract sketch `launcher.capnp` (text only, not yet in `fs-interfaces`)
```
interface Launcher {
  sources        @0 () -> (list :List(Source));
  addSource      @1 (url :Text) -> (id :Text);
  setEnabled     @2 (id :Text, enabled :Bool) -> ();
  removeSource   @3 (id :Text) -> ();        # error referenceSource for the built-in one
  available      @4 () -> (list :List(Package));
  installed      @5 () -> (list :List(Package));
  install        @6 (name :Text) -> (job :Job);
  update         @7 (name :Text) -> (job :Job);   # self-update is update("fs-launcher")
  watch          @8 (job :Job, sink :Sink) -> ();
}
struct Source  { id @0 :Text; url @1 :Text; enabled @2 :Bool; reference @3 :Bool; }
struct Package { name @0 :Text; kind @1 :Kind; version @2 :Text; format @3 :Format; source @4 :Text; }
enum Kind   { program @0; frontend @1; component @2; theme @3; }
enum Format { rpm @0; flatpak @1; appImage @2; tarGz @3; }
enum Phase  { pending @0; verified @1; installing @2; installed @3; failed @4; rolledBack @5; }
```
Risk marks (ADR 0021): `install`, `update`, `removeSource` are write operations set by the engine.

## Provisional decisions
- Versions are `major.minor.patch` numbers only (SemVer subset); pre-release tags later.
- Only one trusted fingerprint for now; sources may later bring their own keys, never replacing the built-in one.
- Texts a person reads (`words.rs`) come with the first frontend; the core only has English developer errors.
- Name of the program "Launcher" and crate names are provisional.

## State
Design and core traits with in-memory test doubles. Not built: network, `gix`, OpenPGP verification, real
installers, self-replacement of the executable, GUI/CLI, the capnp file, persistence of the source list.
