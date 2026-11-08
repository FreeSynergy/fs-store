//! Network and Git: `HttpsFetch` (release files) and `clone_catalog` (`gix`, shallow clone of a source).

use std::io::Read;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::service::Fetch;

/// Largest file we download (a release archive), in bytes.
pub const MAX_DOWNLOAD: u64 = 512 * 1024 * 1024;

/// Downloads over HTTPS only; plain `http` is refused.
#[derive(Debug, Default)]
pub struct HttpsFetch;

impl Fetch for HttpsFetch {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        if !url.starts_with("https://") {
            return Err(format!("{url}: only https addresses are downloaded"));
        }
        let resp = ureq::get(url).call().map_err(|e| format!("{url}: {e}"))?;
        let mut out = Vec::new();
        resp.into_reader()
            .take(MAX_DOWNLOAD + 1)
            .read_to_end(&mut out)
            .map_err(|e| format!("{url}: {e}"))?;
        if out.len() as u64 > MAX_DOWNLOAD {
            return Err(format!(
                "{url}: larger than the allowed {MAX_DOWNLOAD} bytes"
            ));
        }
        Ok(out)
    }
}

/// Clones the newest state of a source (depth 1) into `dest` with `gix`; `dest` must not exist or be empty.
///
/// # Errors
/// Text describing the failure.
pub fn clone_catalog(url: &str, dest: &Path) -> Result<(), String> {
    let stop = AtomicBool::new(false);
    let mut prep = gix::prepare_clone(url, dest).map_err(|e| e.to_string())?;
    prep = prep.with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(NonZeroU32::MIN));
    let (mut checkout, _) = prep
        .fetch_then_checkout(gix::progress::Discard, &stop)
        .map_err(|e| e.to_string())?;
    checkout
        .main_worktree(gix::progress::Discard, &stop)
        .map_err(|e| e.to_string())?;
    Ok(())
}
