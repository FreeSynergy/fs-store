//! `fs-store`: a thin test front end for the engine. The GUI is the goal (sprint, step 6).
//!
//! ```text
//! fs-store sources [add <url> | enable <id> | disable <id> | remove <id>]
//! fs-store available [<source>]          # source: an id from the list, or a Git address; default: all enabled
//! fs-store install <name> [<source>]
//! fs-store update  <name> [<source>]     # only if the catalog offers a newer version
//! fs-store installed
//! Option: --trust-key <file>             # also trust this public key for this run (test key of a local source)
//! ```
//! A source is a Git address (https or file://) holding `catalog.xml` and `catalog.xml.asc`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use fs_store_catalog::{Catalog, CatalogEntry, parse_catalog};
use fs_store_core::{SignaturePolicy, SourceId, SourceRepository, Version};
use fs_store_engine::ProcRunningCheck;
use fs_store_engine::net::{HttpsFetch, clone_catalog};
use fs_store_engine::service::{Fetch, FileFetch, Installer};
use fs_store_pgp::PgpVerifier;
use fs_store_sources::FileSources;

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

fn installed(root: &Path, name: &str) -> Option<Version> {
    let target = fs::read_link(root.join("current").join(name)).ok()?;
    Version::parse(target.file_name()?.to_str()?)
}

fn sources(root: &Path) -> Result<FileSources, String> {
    FileSources::load(&root.join("sources.toml")).map_err(|e| format!("{e:?}"))
}

/// Source argument to Git addresses: an id from the list, a Git address, or all enabled sources.
fn addresses(root: &Path, arg: Option<&String>) -> Result<Vec<String>, String> {
    let list = sources(root)?.list();
    match arg {
        None => Ok(list
            .into_iter()
            .filter(|s| s.enabled)
            .map(|s| s.url)
            .collect()),
        Some(a) => Ok(vec![
            list.into_iter()
                .find(|s| s.id.0 == *a)
                .map_or_else(|| a.clone(), |s| s.url),
        ]),
    }
}

/// Clones one source and returns its catalog text and signature.
fn fetch_source(root: &Path, url: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let work = root.join("sources").join("work");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(work.parent().unwrap_or(root)).map_err(|e| e.to_string())?;
    clone_catalog(url, &work).map_err(|e| format!("{url}: {e}"))?;
    let catalog = fs::read(work.join("catalog.xml")).map_err(|e| format!("{url}: {e}"))?;
    let sig = fs::read(work.join("catalog.xml.asc")).map_err(|e| format!("{url}: {e}"))?;
    Ok((catalog, sig))
}

fn read_catalog(bytes: &[u8]) -> Result<Catalog, String> {
    parse_catalog(&String::from_utf8_lossy(bytes)).map_err(|e| e.0)
}

fn id_from_url(url: &str) -> String {
    url.trim_end_matches(['/', '.'])
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("source")
        .trim_end_matches(".git")
        .to_owned()
}

fn cmd_sources(root: &Path, rest: &[String]) -> Result<(), String> {
    let mut list = sources(root)?;
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let err = |e| format!("{e:?}");
    match rest {
        [] => {
            for s in list.list() {
                println!(
                    "{} {} {}{}",
                    s.id.0,
                    if s.enabled { "on " } else { "off" },
                    s.url,
                    if s.reference { "  (reference)" } else { "" }
                );
            }
            return Ok(());
        }
        [c, url] if c == "add" => list.add(url, SourceId(id_from_url(url))).map_err(err)?,
        [c, id] if c == "enable" => list.set_enabled(&SourceId(id.clone()), true).map_err(err)?,
        [c, id] if c == "disable" => list
            .set_enabled(&SourceId(id.clone()), false)
            .map_err(err)?,
        [c, id] if c == "remove" => list.remove(&SourceId(id.clone())).map_err(err)?,
        _ => {
            return Err(
                "usage: fs-store sources [add <url> | enable <id> | disable <id> | remove <id>]"
                    .into(),
            );
        }
    }
    list.save().map_err(|e| format!("{e:?}"))
}

