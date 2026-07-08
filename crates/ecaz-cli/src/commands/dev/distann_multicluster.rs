//! `ec_distann` real multi-instance fixture (Task 165 M3, Slice A).
//!
//! Spins up N real PostgreSQL 18 instances, loads an **identical, deterministic**
//! corpus in identical order into a fresh table on each (so local-mode `vec_id`s
//! — hashed from the heap TID — and the seed-deterministic global graph are
//! byte-identical across nodes), builds an `ec_distann` index on each, wires the
//! coordinator's roster to all N nodes, and drives a real cross-process
//! distinct-recall comparison plus a fail-closed transport drill.
//!
//! Distribution model (honest): the index is *replicated* and the roster
//! partitions **ownership of serving** (each node answers `expand` /
//! `materialize_row_payloads` only for its owned vec_ids). This exercises the
//! genuine cross-instance read path — remote-owned hits are shipped from another
//! process and reconstructed by the coordinator's CustomScan — with correct
//! (single-node-equal) recall. True disjoint-shard storage needs a
//! build-global-then-distribute step (a follow-up); it is not required to prove
//! the multi-node read gate.

use clap::{Args, Subcommand};
use color_eyre::eyre::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use super::support::{find_pgrx_install, repo_root, resolve_pgrx_home, run_status};

#[derive(Subcommand, Debug)]
pub enum DistannMulticlusterCommand {
    /// Spin up N real PG18 instances, replicate a deterministic ec_distann
    /// corpus, wire the roster, and run the multi-node distinct-recall gate.
    #[command(name = "local-multinode-pg18")]
    LocalMultinodePg18(LocalMultinodePg18Args),
}

#[derive(Args, Debug)]
pub struct LocalMultinodePg18Args {
    #[arg(long, default_value_t = 18)]
    pub pg: u16,
    #[arg(long)]
    pub pgbin: Option<PathBuf>,
    #[arg(long)]
    pub pgrx_home: Option<PathBuf>,
    #[arg(long)]
    pub run_dir: Option<PathBuf>,
    #[arg(long)]
    pub artifact_dir: Option<PathBuf>,
    /// Number of real PG instances (node 1 is the coordinator).
    #[arg(long, default_value_t = 3)]
    pub nodes: u32,
    /// First TCP port; node k listens on base_port + (k - 1).
    #[arg(long, default_value_t = 39710)]
    pub base_port: u16,
    /// Deterministic corpus row count (per node; replicated).
    #[arg(long, default_value_t = 2000)]
    pub rows: u32,
    /// Vector dimension.
    #[arg(long, default_value_t = 16)]
    pub dim: u32,
    /// ec_distann graph degree reloption.
    #[arg(long, default_value_t = 32)]
    pub graph_degree: u32,
    /// Query count for the recall comparison.
    #[arg(long, default_value_t = 50)]
    pub queries: u32,
    /// top-k for the recall comparison.
    #[arg(long, default_value_t = 10)]
    pub top_k: u32,
    /// Keep the instances running after the run (for manual inspection).
    #[arg(long, default_value_t = false)]
    pub keep_running: bool,
}

impl DistannMulticlusterCommand {
    pub async fn run(&self) -> Result<()> {
        match self {
            DistannMulticlusterCommand::LocalMultinodePg18(args) => {
                run_local_multinode_pg18(args).await
            }
        }
    }
}

struct Node {
    node_id: u32,
    port: u16,
    data_dir: PathBuf,
    log_file: PathBuf,
}

