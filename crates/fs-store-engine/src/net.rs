//! Pattern: Adapter (`ureq` and `gix` behind our `Fetch` trait and a clone function).
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
        // Servers have bad minutes (GitHub answered 5xx and timeouts on 2026-11-08): try three times with a pause,
        // but only for what can pass (5xx, no answer); a 404 or a refusal is answered at once.
        let mut last = String::new();
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_secs(2 * attempt));
            }
            match once(url) {
                Ok(bytes) => return Ok(bytes),
                Err((message, retry)) => {
                    last = message;
                    if !retry {
                        break;
                    }
                }
            }
        }
        Err(last)
    }
}

/// One download; the flag says whether trying again can help.
fn once(url: &str) -> Result<Vec<u8>, (String, bool)> {
    // A server that does not answer must not hang the store (the window shows the error instead).
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(20))
        .timeout_read(std::time::Duration::from_secs(60))
        .build();
    let resp = agent.get(url).call().map_err(|e| {
        let retry = match &e {
            ureq::Error::Status(code, _) => *code >= 500,
            ureq::Error::Transport(_) => true,
        };
        (format!("{url}: {e}"), retry)
    })?;
    let mut out = Vec::new();
    resp.into_reader()
        .take(MAX_DOWNLOAD + 1)
        .read_to_end(&mut out)
        .map_err(|e| (format!("{url}: {e}"), true))?;
    if out.len() as u64 > MAX_DOWNLOAD {
        return Err((
            format!("{url}: larger than the allowed {MAX_DOWNLOAD} bytes"),
            false,
        ));
    }
    Ok(out)
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
