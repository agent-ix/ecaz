//! `ecaz bench diskann-graph` — persisted graph-shape diagnostics.
//!
//! This command calls the extension's read-only
//! `ec_diskann_index_graph_summary()` function through the normal CLI
//! database connection. It exists so DiskANN tuning packets do not need
//! ad-hoc SQL or direct `psql` invocations for graph-quality claims.

use std::{fmt::Write as _, path::PathBuf};

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};

use crate::{
    profiles,
    psql::{self, ConnectionOptions},
};

type SummaryRows = Vec<(String, String)>;

#[derive(Args, Debug)]
pub struct GraphArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
    /// Optional DiskANN index name. Defaults to the sole ec_diskann index on
    /// `<prefix>_corpus`.
    #[arg(long)]
    pub index: Option<String>,
    /// Write the rendered diagnostics to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

pub async fn run(conn: &ConnectionOptions, args: GraphArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if let Some(index) = &args.index {
        profiles::validate_ident(index).wrap_err_with(|| format!("invalid index {:?}", index))?;
    }
    let corpus_table = format!("{}_corpus", args.prefix);

    let client = psql::connect(conn).await?;
    if !psql::relation_exists(&client, &corpus_table, 'r').await? {
        return Err(eyre!("no corpus table {:?} in this database", corpus_table));
    }

    let index_name = match args.index {
        Some(index) => {
            ensure_diskann_index(&client, &corpus_table, &index).await?;
            index
        }
        None => select_diskann_index(&client, &corpus_table).await?,
    };
    let summary = fetch_summary(&client, &index_name).await?;
    let rendered = render_summary(&args.prefix, &corpus_table, &index_name, &summary);
    crate::ecaz_println!("{rendered}");
    if let Some(path) = args.log_output {
        std::fs::write(&path, &rendered)
            .wrap_err_with(|| format!("writing graph diagnostics to {}", path.display()))?;
    }
    Ok(())
}

async fn select_diskann_index(
    client: &tokio_postgres::Client,
    corpus_table: &str,
) -> Result<String> {
    let rows = client
        .query(
            "SELECT i.relname
             FROM pg_class t
             JOIN pg_index ix ON ix.indrelid = t.oid
             JOIN pg_class i ON i.oid = ix.indexrelid
             JOIN pg_am am ON am.oid = i.relam
             WHERE t.relname = $1
               AND am.amname = 'ec_diskann'
             ORDER BY i.relname",
            &[&corpus_table],
        )
        .await
        .wrap_err_with(|| format!("listing ec_diskann indexes on {corpus_table:?}"))?;
    match rows.as_slice() {
        [] => Err(eyre!(
            "{} on {:?}",
            super::missing_am_error(&profiles::EC_DISKANN, "ec_diskann"),
            corpus_table
        )),
        [row] => Ok(row.get(0)),
        _ => {
            let names = rows
                .iter()
                .map(|row| row.get::<_, String>(0))
                .collect::<Vec<_>>()
                .join(", ");
            Err(eyre!(
                "multiple ec_diskann indexes on {corpus_table:?}; pass --index (found {names})"
            ))
        }
    }
}

async fn ensure_diskann_index(
    client: &tokio_postgres::Client,
    corpus_table: &str,
    index_name: &str,
) -> Result<()> {
    let row = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_class t
                JOIN pg_index ix ON ix.indrelid = t.oid
                JOIN pg_class i ON i.oid = ix.indexrelid
                JOIN pg_am am ON am.oid = i.relam
                WHERE t.relname = $1
                  AND i.relname = $2
                  AND am.amname = 'ec_diskann'
             )",
            &[&corpus_table, &index_name],
        )
        .await
        .wrap_err_with(|| format!("checking ec_diskann index {index_name:?}"))?;
    if !row.get::<_, bool>(0) {
        return Err(eyre!(
            "{index_name:?} is not an ec_diskann index on {corpus_table:?}"
        ));
    }
    Ok(())
}

async fn fetch_summary(client: &tokio_postgres::Client, index_name: &str) -> Result<SummaryRows> {
    let rows = client
        .query(
            "SELECT metric, value
             FROM ec_diskann_index_graph_summary($1::regclass::oid)",
            &[&index_name],
        )
        .await
        .wrap_err_with(|| format!("reading ec_diskann graph summary for {index_name:?}"))?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("metric"), row.get("value")))
        .collect())
}

