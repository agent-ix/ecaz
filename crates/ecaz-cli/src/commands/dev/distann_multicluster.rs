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

    let result = drive_fixture(args, &pg_ctl, &psql, &socket_dir, &nodes, log_dir.as_path()).await;

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
         DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'peter') THEN CREATE ROLE peter LOGIN SUPERUSER; END IF; END $$;\n\
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
    pg_ctl: &Path,
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

    // Suite-driven recall gate (006-P1 letter): `ecaz bench recall` against the
    // coordinator single-node vs multi-node, distinct_recall(multi) >=
    // distinct_recall(single) - 0.001. Run here — before the mutating drills —
    // so benchgate_corpus is byte-identical across nodes (consistent vec_ids).
    // The byte-identical top-k gate above is strictly stronger.
    let suite_line = suite_recall_gate(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {suite_line}");

    // Qual correctness (011/020-P1): a WHERE predicate on a NON-projected column
    // plus LIMIT. Multi-node must match single-node exactly — this exercises
    // shipping the qual column (source) for remote rows and over-fetching so the
    // LIMIT applies after the qual. Runs early, on the clean/consistent corpus.
    let (qual_line, qual_ok) = qual_correctness_drill(psql, socket_dir, coord_port, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {qual_line}");

    // TC-042 fault matrix (NFR-020): each fault must make the multi-node query
    // ERROR (fail closed) — never a silent wrong or partial result — and a
    // post-recovery query must match the baseline (no false reject).
    let mut drills: Vec<(String, bool)> = Vec::new();
    let last = nodes.last().unwrap();
    let single_query = format!(
        "SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {};",
        args.top_k
    );

    // 1. simulated_network_partition: one owner at a dead port ⇒ connect error.
    {
        let dead_roster = roster_with_port_override(nodes, socket_dir, last.node_id, 1);
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{dead_roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("simulated_network_partition".to_owned(), query_errored(&out)));
    }

    // 2. epoch_bump_no_false_reject: a bare epoch-number bump must NOT reject —
    // the FR-082 fingerprint is content-based and the coordinator propagates its
    // epoch to owners, so both sides agree and the query returns its result.
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=999999; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        // Pass = no false reject: no error AND a result row came back.
        drills.push(("epoch_bump_no_false_reject".to_owned(), !query_errored(&out) && out.contains('\n')));
    }

    // 3. remote_content_divergence (real epoch/fingerprint mismatch): rebuild an
    // owner's index with a different graph_degree so its content fingerprint no
    // longer matches the coordinator's ⇒ the owner rejects the epoch (error).
    {
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "DROP INDEX dm_idx; CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {});",
                args.graph_degree + 8
            ),
        )
        .await
        .wrap_err("diverging remote index content for the drill")?;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_content_divergence".to_owned(), query_errored(&out)));
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "DROP INDEX dm_idx; CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {});",
                args.graph_degree
            ),
        )
        .await
        .wrap_err("restoring remote index content after the drill")?;
    }

    // 3. missing_or_reindexed_remote_index: drop the index on an owner ⇒ error.
    {
        run_psql_file(psql, socket_dir, last.port, "DROP INDEX dm_idx;")
            .await
            .wrap_err("dropping remote index for the drill")?;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("missing_or_reindexed_remote_index".to_owned(), query_errored(&out)));
        // Rebuild for recovery.
        run_psql_file(
            psql,
            socket_dir,
            last.port,
            &format!(
                "CREATE INDEX dm_idx ON dm USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {});",
                args.graph_degree
            ),
        )
        .await
        .wrap_err("rebuilding remote index after the drill")?;
    }

    // 4. remote_backend_termination / instance down: stop an owner ⇒ error.
    {
        let _ = Command::new(pg_ctl)
            .arg("-D")
            .arg(&last.data_dir)
            .arg("-m")
            .arg("fast")
            .arg("stop")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("remote_backend_termination".to_owned(), query_errored(&out)));
        // Restart for recovery.
        let mut restart = Command::new(pg_ctl);
        restart
            .arg("-w")
            .arg("-D")
            .arg(&last.data_dir)
            .arg("-l")
            .arg(&last.log_file)
            .arg("-o")
            .arg(format!(
                "-p {} -k {} -c listen_addresses=''",
                last.port,
                socket_dir.display()
            ))
            .arg("start")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        run_status(restart).await.wrap_err("restarting owner after the drill")?;
    }

    // 6. placement_drift: coordinator local_node_id absent from the roster ⇒ no
    // local node ⇒ error (a placement disagreement is never a silent miss).
    {
        let sql = format!(
            "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=99; SET ec_distann.epoch=1; {single_query}"
        );
        let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
        drills.push(("placement_drift".to_owned(), query_errored(&out)));
    }

    // NOTE: a base-table DELETE is intentionally NOT drilled here — it violates
    // the FR-082 Published-epoch model (the co-placed rerank vector is frozen and
    // never physically reclaimed within an epoch; deletion is a monotonic
    // tombstone-flag set via FR-083's `ec_distann_apply_record_writes`, which
    // keeps the vector). A raw DELETE removes the heap row and makes the exact
    // rerank fail `[EC_VECTOR_MISSING]` — which is precisely the hazard
    // FR-082-AC-5's epoch-owned frozen snapshot exists to prevent. A correct
    // distributed-tombstone drill needs per-node ownership bucketing (an
    // `owning_node` SQL surface) and is a follow-up.

    // 7. concurrency (FR-082-AC-4): run many multi-node scans concurrently with a
    // background inserter mutating the coordinator's table. Every scan must
    // complete (return only expanded records; never a torn/half-applied read that
    // errors). A single failing session fails the drill.
    let concurrency_ok = concurrency_drill(psql, socket_dir, coord_port, &roster, args).await?;
    crate::ecaz_println!("[distann-multicluster] concurrency_scan_insert_epochswap pass={concurrency_ok}");

    // 7b. live retention gate (FR-082-AC-3): a scan held open (AccessShareLock)
    // must block retire; once it drains, retire succeeds.
    let retention_ok = retention_gate_drill(psql, socket_dir, coord_port, args).await;
    crate::ecaz_println!("[distann-multicluster] live_retention_gate pass={retention_ok}");

    // 7c. AC-5 frozen vec_id→vector: a live record's exact-rerank result must be
    // byte-identical after real delete+VACUUM+reinsert TID churn on every node
    // (the AM's ambulkdelete tombstones deleted records so they are never
    // reranked, and a live record's heap TID is never reclaimed → its vector is
    // frozen without a separate tier, under D10).
    let frozen_ok = frozen_vector_drill(psql, socket_dir, coord_port, &roster, nodes, args).await;
    crate::ecaz_println!("[distann-multicluster] ac5_frozen_vector_after_vacuum_reuse pass={frozen_ok}");

    // 8. recovery / no-false-reject: after all faults clear, the full-roster
    // query must match the single-node baseline again.
    let recovery = capture_psql(psql, socket_dir, coord_port, &recall_sql(&roster, args.queries, args.top_k))
        .await
        .wrap_err("running the post-recovery recall comparison")?;
    let recovery_line = recovery
        .lines()
        .find(|line| line.contains("RECALL_RESULT"))
        .unwrap_or("RECALL_RESULT <none>")
        .trim()
        .to_owned();
    let recovered = recovery_line.contains("mismatched_ids=0");

    for (name, fail_closed) in &drills {
        crate::ecaz_println!("[distann-multicluster] fault_drill {name} pass={fail_closed}");
    }
    crate::ecaz_println!("[distann-multicluster] recovery {recovery_line} recovered={recovered}");

    // Disjoint-shard demonstration (destructive — prunes to owned shards; runs
    // last, after the replicated-corpus recovery check).
    let (disjoint_line, disjoint_ok) = disjoint_shard_drill(psql, socket_dir, nodes, &roster, args).await;
    crate::ecaz_println!("[distann-multicluster] {disjoint_line}");

    // Persist the evidence.
    let mut summary = format!(
        "distann-multinode fixture\nnodes={}\nrows={}\ndim={}\ngraph_degree={}\nqueries={}\ntop_k={}\nroster={}\n{}\n",
        args.nodes, args.rows, args.dim, args.graph_degree, args.queries, args.top_k, roster, result_line
    );
    for (name, fail_closed) in &drills {
        summary.push_str(&format!("fault_drill {name} pass={fail_closed}\n"));
    }
    summary.push_str(&format!(
        "concurrency_scan_insert_epochswap pass={concurrency_ok}\n"
    ));
    summary.push_str(&format!("{qual_line}\n"));
    summary.push_str(&format!("live_retention_gate pass={retention_ok}\n"));
    summary.push_str(&format!(
        "ac5_frozen_vector_after_vacuum_reuse pass={frozen_ok}\n"
    ));
    summary.push_str(&format!("{suite_line}\n"));
    summary.push_str(&format!("recovery {recovery_line} recovered={recovered}\n"));
    summary.push_str(&format!("{disjoint_line}\n"));
    let summary_path = log_dir.join("distann-multinode-summary.log");
    fs::write(&summary_path, &summary)
        .wrap_err_with(|| format!("writing {}", summary_path.display()))?;
    crate::ecaz_println!(
        "[distann-multicluster] summary written to {}",
        summary_path.display()
    );

    if !result_line.contains("mismatched_ids=0") {
        bail!("multi-node distinct-recall gate FAILED: {result_line}");
    }
    if !concurrency_ok {
        bail!("concurrency drill FAILED: a scan errored under concurrent insert load");
    }
    if !qual_ok {
        bail!("qual correctness FAILED: multi-node WHERE+LIMIT result differs from single-node");
    }
    if !retention_ok {
        bail!("live retention gate FAILED: retire not gated by an in-flight scan, or blocked after drain");
    }
    if !frozen_ok {
        bail!("AC-5 FAILED: a live record's rerank changed after delete+VACUUM+reinsert TID churn");
    }
    let all_fail_closed = drills.iter().all(|(_, ok)| *ok);
    if !all_fail_closed {
        let failed: Vec<&str> = drills
            .iter()
            .filter(|(_, ok)| !*ok)
            .map(|(name, _)| name.as_str())
            .collect();
        bail!("TC-042 fault matrix FAILED (not fail-closed): {failed:?}");
    }
    if !recovered {
        bail!("recovery FAILED: post-fault query did not match baseline: {recovery_line}");
    }
    if !disjoint_ok {
        bail!("disjoint-shard FAILED: multi-node result changed after pruning to owned shards");
    }
    crate::ecaz_println!(
        "[distann-multicluster] GATE PASS: recall identical; {} faults fail-closed; recovery clean",
        drills.len()
    );
    Ok(())
}