fn cmd_installed(root: &Path) {
    for e in fs::read_dir(root.join("current"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.starts_with('.') {
            println!(
                "{name} {}",
                installed(root, &name).map_or("?".into(), |v| v.to_string())
            );
        }
    }
}

fn cmd_available(root: &Path, arg: Option<&String>) -> Result<(), String> {
    let mut failed = false;
    for url in addresses(root, arg)? {
        let result = fetch_source(root, &url).and_then(|(c, _)| read_catalog(&c));
        match result {
            Ok(cat) => {
                for e in cat.entries {
                    println!("{} {}", e.name.as_str(), e.version);
                }
                for s in cat.skipped {
                    eprintln!("skipped {}: {}", s.id, s.reason);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                failed = true;
            }
        }
    }
    if failed {
        Err("a source could not be read".into())
    } else {
        Ok(())
    }
}

/// Installs or updates; `only_newer` makes it an update.
fn cmd_install(
    root: &Path,
    name: &str,
    arg: Option<&String>,
    extra: Option<&String>,
    only_newer: bool,
) -> Result<(), String> {
    let (verifier, policy) = match extra {
        None => (PgpVerifier::builtin(), SignaturePolicy::default()),
        Some(file) => {
            let armored = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
            let v = PgpVerifier::builtin_and(&armored).map_err(|e| format!("{file}: {e:?}"))?;
            let fp = v.fingerprints().pop().ok_or("no key")?;
            eprintln!("fs-store: also trusting test key {fp} for this run");
            (v, SignaturePolicy::default().also_trusting(&fp))
        }
    };
    let running = ProcRunningCheck::new(root);
    let installer = Installer {
        root: root.to_owned(),
        staging: root.join("staging"),
        verifier: &verifier,
        fetch: &AnyFetch,
        running: &running,
        policy,
    };
    let current = installed(root, name);
    if only_newer && current.is_none() {
        return Err(format!("{name} is not installed; use install"));
    }
    let mut last = format!("{name} is not offered by any source");
    for url in addresses(root, arg)? {
        let (catalog, sig) = match fetch_source(root, &url) {
            Ok(x) => x,
            Err(e) => {
                last = e;
                continue;
            }
        };
        let offered: Option<CatalogEntry> = read_catalog(&catalog)?
            .entries
            .into_iter()
            .find(|e| e.name.as_str() == name);
        let Some(offered) = offered else { continue };
        if only_newer && current.is_some_and(|c| offered.version <= c) {
            println!(
                "{name} is up to date ({})",
                current.map_or_else(String::new, |c| c.to_string())
            );
            return Ok(());
        }
        let v = installer
            .install(&catalog, &sig, name, current)
            .map_err(|e| format!("{url}: {e:?}"))?;
        println!("installed {name} {v}");
        return Ok(());
    }
    Err(last)
}

fn run(all: &[String]) -> Result<(), String> {
    // `--trust-key <file>`: also trust this public key (test key of a local source). Never the default.
    let extra = all
        .iter()
        .position(|a| a == "--trust-key")
        .map(|i| all.get(i + 1).ok_or("--trust-key needs a file"))
        .transpose()?;
    let mut args = Vec::new();
    let mut it = all.iter();
    while let Some(a) = it.next() {
        if a == "--trust-key" {
            it.next();
        } else {
            args.push(a.clone());
        }
    }
    let root = root();
    match args.as_slice() {
        [c, rest @ ..] if c == "sources" => cmd_sources(&root, rest),
        [c] if c == "installed" => {
            cmd_installed(&root);
            Ok(())
        }
        [c] if c == "available" => cmd_available(&root, None),
        [c, s] if c == "available" => cmd_available(&root, Some(s)),
        [c, n] if c == "install" => cmd_install(&root, n, None, extra, false),
        [c, n, s] if c == "install" => cmd_install(&root, n, Some(s), extra, false),
        [c, n] if c == "update" => cmd_install(&root, n, None, extra, true),
        [c, n, s] if c == "update" => cmd_install(&root, n, Some(s), extra, true),
        _ => Err("usage: fs-store sources | available [source] | install <name> [source] | update <name> [source] | installed".into()),
    }
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