async fn run_local_multinode_pg18(args: &LocalMultinodePg18Args) -> Result<()> {
    if args.pg != 18 {
        bail!("distann local-multinode requires --pg 18, got {}", args.pg);
    }
    if args.nodes < 2 {
        bail!("distann local-multinode needs at least 2 nodes, got {}", args.nodes);
    }
    let repo_root = repo_root()?;
    let pgbin = match args.pgbin.clone() {
        Some(path) => path,
        None => {
            let pgrx_home = resolve_pgrx_home(args.pgrx_home.as_ref());
            find_pgrx_install(args.pg, &pgrx_home)?.bin_dir
        }
    };
    let pg_ctl = pgbin.join("pg_ctl");
    let psql = pgbin.join("psql");

    let run_dir = args
        .run_dir
        .clone()
        .unwrap_or_else(|| repo_root.join("target/distann-local-multinode"));
    let socket_dir = run_dir.join("sockets");
    let log_dir = args.artifact_dir.clone().unwrap_or_else(|| run_dir.join("logs"));
    if run_dir.exists() {
        // Best-effort stop of a prior run before wiping.
        for k in 0..args.nodes {
            let data_dir = run_dir.join(format!("node{}", k + 1));
            let _ = Command::new(&pg_ctl)
                .arg("-D")
                .arg(&data_dir)
                .arg("-m")
                .arg("immediate")
                .arg("stop")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }
        fs::remove_dir_all(&run_dir).wrap_err_with(|| format!("clearing {}", run_dir.display()))?;
    }
    fs::create_dir_all(&socket_dir)?;
    fs::create_dir_all(&log_dir)?;

    let nodes: Vec<Node> = (0..args.nodes)
        .map(|k| Node {
            node_id: k + 1,
            port: args.base_port + k as u16,
            data_dir: run_dir.join(format!("node{}", k + 1)),
            log_file: log_dir.join(format!("node{}-postgres.log", k + 1)),
        })
        .collect();

    crate::ecaz_println!("[distann-multicluster] repo={}", repo_root.display());
    crate::ecaz_println!("[distann-multicluster] pgbin={}", pgbin.display());
    crate::ecaz_println!(
        "[distann-multicluster] nodes={} base_port={} rows={} dim={}",
        args.nodes,
        args.base_port,
        args.rows,
        args.dim
    );

    // initdb + start + extension on every node.
    for node in &nodes {
        let mut command = Command::new(&pg_ctl);
        command
            .arg("initdb")
            .arg("-D")
            .arg(&node.data_dir)
            .arg("-o")
            .arg("-A trust -U postgres")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(command)
            .await
            .wrap_err_with(|| format!("initdb node {}", node.node_id))?;
    }
    for node in &nodes {
        let mut command = Command::new(&pg_ctl);
        command
            .arg("-w")
            .arg("-D")
            .arg(&node.data_dir)
            .arg("-l")
            .arg(&node.log_file)
            .arg("-o")
            .arg(format!(
                "-p {} -k {} -c listen_addresses=''",
                node.port,
                socket_dir.display()
            ))
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(command)
            .await
            .wrap_err_with(|| format!("start node {}", node.node_id))?;
    }

    let result = drive_fixture(args, &psql, &socket_dir, &nodes, log_dir.as_path()).await;

    if !args.keep_running {
        for node in &nodes {
            let _ = Command::new(&pg_ctl)
                .arg("-D")
                .arg(&node.data_dir)
                .arg("-m")
                .arg("fast")
                .arg("stop")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }
    } else {
        crate::ecaz_println!(
            "[distann-multicluster] instances left running under {}",
            run_dir.display()
        );
    }
    result
}

/// libpq conninfo for a node over the shared socket dir.
fn conninfo(socket_dir: &Path, port: u16) -> String {
    format!(
        "host={} port={} dbname=postgres user=postgres",
        socket_dir.display(),
        port
    )
}

/// The identical, deterministic corpus + index setup run on every node.
fn setup_sql(args: &LocalMultinodePg18Args) -> String {
    format!(
        "CREATE EXTENSION IF NOT EXISTS ecaz;\n\
         DROP TABLE IF EXISTS dm;\n\
         CREATE TABLE dm (id bigint, source real[], embedding ecvector);\n\
         INSERT INTO dm\n\
         SELECT g,\n\
                arr,\n\
                encode_to_ecvector(arr, 4, 42)\n\
         FROM (\n\
           SELECT g,\n\
                  (SELECT array_agg((sin(g * 0.017 * (d + 1)) + cos(g * 0.0031 * (d + 1)))::real)\n\
                     FROM generate_series(0, {dim} - 1) AS d) AS arr\n\
           FROM generate_series(1, {rows}) AS g\n\
         ) s;\n\
         CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops)\n\
           WITH (graph_degree = {gd});\n",
        dim = args.dim,
        rows = args.rows,
        gd = args.graph_degree,
    )
}

/// The coordinator-side recall comparison: single-node (empty roster) vs the
/// full multi-node roster (CustomScan → owner row shipping), asserting the top-k
/// id sets are identical (distinct_recall delta 0 ⇒ ≥ single − 0.001).
fn recall_sql(roster: &str, queries: u32, top_k: u32) -> String {
    format!(
        "SET enable_seqscan = off;\n\
         DROP TABLE IF EXISTS q; CREATE TEMP TABLE q AS SELECT id AS qid, source AS v FROM dm WHERE id <= {queries};\n\
         SET ec_distann.roster = ''; SET ec_distann.local_node_id = 1; SET ec_distann.epoch = 0;\n\
         DROP TABLE IF EXISTS base; CREATE TEMP TABLE base AS\n\
           SELECT q.qid, r.id FROM q CROSS JOIN LATERAL\n\
             (SELECT id FROM dm ORDER BY embedding <#> q.v LIMIT {top_k}) r;\n\
         SET ec_distann.roster = '{roster}'; SET ec_distann.local_node_id = 1; SET ec_distann.epoch = 1;\n\
         DROP TABLE IF EXISTS two; CREATE TEMP TABLE two AS\n\
           SELECT q.qid, r.id FROM q CROSS JOIN LATERAL\n\
             (SELECT id FROM dm ORDER BY embedding <#> q.v LIMIT {top_k}) r;\n\
         SET ec_distann.roster = '';\n\
         SELECT 'RECALL_RESULT'\n\
           || ' n_queries=' || count(DISTINCT qid)\n\
           || ' identical=' || count(DISTINCT qid) FILTER (WHERE mismatch = 0)\n\
           || ' mismatched_ids=' || coalesce(sum(mismatch), 0)\n\
         FROM (\n\
           SELECT q.qid,\n\
             (SELECT count(*) FROM (SELECT id FROM base WHERE qid=q.qid EXCEPT SELECT id FROM two WHERE qid=q.qid) d)\n\
           + (SELECT count(*) FROM (SELECT id FROM two WHERE qid=q.qid EXCEPT SELECT id FROM base WHERE qid=q.qid) d) AS mismatch\n\
           FROM q\n\
         ) s;\n"
    )
}

