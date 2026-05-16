use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tokio::process::Command;

use super::support::{
    find_pgrx_install, repo_root, resolve_pgrx_home, run_status, PgrxInstall, DEFAULT_PG_MAJOR,
};

#[derive(Subcommand, Debug)]
pub enum InstallCommand {
    /// Install the ecaz pg_test build into a local pgrx tree and verify the backend artifact.
    EcazPgTest(InstallEcazPgTestArgs),
    /// Install pgvector into the selected pg_config tree for side-by-side comparison lanes.
    Pgvector(InstallPgvectorArgs),
    /// Install pgvectorscale into the selected pg_config tree for DiskANN comparison lanes.
    Vectorscale(InstallVectorscaleArgs),
}

impl InstallCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        match self {
            InstallCommand::EcazPgTest(args) => run_ecaz_pg_test(args).await,
            InstallCommand::Pgvector(args) => run_pgvector(args).await,
            InstallCommand::Vectorscale(args) => run_vectorscale(args).await,
        }
    }
}

#[derive(Args, Debug)]
pub struct InstallEcazPgTestArgs {
    /// PostgreSQL major version to install against.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit pg_config path. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pg_config: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct InstallPgvectorArgs {
    /// pgvector repository checkout.
    #[arg(long, env = "PGVECTOR_REPO", default_value_os_t = default_pgvector_repo())]
    repo: PathBuf,

    /// PostgreSQL major version to install against.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit pg_config path. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pg_config: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct InstallVectorscaleArgs {
    /// pgvectorscale extension crate checkout.
    #[arg(
        long,
        env = "PGVECTORSCALE_REPO",
        default_value_os_t = default_vectorscale_repo()
    )]
    repo: PathBuf,

    /// cargo-pgrx binary to use for the install.
    #[arg(
        long,
        env = "CARGO_PGRX_BIN",
        default_value_os_t = default_cargo_pgrx_bin()
    )]
    cargo_pgrx: PathBuf,

    /// PostgreSQL major version to install against.
    #[arg(long, default_value_t = DEFAULT_PG_MAJOR)]
    pg: u16,

    /// Override PGRX_HOME.
    #[arg(long)]
    pgrx_home: Option<PathBuf>,

    /// Explicit pg_config path. Defaults to the newest matching pgrx install.
    #[arg(long)]
    pg_config: Option<PathBuf>,
}

async fn run_ecaz_pg_test(args: InstallEcazPgTestArgs) -> Result<()> {
    let repo_root = repo_root()?;
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = resolve_install(args.pg, args.pg_config, &pgrx_home)?;
    crate::ecaz_println!("[install] repo={}", repo_root.display());
    crate::ecaz_println!("[install] pgrx_home={}", pgrx_home.display());
    crate::ecaz_println!("[install] pg_config={}", install.pg_config.display());

    let mut command = Command::new("cargo");
    command
        .arg("pgrx")
        .arg("install")
        .arg("--pg-config")
        .arg(&install.pg_config)
        .arg("--release")
        .arg("--features")
        .arg(format!("pg{} pg_test", args.pg))
        .arg("--no-default-features")
        .current_dir(&repo_root)
        .env("PGRX_HOME", &pgrx_home);
    run_status(command).await?;

    let release_artifact = repo_root
        .join("target/release")
        .join(ecaz_built_library_name());
    let installed_backend =
        pg_config_value(&install.pg_config, "--pkglibdir")?.join(ecaz_installed_library_name());
    assert_matching_backend(&release_artifact, &installed_backend)?;
    crate::ecaz_println!("[install] backend artifact assertion passed");
    crate::ecaz_println!(
        "[install] installed_backend={}",
        installed_backend.display()
    );
    crate::ecaz_println!("[install] sha256={}", sha256_hex(&installed_backend)?);
    Ok(())
}

async fn run_pgvector(args: InstallPgvectorArgs) -> Result<()> {
    if !args.repo.is_dir() {
        bail!("pgvector repo not found: {}", args.repo.display());
    }
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = resolve_install(args.pg, args.pg_config, &pgrx_home)?;
    crate::ecaz_println!("[install] pgvector_repo={}", args.repo.display());
    crate::ecaz_println!("[install] pgrx_home={}", pgrx_home.display());
    crate::ecaz_println!("[install] pg_config={}", install.pg_config.display());

    let mut command = Command::new("make");
    command
        .arg("-C")
        .arg(&args.repo)
        .arg(format!("PG_CONFIG={}", install.pg_config.display()))
        .arg("install");
    run_status(command).await?;
    crate::ecaz_println!("[install] finished installing pgvector");
    Ok(())
}

