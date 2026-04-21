//! `ecaz corpus list` — enumerate loaded corpora in the configured database.
//!
//! A "loaded corpus" is any table named `<prefix>_corpus`. We join against
//! a sibling `<prefix>_queries` when present so the output tells you at a
//! glance which corpora are also wired for benchmarking.

use color_eyre::eyre::{Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};

use crate::psql;

pub async fn run(database: &str) -> Result<()> {
    let client = psql::connect(database).await?;
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
                    (SELECT count(*) FROM pg_index ix WHERE ix.indrelid = c.oid) AS n_indexes
             FROM corpora c
             ORDER BY c.prefix",
            &[],
        )
        .await
        .wrap_err("listing corpora")?;

    if rows.is_empty() {
        println!("(no corpora loaded in {database})");
        return Ok(());
    }

    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec!["prefix", "rows (est.)", "queries?", "indexes"]);
    for r in rows {
        let prefix: String = r.get(0);
        let est_rows: i64 = r.get(1);
        let has_queries: bool = r.get(2);
        let n_indexes: i64 = r.get(3);
        t.add_row(vec![
            Cell::new(prefix),
            Cell::new(est_rows),
            Cell::new(if has_queries { "yes" } else { "no" }),
            Cell::new(n_indexes),
        ]);
    }
    println!("{t}");
    Ok(())
}
