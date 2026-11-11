# fs-store: gaps of the install workflow, for git-bug entries (2026-11-12)

Source: `/srv/FreeSynergy/.scratch/workflows/C-install.md`, section 5. Paths relative to `crates/`. One entry per item; the coordinator creates them.
Status as of this commit: 2 is fixed, 1 is half fixed.

| # | Title | Where | Status |
|---|-------|-------|--------|
| 1 | Install/remove flow exists twice (commands and window) | `fs-store-cli/src/batch.rs` (`finish`), `fs-store-cli/src/updates.rs` (`remove`) vs `fs-store-app/src/engine.rs` (`remember`), `engine_remove.rs` | Half fixed: the follow-up steps (facets, program links, menu entries, unlink) are now ONE function in `fs-store-engine/src/after.rs` (`settle_install`, `settle_remove`), same order in both. Still two: icon (`store_icon`) is stored only by the commands, the window only reads it; register handling is repeated. |
| 2 | Silent `let _ =` in follow-up steps | was `engine.rs:125,127`, `engine_remove.rs:20,27`, `updates.rs:47,53`, `batch.rs:149,157` | **Fixed**: every problem comes back as a sentence (`JobError::Incomplete`, `Err` of uninstall, stderr and exit code of the commands); tests `fs-store-engine/tests/after.rs`, `fs-store-app/tests/model_jobs.rs`. Left on purpose: temp-file cleanup (`targz.rs:84,184,189`, `native.rs:30`, `rpm.rs:81`, `deb.rs:52`), `notify-send`/`gdbus` (best effort by design), and `start_server` after a failed update (`fs-store-engine/src/batch.rs:97,109`): that last one still swallows a failed restart of the old server and should be reported. |
| 3 | Undo exists only for tar.gz; remove deletes all versions; old versions are never pruned | `fs-store-engine/src/rpm.rs:138` (`restore` empty), `remove.rs:39`, `targz.rs` (versions stay) | open |
| 4 | Reading a source writes `serials.toml` (not purely "look") | `fs-store-sources/src/serials.rs:90`, `fs-store-app/src/engine.rs:101` | open |
| 5 | Rules hidden as predicates, not as preconditions: plain name, never replace foreign files/entries, never SIGKILL/PID<=1/own PID, running check blind for `/usr` | `fs-store-core/src/package.rs` (`PackageName`), `fs-store-engine/src/links.rs:93-104`, `facets.rs:22-29,103`, `end.rs`, `proc.rs`, `DESIGN.md:103` | open |
| 6 | Format/platform choice decides the step sequence at plan time | `fs-store-engine/src/prepare.rs:94-133`, `mode.rs:85-127`, `fs-store-catalog/src/catalog.rs` (`fits_this_machine`) | open |
| 7 | Batch semantics: all rpm/deb of a batch in one `pkexec` call, same cause for all | `fs-store-engine/src/batch.rs:173-238` | open |
| 8 | Environment and limits as hidden parameters (`FS_STORE_BIN`, `FS_STORE_APPS`, `FS_STORE_ROOT`, `HOME`, `FS_STORE_TUF_ROOT`; 35 s, 5 s, 4 GiB/100 000 entries, 512 MiB) | `links.rs:55`, `facets.rs:151`, `end.rs`, `targz.rs:11-13`, `net.rs` | open |
| 9 | `requires` (ADR 0068) is not a precondition of install; `resolve` is built but called nowhere | `fs-store-core/src/capability.rs`, `fs-store-engine/src/prepare.rs` (`plan`) | open, needs the contract fields (A3) |
| 10 | Register is thin: source fixed as `"catalog"`, no signer, no format | `fs-store-engine/src/prepare.rs:187`, `fs-store-sources/src/registry.rs` | open |
| 11 | `pkexec` is a human confirmation inside a machine step; the preview must come before the password | `fs-store-engine/src/rpm.rs:130`, `deb.rs:49` | open |
| 12 | "The person's data is never removed" exists only in comments | `fs-store-engine/src/remove.rs`, `rpm.rs:143`, `fs-store-app/src/engine_remove.rs` | open |
