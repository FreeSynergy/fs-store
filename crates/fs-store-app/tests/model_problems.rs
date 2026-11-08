//! Tests of the view-model: problems reported by the backend are told to the person.

mod common;

use std::sync::Arc;

use fs_store_app::backend::{
    BackendError, InstalledRow, JobError, JobKind, Listing, Phase, SourceRow, StoreBackend,
};
use fs_store_app::model::StoreModel;
use fs_store_core::Version;

#[test]
fn unreadable_and_skipped_are_told() {
    struct Odd;
    impl StoreBackend for Odd {
        fn sources(&self) -> Result<Vec<SourceRow>, BackendError> {
            Ok(Vec::new())
        }
        fn add_source(&self, _: &str) -> Result<SourceRow, BackendError> {
            Err(BackendError::Other("no".into()))
        }
        fn set_enabled(&self, _: &str, _: bool) -> Result<(), BackendError> {
            Ok(())
        }
        fn remove_source(&self, _: &str) -> Result<(), BackendError> {
            Ok(())
        }
        fn available(&self) -> Result<Listing, BackendError> {
            Ok(Listing {
                packages: Vec::new(),
                skipped: vec![("x".into(), "no signature".into())],
                unreadable: vec![("s".into(), "offline".into())],
            })
        }
        fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
            Err(BackendError::Other("disk".into()))
        }
        fn run(&self, _: JobKind, _: &str, _: &mut dyn FnMut(Phase)) -> Result<Version, JobError> {
            Err(JobError::NotOffered)
        }
    }
    let mut m = StoreModel::new(Arc::new(Odd));
    m.refresh();
    assert_eq!(
        m.problem_lines(),
        [
            "The source s could not be read: offline",
            "x is not offered: no signature",
            "The store could not do this: disk",
        ]
    );
}
