# SPIRE Scale Packet Artifact Manifest

Head SHA: to be filled by the measurement run.
Packet/topic: `30629-spire-scale-packet-runbook`

This manifest is a scaffold for the controlled AWS/RDS-class SPIRE scale
packet. Do not cite scale claims from this packet until the artifact rows below
are replaced with real run outputs.

| Artifact | Lane | Fixture | Storage format | Rerank mode | Command | Timestamp | Isolated one-index-per-table | Key result lines |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `load.log` | load | real10k or larger configured corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only load --log-file review/30629-spire-scale-packet-runbook/artifacts/load.log` | pending | yes | pending |
| `storage.log` | storage | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only storage --log-file review/30629-spire-scale-packet-runbook/artifacts/storage.log` | pending | yes | pending |
| `explain.log` | planner | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only explain --log-file review/30629-spire-scale-packet-runbook/artifacts/explain.log` | pending | yes | pending |
| `latency.log` | latency | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only latency --log-file review/30629-spire-scale-packet-runbook/artifacts/latency.log` | pending | yes | pending |
| `recall.log` | recall | same loaded corpus | SPIRE profile value | profile value | `ecaz bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --only recall --log-file review/30629-spire-scale-packet-runbook/artifacts/recall.log` | pending | yes | pending |

## Required Environment Record

- Instance class:
- Storage class and IOPS:
- PostgreSQL version:
- Extension SHA:
- Dataset and row count:
- Query count:
- Warmup policy:
- Shared buffers:
- Maintenance work mem:
- Effective `ec_spire` reloptions:
- Comparison AMs and reloptions:

## Local PG18 Preflight

These artifacts were generated on the local pgrx PG18 scratch cluster after
the Phase 8 `nprobe_per_level` pull-forward. They are command-readiness
evidence only; they do not satisfy the controlled AWS/RDS-class scale gate.

- Head SHA: `9f9869c013424f9b5b104c5096533c69557ca6a8`
- Packet/topic: `30629-spire-scale-packet-runbook`
- Fixture: `target/real-corpus/ec_hnsw_real_10k`, 10,000 corpus rows, 200
  query rows, 1536 dimensions
- Lane/profile: local recursive `ec_spire`
- Storage format: `turboquant`
- Rerank mode: `rerank_width=25`
- Reloptions: `nlists=32`, `nprobe=24`, `recursive_fanout=2`,
  `nprobe_per_level=2`
- Isolated one-index-per-table: yes, prefix `task30_spire_scale_local`

| Artifact | Lane | Command | Key result lines |
| --- | --- | --- | --- |
| `local-pg18-version.log` | setup | `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "select version()" --log-output review/30629-spire-scale-packet-runbook/artifacts/local-pg18-version.log` | PostgreSQL 18.3 on x86_64-pc-linux-gnu. |
| `local-create-database.log` | setup | `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "create database tqvector_bench" --log-output review/30629-spire-scale-packet-runbook/artifacts/local-create-database.log` | `CREATE DATABASE`. |
| `local-load-real10k-recursive.log` | load | `target/debug/ecaz corpus load --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_spire_scale_local --profile ec_spire --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --manifest-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_manifest.json --allow-manifest-mismatch --bits 4 --seed 42 --reloption storage_format=turboquant --reloption nlists=32 --reloption nprobe=24 --reloption rerank_width=25 --reloption recursive_fanout=2 --reloption nprobe_per_level=2 --log-file review/30629-spire-scale-packet-runbook/artifacts/local-load-real10k-recursive.log` | Built `task30_spire_scale_local_idx` in 83.26s; completed prefix in 112.79s. |
| `local-storage-real10k-recursive.log` | storage | `target/debug/ecaz bench storage --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_spire_scale_local --log-file review/30629-spire-scale-packet-runbook/artifacts/local-storage-real10k-recursive.log` | Table total 168.0 MiB; SPIRE index 8.2 MiB, 859.3 B/row. |
| `local-explain-real10k-recursive.sql` / `local-explain-real10k-recursive.log` | explain/planner | `target/debug/ecaz dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --file review/30629-spire-scale-packet-runbook/artifacts/local-explain-real10k-recursive.sql --log-output review/30629-spire-scale-packet-runbook/artifacts/local-explain-real10k-recursive.log` | Index scan cost `31.31..1848.06`; execution 72.306 ms; `effective_nprobe_per_level={24,2}` and `configured_above_level_1`. |
| `local-latency-real10k-recursive-table.log` | latency | `target/debug/ecaz bench latency --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_spire_scale_local --profile ec_spire --k 10 --concurrency 1 --iterations 100 --sweep 8,24 --rerank-width 25 --bits 4 --seed 42 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30629-spire-scale-packet-runbook/artifacts/local-latency-real10k-recursive-table.log` | nprobe 8 p50 50.8 ms / p95 57.5 ms; nprobe 24 p50 50.0 ms / p95 56.2 ms. |
| `local-recall-real10k-recursive-table.log` / `local-truth-real10k-recursive-k10.json` | recall | `target/debug/ecaz bench recall --database tqvector_bench --host /home/peter/.pgrx --port 28818 --prefix task30_spire_scale_local --profile ec_spire --k 10 --sweep 8,24 --rerank-width 25 --queries-limit 100 --bits 4 --seed 42 --force-index --truth-cache-file review/30629-spire-scale-packet-runbook/artifacts/local-truth-real10k-recursive-k10.json --log-output review/30629-spire-scale-packet-runbook/artifacts/local-recall-real10k-recursive-table.log` | nprobe 8 recall@10 0.9900 / NDCG@10 0.9993; nprobe 24 recall@10 0.9900 / NDCG@10 0.9993. |

## Completion Gate

The request packet can mark the scale item complete only after this manifest
contains packet-local raw logs and key result lines for load, storage, explain,
latency, and recall.
