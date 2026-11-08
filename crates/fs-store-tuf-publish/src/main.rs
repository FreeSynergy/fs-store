//! `fs-store-tuf-publish publish <dir> [--targets-days N] [--snapshot-days N] [--timestamp-days N]`
//! `fs-store-tuf-publish generate <dir> [--root-keys 3] [--threshold 2] [--root-days 365]`

use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use fs_store_tuf_publish::{Expiries, generate_keys, publish};

const USAGE: &str = "usage: fs-store-tuf-publish publish <dir> [--targets-days N] [--snapshot-days N] [--timestamp-days N]\n       fs-store-tuf-publish generate <dir> [--root-keys 3] [--threshold 2] [--root-days 365]";

/// Value of `--name N` in `args`, or `default`.
fn option(args: &[String], name: &str, default: u64) -> Result<u64, String> {
    match args.iter().position(|a| a == name) {
        None => Ok(default),
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| format!("{name} needs a number")),
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let (Some(cmd), Some(dir)) = (args.first(), args.get(1)) else {
        return Err(USAGE.to_owned());
    };
    let (dir, rest) = (Path::new(dir), &args[2..]);
    let days = |d: u64| Duration::from_secs(d.saturating_mul(86_400));
    match cmd.as_str() {
        "publish" => {
            let d = Expiries::default();
            let e = Expiries {
                targets: option(rest, "--targets-days", d.targets.as_secs() / 86_400).map(days)?,
                snapshot: option(rest, "--snapshot-days", d.snapshot.as_secs() / 86_400)
                    .map(days)?,
                timestamp: option(rest, "--timestamp-days", d.timestamp.as_secs() / 86_400)
                    .map(days)?,
            };
            let v = publish(dir, &e).map_err(|e| e.to_string())?;
            Ok(format!(
                "published: targets v{}, snapshot v{}, timestamp v{}",
                v.targets, v.snapshot, v.timestamp
            ))
        }
        "generate" => {
            let keys =
                usize::try_from(option(rest, "--root-keys", 3)?).map_err(|e| e.to_string())?;
            let threshold = option(rest, "--threshold", 2)?;
            let validity = days(option(rest, "--root-days", 365)?);
            generate_keys(dir, keys, threshold, validity).map_err(|e| e.to_string())?;
            Ok(format!(
                "keys and root.json written to {}; keep root-*.pk8 offline",
                dir.display()
            ))
        }
        _ => Err(USAGE.to_owned()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
