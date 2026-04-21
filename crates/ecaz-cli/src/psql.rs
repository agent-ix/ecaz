//! Postgres plumbing for the CLI.
//!
//! v1 goal: a thin `connect` helper over `tokio-postgres` plus a small set
//! of pure-SQL builders that live on top of `profiles` + `reloptions`.
//! Everything that issues DDL/DML lives on top of `connect`; nothing else
//! in the crate should shell out to `psql`.
//!
//! Connection defaults come from libpq environment variables (`PGHOST`,
//! `PGPORT`, `PGUSER`, …). The `database` argument is passed explicitly so
//! commands can be pointed at a fixture DB without mutating environment
//! state.

use color_eyre::eyre::{Context, Result};
use tokio_postgres::{Client, NoTls};

/// Open a connection to the named database using libpq-style environment
/// variables for everything else (host, port, user, password, ssl mode).
pub async fn connect(database: &str) -> Result<Client> {
    let mut config = tokio_postgres::Config::new();
    config.dbname(database);
    if let Ok(host) = std::env::var("PGHOST") {
        config.host(&host);
    }
    if let Ok(port) = std::env::var("PGPORT") {
        if let Ok(p) = port.parse() {
            config.port(p);
        }
    }
    if let Ok(user) = std::env::var("PGUSER") {
        config.user(&user);
    }
    if let Ok(password) = std::env::var("PGPASSWORD") {
        config.password(&password);
    }

    let (client, connection) = config
        .connect(NoTls)
        .await
        .wrap_err_with(|| format!("connecting to Postgres database {database:?}"))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(error = %e, "postgres connection task failed");
        }
    });

    Ok(client)
}

/// Does a relation with the given name and `relkind` exist?
pub async fn relation_exists(
    client: &Client,
    name: &str,
    relkind: char,
) -> Result<bool> {
    let row = client
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM pg_class WHERE relname = $1 AND relkind = $2)",
            &[&name, &(relkind.to_string())],
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
    fn hnsw_index_sql_matches_legacy_loader_shape() {
        let opts = vec![
            ("m".into(), "8".into()),
            ("ef_construction".into(), "128".into()),
            ("build_source_column".into(), "source".into()),
        ];
        let sql = build_create_index_sql(
            "dbpedia_10k_corpus",
            "dbpedia_10k_m8_idx",
            &EC_HNSW,
            &opts,
        );
        assert!(sql.contains("USING ec_hnsw (embedding ecvector_ip_ops)"));
        assert!(sql.contains("m = 8"));
        assert!(sql.contains("build_source_column = 'source'"));
        assert!(!sql.contains("storage_format"));
    }

    #[test]
    fn diskann_index_sql_uses_diskann_opclass_and_no_default_reloptions() {
        let sql =
            build_create_index_sql("dbpedia_10k_corpus", "dbpedia_10k_idx", &EC_DISKANN, &[]);
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
