//! `ecaz corpus list` — enumerate loaded corpora in the configured database.
//!
//! A "loaded corpus" is any table named `<prefix>_corpus`. We join against
//! a sibling `<prefix>_queries` when present so the output tells you at a
//! glance which corpora are also wired for benchmarking and which
//! profile-backed index families already exist on each corpus.

use color_eyre::eyre::{Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};

use crate::profiles;
use crate::psql::{self, ConnectionOptions};

pub async fn run(conn: &ConnectionOptions) -> Result<()> {
    let client = psql::connect(conn).await?;
    let rows = client
        .query(
            "WITH corpora AS (
                 SELECT substring(relname from '^(.*)_corpus$') AS prefix, oid
                 FROM pg_class
                 WHERE relkind = 'r' AND relname LIKE '%\\_corpus' ESCAPE '\\'
             )
             SELECT c.prefix,
                    (SELECT reltuples::bigint FROM pg_class WHERE oid = c.oid) AS est_rows,
                    EXISTS (SELECT 1 FROM pg_class q
                            WHERE q.relkind = 'r' AND q.relname = c.prefix || '_queries')
                        AS has_queries,
                    COUNT(ix.indexrelid)::bigint AS n_indexes,
                    COALESCE(
                        array_agg(DISTINCT am.amname ORDER BY am.amname)
                            FILTER (WHERE am.amname IS NOT NULL),
                        '{}'::text[]
                    ) AS access_methods
             FROM corpora c
             LEFT JOIN pg_index ix ON ix.indrelid = c.oid
             LEFT JOIN pg_class i ON i.oid = ix.indexrelid
             LEFT JOIN pg_am am ON am.oid = i.relam
             GROUP BY c.prefix, c.oid
             ORDER BY c.prefix",
            &[],
        )
        .await
        .wrap_err("listing corpora")?;

    if rows.is_empty() {
        println!("(no corpora loaded in {})", conn.database);
        return Ok(());
    }

    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec![
        "prefix",
        "rows (est.)",
        "queries?",
        "indexes",
        "access methods",
        "profiles",
    ]);
    for r in rows {
        let prefix: String = r.get(0);
        let est_rows: i64 = r.get(1);
        let has_queries: bool = r.get(2);
        let n_indexes: i64 = r.get(3);
        let access_methods: Vec<String> = r.get(4);
        let profile_names = profile_names_for_access_methods(&access_methods);
        t.add_row(vec![
            Cell::new(prefix),
            Cell::new(est_rows),
            Cell::new(if has_queries { "yes" } else { "no" }),
            Cell::new(n_indexes),
            Cell::new(format_name_list(&access_methods)),
            Cell::new(format_name_list(&profile_names)),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn profile_names_for_access_methods(access_methods: &[String]) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = access_methods
        .iter()
        .filter_map(|am| profiles::resolve_by_access_method(am).map(|p| p.name))
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

fn format_name_list<T: AsRef<str>>(names: &[T]) -> String {
    if names.is_empty() {
        "<none>".to_owned()
    } else {
        names
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::{format_name_list, profile_names_for_access_methods};

    #[test]
    fn profile_names_for_access_methods_sorts_and_dedups_matches() {
        let access_methods = vec![
            "ec_hnsw".to_owned(),
            "ec_diskann".to_owned(),
            "ec_hnsw".to_owned(),
        ];
        assert_eq!(
            profile_names_for_access_methods(&access_methods),
            vec!["ec_diskann", "ec_hnsw"]
        );
    }

    #[test]
    fn profile_names_for_access_methods_ignores_unknown_access_methods() {
        let access_methods = vec!["btree".to_owned(), "ec_diskann".to_owned()];
        assert_eq!(
            profile_names_for_access_methods(&access_methods),
            vec!["ec_diskann"]
        );
    }

    #[test]
    fn format_name_list_empty_is_none() {
        let names: Vec<String> = Vec::new();
        assert_eq!(format_name_list(&names), "<none>");
    }

    #[test]
    fn format_name_list_joins_names_in_order() {
        let names = vec!["ec_diskann".to_owned(), "ec_hnsw".to_owned()];
        assert_eq!(format_name_list(&names), "ec_diskann, ec_hnsw");
    }
}
