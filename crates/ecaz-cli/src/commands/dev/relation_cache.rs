use clap::Args;
use color_eyre::eyre::{bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

use crate::psql::ConnectionOptions;

#[derive(Args, Debug)]
pub struct EvictRelationCacheArgs {
    /// Benchmark prefix whose `<prefix>_corpus` table and related indexes/toast
    /// files should be evicted from the local OS page cache.
    #[arg(long = "prefix")]
    prefixes: Vec<String>,

    /// Explicit relation to evict. Repeatable; resolved as a PostgreSQL
    /// regclass in the target database.
    #[arg(long = "relation")]
    relations: Vec<String>,

    /// Print the files that would be evicted without calling posix_fadvise.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RelationPath {
    relname: String,
    relkind: String,
    relpath: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvictFile {
    relation: RelationPath,
    path: PathBuf,
    bytes: u64,
}

pub async fn run(conn: &ConnectionOptions, args: EvictRelationCacheArgs) -> Result<()> {
    if args.prefixes.is_empty() && args.relations.is_empty() {
        bail!("provide at least one --prefix or --relation");
    }

    let client = crate::psql::connect(conn).await?;
    let data_dir = data_directory(&client).await?;
    let mut relations = BTreeSet::new();

    for prefix in &args.prefixes {
        relations.extend(resolve_prefix_relations(&client, prefix).await?);
    }
    for relation in &args.relations {
        relations.extend(resolve_explicit_relation(&client, relation).await?);
    }

    if relations.is_empty() {
        bail!("no local relation files resolved for eviction");
    }

    let mut files = Vec::new();
    for relation in relations {
        files.extend(resolve_relation_files(&data_dir, &relation)?);
    }
    if files.is_empty() {
        bail!("resolved relations have no local relation files to evict");
    }

    let total_bytes: u64 = files.iter().map(|file| file.bytes).sum();
    println!(
        "cache_evict_start database={} dry_run={} data_directory={} relations={} files={} bytes={}",
        conn.database,
        args.dry_run,
        data_dir.display(),
        files
            .iter()
            .map(|file| file.relation.relname.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        files.len(),
        total_bytes
    );

    let eviction_mode = if args.dry_run {
        None
    } else {
        Some(evict_files(&files)?)
    };
    let mut relation_bytes: BTreeMap<String, u64> = BTreeMap::new();
    for file in &files {
        *relation_bytes
            .entry(file.relation.relname.clone())
            .or_default() += file.bytes;
        if args.dry_run {
            println!(
                "cache_evict_file status=dry_run relation={} relkind={} bytes={} path={}",
                file.relation.relname,
                file.relation.relkind,
                file.bytes,
                file.path.display()
            );
            continue;
        }
        let status = match eviction_mode {
            #[cfg(any(target_os = "linux", target_os = "android"))]
            Some(EvictionMode::PerFile) => "evicted",
            #[cfg(target_os = "macos")]
            Some(EvictionMode::MacNoCache) => "evicted_macos_f_nocache",
            None => "dry_run",
        };
        println!(
            "cache_evict_file status={} relation={} relkind={} bytes={} path={}",
            status,
            file.relation.relname,
            file.relation.relkind,
            file.bytes,
            file.path.display()
        );
    }

    for (relation, bytes) in relation_bytes {
        println!("cache_evict_relation relation={relation} bytes={bytes}");
    }
    println!(
        "cache_evict_summary database={} dry_run={} relations={} files={} bytes={}",
        conn.database,
        args.dry_run,
        files
            .iter()
            .map(|file| file.relation.relname.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        files.len(),
        total_bytes
    );
    Ok(())
}

async fn data_directory(client: &tokio_postgres::Client) -> Result<PathBuf> {
    let row = client
        .query_one("SHOW data_directory", &[])
        .await
        .wrap_err("querying data_directory")?;
    Ok(PathBuf::from(row.get::<_, String>(0)))
}

async fn resolve_prefix_relations(
    client: &tokio_postgres::Client,
    prefix: &str,
) -> Result<Vec<RelationPath>> {
    let table = format!("{prefix}_corpus");
    let rows = client
        .query(
            "
            WITH base AS (
                SELECT c.oid, c.reltoastrelid
                FROM pg_class c
                WHERE c.relname = $1
                  AND c.relkind IN ('r', 'p')
            ),
            rels AS (
                SELECT oid FROM base
                UNION
                SELECT ix.indexrelid
                FROM pg_index ix
                JOIN base b ON ix.indrelid = b.oid
                UNION
                SELECT reltoastrelid
                FROM base
                WHERE reltoastrelid <> 0
                UNION
                SELECT ix.indexrelid
                FROM pg_index ix
                JOIN base b ON ix.indrelid = b.reltoastrelid
            )
            SELECT c.relname,
                   c.relkind::text,
                   pg_relation_filepath(c.oid)
            FROM rels
            JOIN pg_class c ON c.oid = rels.oid
            WHERE pg_relation_filepath(c.oid) IS NOT NULL
            ORDER BY c.relname
            ",
            &[&table],
        )
        .await
        .wrap_err_with(|| format!("resolving local relations for prefix {prefix:?}"))?;
    if rows.is_empty() {
        bail!("prefix {prefix:?} did not resolve table {table:?}");
    }
    relation_rows(rows)
}

async fn resolve_explicit_relation(
    client: &tokio_postgres::Client,
    relation: &str,
) -> Result<Vec<RelationPath>> {
    let rows = client
        .query(
            "
            SELECT c.relname,
                   c.relkind::text,
                   pg_relation_filepath(c.oid)
            FROM pg_class c
            WHERE c.oid = $1::regclass
              AND pg_relation_filepath(c.oid) IS NOT NULL
            ",
            &[&relation],
        )
        .await
        .wrap_err_with(|| format!("resolving relation {relation:?}"))?;
    relation_rows(rows)
}

fn relation_rows(rows: Vec<tokio_postgres::Row>) -> Result<Vec<RelationPath>> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let relname = row.get::<_, String>(0);
        let relkind = row.get::<_, String>(1);
        let relpath = row.get::<_, Option<String>>(2);
        if let Some(relpath) = relpath {
            out.push(RelationPath {
                relname,
                relkind,
                relpath: PathBuf::from(relpath),
            });
        }
    }
    Ok(out)
}

fn resolve_relation_files(data_dir: &Path, relation: &RelationPath) -> Result<Vec<EvictFile>> {
    let relation_path = data_dir.join(&relation.relpath);
    let parent = relation_path
        .parent()
        .ok_or_else(|| color_eyre::eyre::eyre!("relation path has no parent"))?;
    let base = relation_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| color_eyre::eyre::eyre!("relation path has no file name"))?;
    let mut files = Vec::new();
    for entry in std::fs::read_dir(parent)
        .wrap_err_with(|| format!("reading relation directory {}", parent.display()))?
    {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        if !relation_file_name_matches(base, name) {
            continue;
        }
        let metadata = entry.metadata()?;
        if !metadata.is_file() {
            continue;
        }
        files.push(EvictFile {
            relation: relation.clone(),
            path: entry.path(),
            bytes: metadata.len(),
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn relation_file_name_matches(base: &str, name: &str) -> bool {
    name == base
        || name
            .strip_prefix(base)
            .is_some_and(|tail| tail.starts_with('.') || tail.starts_with('_'))
}

#[derive(Clone, Copy)]
enum EvictionMode {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    PerFile,
    #[cfg(target_os = "macos")]
    MacNoCache,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn evict_files(files: &[EvictFile]) -> Result<EvictionMode> {
    for file in files {
        evict_file(file)?;
    }
    Ok(EvictionMode::PerFile)
}

#[cfg(target_os = "macos")]
fn evict_files(files: &[EvictFile]) -> Result<EvictionMode> {
    for file in files {
        let handle = File::open(&file.path)
            .wrap_err_with(|| format!("opening relation file {}", file.path.display()))?;
        let rc = unsafe { libc::fcntl(handle.as_raw_fd(), libc::F_NOCACHE, 1) };
        if rc == -1 {
            let error = std::io::Error::last_os_error();
            bail!(
                "fcntl(F_NOCACHE) failed for {}: {error}",
                file.path.display()
            );
        }
    }
    Ok(EvictionMode::MacNoCache)
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
fn evict_files(files: &[EvictFile]) -> Result<EvictionMode> {
    let Some(file) = files.first() else {
        bail!("no relation files to evict");
    };
    let _handle = File::open(&file.path)
        .wrap_err_with(|| format!("opening relation file {}", file.path.display()))?;
    bail!("relation-cache eviction requires posix_fadvise(DONTNEED) or macOS F_NOCACHE, unavailable on this platform");
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn evict_file(file: &EvictFile) -> Result<()> {
    let handle = File::open(&file.path)
        .wrap_err_with(|| format!("opening relation file {}", file.path.display()))?;
    let rc = unsafe { libc::posix_fadvise(handle.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED) };
    if rc != 0 {
        bail!(
            "posix_fadvise(DONTNEED) failed for {} with errno {rc}",
            file.path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_file_match_includes_segments_and_forks() {
        assert!(relation_file_name_matches("12345", "12345"));
        assert!(relation_file_name_matches("12345", "12345.1"));
        assert!(relation_file_name_matches("12345", "12345_fsm"));
        assert!(relation_file_name_matches("12345", "12345_vm"));
        assert!(!relation_file_name_matches("12345", "123456"));
        assert!(!relation_file_name_matches("12345", "12345x"));
    }
}
