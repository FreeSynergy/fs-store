//! `fs-store available|install|installed <source>`: a thin test front end. The GUI is the goal (sprint, step 6).
//! `<source>` is a Git address (https or file://) holding `catalog.xml` and `catalog.xml.asc`.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use fs_store_catalog::parse_catalog;
use fs_store_core::Version;
use fs_store_engine::ProcRunningCheck;
use fs_store_engine::net::{HttpsFetch, clone_catalog};
use fs_store_engine::service::{Fetch, FileFetch, Installer};
use fs_store_pgp::PgpVerifier;

struct AnyFetch;
impl Fetch for AnyFetch {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        if url.starts_with("file://") {
            FileFetch.get(url)
        } else {
            HttpsFetch.get(url)
        }
    }
}

fn root() -> PathBuf {
    std::env::var_os("FS_STORE_ROOT").map_or_else(
        || {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join(".local/share/fs-store")
        },
        PathBuf::from,
    )
}

fn installed(root: &std::path::Path, name: &str) -> Option<Version> {
    let target = fs::read_link(root.join("current").join(name)).ok()?;
    Version::parse(target.file_name()?.to_str()?)
}

fn run(args: &[String]) -> Result<(), String> {
    let root = root();
    let (cmd, source) = match args {
        [c, s, ..] if c == "available" || c == "install" => (c.as_str(), s.as_str()),
        [c] if c == "installed" => {
            for e in fs::read_dir(root.join("current"))
                .into_iter()
                .flatten()
                .flatten()
            {
                let name = e.file_name().to_string_lossy().into_owned();
                if !name.starts_with('.') {
                    println!(
                        "{name} {}",
                        installed(&root, &name)
                            .map_or("?".into(), |v| format!("{}.{}.{}", v.0, v.1, v.2))
                    );
                }
            }
            return Ok(());
        }
        _ => {
            return Err(
                "usage: fs-store available <source> | install <source> <name> | installed".into(),
            );
        }
    };
    let work = root.join("sources").join("work");
    let _ = fs::remove_dir_all(&work);
    clone_catalog(source, &work)?;
    let catalog = fs::read(work.join("catalog.xml")).map_err(|e| e.to_string())?;
    let sig = fs::read(work.join("catalog.xml.asc")).map_err(|e| e.to_string())?;
    if cmd == "available" {
        let parsed = parse_catalog(&String::from_utf8_lossy(&catalog)).map_err(|e| e.0)?;
        for e in parsed.entries {
            println!(
                "{} {}.{}.{}",
                e.name.as_str(),
                e.version.0,
                e.version.1,
                e.version.2
            );
        }
        for s in parsed.skipped {
            eprintln!("skipped {}: {}", s.id, s.reason);
        }
        return Ok(());
    }
    let name = args.get(2).ok_or("install needs a package name")?;
    let verifier = PgpVerifier::builtin();
    let running = ProcRunningCheck::new(&root);
    let i = Installer {
        root: root.clone(),
        staging: root.join("staging"),
        verifier: &verifier,
        fetch: &AnyFetch,
        running: &running,
    };
    let v = i
        .install(&catalog, &sig, name, installed(&root, name))
        .map_err(|e| format!("{e:?}"))?;
    println!("installed {name} {}.{}.{}", v.0, v.1, v.2);
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fs-store: {e}");
            ExitCode::FAILURE
        }
    }
}
