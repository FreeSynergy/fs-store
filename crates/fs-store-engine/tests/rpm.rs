mod common;

use std::fs;

use common::{Idle, verified};
use fs_store_core::*;
use fs_store_engine::{Commander, Rpm};

#[derive(Default)]
struct Fake {
    calls: Vec<(String, Vec<String>, Option<Vec<u8>>)>,
    fail: bool,
}
impl Commander for &mut Fake {
    fn run(&mut self, program: &str, args: &[&str]) -> Result<(), String> {
        let file = args.last().and_then(|p| fs::read(p).ok());
        self.calls.push((
            program.into(),
            args.iter().map(|s| (*s).into()).collect(),
            file,
        ));
        if self.fail {
            Err("dnf failed".into())
        } else {
            Ok(())
        }
    }
}

#[test]
fn dnf_gets_the_verified_bytes_through_pkexec_and_the_file_is_removed_afterwards() {
    let tmp = tempfile::tempdir().unwrap();
    let mut fake = Fake::default();
    let mut s = Rpm::new(
        &tmp.path().join("rpm"),
        vec![b"RPMBYTES".to_vec()],
        &mut fake,
    );
    run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    )
    .unwrap();
    drop(s);
    let (prog, args, file) = &fake.calls[0];
    assert_eq!(prog, "pkexec");
    assert_eq!(&args[..3], ["dnf", "install", "-y"]);
    assert_eq!(file.as_deref(), Some(&b"RPMBYTES"[..]));
    assert_eq!(fs::read_dir(tmp.path().join("rpm")).unwrap().count(), 0);
}

#[test]
fn a_failing_dnf_is_reported_as_rolled_back_and_leaves_no_file() {
    let tmp = tempfile::tempdir().unwrap();
    let mut fake = Fake {
        fail: true,
        ..Fake::default()
    };
    let mut s = Rpm::new(&tmp.path().join("rpm"), vec![b"x".to_vec()], &mut fake);
    let r = run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    );
    assert_eq!(r, Err(UpdateError::RolledBack));
    assert_eq!(fs::read_dir(tmp.path().join("rpm")).unwrap().count(), 0);
}

#[test]
fn several_rpms_go_to_one_dnf_call() {
    let tmp = tempfile::tempdir().unwrap();
    let mut fake = Fake::default();
    let mut s = Rpm::new(
        &tmp.path().join("rpm"),
        vec![b"A".to_vec(), b"B".to_vec()],
        &mut fake,
    );
    run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    )
    .unwrap();
    drop(s);
    assert_eq!(fake.calls.len(), 1);
    let args = &fake.calls[0].1;
    assert_eq!(&args[..3], ["dnf", "install", "-y"]);
    assert_eq!(args.len(), 6); // dnf install -y <option> and two files
    assert_eq!(fs::read_dir(tmp.path().join("rpm")).unwrap().count(), 0);
}
