mod common;

use std::fs;

use common::{Idle, verified};
use fs_store_core::*;
use fs_store_engine::{Commander, Deb};

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
            Err("apt-get failed".into())
        } else {
            Ok(())
        }
    }
}

#[test]
fn apt_gets_the_verified_bytes_through_pkexec_and_the_file_is_removed_afterwards() {
    let tmp = tempfile::tempdir().unwrap();
    let mut fake = Fake::default();
    let mut s = Deb::new(
        &tmp.path().join("deb"),
        vec![b"DEBBYTES".to_vec()],
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
    assert_eq!(&args[..4], ["apt-get", "install", "-y", "--no-remove"]);
    assert_eq!(file.as_deref(), Some(&b"DEBBYTES"[..]));
    assert_eq!(fs::read_dir(tmp.path().join("deb")).unwrap().count(), 0);
}

#[test]
fn a_failing_apt_is_reported_as_rolled_back_and_leaves_no_file() {
    let tmp = tempfile::tempdir().unwrap();
    let mut fake = Fake {
        fail: true,
        ..Fake::default()
    };
    let mut s = Deb::new(&tmp.path().join("deb"), vec![b"x".to_vec()], &mut fake);
    let r = run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    );
    assert_eq!(
        r,
        Err(UpdateError::RolledBack(fs_store_core::InstallError(
            "apt-get failed".into()
        )))
    ); // the cause comes up, it is not swallowed
    assert_eq!(fs::read_dir(tmp.path().join("deb")).unwrap().count(), 0);
}

#[test]
fn several_debs_go_to_one_apt_call() {
    let tmp = tempfile::tempdir().unwrap();
    let mut fake = Fake::default();
    let mut s = Deb::new(
        &tmp.path().join("deb"),
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
    assert_eq!(&args[..4], ["apt-get", "install", "-y", "--no-remove"]);
    assert_eq!(args.len(), 6); // apt-get install -y --no-remove and two files
    assert_eq!(fs::read_dir(tmp.path().join("deb")).unwrap().count(), 0);
}

#[test]
#[allow(clippy::mut_mut)] // the fake lives on in the test as `&mut Fake`, which is what implements `Commander`
fn removing_a_deb_asks_apt_through_pkexec_and_refuses_odd_names() {
    let mut fake = Fake::default();
    let mut runner = &mut fake;
    fs_store_engine::remove_deb("tasks", &mut runner).unwrap();
    assert!(fs_store_engine::remove_deb("../x", &mut runner).is_err());
    assert!(fs_store_engine::remove_deb("-rf", &mut runner).is_err());
    assert_eq!(fake.calls.len(), 1);
    assert_eq!(fake.calls[0].0, "pkexec");
    assert_eq!(fake.calls[0].1, ["apt-get", "remove", "-y", "tasks"]);
}
