# Artifact Manifest: SPIRE Boundary Recall Study

- head SHA: `54eece753ac9e262a88a3dca894dd6f44b6d897c`
- packet/topic: `30548-spire-boundary-recall-study`
- lane: Task 30 SPIRE Phase 5 boundary replication recall/storage study
- fixture: real 10k corpus from `target/real-corpus/ec_hnsw_real_10k`
- storage format: `turboquant`
- rerank mode: heap rerank with `rerank_width=25`
- surface: isolated one-index-per-table prefixes:
  - `task30_spire_boundary_off`
  - `task30_spire_boundary_rep1`
- timestamp: 2026-05-06 19:00-19:35 America/Los_Angeles

## Setup

Current extension install:

```text
cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18
```

Current CLI build:

```text
cargo build -p ecaz-cli
```

The scratch database had the AM installed but not the newest diagnostic SQL
function declarations in `pg_proc`, so the packet declared
`ec_spire_index_leaf_snapshot(oid)` against the installed `ecaz.so` before
capturing leaf assignment totals. The measured indexes were already built with
the current extension library and the intended reloptions.

## `load_real10k_boundary_off.log`

- head SHA: `54eece753ac9e262a88a3dca894dd6f44b6d897c`
- packet/topic: `30548-spire-boundary-recall-study`
- lane: boundary replication off
- fixture: real 10k / 200 queries / 1536 dimensions
- storage format / rerank mode: `turboquant` / `rerank_width=25`
- command used:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 corpus load --prefix task30_spire_boundary_off --profile ec_spire --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --manifest-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_manifest.json --allow-manifest-mismatch --dim 1536 --storage-format turboquant --reloption nlists=32 --reloption nprobe=24 --reloption rerank_width=25 --reloption boundary_replica_count=0 --log-file review/30548-spire-boundary-recall-study/artifacts/load_real10k_boundary_off.log
```

- key lines:
  - `task30_spire_boundary_off_turboquant_idx already exists with reloptions=[nlists=32, nprobe=24, rerank_width=25, boundary_replica_count=0, storage_format=turboquant]; skipping rebuild`
  - `corpus: 10000 rows`
  - `queries: 200 rows`

## `load_real10k_boundary_rep1.log`

- head SHA: `54eece753ac9e262a88a3dca894dd6f44b6d897c`
- packet/topic: `30548-spire-boundary-recall-study`
- lane: boundary replication on, `boundary_replica_count=1`
- fixture: real 10k / 200 queries / 1536 dimensions
- storage format / rerank mode: `turboquant` / `rerank_width=25`
- command used:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 corpus load --prefix task30_spire_boundary_rep1 --profile ec_spire --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --manifest-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_manifest.json --allow-manifest-mismatch --dim 1536 --storage-format turboquant --reloption nlists=32 --reloption nprobe=24 --reloption rerank_width=25 --reloption boundary_replica_count=1 --log-file review/30548-spire-boundary-recall-study/artifacts/load_real10k_boundary_rep1.log
```

- key lines:
  - `task30_spire_boundary_rep1_turboquant_idx already exists with reloptions=[nlists=32, nprobe=24, rerank_width=25, boundary_replica_count=1, storage_format=turboquant]; skipping rebuild`
  - `corpus: 10000 rows`
  - `queries: 200 rows`

## Recall Tables

Commands:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task30_spire_boundary_off --profile ec_spire --k 10 --sweep 4,8,16,24 --force-index --truth-cache-file review/30548-spire-boundary-recall-study/artifacts/real10k_truth_k10.json --log-file review/30548-spire-boundary-recall-study/artifacts/recall_real10k_boundary_off_cli.log --log-output review/30548-spire-boundary-recall-study/artifacts/recall_real10k_boundary_off_table.log
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task30_spire_boundary_rep1 --profile ec_spire --k 10 --sweep 4,8,16,24 --force-index --truth-cache-file review/30548-spire-boundary-recall-study/artifacts/real10k_truth_k10.json --log-file review/30548-spire-boundary-recall-study/artifacts/recall_real10k_boundary_rep1_cli.log --log-output review/30548-spire-boundary-recall-study/artifacts/recall_real10k_boundary_rep1_table.log
```

- `recall_real10k_boundary_off_table.log`
  - `nprobe=4 recall@k=0.9950 ndcg@k=0.9998 mean q-time=40.17 ms`
  - `nprobe=8 recall@k=0.9985 ndcg@k=0.9999 mean q-time=62.59 ms`
  - `nprobe=16 recall@k=1.0000 ndcg@k=1.0000 mean q-time=103.27 ms`
  - `nprobe=24 recall@k=1.0000 ndcg@k=1.0000 mean q-time=139.61 ms`
- `recall_real10k_boundary_rep1_table.log`
  - `nprobe=4 recall@k=0.9975 ndcg@k=0.9999 mean q-time=74.65 ms`
  - `nprobe=8 recall@k=0.9990 ndcg@k=1.0000 mean q-time=120.52 ms`
  - `nprobe=16 recall@k=1.0000 ndcg@k=1.0000 mean q-time=206.87 ms`
  - `nprobe=24 recall@k=1.0000 ndcg@k=1.0000 mean q-time=289.65 ms`
- `real10k_truth_k10.json`: packet-local exact truth cache.

## Storage and Assignment Artifacts

Commands:

```text
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench storage --prefix task30_spire_boundary_off --log-file review/30548-spire-boundary-recall-study/artifacts/storage_real10k_boundary_off.log
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 bench storage --prefix task30_spire_boundary_rep1 --log-file review/30548-spire-boundary-recall-study/artifacts/storage_real10k_boundary_rep1.log
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -A -F $'\t' -o review/30548-spire-boundary-recall-study/artifacts/leaf_snapshot_boundary_off.tsv -c "SELECT count(*) AS leaf_count, sum(base_assignment_count) AS base_assignments, sum(base_primary_assignment_count) AS base_primary, sum(base_boundary_replica_assignment_count) AS base_boundary, sum(effective_assignment_count) AS effective_assignments, sum(effective_boundary_replica_assignment_count) AS effective_boundary FROM ec_spire_index_leaf_snapshot('task30_spire_boundary_off_turboquant_idx'::regclass::oid);"
/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -A -F $'\t' -o review/30548-spire-boundary-recall-study/artifacts/leaf_snapshot_boundary_rep1.tsv -c "SELECT count(*) AS leaf_count, sum(base_assignment_count) AS base_assignments, sum(base_primary_assignment_count) AS base_primary, sum(base_boundary_replica_assignment_count) AS base_boundary, sum(effective_assignment_count) AS effective_assignments, sum(effective_boundary_replica_assignment_count) AS effective_boundary FROM ec_spire_index_leaf_snapshot('task30_spire_boundary_rep1_turboquant_idx'::regclass::oid);"
```

- `storage_real10k_boundary_off.log`
  - index size `8.2 MiB`
  - index per row `857.7 B`
  - total size `168.0 MiB`
- `storage_real10k_boundary_rep1.log`
  - index size `16.0 MiB`
  - index per row `1673.6 B`
  - total size `175.8 MiB`
- `leaf_snapshot_boundary_off.tsv`
  - `leaf_count=32`
  - `base_assignments=10000`
  - `base_primary=10000`
  - `base_boundary=0`
  - `effective_assignments=10000`
  - `effective_boundary=0`
- `leaf_snapshot_boundary_rep1.tsv`
  - `leaf_count=32`
  - `base_assignments=20000`
  - `base_primary=10000`
  - `base_boundary=10000`
  - `effective_assignments=20000`
  - `effective_boundary=10000`
