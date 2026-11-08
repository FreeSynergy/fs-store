#![allow(dead_code)]
use std::fs;
use std::path::Path;

use fs_store_core::*;

pub fn archive(dir: &Path, file: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(format!("{file}.tar.gz"));
    let enc = flate2::write::GzEncoder::new(
        fs::File::create(&path).unwrap(),
        flate2::Compression::fast(),
    );
    let mut b = tar::Builder::new(enc);
    let mut h = tar::Header::new_gnu();
    h.set_size(content.len() as u64);
    h.set_mode(0o755);
    h.set_cksum();
    b.append_data(&mut h, "bin/prog", content.as_bytes())
        .unwrap();
    b.into_inner().unwrap().finish().unwrap();
    path
}

pub fn verified(name: &str, v: Version) -> VerifiedPackage {
    let p = Package {
        name: PackageName::new(name).unwrap(),
        kind: Kind::Program,
        version: v,
        format: Format::TarGz,
        source: SourceId(REFERENCE_ID.to_owned()),
    };
    SignaturePolicy::default()
        .admit(
            p,
            SignatureOutcome::Valid {
                fingerprint: TRUSTED_FINGERPRINT.into(),
            },
        )
        .unwrap()
}

pub struct Idle;
impl RunningCheck for Idle {
    fn is_running(&self, _: &str) -> bool {
        false
    }
}
