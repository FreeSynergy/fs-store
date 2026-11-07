//! Tests of the state machine and of update with rollback.
mod common;
use common::*;
use fs_launcher_core::*;

#[test]
fn happy_path() {
    use Event::*;
    let mut s = InstallState::Pending;
    for e in [Verify, Start, Finish] {
        s = s.next(e).unwrap();
    }
    assert_eq!(s, InstallState::Installed);
}

#[test]
fn failed_only_leads_to_rolled_back() {
    use Event::*;
    let s = InstallState::Installing.next(Fail).unwrap();
    assert_eq!(s, InstallState::Failed);
    for e in [Verify, Start, Finish, Fail] {
        assert!(s.next(e).is_err());
    }
    assert_eq!(s.next(Rollback), Ok(InstallState::RolledBack));
}

#[test]
fn cannot_install_without_verification() {
    assert!(InstallState::Pending.next(Event::Start).is_err());
}

#[test]
fn update_installs() {
    let v = verified((2, 0, 0));
    let mut inst = FakeInstaller::default();
    let u = Update { package: &v, previous: Some(Version(1, 0, 0)) };
    assert_eq!(run_update(&u, &mut inst, &Running(false)), Ok(InstallState::Installed));
    assert_eq!(inst.installed, vec![Version(2, 0, 0)]);
}

#[test]
fn running_program_is_not_replaced() {
    let v = verified((2, 0, 0));
    let mut inst = FakeInstaller::default();
    let u = Update { package: &v, previous: None };
    assert_eq!(run_update(&u, &mut inst, &Running(true)), Err(UpdateError::Running));
    assert!(inst.installed.is_empty() && inst.restored.is_empty());
}

#[test]
fn failed_update_is_rolled_back_to_previous() {
    let v = verified((2, 0, 0));
    let mut inst = FakeInstaller { fail_install: true, ..Default::default() };
    let u = Update { package: &v, previous: Some(Version(1, 0, 0)) };
    assert_eq!(run_update(&u, &mut inst, &Running(false)), Err(UpdateError::RolledBack));
    assert_eq!(inst.restored, vec![Some(Version(1, 0, 0))]);
}

#[test]
fn failed_rollback_is_reported() {
    let v = verified((2, 0, 0));
    let mut inst = FakeInstaller { fail_install: true, fail_restore: true, ..Default::default() };
    let u = Update { package: &v, previous: None };
    assert_eq!(run_update(&u, &mut inst, &Running(false)), Err(UpdateError::RollbackFailed));
}
