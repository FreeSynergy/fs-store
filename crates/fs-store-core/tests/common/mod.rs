//! Simple test doubles for the launcher traits.
#![allow(dead_code)]

use fs_store_core::*;

pub fn pkg(version: (u64, u64, u64)) -> Package {
    Package {
        name: PackageName::new("fs-demo").unwrap(),
        kind: Kind::Program,
        version: Version::new(version.0, version.1, version.2),
        format: Format::TarGz,
        source: SourceId("fs".into()),
    }
}

pub fn good() -> SignatureOutcome {
    SignatureOutcome::Valid {
        fingerprint: TRUSTED_FINGERPRINT.into(),
    }
}

pub fn verified(version: (u64, u64, u64)) -> VerifiedPackage {
    SignaturePolicy::default()
        .admit(pkg(version), good())
        .unwrap()
}

/// In-memory source list with the reference source built in.
pub struct MemSources(pub Vec<Source>);

impl MemSources {
    pub fn new() -> Self {
        Self(vec![Source {
            id: SourceId("fs".into()),
            url: "https://github.com/FreeSynergy/fs-catalog".into(),
            enabled: true,
            reference: true,
        }])
    }
    fn find(&mut self, id: &SourceId) -> Result<usize, SourceError> {
        self.0
            .iter()
            .position(|s| &s.id == id)
            .ok_or(SourceError::NotFound)
    }
}

impl SourceRepository for MemSources {
    fn list(&self) -> Vec<Source> {
        self.0.clone()
    }
    fn add(&mut self, url: &str, id: SourceId) -> Result<(), SourceError> {
        if self.0.iter().any(|s| s.id == id) {
            return Err(SourceError::Duplicate);
        }
        self.0.push(Source {
            id,
            url: url.into(),
            enabled: true,
            reference: false,
        });
        Ok(())
    }
    fn set_enabled(&mut self, id: &SourceId, enabled: bool) -> Result<(), SourceError> {
        let i = self.find(id)?;
        self.0[i].enabled = enabled;
        Ok(())
    }
    fn remove(&mut self, id: &SourceId) -> Result<(), SourceError> {
        let i = self.find(id)?;
        if self.0[i].reference {
            return Err(SourceError::ReferenceSource);
        }
        self.0.remove(i);
        Ok(())
    }
}

/// Installer that records calls; can be told to fail.
#[derive(Default)]
pub struct FakeInstaller {
    pub fail_install: bool,
    pub fail_restore: bool,
    pub installed: Vec<Version>,
    pub restored: Vec<Option<Version>>,
}

impl InstallStrategy for FakeInstaller {
    fn format(&self) -> Format {
        Format::TarGz
    }
    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError> {
        if self.fail_install {
            return Err(InstallError("boom".into()));
        }
        self.installed.push(package.package().version);
        Ok(())
    }
    fn restore(&mut self, _name: &str, previous: Option<Version>) -> Result<(), InstallError> {
        if self.fail_restore {
            return Err(InstallError("no undo".into()));
        }
        self.restored.push(previous);
        Ok(())
    }
}

pub struct Running(pub bool);

impl RunningCheck for Running {
    fn is_running(&self, _name: &str) -> bool {
        self.0
    }
}

/// Verifier that returns a fixed outcome.
pub struct FixedVerifier(pub SignatureOutcome);

impl SignatureVerifier for FixedVerifier {
    fn verify(&self, _data: &[u8], _signature: Option<&[u8]>) -> SignatureOutcome {
        self.0.clone()
    }
}