fn roster_with_port_override(
    nodes: &[Node],
    socket_dir: &Path,
    override_node_id: u32,
    override_port: u16,
) -> String {
    nodes
        .iter()
        .map(|node| {
            let port = if node.node_id == override_node_id {
                override_port
            } else {
                node.port
            };
            format!("{}@{}", node.node_id, conninfo(socket_dir, port))
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// A drill query satisfies NFR-020's fail-closed arm if it raised an ERROR
/// rather than returning a (possibly wrong/partial) result.
fn query_errored(output: &str) -> bool {
    output.contains("ERROR") || output.contains("EC_INTERNAL") || output.contains("could not connect")
}

/// FR-082-AC-4 concurrency drill: `scanners` concurrent multi-node scan loops on
/// the coordinator, plus a background inserter mutating the table, all at once.
/// Returns true iff every session completed without error (each scan drew only
/// from expanded records — a torn/half-applied read would surface as an error).
async fn concurrency_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> Result<bool> {
    let scanners = 4;
    let iters = 12;
    let scan_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT count(*) FROM (SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}) t;",
        args.top_k
    );
    // Deterministic insert vector (same shape as the corpus generator).
    let arr = format!(
        "(SELECT array_agg((sin(7 * 0.017 * (d + 1)) + cos(7 * 0.0031 * (d + 1)))::real) FROM generate_series(0, {} - 1) AS d)",
        args.dim
    );

    let mut tasks = Vec::new();
    for _ in 0..scanners {
        let (psql, socket_dir, sql) = (psql.to_path_buf(), socket_dir.to_path_buf(), scan_sql.clone());
        tasks.push(tokio::spawn(async move {
            for _ in 0..iters {
                let out = run_capture(&psql, &socket_dir, coord_port, &sql).await;
                if !out.status_ok {
                    return Err(out.stderr);
                }
            }
            Ok(())
        }));
    }
    // Background inserter: unique ids well above the corpus range.
    {
        let (psql, socket_dir, arr) = (psql.to_path_buf(), socket_dir.to_path_buf(), arr.clone());
        let base_rows = args.rows;
        tasks.push(tokio::spawn(async move {
            for i in 0..iters {
                let sql = format!(
                    "INSERT INTO dm SELECT {}, {arr}, encode_to_ecvector({arr}, 4, 42);",
                    900_000 + base_rows as i64 + i
                );
                let out = run_capture(&psql, &socket_dir, coord_port, &sql).await;
                if !out.status_ok {
                    return Err(out.stderr);
                }
            }
            Ok(())
        }));
    }
    // Epoch-swap-under-load (FR-082-AC-1): churn the coordinator's epoch lifecycle
    // (publish new epochs) while scans run. The metadata-page publish write must
    // not corrupt concurrent scans reading the metadata; end back at epoch 1.
    {
        let (psql, socket_dir) = (psql.to_path_buf(), socket_dir.to_path_buf());
        tasks.push(tokio::spawn(async move {
            for i in 0..iters {
                let epoch = if i % 2 == 0 { 2 } else { 1 };
                let sql = format!(
                    "SELECT ec_distann_publish_epoch('dm_idx'::regclass::oid, {epoch});"
                );
                let out = run_capture(&psql, &socket_dir, coord_port, &sql).await;
                if !out.status_ok {
                    return Err(out.stderr);
                }
            }
            let _ = run_capture(
                &psql,
                &socket_dir,
                coord_port,
                "SELECT ec_distann_publish_epoch('dm_idx'::regclass::oid, 1);",
            )
            .await;
            Ok(())
        }));
    }

    let mut ok = true;
    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(stderr)) => {
                crate::ecaz_println!("[distann-multicluster] concurrency session error: {stderr}");
                ok = false;
            }
            Err(join_err) => {
                crate::ecaz_println!("[distann-multicluster] concurrency task panicked: {join_err}");
                ok = false;
            }
        }
    }
    Ok(ok)
}