fn render_summary(
    prefix: &str,
    corpus_table: &str,
    index_name: &str,
    summary: &SummaryRows,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "DiskANN graph diagnostics for prefix={prefix} corpus={corpus_table} index={index_name}"
    )
    .expect("writing to String should not fail");

    let mut header = Table::new();
    header.load_preset(UTF8_FULL);
    header.set_header(vec!["field", "value"]);
    add_rows(&mut header, summary, METADATA_KEYS);
    writeln!(out, "{header}").expect("writing to String should not fail");

    let mut topology = Table::new();
    topology.load_preset(UTF8_FULL);
    topology.set_header(vec!["field", "value"]);
    add_rows(&mut topology, summary, TOPOLOGY_KEYS);
    writeln!(out, "{topology}").expect("writing to String should not fail");

    let mut edges = Table::new();
    edges.load_preset(UTF8_FULL);
    edges.set_header(vec!["field", "value"]);
    add_rows(&mut edges, summary, EDGE_KEYS);
    writeln!(out, "{edges}").expect("writing to String should not fail");

    let mut degrees = Table::new();
    degrees.load_preset(UTF8_FULL);
    degrees.set_header(vec![
        "direction",
        "zero",
        "min",
        "avg",
        "p50",
        "p95",
        "p99",
        "max",
    ]);
    degrees.add_row(vec![
        Cell::new("out"),
        Cell::new(value(summary, "zero_out_degree_count")),
        Cell::new(value(summary, "min_out_degree")),
        Cell::new(value(summary, "avg_out_degree")),
        Cell::new(value(summary, "p50_out_degree")),
        Cell::new(value(summary, "p95_out_degree")),
        Cell::new(value(summary, "p99_out_degree")),
        Cell::new(value(summary, "max_out_degree")),
    ]);
    degrees.add_row(vec![
        Cell::new("in"),
        Cell::new(value(summary, "zero_in_degree_count")),
        Cell::new(value(summary, "min_in_degree")),
        Cell::new(value(summary, "avg_in_degree")),
        Cell::new(value(summary, "p50_in_degree")),
        Cell::new(value(summary, "p95_in_degree")),
        Cell::new(value(summary, "p99_in_degree")),
        Cell::new(value(summary, "max_in_degree")),
    ]);
    write!(out, "{degrees}").expect("writing to String should not fail");
    out
}

const METADATA_KEYS: &[&str] = &[
    "block_count",
    "dimensions",
    "graph_degree_r",
    "build_list_size_l",
    "alpha",
    "inserted_since_rebuild",
    "needs_medoid_refresh",
];

const TOPOLOGY_KEYS: &[&str] = &[
    "node_count",
    "live_node_count",
    "non_live_node_count",
    "entry_point_live",
    "reachable_live_node_count",
    "unreachable_live_node_count",
    "reachable_live_fraction",
];

const EDGE_KEYS: &[&str] = &[
    "neighbor_ref_count",
    "live_neighbor_ref_count",
    "dead_neighbor_ref_count",
    "invalid_neighbor_ref_count",
    "self_neighbor_ref_count",
    "duplicate_neighbor_ref_count",
    "unresolvable_neighbor_ref_count",
];

fn add_rows(table: &mut Table, summary: &SummaryRows, keys: &[&str]) {
    for key in keys {
        table.add_row(vec![Cell::new(*key), Cell::new(value(summary, key))]);
    }
}

fn value(summary: &SummaryRows, key: &str) -> String {
    summary
        .iter()
        .find_map(|(metric, value)| (metric == key).then(|| value.clone()))
        .unwrap_or_else(|| "<missing>".to_owned())
}

#[cfg(test)]
mod tests {
    use super::render_summary;

    #[test]
    fn render_summary_includes_reachability_and_degree_rows() {
        let summary = vec![
            ("block_count".to_owned(), "10".to_owned()),
            ("dimensions".to_owned(), "1536".to_owned()),
            ("graph_degree_r".to_owned(), "32".to_owned()),
            ("build_list_size_l".to_owned(), "100".to_owned()),
            ("alpha".to_owned(), "1.200000".to_owned()),
            ("inserted_since_rebuild".to_owned(), "0".to_owned()),
            ("needs_medoid_refresh".to_owned(), "false".to_owned()),
            ("node_count".to_owned(), "100".to_owned()),
            ("live_node_count".to_owned(), "100".to_owned()),
            ("non_live_node_count".to_owned(), "0".to_owned()),
            ("entry_point_live".to_owned(), "true".to_owned()),
            ("reachable_live_node_count".to_owned(), "99".to_owned()),
            ("unreachable_live_node_count".to_owned(), "1".to_owned()),
            ("reachable_live_fraction".to_owned(), "0.990000".to_owned()),
            ("neighbor_ref_count".to_owned(), "3200".to_owned()),
            ("live_neighbor_ref_count".to_owned(), "3200".to_owned()),
            ("dead_neighbor_ref_count".to_owned(), "0".to_owned()),
            ("invalid_neighbor_ref_count".to_owned(), "0".to_owned()),
            ("self_neighbor_ref_count".to_owned(), "0".to_owned()),
            ("duplicate_neighbor_ref_count".to_owned(), "0".to_owned()),
            ("unresolvable_neighbor_ref_count".to_owned(), "0".to_owned()),
            ("zero_out_degree_count".to_owned(), "0".to_owned()),
            ("min_out_degree".to_owned(), "24".to_owned()),
            ("avg_out_degree".to_owned(), "31.500000".to_owned()),
            ("p50_out_degree".to_owned(), "32".to_owned()),
            ("p95_out_degree".to_owned(), "32".to_owned()),
            ("p99_out_degree".to_owned(), "32".to_owned()),
            ("max_out_degree".to_owned(), "32".to_owned()),
            ("zero_in_degree_count".to_owned(), "2".to_owned()),
            ("min_in_degree".to_owned(), "0".to_owned()),
            ("avg_in_degree".to_owned(), "31.500000".to_owned()),
            ("p50_in_degree".to_owned(), "30".to_owned()),
            ("p95_in_degree".to_owned(), "64".to_owned()),
            ("p99_in_degree".to_owned(), "80".to_owned()),
            ("max_in_degree".to_owned(), "96".to_owned()),
        ];
        let rendered = render_summary("p", "p_corpus", "p_idx", &summary);
        assert!(rendered.contains("reachable_live_fraction"));
        assert!(rendered.contains("0.990000"));
        assert!(rendered.contains("neighbor_ref_count"));
        assert!(rendered.contains("direction"));
    }
}
