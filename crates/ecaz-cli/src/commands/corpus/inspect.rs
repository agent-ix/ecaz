//! `ecaz corpus inspect` — show row counts, embedding column type, and
//! indexes built on a loaded corpus.
//!
//! Thin read-only helper on top of `pg_class` + `information_schema`.
//! Unlike `load`, this command never issues DDL/DML; safe to run against
//! a live benchmark database.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};

use crate::profiles;
use crate::psql;

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
}

pub async fn run(database: &str, args: InspectArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);

    let client = psql::connect(database).await?;
    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!("no corpus table {:?} in this database", corpus_table));
    }

    let corpus_rows = psql::row_count(&client, &corpus_table).await?;
    let queries_rows = if psql::relation_exists(&client, &queries_table, 'r').await? {
        psql::row_count(&client, &queries_table).await?
    } else {
        -1
    };
    let embedding_type = client
        .query_one(
            "SELECT format_type(a.atttypid, a.atttypmod)
             FROM pg_attribute a
             JOIN pg_class c ON c.oid = a.attrelid
             WHERE c.relname = $1 AND a.attname = 'embedding' AND NOT a.attisdropped",
            &[&corpus_table],
        )
        .await
        .ok()
        .map(|r| r.get::<_, String>(0))
        .unwrap_or_else(|| "<missing>".to_owned());

    let mut header = Table::new();
    header.load_preset(UTF8_FULL);
    header.set_header(vec!["field", "value"]);
    header.add_row(vec!["prefix".into(), Cell::new(&args.prefix)]);
    header.add_row(vec![
        "corpus".into(),
        Cell::new(format!("{corpus_table} ({corpus_rows} rows)")),
    ]);
    header.add_row(vec![
        "queries".into(),
        Cell::new(if queries_rows < 0 {
            format!("{queries_table} <missing>")
        } else {
            format!("{queries_table} ({queries_rows} rows)")
        }),
    ]);
    header.add_row(vec!["embedding type".into(), Cell::new(embedding_type)]);
    println!("{header}");

    // Indexes on <prefix>_corpus. Pull AM + reloptions in one shot so the
    // output matches what `corpus load` thinks is present.
    let rows = client
        .query(
            "SELECT i.relname, am.amname,
                    COALESCE(i.reloptions, '{}')::text,
                    pg_relation_size(i.oid)
             FROM pg_class t
             JOIN pg_index ix ON ix.indrelid = t.oid
             JOIN pg_class i  ON i.oid = ix.indexrelid
             JOIN pg_am    am ON am.oid = i.relam
             WHERE t.relname = $1
             ORDER BY i.relname",
            &[&corpus_table],
        )
        .await
        .wrap_err("listing indexes")?;

    let mut idx = Table::new();
    idx.load_preset(UTF8_FULL);
    idx.set_header(vec!["index", "access method", "reloptions", "size"]);
    if rows.is_empty() {
        idx.add_row(vec![
            Cell::new("<none>"),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
        ]);
    } else {
        for r in rows {
            let name: String = r.get(0);
            let am: String = r.get(1);
            let opts: String = r.get(2);
            let size: i64 = r.get(3);
            idx.add_row(vec![
                Cell::new(name),
                Cell::new(am),
                Cell::new(opts),
                Cell::new(format_bytes(size)),
            ]);
        }
    }
    println!("{idx}");
    Ok(())
}

fn format_bytes(n: i64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    if n < 0 {
        return format!("{n}");
    }
    let mut v = n as f64;
    let mut u = 0;
    while v >= 1024.0 && u + 1 < UNITS.len() {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{n} {}", UNITS[0])
    } else {
        format!("{v:.1} {}", UNITS[u])
    }
}

#[cfg(test)]
mod tests {
    use super::format_bytes;

    #[test]
    fn format_bytes_scales_through_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KiB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MiB");
        assert_eq!(format_bytes(3 * 1024 * 1024 * 1024), "3.0 GiB");
    }

    #[test]
    fn format_bytes_handles_boundary_at_1024() {
        // Just below: still bytes. At/above: promote unit so the number
        // never displays as "1024.0 KiB".
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
    }

    #[test]
    fn format_bytes_falls_back_to_raw_for_negative() {
        // pg_relation_size is i64 — a bogus negative should round-trip so a
        // reviewer sees something unambiguously wrong rather than "-1.0 B".
        assert_eq!(format_bytes(-1), "-1");
    }

    #[test]
    fn format_bytes_caps_at_largest_unit() {
        // Numbers beyond TiB reuse the last unit instead of overflowing the
        // array; 2 PiB shows as "2048.0 TiB".
        let two_pib = 2_i64 * 1024 * 1024 * 1024 * 1024 * 1024;
        assert_eq!(format_bytes(two_pib), "2048.0 TiB");
    }
}