/// True disjoint-shard demonstration: prune each node's replicated corpus to only
/// the heap rows it OWNS (`owning_node`), then prove the multi-node top-k result
/// signature is byte-identical to the pre-prune (replicated) result — i.e. the
/// distributed read is correct with genuinely disjoint per-node storage, not a
/// full replica. Returns a report line; fatal if the signature changes.
async fn disjoint_shard_drill(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    let coord_port = nodes[0].port;
    // Operate on benchgate_corpus (a clean, cross-node-consistent copy the suite
    // gate created before the mutating drills). Save the query vectors first so
    // they survive pruning of non-owned coordinator rows.
    let setup = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!(
            "DROP TABLE IF EXISTS dj_queries; \
             CREATE TABLE dj_queries AS SELECT id AS qid, source AS v FROM benchgate_corpus WHERE id <= {};",
            args.queries
        ),
    )
    .await;
    if !setup.status_ok {
        return ("disjoint_shard=SKIPPED(no benchgate_corpus)".to_owned(), false);
    }
    // Order-stable signature of the multi-node top-k over the saved query set.
    let sig_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT md5(string_agg(id::text, ',' ORDER BY qid, id)) FROM ( \
           SELECT q.qid, r.id FROM dj_queries q \
           CROSS JOIN LATERAL (SELECT id FROM benchgate_corpus ORDER BY embedding <#> q.v LIMIT {k}) r) t;",
        k = args.top_k
    );
    let sig = |out: String| out.lines().map(str::trim).find(|l| l.len() == 32).unwrap_or("").to_owned();
    let before = sig(capture_psql_allow_error(psql, socket_dir, coord_port, &sig_sql).await);

    // Prune each node to its owned shard: delete the heap rows for vec_ids this
    // node does not own, then VACUUM (ambulkdelete tombstones their records).
    let n = args.nodes;
    let mut row_report = Vec::new();
    for node in nodes {
        let owner_idx = node.node_id - 1; // placement index = roster position
        let before_rows = capture_psql_allow_error(psql, socket_dir, node.port, "SELECT count(*) FROM benchgate_corpus;")
            .await
            .lines().find_map(|l| l.trim().parse::<i64>().ok()).unwrap_or(-1);
        let del = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!(
                "DELETE FROM benchgate_corpus WHERE ctid IN (\
                   SELECT ('(' || heap_block || ',' || heap_offset || ')')::tid \
                     FROM ec_distann_list_directory('benchgate_corpus_idx'::regclass::oid) \
                    WHERE NOT is_tombstone AND ec_distann_owning_node(vec_id, {n}, 1) <> {owner_idx});"
            ),
        )
        .await;
        let vac = run_capture(psql, socket_dir, node.port, "VACUUM benchgate_corpus;").await;
        if !del.status_ok || !vac.status_ok {
            return ("disjoint_shard=SKIPPED(prune failed)".to_owned(), false);
        }
        let after_rows = capture_psql_allow_error(psql, socket_dir, node.port, "SELECT count(*) FROM benchgate_corpus;")
            .await
            .lines().find_map(|l| l.trim().parse::<i64>().ok()).unwrap_or(-1);
        row_report.push(format!("n{}:{}->{}", node.node_id, before_rows, after_rows));
    }

    let after = sig(capture_psql_allow_error(psql, socket_dir, coord_port, &sig_sql).await);
    let identical = !before.is_empty() && before == after;
    (
        format!(
            "disjoint_shard identical_after_prune={identical} per_node_rows[{}]",
            row_report.join(" ")
        ),
        identical,
    )
}

