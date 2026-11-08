use std::fs;

use fs_store_engine::store_icon;
use sha2::{Digest, Sha256};

fn png() -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
    b.extend_from_slice(b"rest");
    b
}

#[test]
fn only_a_png_with_the_right_hash_is_stored() {
    let tmp = tempfile::tempdir().unwrap();
    let bytes = png();
    let hash = format!("{:x}", Sha256::digest(&bytes));
    assert!(!store_icon(tmp.path(), "people", &bytes, "00").unwrap());
    assert!(!store_icon(tmp.path(), "people", b"not a png", &hash).unwrap());
    assert!(!store_icon(tmp.path(), "../x", &bytes, &hash).unwrap());
    assert!(!tmp.path().join("icons").exists());
    assert!(store_icon(tmp.path(), "people", &bytes, &hash).unwrap());
    assert_eq!(
        fs::read(tmp.path().join("icons/people.png")).unwrap(),
        bytes
    );
}
