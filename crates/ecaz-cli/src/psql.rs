//! Postgres plumbing for the CLI.
//!
//! v1 goal: a thin `connect` helper over `tokio-postgres` plus a small set
//! of pure-SQL builders that live on top of `profiles` + `reloptions`.
//! Everything that issues DDL/DML lives on top of `connect`; nothing else
//! in the crate should shell out to `psql`.
//!
//! Connection defaults are resolved at the clap layer from explicit
//! global flags (`--database`, `--host`, `--port`, `--user`, `--password`)
//! with libpq env vars as fallback. This module consumes the concrete
//! connection options so commands can target a fixture DB or scratch
//! cluster without mutating process environment.

use color_eyre::eyre::{Context, Result};
use tokio_postgres::{Client, Config, NoTls};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionOptions {
    pub database: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
}

pub type ConnectParams = ConnectionOptions;

impl ConnectionOptions {
    pub fn config(&self) -> Config {
        let mut config = Config::new();
        config.dbname(&self.database);
        if let Some(host) = &self.host {
            config.host(host);
        }
        if let Some(port) = self.port {
            config.port(port);
        }
        if let Some(user) = &self.user {
            config.user(user);
        }
        if let Some(password) = &self.password {
            config.password(password);
        }
        config
    }
}

/// Open a connection using the already-resolved connection options.
pub async fn connect(options: &ConnectionOptions) -> Result<Client> {
    let config = options.config();
    let (client, connection) = config
        .connect(NoTls)
        .await
        .wrap_err_with(|| format!("connecting to Postgres database {:?}", options.database))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(error = %e, "postgres connection task failed");
        }
    });

    Ok(client)
}

pub async fn connect_with(params: &ConnectParams) -> Result<Client> {
    connect(params).await
}

fn encode_relkind(relkind: char) -> Result<i8> {
    i8::try_from(u32::from(relkind))
        .wrap_err_with(|| format!("relkind {:?} must be an ASCII catalog code", relkind))
}

/// Does a relation with the given name and `relkind` exist?
pub async fn relation_exists(client: &Client, name: &str, relkind: char) -> Result<bool> {
    let relkind = encode_relkind(relkind)?;
    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_class
                WHERE relname = $1
                  AND relkind = $2::\"char\"
            )",
            &[&name, &relkind],
        )
        .await
        .wrap_err_with(|| format!("checking relation {name:?} exists"))?;
    Ok(row.get::<_, bool>(0))
}

/// Row count for `table`, assumed to exist.
pub async fn row_count(client: &Client, table: &str) -> Result<i64> {
    let sql = format!("SELECT count(*) FROM {table}");
    let row = client
        .query_one(sql.as_str(), &[])
        .await
        .wrap_err_with(|| format!("counting rows in {table:?}"))?;
    Ok(row.get::<_, i64>(0))
}

/// Count indexes on `table` whose access method matches `am`.
pub async fn index_count_with_am(client: &Client, table: &str, am: &str) -> Result<i64> {
    let row = client
        .query_one(
            "SELECT count(*)
             FROM pg_class t
             JOIN pg_index ix ON ix.indrelid = t.oid
             JOIN pg_class i  ON i.oid = ix.indexrelid
             JOIN pg_am    pam ON pam.oid = i.relam
             WHERE t.relname = $1
               AND pam.amname = $2",
            &[&table, &am],
        )
        .await
        .wrap_err_with(|| format!("counting {am:?} indexes on {table:?}"))?;
    Ok(row.get::<_, i64>(0))
}

/// Bias the session toward the ordered ANN index path instead of seqscan/sort
/// fallbacks. Measurement commands use this so they time the selected access
/// method rather than an arbitrary planner alternative.
pub async fn prefer_ordered_ann_path(client: &Client) -> Result<()> {
    client
        .batch_execute(
            "SET enable_seqscan = off;
             SET enable_bitmapscan = off;
             SET enable_sort = off",
        )
        .await
        .wrap_err("forcing ordered ANN plan shape")?;
    Ok(())
}

/// Does an index with the given `pg_class.reloptions` prefix exist? The
/// caller passes the canonical `key=value` list; a match means every
/// listed reloption is present (via array containment), so other
/// reloptions we don't care about don't cause false negatives.
pub async fn index_exists_with_reloptions(
    client: &Client,
    index: &str,
    reloptions: &[(String, String)],
) -> Result<bool> {
    let normalized = crate::reloptions::normalize_list(reloptions);
    if normalized.is_empty() {
        let row = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM pg_class WHERE relname = $1 AND relkind = 'i')",
                &[&index],
            )
            .await?;
        return Ok(row.get::<_, bool>(0));
    }
    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM pg_class
                WHERE relname = $1 AND relkind = 'i'
                AND reloptions @> $2::text[]
            )",
            &[&index, &normalized],
        )
        .await?;
    Ok(row.get::<_, bool>(0))
}

