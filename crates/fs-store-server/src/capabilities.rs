//! What this server can do, said honestly (provider rule, ADR 0045).
//!
//! Pattern: Value object — one [`Capabilities`] built once; `capabilities`, `supportReport` and every
//! refusal answer from it, so they cannot disagree with the methods.

use fs_interfaces::Unsupported;
use fs_interfaces::provider::Capabilities;
use fs_interfaces::rpc::unsupported;

/// The contract version this server speaks.
pub(crate) const CONTRACT_VERSION: &str = "store 0.1 (draft)";

const NO_CALLER: &str = "this write needs a person who asks; the transport does not know the caller yet (ADR 0054), so the store does not act on the socket. Use the window or `fs-store` on the command line";

/// The capabilities of the first slice: reads work, writes and jobs are named as not possible.
pub(crate) fn capabilities() -> Capabilities {
    let mut c = Capabilities::new("fs-store");
    c.supported("source.read").supported("package.read");
    for id in [
        "source.add",
        "source.enable",
        "source.remove",
        "source.refresh",
        "package.install",
        "package.update",
        "package.uninstall",
    ] {
        c.unsupported(id, NO_CALLER);
    }
    c.unsupported(
        "job.read",
        "jobs run in the window or on the command line for now; the server starts none, so there is none to read",
    );
    c.unsupported(
        "job.subscribe",
        "no job runs through the server in this slice, so there is nothing to announce",
    );
    c
}

/// The refusal for `capability`, as the provider rule wants it.
pub(crate) fn refuse(capability: &str) -> capnp::Error {
    unsupported(
        &capabilities()
            .why_not(capability)
            .unwrap_or_else(|| Unsupported::new(capability, "not possible")),
    )
}
