//! The program language comes from `FS_LANG`, else `LANG`; an EMPTY `FS_LANG` does not hide `LANG`. One test only: it changes the environment.

use fs_store_core::language::{from_environment, preferred};

#[test]
fn an_empty_fs_lang_falls_back_to_lang_and_a_set_one_wins() {
    // SAFETY: this file has one test, so nothing else reads the environment meanwhile
    unsafe {
        std::env::set_var("LANG", "de_DE.UTF-8");
        std::env::set_var("FS_LANG", "");
    }
    assert_eq!(preferred(), ["de", "en"]);
    unsafe { std::env::set_var("FS_LANG", "  ") };
    assert_eq!(from_environment(), "de_DE.UTF-8");
    // counter-proof: a real FS_LANG wins over LANG; neither set gives English only
    unsafe { std::env::set_var("FS_LANG", "en") };
    assert_eq!(preferred(), ["en"]);
    unsafe {
        std::env::remove_var("FS_LANG");
        std::env::remove_var("LANG");
    }
    assert_eq!(preferred(), ["en"]);
}