/// Build a `CREATE INDEX <name> ON <table> USING <am> (embedding <opclass>) WITH (...)`
/// statement for the given profile and reloption list. The profile controls
/// the access method and operator class; `reloptions` is passed through
/// verbatim (already parsed into key/value pairs).
pub fn build_create_index_sql(
    corpus_table: &str,
    index_name: &str,
    profile: &crate::profiles::IndexProfile,
    reloptions: &[(String, String)],
) -> String {
    let with_clause = crate::reloptions::format_with_clause(reloptions);
    format!(
        "CREATE INDEX {index_name} ON {corpus_table}\n        USING {am} (embedding {opclass}){with_clause}",
        am = profile.access_method,
        opclass = profile.operator_class,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW};

    #[test]
    fn connection_options_config_sets_explicit_overrides() {
        let options = ConnectionOptions {
            database: "bench".into(),
            host: Some("/home/peter/.pgrx".into()),
            port: Some(28818),
            user: Some("peter".into()),
            password: Some("secret".into()),
        };
        let config = options.config();
        assert_eq!(config.get_dbname(), Some("bench"));
        assert_eq!(config.get_hosts().len(), 1);
        assert_eq!(config.get_ports(), &[28818]);
        assert_eq!(config.get_user(), Some("peter"));
        assert_eq!(config.get_password(), Some(&b"secret"[..]));
    }

    #[test]
    fn relkind_ascii_codes_encode_to_postgres_char() {
        assert_eq!(encode_relkind('r').unwrap(), b'r' as i8);
        assert_eq!(encode_relkind('i').unwrap(), b'i' as i8);
    }

    #[test]
    fn relkind_rejects_non_ascii_catalog_codes() {
        let err = encode_relkind('λ').unwrap_err().to_string();
        assert!(err.contains("must be an ASCII catalog code"), "got: {err}");
    }

    #[test]
    fn hnsw_index_sql_matches_legacy_loader_shape() {
        let opts = vec![
            ("m".into(), "8".into()),
            ("ef_construction".into(), "128".into()),
            ("build_source_column".into(), "source".into()),
        ];
        let sql =
            build_create_index_sql("dbpedia_10k_corpus", "dbpedia_10k_m8_idx", &EC_HNSW, &opts);
        assert!(sql.contains("USING ec_hnsw (embedding ecvector_ip_ops)"));
        assert!(sql.contains("m = 8"));
        assert!(sql.contains("build_source_column = 'source'"));
        assert!(!sql.contains("storage_format"));
    }

    #[test]
    fn diskann_index_sql_uses_diskann_opclass_and_no_default_reloptions() {
        let sql = build_create_index_sql("dbpedia_10k_corpus", "dbpedia_10k_idx", &EC_DISKANN, &[]);
        assert!(sql.contains("USING ec_diskann (embedding ecvector_diskann_ip_ops)"));
        assert!(!sql.contains("WITH ("));
    }

    #[test]
    fn build_index_sql_with_empty_reloptions_omits_with_clause() {
        let sql = build_create_index_sql("t", "idx", &EC_HNSW, &[]);
        assert!(!sql.contains("WITH"));
        assert!(sql.contains("USING ec_hnsw (embedding ecvector_ip_ops)"));
    }

    #[test]
    fn build_index_sql_renders_multiple_reloptions_in_order() {
        let opts = vec![
            ("m".into(), "8".into()),
            ("ef_construction".into(), "128".into()),
        ];
        let sql = build_create_index_sql("t", "idx", &EC_HNSW, &opts);
        let m_pos = sql.find("m = 8").expect("m missing");
        let ef_pos = sql.find("ef_construction = 128").expect("ef missing");
        assert!(m_pos < ef_pos, "reloption order not preserved: {sql}");
    }

    #[test]
    fn diskann_index_sql_quotes_strings_but_not_numerics() {
        let opts = vec![
            ("graph_degree".into(), "48".into()),
            ("alpha".into(), "1.2".into()),
            ("storage_format".into(), "pq_fastscan".into()),
        ];
        let sql =
            build_create_index_sql("dbpedia_10k_corpus", "dbpedia_10k_idx", &EC_DISKANN, &opts);
        assert!(sql.contains("graph_degree = 48"));
        assert!(sql.contains("alpha = 1.2"));
        assert!(sql.contains("storage_format = 'pq_fastscan'"));
    }
}