async fn run_vectorscale(args: InstallVectorscaleArgs) -> Result<()> {
    if !args.repo.is_dir() {
        bail!("pgvectorscale repo not found: {}", args.repo.display());
    }
    if !args.cargo_pgrx.is_file() {
        bail!("cargo-pgrx binary not found: {}", args.cargo_pgrx.display());
    }
    let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
    let install = resolve_install(args.pg, args.pg_config, &pgrx_home)?;
    crate::ecaz_println!("[install] pgvectorscale_repo={}", args.repo.display());
    crate::ecaz_println!("[install] cargo_pgrx={}", args.cargo_pgrx.display());
    crate::ecaz_println!("[install] pgrx_home={}", pgrx_home.display());
    crate::ecaz_println!("[install] pg_config={}", install.pg_config.display());

    let mut command = Command::new(&args.cargo_pgrx);
    command
        .arg("pgrx")
        .arg("install")
        .arg("--release")
        .arg("--pg-config")
        .arg(&install.pg_config)
        .current_dir(&args.repo);
    run_status(command).await?;
    crate::ecaz_println!("[install] finished installing pgvectorscale");
    Ok(())
}

fn resolve_install(
    pg: u16,
    explicit_pg_config: Option<PathBuf>,
    pgrx_home: &PathBuf,
) -> Result<PgrxInstall> {
    if let Some(pg_config) = explicit_pg_config {
        let root = pg_config
            .parent()
            .and_then(std::path::Path::parent)
            .map(PathBuf::from)
            .context("resolving install root from --pg-config")?;
        let version_label = root
            .parent()
            .and_then(std::path::Path::parent)
            .and_then(std::path::Path::file_name)
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("pg{pg}"));
        return Ok(PgrxInstall {
            version_label,
            bin_dir: root.join("bin"),
            root,
            pg_config,
        });
    }
    find_pgrx_install(pg, pgrx_home)
}

fn assert_matching_backend(release_artifact: &PathBuf, installed_backend: &PathBuf) -> Result<()> {
    if !release_artifact.is_file() {
        bail!(
            "expected release artifact missing: {}",
            release_artifact.display()
        );
    }
    if !installed_backend.is_file() {
        bail!("installed backend missing: {}", installed_backend.display());
    }
    let built = fs::read(release_artifact)
        .wrap_err_with(|| format!("reading {}", release_artifact.display()))?;
    let installed = fs::read(installed_backend)
        .wrap_err_with(|| format!("reading {}", installed_backend.display()))?;
    if built != installed {
        bail!(
            "backend .so mismatch after install\nbuilt={}\ninstalled={}\nbuilt_sha256={}\ninstalled_sha256={}",
            release_artifact.display(),
            installed_backend.display(),
            sha256_hex(release_artifact)?,
            sha256_hex(installed_backend)?
        );
    }
    Ok(())
}

fn sha256_hex(path: &PathBuf) -> Result<String> {
    let bytes = fs::read(path).wrap_err_with(|| format!("reading {}", path.display()))?;
    let mut digest = Sha256::new();
    digest.update(bytes);
    Ok(format!("{:x}", digest.finalize()))
}

fn ecaz_built_library_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "libecaz.dylib"
    } else {
        "libecaz.so"
    }
}

fn ecaz_installed_library_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "ecaz.dylib"
    } else {
        "ecaz.so"
    }
}

fn pg_config_value(pg_config: &PathBuf, flag: &str) -> Result<PathBuf> {
    let output = StdCommand::new(pg_config)
        .arg(flag)
        .output()
        .wrap_err_with(|| format!("running {} {flag}", pg_config.display()))?;
    if !output.status.success() {
        bail!(
            "{} {flag} failed with status {}",
            pg_config.display(),
            output.status
        );
    }
    let value = String::from_utf8(output.stdout)
        .wrap_err_with(|| format!("decoding {} {flag} output", pg_config.display()))?;
    Ok(PathBuf::from(value.trim()))
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn default_pgvector_repo() -> PathBuf {
    home_dir().join("dev_bak/pgvector")
}

fn default_vectorscale_repo() -> PathBuf {
    home_dir().join("dev_bak/pgvectorscale/pgvectorscale")
}

fn default_cargo_pgrx_bin() -> PathBuf {
    home_dir().join(".cargo/bin/cargo-pgrx")
}
