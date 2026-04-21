//! `ecaz bench storage` — on-disk accounting for a loaded corpus.
//!
//! Reports:
//! - corpus table: heap, toast, indexes, total
//! - per-vector-row bytes (total / rows) so different profiles are
//!   directly comparable across corpora of different sizes
//! - per-index size with access method and reloptions so a sweep across
//!   `corpus load ... --m 8,16,32` produces a readable breakdown
//!
//! # Purity boundary
//!
//! `per_row_bytes` and `format_bytes` are pure. Everything else is a
//! straight pg_class / pg_relation_size query.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};

use crate::profiles;
use crate::psql;

#[derive(Args, Debug)]
pub struct StorageArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
}

pub async fn run(database: &str, args: StorageArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    let corpus_table = format!("{}_corpus", args.prefix);

    let client = psql::connect(database).await?;
    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!("no corpus table {:?} in this database", corpus_table));
    }
    let rows = psql::row_count(&client, &corpus_table).await? as u64;

    // Table-level accounting. `pg_table_size` includes heap + toast +
    // visibility map + free-space map. `pg_indexes_size` is the sum of
    // every index on the relation. Together they equal `pg_total_relation_size`.
    let table_row = client
        .query_one(
            "SELECT pg_table_size(oid), pg_indexes_size(oid),
                    pg_total_relation_size(oid), pg_relation_size(oid, 'main')
             FROM pg_class WHERE relname = $1 AND relkind = 'r'",
            &[&corpus_table],
        )
        .await
        .wrap_err("reading table size")?;
    let table_size: i64 = table_row.get(0);
    let indexes_size: i64 = table_row.get(1);
    let total_size: i64 = table_row.get(2);
    let heap_size: i64 = table_row.get(3);

    let mut header = Table::new();
    header.load_preset(UTF8_FULL);
    header.set_header(vec!["field", "value"]);
    header.add_row(vec!["prefix".into(), Cell::new(&args.prefix)]);
    header.add_row(vec!["corpus".into(), Cell::new(&corpus_table)]);
    header.add_row(vec!["rows".into(), Cell::new(rows)]);
    header.add_row(vec!["heap".into(), Cell::new(format_bytes(heap_size))]);
    header.add_row(vec![
        "table (heap + toast + fsm/vm)".into(),
        Cell::new(format_bytes(table_size)),
    ]);
    header.add_row(vec![
        "indexes".into(),
        Cell::new(format_bytes(indexes_size)),
    ]);
    header.add_row(vec!["total".into(), Cell::new(format_bytes(total_size))]);
    header.add_row(vec![
        "per row (total)".into(),
        Cell::new(format!("{:.1} B", per_row_bytes(total_size, rows))),
    ]);
    header.add_row(vec![
        "per row (heap only)".into(),
        Cell::new(format!("{:.1} B", per_row_bytes(heap_size, rows))),
    ]);
    println!("{header}");

    // Per-index breakdown.
    let index_rows = client
        .query(
            "SELECT i.relname, am.amname,
                    COALESCE(i.reloptions, '{}')::text,
                    pg_relation_size(i.oid)
             FROM pg_class t
             JOIN pg_index ix ON ix.indrelid = t.oid
             JOIN pg_class i  ON i.oid = ix.indexrelid
             JOIN pg_am    am ON am.oid = i.relam
             WHERE t.relname = $1
             ORDER BY pg_relation_size(i.oid) DESC, i.relname",
            &[&corpus_table],
        )
        .await
        .wrap_err("listing indexes")?;

    let mut idx = Table::new();
    idx.load_preset(UTF8_FULL);
    idx.set_header(vec![
        "index",
        "access method",
        "profile",
        "reloptions",
        "size",
        "per row",
    ]);
    if index_rows.is_empty() {
        idx.add_row(vec![
            Cell::new("<none>"),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
        ]);
    } else {
        for r in index_rows {
            let name: String = r.get(0);
            let am: String = r.get(1);
            let opts: String = r.get(2);
            let size: i64 = r.get(3);
            idx.add_row(vec![
                Cell::new(name),
                Cell::new(&am),
                Cell::new(profile_label_for_access_method(&am)),
                Cell::new(opts),
                Cell::new(format_bytes(size)),
                Cell::new(format!("{:.1} B", per_row_bytes(size, rows))),
            ]);
        }
    }
    println!("{idx}");
    Ok(())
}

/// Bytes per row, safe on zero-row corpora (returns 0.0 rather than NaN).
/// Kept in f64 so a 1k-row × 10 MiB index reports "10240.0 B" with
/// precision, not a rounded integer.
pub fn per_row_bytes(size: i64, rows: u64) -> f64 {
    if rows == 0 || size <= 0 {
        return 0.0;
    }
    size as f64 / rows as f64
}

/// Human-readable byte size. Mirrors `corpus::inspect::format_bytes` and
/// is duplicated intentionally so each command owns its rendering rules.
pub fn format_bytes(n: i64) -> String {
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
        format!("{n} B")
    } else {
        format!("{v:.1} {}", UNITS[u])
    }
}

fn profile_label_for_access_method(access_method: &str) -> &'static str {
    profiles::resolve_by_access_method(access_method)
        .map(|p| p.name)
        .unwrap_or("<unknown>")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- per_row_bytes ---

    #[test]
    fn per_row_bytes_computes_floating_ratio() {
        assert!((per_row_bytes(10_240, 1_000) - 10.24).abs() < 1e-9);
        assert!((per_row_bytes(1_536, 1) - 1_536.0).abs() < 1e-9);
    }

    #[test]
    fn per_row_bytes_zero_rows_is_zero_not_nan() {
        let got = per_row_bytes(123_456, 0);
        assert!(got.is_finite() && got == 0.0, "got {got}");
    }

    #[test]
    fn per_row_bytes_zero_or_negative_size_is_zero() {
        assert_eq!(per_row_bytes(0, 100), 0.0);
        assert_eq!(per_row_bytes(-1, 100), 0.0);
    }

    #[test]
    fn per_row_bytes_handles_billion_row_corpus_without_overflow() {
        // 1 TiB over 1 billion rows ≈ 1099.51 bytes/row. Guard the i64→f64
        // conversion doesn't silently lose precision on this magnitude.
        let tib = 1024_i64 * 1024 * 1024 * 1024;
        let got = per_row_bytes(tib, 1_000_000_000);
        assert!((got - 1_099.511_627_776).abs() < 1e-3, "got {got}");
    }

    // --- format_bytes ---

    #[test]
    fn format_bytes_scales_units_and_promotes_at_1024() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MiB");
    }

    #[test]
    fn format_bytes_negative_falls_back_to_raw() {
        assert_eq!(format_bytes(-1), "-1");
    }

    #[test]
    fn profile_label_for_access_method_maps_known_profiles() {
        assert_eq!(profile_label_for_access_method("ec_hnsw"), "ec_hnsw");
        assert_eq!(profile_label_for_access_method("ec_diskann"), "ec_diskann");
    }

    #[test]
    fn profile_label_for_access_method_marks_unknown_access_methods() {
        assert_eq!(profile_label_for_access_method("btree"), "<unknown>");
    }
}
