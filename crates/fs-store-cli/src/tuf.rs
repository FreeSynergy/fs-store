//! TUF check of a fetched source (ADR 0062).

use std::fs;
use std::path::Path;

/// The project's built-in TUF root (`keys/tuf-root.json`), compiled in when the file exists at build time.
#[cfg(builtin_tuf_root)]
pub(crate) fn builtin_root() -> Option<&'static [u8]> {
    Some(include_bytes!("../keys/tuf-root.json"))
}

/// The project's built-in TUF root: none yet (the person has not made the root keys), so the check is not active by default.
#[cfg(not(builtin_tuf_root))]
pub(crate) fn builtin_root() -> Option<&'static [u8]> {
    None
}

/// The trusted root: `FS_STORE_TUF_ROOT=<root.json>` if set, else the built-in one, else none.
fn trusted_root() -> Result<Option<Vec<u8>>, String> {
    match std::env::var_os("FS_STORE_TUF_ROOT") {
        Some(file) => fs::read(file)
            .map(Some)
            .map_err(|e| format!("FS_STORE_TUF_ROOT: {e}")),
        None => Ok(builtin_root().map(<[u8]>::to_vec)),
    }
}

/// When a trusted root exists, the signed `tuf/` directory of the source must list this exact `catalog.xml`.
/// Without a root nothing changes: the OpenPGP signature and the freshness check still apply.
pub(crate) fn tuf_check(work: &Path, url: &str, catalog: &[u8]) -> Result<(), String> {
    let Some(root) = trusted_root()? else {
        return Ok(());
    };
    let (meta, targets) = (work.join("tuf/metadata"), work.join("tuf/targets"));
    if !meta.is_dir() {
        return Err(format!("{url}: no TUF metadata, but a TUF root is trusted"));
    }
    let signed = fs_store_tuf::read_target(&root, &meta, &targets, "catalog.xml")
        .map_err(|e| format!("{url}: TUF: {e:?}"))?;
    if signed == catalog {
        Ok(())
    } else {
        Err(format!(
            "{url}: catalog.xml is not the file the TUF metadata signs"
        ))
    }
}
