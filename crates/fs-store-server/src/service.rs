//! The `Store` root interface.
//!
//! Pattern: Adapter + Null object for the writes — [`StoreServer`] translates the reads into
//! [`StoreView`] calls and answers every write with the provider-rule `Unsupported`. Blocking work
//! (a catalog is cloned from Git) runs on the blocking pool, never on the RPC thread.

use std::rc::Rc;
use std::sync::Arc;

use capnp_rpc::new_client;
use fs_interfaces::store_capnp::store;

use crate::capabilities::{CONTRACT_VERSION, capabilities, refuse};
use crate::view::StoreView;
use crate::wire::{len, split_for_wire, write_packages, write_skipped, write_sources};

/// The root capability of the store.
pub struct StoreServer {
    view: Arc<dyn StoreView>,
}

impl StoreServer {
    /// Serve what `view` knows.
    #[must_use]
    pub fn new(view: Arc<dyn StoreView>) -> Self {
        Self { view }
    }

    /// The capability to hand to `fs_interfaces::rpc::serve`.
    #[must_use]
    pub fn into_client(self) -> store::Client {
        new_client(self)
    }

    /// Run a blocking read of the view off the RPC thread.
    async fn read<T, F>(&self, f: F) -> Result<T, capnp::Error>
    where
        T: Send + 'static,
        F: FnOnce(&dyn StoreView) -> Result<T, String> + Send + 'static,
    {
        let view = self.view.clone();
        tokio_blocking(move || f(view.as_ref()))
            .await
            .map_err(capnp::Error::failed)
    }
}

/// `spawn_blocking` without a direct dependency on the runtime's name in every call site.
async fn tokio_blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| format!("the read was cut short: {e}"))?
}

type Done = Result<(), capnp::Error>;

impl store::Server for StoreServer {
    async fn capabilities(
        self: Rc<Self>,
        _: store::CapabilitiesParams,
        mut results: store::CapabilitiesResults,
    ) -> Done {
        capabilities().write_list(CONTRACT_VERSION, results.get().init_capabilities());
        Ok(())
    }

    async fn support_report(
        self: Rc<Self>,
        _: store::SupportReportParams,
        mut results: store::SupportReportResults,
    ) -> Done {
        capabilities().write(CONTRACT_VERSION, results.get().init_report());
        Ok(())
    }

    async fn sources(
        self: Rc<Self>,
        _: store::SourcesParams,
        mut results: store::SourcesResults,
    ) -> Done {
        let list = self.read(|v| v.sources()).await?;
        write_sources(&list, results.get().init_list(len(list.len())));
        Ok(())
    }

    async fn available(
        self: Rc<Self>,
        _: store::AvailableParams,
        mut results: store::AvailableResults,
    ) -> Done {
        let (packages, mut skipped) = self.read(|v| v.available()).await?;
        let (packages, left) = split_for_wire(packages);
        skipped.extend(left);
        let mut out = results.get();
        write_packages(&packages, out.reborrow().init_list(len(packages.len())));
        write_skipped(&skipped, out.init_skipped(len(skipped.len())));
        Ok(())
    }

    async fn installed(
        self: Rc<Self>,
        _: store::InstalledParams,
        mut results: store::InstalledResults,
    ) -> Done {
        let (list, _) = split_for_wire(self.read(|v| v.installed()).await?);
        write_packages(&list, results.get().init_list(len(list.len())));
        Ok(())
    }

    async fn add_source(
        self: Rc<Self>,
        _: store::AddSourceParams,
        _: store::AddSourceResults,
    ) -> Done {
        Err(refuse("source.add"))
    }

    async fn set_enabled(
        self: Rc<Self>,
        _: store::SetEnabledParams,
        _: store::SetEnabledResults,
    ) -> Done {
        Err(refuse("source.enable"))
    }

    async fn remove_source(
        self: Rc<Self>,
        _: store::RemoveSourceParams,
        _: store::RemoveSourceResults,
    ) -> Done {
        Err(refuse("source.remove"))
    }

    async fn install(self: Rc<Self>, _: store::InstallParams, _: store::InstallResults) -> Done {
        Err(refuse("package.install"))
    }

    async fn update(self: Rc<Self>, _: store::UpdateParams, _: store::UpdateResults) -> Done {
        Err(refuse("package.update"))
    }

    async fn uninstall(
        self: Rc<Self>,
        _: store::UninstallParams,
        _: store::UninstallResults,
    ) -> Done {
        Err(refuse("package.uninstall"))
    }

    async fn refresh(self: Rc<Self>, _: store::RefreshParams, _: store::RefreshResults) -> Done {
        Err(refuse("source.refresh"))
    }

    async fn job(self: Rc<Self>, _: store::JobParams, _: store::JobResults) -> Done {
        Err(refuse("job.read"))
    }

    async fn subscribe(
        self: Rc<Self>,
        _: store::SubscribeParams,
        _: store::SubscribeResults,
    ) -> Done {
        Err(refuse("job.subscribe"))
    }
}
