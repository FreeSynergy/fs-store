//! Pattern: Command (`fs-store owner <path>` and `fs-store verify [name]`). Both answer from the store's own list of files;
//! `rpm -qf` / `dpkg -S` are only added as a cross-check for a path that is none of ours.

use std::io::IsTerminal;
use std::path::Path;

use fs_store_engine::Verdict;

fn red(text: &str) -> String {
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
        format!("\x1b[31m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

fn source_of(root: &Path) -> impl Fn(&str) -> String {
    let registry = fs_store_sources::Registry::load(&root.join("installed.toml")).ok();
    move |name| {
        registry
            .as_ref()
            .and_then(|r| r.get(name))
            .map(|e| e.source.clone())
            .unwrap_or_default()
    }
}

/// `fs-store owner <path>`: which package, version, source and signature a file belongs to, and how sure that is.
pub(crate) fn owner(root: &Path, path: &str) -> Result<(), String> {
    let Some(o) = fs_store_engine::owner(root, Path::new(path))? else {
        match fs_store_engine::system_owner(Path::new(path)) {
            Some(s) => {
                println!("{path}: not installed by this store; system cross-check says {s}");
            }
            None => println!("{path}: not installed by this store"),
        }
        return Ok(());
    };
    let m = &o.manifest;
    println!("{path}");
    println!("  package:   {} {}", m.name, m.version);
    println!("  source:    {}", or_unknown(&m.source));
    println!("  signature: {}", or_unknown(&m.signer));
    println!("  checksum:  {}", o.file.hash);
    println!("  proof:     {}", o.proof());
    if o.verdict == Verdict::Intact {
        Ok(())
    } else {
        Err(red(&format!("{path}: {}", o.proof())))
    }
}

fn or_unknown(text: &str) -> &str {
    if text.is_empty() { "unknown" } else { text }
}

/// `fs-store verify [name]`: our own installations against their lists; exit code 1 when a file changed or is missing.
pub(crate) fn verify(root: &Path, only: Option<&str>) -> Result<(), String> {
    let reports = fs_store_engine::verify(root, only, &source_of(root))?;
    let mut bad = 0usize;
    for r in &reports {
        let m = &r.manifest;
        let note = if r.made_now {
            " (list made now from the tree found: older changes cannot be seen)"
        } else if m.basis == fs_store_sources::Basis::Retroactive {
            " (list made afterwards, not at install)"
        } else {
            ""
        };
        println!("{} {}: {} files{note}", m.name, m.version, r.files.len());
        for (path, verdict) in &r.files {
            match verdict {
                Verdict::Intact => {}
                Verdict::Changed => {
                    bad += 1;
                    println!("  {}", red(&format!("changed  {path}")));
                }
                Verdict::Missing => {
                    bad += 1;
                    println!("  {}", red(&format!("missing  {path}")));
                }
            }
        }
    }
    if bad == 0 {
        println!("all {} packages intact", reports.len());
        Ok(())
    } else {
        Err(red(&format!("{bad} files changed or missing")))
    }
}
