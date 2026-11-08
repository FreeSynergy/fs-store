//! Tells the compiler whether the project's TUF root (`keys/tuf-root.json`) exists; it is absent until the person has made it.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(builtin_tuf_root)");
    println!("cargo::rerun-if-changed=keys/tuf-root.json");
    if std::path::Path::new("keys/tuf-root.json").is_file() {
        println!("cargo::rustc-cfg=builtin_tuf_root");
    }
}
