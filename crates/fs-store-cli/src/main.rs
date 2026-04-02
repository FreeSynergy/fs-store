#![deny(clippy::all, clippy::pedantic, warnings)]
// fs-store CLI — command-line interface for the FreeSynergy Store.
//
// Source detection (highest priority first):
//   1. --local <path>  flag
//   2. FS_STORE_LOCAL  environment variable
//   3. Official HTTP store (default)
//
// Commands:
//   list              — list all packages (filterable by namespace + search)
//   info <id>         — detailed view of one package
//   installed         — list locally installed packages
//   install <id>      — install a package via the Pipeline
//   remove <id>       — remove an installed package
//   update [id]       — update one or all installed packages

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use fs_store::{
    InstallContext, InstallKind, InstallRequest, InstallTarget, Inventory, Pipeline, StoreReader,
    StoreSettings, StoreSource,
};

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

    /// Override the fs-inventory base URL (e.g. `http://localhost:8082`).
    #[arg(long, env = "FS_INVENTORY_URL", global = true)]
    inventory_url: Option<String>,

    /// Override the fs-registry base URL (e.g. `http://localhost:8081`).
    #[arg(long, env = "FS_REGISTRY_URL", global = true)]
    registry_url: Option<String>,

    /// Override the fs-bus base URL (e.g. `http://localhost:8090`).
    #[arg(long, env = "FS_BUS_URL", global = true)]
    bus_url: Option<String>,

    #[command(subcommand)]
    command: Command,
}

// ── Install target ─────────────────────────────────────────────────────────────

#[derive(Clone, ValueEnum)]
enum TargetArg {
    Container,
    Rpm,
    Deb,
    Flatpak,
    Appimage,
}

impl From<TargetArg> for InstallTarget {
    fn from(t: TargetArg) -> Self {
        match t {
            TargetArg::Container => Self::Container,
            TargetArg::Rpm => Self::Rpm,
            TargetArg::Deb => Self::Deb,
            TargetArg::Flatpak => Self::Flatpak,
            TargetArg::Appimage => Self::AppImage,
        }
    }
}

