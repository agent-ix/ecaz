use color_eyre::eyre::{bail, eyre, Context, Result};
use std::cmp::Ordering;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub(crate) const SCRATCH_DEFAULT_PORT: u16 = 28817;
pub(crate) const PG18_PRELOAD_DEFAULT_PORT: u16 = 28818;
pub(crate) const PG18_PARALLEL_SCAN_DEFAULT_PORT: u16 = 28819;

#[derive(Debug, Clone)]
pub(crate) struct PgrxInstall {
    pub(crate) version_label: String,
    pub(crate) root: PathBuf,
    pub(crate) bin_dir: PathBuf,
    pub(crate) pg_config: PathBuf,
}

pub(crate) fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .wrap_err("resolving repo root from crates/ecaz-cli")
}

pub(crate) fn default_pgrx_home() -> PathBuf {
    env::var_os("PGRX_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".pgrx")))
        .unwrap_or_else(|| PathBuf::from(".pgrx"))
}

pub(crate) fn resolve_pgrx_home(explicit: Option<&PathBuf>) -> PathBuf {
    explicit.cloned().unwrap_or_else(default_pgrx_home)
}

pub(crate) fn find_pgrx_install(major: u16, pgrx_home: &Path) -> Result<PgrxInstall> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(pgrx_home)
        .wrap_err_with(|| format!("reading pgrx home {}", pgrx_home.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(&major.to_string()) {
            continue;
        }
        let root = entry.path().join("pgrx-install");
        let pg_config = root.join("bin/pg_config");
        if pg_config.is_file() {
            candidates.push((name, root, pg_config));
        }
    }
    candidates.sort_by(|a, b| compare_version_labels(&a.0, &b.0));
    let Some((version_label, root, pg_config)) = candidates.pop() else {
        bail!(
            "could not find a PG{} pgrx install under {}",
            major,
            pgrx_home.display()
        );
    };
    let bin_dir = root.join("bin");
    Ok(PgrxInstall {
        version_label,
        root,
        bin_dir,
        pg_config,
    })
}

fn compare_version_labels(lhs: &str, rhs: &str) -> Ordering {
    let lhs_parts = lhs
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect::<Vec<_>>();
    let rhs_parts = rhs
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect::<Vec<_>>();
    lhs_parts.cmp(&rhs_parts).then_with(|| lhs.cmp(rhs))
}

pub(crate) async fn run_status(mut command: Command) -> Result<()> {
    let debug = format!("{command:?}");
    let status = command
        .status()
        .await
        .wrap_err_with(|| format!("running {debug}"))?;
    if !status.success() {
        bail!("{debug} failed with status {status}");
    }
    Ok(())
}

pub(crate) fn scratch_socket_dir(
    explicit: Option<&PathBuf>,
    pgrx_home: &Path,
    port: u16,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.clone());
    }
    if let Some(path) = env::var_os("TQV_PG_SOCKET_DIR") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("PGHOST") {
        return Ok(PathBuf::from(path));
    }
    let socket_dir = pgrx_home.to_path_buf();
    let socket_name = format!(".s.PGSQL.{port}");
    let socket_path = socket_dir.join(&socket_name);
    if socket_path.exists() {
        return Ok(socket_dir);
    }
    Err(eyre!(
        "expected scratch socket at {}; pass --socket-dir or --host explicitly if the cluster lives elsewhere",
        socket_path.display()
    ))
}

pub(crate) fn refresh_debug_helpers_sql() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("sql/refresh_adr030_scratch_debug_helpers.sql")
        .canonicalize()
        .wrap_err("resolving bundled scratch debug-helper SQL")?)
}