async fn drive_fixture(
    args: &LocalMultinodePg18Args,
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    log_dir: &Path,
) -> Result<()> {
    // Replicated deterministic corpus + index on every node.
    let setup = setup_sql(args);
    for node in nodes {
        run_psql_file(psql, socket_dir, node.port, &setup)
            .await
            .wrap_err_with(|| format!("corpus/index setup on node {}", node.node_id))?;
        crate::ecaz_println!("[distann-multicluster] node {} loaded + indexed", node.node_id);
    }

    // Coordinator roster: every node by socket conninfo, in node-id order.
    let roster = nodes
        .iter()
        .map(|node| format!("{}@{}", node.node_id, conninfo(socket_dir, node.port)))
        .collect::<Vec<_>>()
        .join(";");

    // Distinct-recall gate on the coordinator (node 1).
    let coord_port = nodes[0].port;
    let recall = recall_sql(&roster, args.queries, args.top_k);
    let out = capture_psql(psql, socket_dir, coord_port, &recall)
        .await
        .wrap_err("running the multi-node recall comparison")?;
    let result_line = out
        .lines()
        .find(|line| line.contains("RECALL_RESULT"))
        .unwrap_or("RECALL_RESULT <none>")
        .trim()
        .to_owned();
    crate::ecaz_println!("[distann-multicluster] {result_line}");

    // Fail-closed transport drill: point one roster node at a dead port and
    // confirm the multi-node query errors (never a silent wrong/partial result).
    let dead_roster = nodes
        .iter()
        .map(|node| {
            let port = if node.node_id == nodes.last().unwrap().node_id {
                1
            } else {
                node.port
            };
            format!("{}@{}", node.node_id, conninfo(socket_dir, port))
        })
        .collect::<Vec<_>>()
        .join(";");
    let drill = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{dead_roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {};",
        args.top_k
    );
    let drill_out = capture_psql_allow_error(psql, socket_dir, coord_port, &drill).await;
    let fail_closed = drill_out.contains("EC_INTERNAL") || drill_out.contains("could not connect");
    crate::ecaz_println!(
        "[distann-multicluster] fault_drill dead_remote_port fail_closed={fail_closed}"
    );

    // Persist the evidence.
    let summary = format!(
        "distann-multinode fixture\nnodes={}\nrows={}\ndim={}\ngraph_degree={}\nqueries={}\ntop_k={}\nroster={}\n{}\nfault_drill dead_remote_port fail_closed={}\n",
        args.nodes, args.rows, args.dim, args.graph_degree, args.queries, args.top_k, roster,
        result_line, fail_closed
    );
    let summary_path = log_dir.join("distann-multinode-summary.log");
    fs::write(&summary_path, &summary)
        .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
    crate::ecaz_println!(
        "[distann-multicluster] summary written to {}",
        summary_path.display()
    );

    let mismatched_zero = result_line.contains("mismatched_ids=0");
    if !mismatched_zero {
        bail!("multi-node distinct-recall gate FAILED: {result_line}");
    }
    if !fail_closed {
        bail!("fault drill FAILED: dead remote port did not fail closed");
    }
    crate::ecaz_println!(
        "[distann-multicluster] GATE PASS: multi-node top-k identical to single-node; fault drill fail-closed"
    );
    Ok(())
}

fn psql_base(psql: &Path, socket_dir: &Path, port: u16) -> Command {
    let mut command = Command::new(psql);
    command
        .arg("-h")
        .arg(socket_dir)
        .arg("-p")
        .arg(port.to_string())
        .arg("-U")
        .arg("postgres")
        .arg("-d")
        .arg("postgres");
    command
}

async fn run_psql_file(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> Result<()> {
    let mut command = psql_base(psql, socket_dir, port);
    command
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-c")
        .arg(sql)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    run_status(command).await
}

async fn capture_psql(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> Result<String> {
    let mut command = psql_base(psql, socket_dir, port);
    command.arg("-v").arg("ON_ERROR_STOP=1").arg("-tAc").arg(sql);
    let output = command.output().await.wrap_err("spawning psql")?;
    if !output.status.success() {
        bail!(
            "psql failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn capture_psql_allow_error(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> String {
    let mut command = psql_base(psql, socket_dir, port);
    command.arg("-tAc").arg(sql);
    match command.output().await {
        Ok(output) => {
            let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
            combined.push_str(&String::from_utf8_lossy(&output.stderr));
            combined
        }
        Err(error) => format!("psql spawn error: {error}"),
    }
}