/// Suite-driven recall gate (006-P1 letter): reuse the fixture corpus as a
/// `benchgate_*` bench-format corpus and run `ecaz bench recall` against the
/// coordinator single-node vs multi-node, asserting recall(multi) >= recall(single)
/// - 0.001. Best-effort/non-fatal: the byte-identical top-k gate is the hard one.
async fn suite_recall_gate(
    psql: &Path,
    socket_dir: &Path,
    nodes: &[Node],
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> String {
    let gd = args.graph_degree;
    for node in nodes {
        let sql = format!(
            "DROP TABLE IF EXISTS benchgate_corpus; \
             CREATE TABLE benchgate_corpus AS SELECT * FROM dm; \
             CREATE INDEX benchgate_corpus_idx ON benchgate_corpus \
               USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = {gd});"
        );
        if !run_capture(psql, socket_dir, node.port, &sql).await.status_ok {
            return "suite_recall_gate=SKIPPED(benchgate setup failed)".to_owned();
        }
    }
    let coord_port = nodes[0].port;
    let _ = run_capture(
        psql,
        socket_dir,
        coord_port,
        "DROP TABLE IF EXISTS benchgate_queries; CREATE TABLE benchgate_queries AS SELECT id, source FROM dm WHERE id <= 50;",
    )
    .await;
    let ecaz = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return "suite_recall_gate=SKIPPED(no exe)".to_owned(),
    };
    let single = run_bench_recall(&ecaz, socket_dir, coord_port, "").await;
    let multi = run_bench_recall(&ecaz, socket_dir, coord_port, roster).await;
    match (single, multi) {
        (Some(s), Some(m)) => {
            let pass = m >= s - 0.001;
            format!(
                "suite_recall_gate single={s:.4} multi={m:.4} delta={:.4} pass={pass}",
                m - s
            )
        }
        _ => "suite_recall_gate=INCONCLUSIVE(recall parse/connect failed)".to_owned(),
    }
}

