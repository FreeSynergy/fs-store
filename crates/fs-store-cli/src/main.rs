// fs-store CLI — command-line interface for the FreeSynergy Store.
//
// Source detection (highest priority first):
//   1. --local <path>  flag
//   2. FS_STORE_LOCAL  environment variable
//   3. Official HTTP store (default)
//
// Commands:
//   list       — list all packages (filterable by namespace and search term)
//   info <id>  — detailed view of one package
//   installed  — list locally installed packages

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use fs_store::{Inventory, StoreReader, StoreSettings, StoreSource};

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "fs-store",
    about = "FreeSynergy Store — manage packages from the command line",
    version,
    arg_required_else_help = true
)]
struct Cli {
    /// Use a local Store checkout instead of the official HTTP source.
    #[arg(long, env = "FS_STORE_LOCAL", global = true, value_name = "PATH")]
    local: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List available packages.
    List {
        /// Filter by namespace: apps, containers, themes, widgets, tasks,
        /// languages, icons, bundles, externals, repos
        #[arg(short, long, value_name = "NS")]
        namespace: Option<String>,

        /// Filter by search term (matches id, name, or summary).
        #[arg(short, long, value_name = "QUERY")]
        search: Option<String>,
    },

    /// Show detailed information about a package.
    Info {
        /// Package id, e.g. "forgejo"
        id: String,
    },

    /// List locally installed packages.
    Installed,
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();
    let reader = make_reader(cli.local);

    let mut settings = StoreSettings::default();
    // Respect FS_STORE_DATA_DIR override for testing.
    if let Ok(dir) = std::env::var("FS_STORE_DATA_DIR") {
        settings.storage.data_dir = PathBuf::from(dir);
    }

    let mut inv = Inventory::new(settings);
    eprint!("Loading catalog… ");
    inv.load(&reader).await?;
    eprintln!("done ({} packages)", inv.states.len());

    match cli.command {
        Command::List { namespace, search } => {
            cmd_list(&inv, namespace.as_deref(), search.as_deref())
        }
        Command::Info { id } => cmd_info(&inv, &id),
        Command::Installed => cmd_installed(&inv),
    }

    Ok(())
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn cmd_list(inv: &Inventory, namespace: Option<&str>, search: Option<&str>) {
    let rows: Vec<_> = inv
        .states
        .iter()
        .filter(|s| {
            if let Some(ns) = namespace {
                inv.namespaces
                    .namespace_of(s.package.id())
                    .map(|n| n == ns)
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .filter(|s| {
            if let Some(q) = search {
                let q = q.to_lowercase();
                s.package.id().to_lowercase().contains(&q)
                    || s.package.name().to_lowercase().contains(&q)
                    || s.package.summary().to_lowercase().contains(&q)
            } else {
                true
            }
        })
        .collect();

    if rows.is_empty() {
        println!("No packages found.");
        return;
    }

    // Dynamic column widths.
    let w_id = rows
        .iter()
        .map(|s| s.package.id().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let w_name = rows
        .iter()
        .map(|s| s.package.name().len())
        .max()
        .unwrap_or(4)
        .max(4);
    let w_ver = rows
        .iter()
        .map(|s| s.package.latest_version().unwrap_or("-").len())
        .max()
        .unwrap_or(7)
        .max(7);
    let w_ns = 10;

    println!(
        "{:<w_id$}  {:<w_name$}  {:>w_ver$}  {:<w_ns$}  SUMMARY",
        "ID", "NAME", "VERSION", "NAMESPACE",
    );
    println!("{}", "-".repeat(w_id + w_name + w_ver + w_ns + 4 + 2 + 10));

    for s in &rows {
        let ns = inv.namespaces.namespace_of(s.package.id()).unwrap_or("-");
        let installed_marker = if s.is_installed() { " ✓" } else { "" };
        println!(
            "{:<w_id$}  {:<w_name$}  {:>w_ver$}  {:<w_ns$}  {}{}",
            s.package.id(),
            s.package.name(),
            s.package.latest_version().unwrap_or("-"),
            ns,
            s.package.summary(),
            installed_marker,
        );
    }
    println!("\n{} package(s)", rows.len());
}

fn cmd_info(inv: &Inventory, id: &str) {
    let Some(state) = inv.package_state(id) else {
        eprintln!("Package '{id}' not found.");
        std::process::exit(1);
    };

    let pkg = &state.package;
    let ns = inv.namespaces.namespace_of(id).unwrap_or("unknown");

    println!("Package:    {} ({})", pkg.name(), pkg.id());
    println!("Namespace:  {ns}");
    println!("Version:    {}", pkg.latest_version().unwrap_or("unknown"));
    println!("Summary:    {}", pkg.summary());

    if !pkg.description().is_empty() {
        println!();
        println!("Description:");
        for line in pkg.description().lines() {
            println!("  {line}");
        }
    }

    if !pkg.tags().is_empty() {
        println!();
        println!("Tags:       {}", pkg.tags().join(", "));
    }

    if let Some(icon) = pkg.icon_path() {
        println!("Icon:       {icon}");
    }

    println!();
    if state.is_installed() {
        let active = state.active().unwrap();
        println!("Installed:  Yes (v{})", active.version);
        println!("Path:       {}", active.install_path.display());
        println!("Since:      {}", active.installed_at.format("%Y-%m-%d"));
    } else {
        println!("Installed:  No");
    }

    if state.has_update() {
        let latest = state.latest_available().unwrap();
        println!("Update:     v{} available", latest.version);
    }
}

fn cmd_installed(inv: &Inventory) {
    let installed: Vec<_> = inv.installed().collect();

    if installed.is_empty() {
        println!("No packages installed.");
        return;
    }

    let w_id = installed
        .iter()
        .map(|s| s.package.id().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let w_name = installed
        .iter()
        .map(|s| s.package.name().len())
        .max()
        .unwrap_or(4)
        .max(4);

    println!(
        "{:<w_id$}  {:<w_name$}  VERSION   ACTIVE  PATH",
        "ID", "NAME"
    );
    println!("{}", "-".repeat(w_id + w_name + 34));

    for s in &installed {
        for rec in &s.installed {
            let active = if rec.is_active { "✓" } else { " " };
            println!(
                "{:<w_id$}  {:<w_name$}  {:<8}  {:<6}  {}",
                s.package.id(),
                s.package.name(),
                rec.version,
                active,
                rec.install_path.display(),
            );
        }
    }
    println!("\n{} package(s) installed", installed.len());
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_reader(local: Option<PathBuf>) -> StoreReader {
    match local {
        Some(path) => {
            eprintln!("Source: local ({})", path.display());
            StoreReader::new(StoreSource::Local(path))
        }
        None => {
            eprintln!("Source: official store");
            StoreReader::official()
        }
    }
}