// ── Subcommands ────────────────────────────────────────────────────────────────

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

    /// Install a package.
    Install {
        /// Package id, e.g. "forgejo" or "freeSynergy.bundle.server"
        id: String,

        /// Install target (default: container for containers, native for apps).
        #[arg(long, value_enum, default_value = "container")]
        target: TargetArg,

        /// Environment variables for container installs (KEY=VALUE, one per flag).
        #[arg(long = "env", value_name = "KEY=VALUE")]
        env_vars: Vec<String>,
    },

    /// Remove an installed package.
    Remove {
        /// Package id, e.g. "forgejo"
        id: String,
    },

    /// Update one or all installed packages.
    Update {
        /// Package id to update. Omit to update all installed packages.
        id: Option<String>,
    },
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
    if let Ok(dir) = std::env::var("FS_STORE_DATA_DIR") {
        settings.storage.data_dir = PathBuf::from(dir);
    }
    let data_dir = settings.storage.data_dir.clone();

    let mut inv = Inventory::new(settings);
    eprint!("Loading catalog… ");
    inv.load(&reader).await?;
    eprintln!("done ({} packages)", inv.states.len());

    match cli.command {
        Command::List { namespace, search } => {
            cmd_list(&inv, namespace.as_deref(), search.as_deref());
        }
        Command::Info { id } => cmd_info(&inv, &id),
        Command::Installed => cmd_installed(&inv),
        Command::Install {
            id,
            target,
            env_vars,
        } => {
            cmd_install(
                &inv,
                &id,
                target.into(),
                env_vars,
                data_dir,
                cli.inventory_url,
                cli.registry_url,
                cli.bus_url,
            )
            .await?;
        }
        Command::Remove { id } => cmd_remove(&mut inv, &id)?,
        Command::Update { id } => cmd_update(&inv, id.as_deref(), data_dir),
    }

    Ok(())
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn cmd_list(inv: &Inventory, namespace: Option<&str>, search: Option<&str>) {
    let rows: Vec<_> = inv
        .states
        .iter()
        .filter(|s| {
            namespace.is_none_or(|ns| {
                inv.namespaces
                    .namespace_of(s.package.id())
                    .is_some_and(|n| n == ns)
            })
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

#[allow(clippy::too_many_arguments)]
async fn cmd_install(
    inv: &Inventory,
    id: &str,
    target: InstallTarget,
    env_kv: Vec<String>,
    fs_dir: PathBuf,
    inventory_url: Option<String>,
    registry_url: Option<String>,
    bus_url: Option<String>,
) -> Result<()> {
    let Some(state) = inv.package_state(id) else {
        eprintln!("Package '{id}' not found.");
        std::process::exit(1);
    };

    let pkg = &state.package;
    let version = pkg.latest_version().unwrap_or("0.0.0").to_string();

    println!(
        "Installing {} v{version} as {}…",
        pkg.name(),
        target.label()
    );

    // Infer install kind from package namespace.
    let kind = infer_kind(inv, id);

    let request = InstallRequest {
        id: id.to_string(),
        name: pkg.name().to_string(),
        kind,
        version,
        store_path: None,
        capabilities: vec![],
        icon: pkg.icon_path().map(String::from),
    };

    let env_vars = env_kv.join("\n");

    let mut ctx = InstallContext::new(request.clone(), target.clone(), fs_dir);
    ctx.env_vars = env_vars;
    ctx.inventory_url = inventory_url;
    ctx.registry_url = registry_url;
    ctx.bus_url = bus_url;

    // Subscribe to progress events and print them.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    ctx.progress = Some(tx);

    let pipeline = Pipeline::for_request(&request, &target);

    // Print progress in a background task.
    let print_task = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            use fs_store::PipelineEvent;
            match event {
                PipelineEvent::StepStarted { step } => print!("  [{step}] "),
                PipelineEvent::StepCompleted { .. } => println!("done"),
                PipelineEvent::StepSkipped { reason, .. } => println!("skipped ({reason})"),
                PipelineEvent::Failed { step, reason } => {
                    println!("FAILED");
                    eprintln!("  Error in [{step}]: {reason}");
                }
                PipelineEvent::Done => {}
            }
        }
    });

    let result = pipeline.run(&mut ctx).await;
    let _ = print_task.await;

    match result {
        Ok(()) => println!("{} installed successfully.", pkg.name()),
        Err(e) => {
            eprintln!("Installation failed: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn cmd_remove(inv: &mut Inventory, id: &str) -> Result<()> {
    if inv.package_state(id).is_none() {
        eprintln!("Package '{id}' not found.");
        std::process::exit(1);
    }

    let (name, version) = {
        let state = inv.package_state(id).unwrap();
        if !state.is_installed() {
            println!("{} is not installed.", state.package.name());
            return Ok(());
        }
        let v = state
            .active()
            .map(|r| r.version.clone())
            .unwrap_or_default();
        (state.package.name().to_string(), v)
    };

    println!("Removing {name}…");

    inv.record_removed(id, &version);
    inv.save_records()?;

    println!("{id} removed.");
    Ok(())
}

fn cmd_update(inv: &Inventory, id: Option<&str>, _fs_dir: PathBuf) {
    let targets: Vec<_> = if let Some(id) = id {
        let Some(state) = inv.package_state(id) else {
            eprintln!("Package '{id}' not found.");
            std::process::exit(1);
        };
        vec![state]
    } else {
        inv.installed().filter(|s| s.has_update()).collect()
    };

    if targets.is_empty() {
        println!("Nothing to update.");
        return;
    }

    for state in targets {
        let latest = state.latest_available().map_or_else(
            || state.package.latest_version().unwrap_or("?").to_string(),
            |r| r.version.clone(),
        );
        println!("  {} → v{latest}", state.package.id());
    }

    println!("\nNote: run 'fs-store install <id>' for each package to complete the update.");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_reader(local: Option<PathBuf>) -> StoreReader {
    if let Some(path) = local {
        eprintln!("Source: local ({})", path.display());
        StoreReader::new(StoreSource::Local(path))
    } else {
        eprintln!("Source: official store");
        StoreReader::official()
    }
}

fn infer_kind(inv: &Inventory, id: &str) -> InstallKind {
    match inv.namespaces.namespace_of(id).unwrap_or("") {
        "containers" => InstallKind::Container,
        "apps" | "managers" => InstallKind::App,
        "themes" => InstallKind::Theme,
        "languages" => InstallKind::Language,
        "bundles" => InstallKind::Bundle,
        _ => InstallKind::Other,
    }
}