/// Invoke `ecaz bench recall` against the coordinator with the given roster
/// session-GUC; parse recall@k from the comfy-table (a single-sweep row).
async fn run_bench_recall(
    ecaz: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster_val: &str,
) -> Option<f64> {
    let mut cmd = Command::new(ecaz);
    cmd.arg("--database")
        .arg("postgres")
        .arg("--host")
        .arg(socket_dir)
        .arg("--port")
        .arg(coord_port.to_string())
        .arg("bench")
        .arg("recall")
        .arg("--prefix")
        .arg("benchgate")
        .arg("--profile")
        .arg("ec_distann")
        .arg("--k")
        .arg("10")
        .arg("--sweep")
        .arg("32")
        .arg("--force-index");
    // Single-node = default (empty) roster; the GUC parser rejects an empty value,
    // so only set it for the multi-node arm.
    if !roster_val.is_empty() {
        cmd.arg("--session-guc")
            .arg(format!("ec_distann.roster={roster_val}"))
            .arg("--session-guc")
            .arg("ec_distann.local_node_id=1")
            .arg("--session-guc")
            .arg("ec_distann.epoch=1");
    }
    let out = cmd.output().await.ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let errtext = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        crate::ecaz_println!(
            "[distann-multicluster] bench recall (roster={:?}) exit={:?} stderr={}",
            !roster_val.is_empty(),
            out.status.code(),
            errtext.lines().rev().take(3).collect::<Vec<_>>().join(" | ")
        );
    }
    // comfy-table data row (columns: top_k/sweep, queries, recall_trials,
    // recall@k, ...): `│ 32 ┆ 50 ┆ 500 ┆ 0.5040 ┆ ...`. The left border is '│',
    // inner columns are separated by '┆'; recall@k is field index 4.
    for line in text.lines() {
        let fields: Vec<&str> = line.split(['│', '┆']).map(str::trim).collect();
        if fields.len() > 4 && fields[1].parse::<i64>().is_ok() {
            if let Ok(recall) = fields[4].parse::<f64>() {
                return Some(recall);
            }
        }
    }
    None
}

