//! Pattern: Command (`fs-store sources ...`: list, add, enable, disable, remove).

use std::fs;
use std::path::Path;

use fs_store_core::{SourceId, SourceRepository};

use crate::commands::sources;

fn id_from_url(url: &str) -> String {
    url.trim_end_matches(['/', '.'])
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("source")
        .trim_end_matches(".git")
        .to_owned()
}

pub(crate) fn run(root: &Path, rest: &[String]) -> Result<(), String> {
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