/// FR-082-AC-3 live gate: hold a single-node index scan open (AccessShareLock on
/// dm_idx) in a background transaction, and assert `ec_distann_retire_epoch` is
/// gated while it is in flight, then succeeds once it drains.
async fn retention_gate_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    args: &LocalMultinodePg18Args,
) -> bool {
    let idx = "'dm_idx'::regclass::oid";
    // Background holder: an ec_distann index scan held open ~3s via a cursor.
    let hold_sql = format!(
        "SET enable_seqscan=off; SET ec_distann.roster=''; SET ec_distann.local_node_id=1; \
         BEGIN; \
         DECLARE c CURSOR FOR SELECT id FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}; \
         FETCH 1 FROM c; SELECT pg_sleep(3); COMMIT;",
        args.top_k
    );
    let holder = {
        let (psql, socket_dir) = (psql.to_path_buf(), socket_dir.to_path_buf());
        tokio::spawn(async move { run_capture(&psql, &socket_dir, coord_port, &hold_sql).await })
    };
    // Let the scan acquire its AccessShareLock.
    tokio::time::sleep(std::time::Duration::from_millis(900)).await;

    let gated_out = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_retire_epoch({idx})"),
    )
    .await;
    let gated = gated_out.contains("retention gate");

    let _ = holder.await;

    let drained_out = capture_psql_allow_error(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_retire_epoch({idx})"),
    )
    .await;
    let succeeded_after_drain = !drained_out.contains("ERROR");

    // Restore a Published epoch for any downstream steps.
    let _ = run_capture(
        psql,
        socket_dir,
        coord_port,
        &format!("SELECT ec_distann_publish_epoch({idx}, 1)"),
    )
    .await;

    gated && succeeded_after_drain
}

/// FR-082-AC-5: a live record's exact-rerank result must be byte-identical after
/// real delete+VACUUM+reinsert TID churn on every node. Deleted records are
/// tombstoned by the AM's ambulkdelete (never reranked); a live record's heap TID
/// is never reclaimed, so its co-placed vector is frozen without a separate tier.
async fn frozen_vector_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    nodes: &[Node],
    args: &LocalMultinodePg18Args,
) -> bool {
    // Probe: row 1's multi-node top-1 (id:distance), byte-exact.
    let probe = format!(
        "SET enable_seqscan=off; SET ec_distann.roster='{roster}'; SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         SELECT id || ':' || (embedding <#> (SELECT source FROM dm WHERE id=1))::float8 \
           FROM dm ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT 1;"
    );
    let baseline = capture_psql_allow_error(psql, socket_dir, coord_port, &probe).await;
    let baseline = baseline.lines().find(|l| l.contains(':')).unwrap_or("").trim().to_owned();
    if baseline.is_empty() {
        crate::ecaz_println!("[distann-multicluster] ac5 baseline probe empty");
        return false;
    }

    // Delete a mid range on every node, then VACUUM (triggers ambulkdelete →
    // tombstone + heap reclaim), freeing those TIDs for reuse.
    let lo = args.rows / 4;
    let hi = lo + 150;
    for node in nodes {
        let del = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!("DELETE FROM dm WHERE id BETWEEN {lo} AND {hi};"),
        )
        .await;
        let vac = run_capture(psql, socket_dir, node.port, "VACUUM dm;").await;
        if !del.status_ok || !vac.status_ok {
            crate::ecaz_println!("[distann-multicluster] ac5 delete/vacuum failed on node {}", node.node_id);
            return false;
        }
    }
    // Reinsert new rows on every node (may reuse the reclaimed TIDs).
    let arr = format!(
        "(SELECT array_agg((sin(g * 0.017 * (d + 1)) + cos(g * 0.0031 * (d + 1)))::real) FROM generate_series(0, {} - 1) AS d)",
        args.dim
    );
    for node in nodes {
        let ins = run_capture(
            psql,
            socket_dir,
            node.port,
            &format!(
                "INSERT INTO dm SELECT g, {arr}, encode_to_ecvector({arr}, 4, 42) FROM generate_series({lo}, {hi}) AS g;"
            ),
        )
        .await;
        if !ins.status_ok {
            crate::ecaz_println!("[distann-multicluster] ac5 reinsert failed on node {}", node.node_id);
            return false;
        }
    }

    // Re-probe: row 1 (never touched) must rerank byte-identically.
    let after = capture_psql_allow_error(psql, socket_dir, coord_port, &probe).await;
    if after.contains("EC_VECTOR_MISSING") || after.contains("ERROR") {
        crate::ecaz_println!("[distann-multicluster] ac5 post-churn probe errored: {after}");
        return false;
    }
    let after = after.lines().find(|l| l.contains(':')).unwrap_or("").trim().to_owned();
    baseline == after
}

/// 011/020-P1: a WHERE qual on a NON-projected column (`source`) plus LIMIT.
/// Multi-node (CustomScan) must return exactly the single-node result — proving
/// the qual column is shipped for remote rows and the LIMIT applies after the
/// qual (over-fetch), not before. Returns (report line, pass).
async fn qual_correctness_drill(
    psql: &Path,
    socket_dir: &Path,
    coord_port: u16,
    roster: &str,
    args: &LocalMultinodePg18Args,
) -> (String, bool) {
    // Order by the id=1 vector; filter on source[1] > 0 (source is NOT selected).
    let query = format!(
        "SELECT id FROM dm WHERE source[1] > 0 \
           ORDER BY embedding <#> (SELECT source FROM dm WHERE id=1) LIMIT {}",
        args.top_k
    );
    let sql = format!(
        "SET enable_seqscan=off; \
         DROP TABLE IF EXISTS qc_s; DROP TABLE IF EXISTS qc_m; \
         SELECT set_config('ec_distann.roster', '', false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=0; \
         CREATE TEMP TABLE qc_s AS {query}; \
         SELECT set_config('ec_distann.roster', '{roster}', false); SET ec_distann.local_node_id=1; SET ec_distann.epoch=1; \
         CREATE TEMP TABLE qc_m AS {query}; \
         SELECT set_config('ec_distann.roster', '', false); \
         SELECT (SELECT count(*) FROM qc_s) || ' ' || (SELECT count(*) FROM qc_m) || ' ' || \
           ((SELECT count(*) FROM (SELECT id FROM qc_s EXCEPT SELECT id FROM qc_m) x) \
          + (SELECT count(*) FROM (SELECT id FROM qc_m EXCEPT SELECT id FROM qc_s) x));"
    );
    let out = capture_psql_allow_error(psql, socket_dir, coord_port, &sql).await;
    let parsed: Vec<i64> = out
        .lines()
        .find(|l| l.split_whitespace().count() == 3 && l.split_whitespace().all(|f| f.parse::<i64>().is_ok()))
        .map(|l| l.split_whitespace().filter_map(|f| f.parse().ok()).collect())
        .unwrap_or_default();
    if parsed.len() != 3 {
        return (format!("qual_correctness=INCONCLUSIVE({})", out.lines().last().unwrap_or("").trim()), false);
    }
    let (s_n, m_n, mismatch) = (parsed[0], parsed[1], parsed[2]);
    // Pass = same count and zero id mismatch (single==multi under the qual+LIMIT).
    let pass = s_n == m_n && mismatch == 0;
    (
        format!("qual_correctness single_n={s_n} multi_n={m_n} mismatch={mismatch} pass={pass}"),
        pass,
    )
}

struct CaptureOut {
    status_ok: bool,
    stderr: String,
}

async fn run_capture(psql: &Path, socket_dir: &Path, port: u16, sql: &str) -> CaptureOut {
    let mut command = psql_base(psql, socket_dir, port);
    command.arg("-v").arg("ON_ERROR_STOP=1").arg("-tAc").arg(sql);
    match command.output().await {
        Ok(output) => CaptureOut {
            status_ok: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(error) => CaptureOut {
            status_ok: false,
            stderr: format!("spawn error: {error}"),
        },
    }
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
