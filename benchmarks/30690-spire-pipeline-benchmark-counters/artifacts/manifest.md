# Manifest: SPIRE Pipeline Benchmark Counters

- head SHA: `8b221c93c5d9f99f56b33268b4d19fa8da6fae13`
- packet/topic: `30690-spire-pipeline-benchmark-counters`
- lane: Task 30 Phase 10.7 performance harness
- fixture: not applicable for this CLI-surface checkpoint
- storage format: not applicable
- rerank mode: command supports `--rerank-width`; no live benchmark claim made
- command used:
  - `cargo test -p ecaz-cli spire_pipeline`
  - `cargo fmt --check`
  - `cargo run -p ecaz-cli -- bench spire-pipeline --help`
  - `git diff --cached --check`
- timestamp: 2026-05-09
- isolated one-index-per-table or shared-table surface: not applicable for this
  code checkpoint; Phase 10.7 points one-index-per-table evidence at
  `review/30686-spire-phase9-quality-baseline`
- key result lines:
  - `cargo test -p ecaz-cli spire_pipeline`: `5 passed; 0 failed`
  - `cargo fmt --check`: exit 0, stable-rustfmt warnings only
  - `cargo run -p ecaz-cli -- bench spire-pipeline --help`: command help
    renders with `Usage: ecaz bench spire-pipeline [OPTIONS] --prefix <PREFIX>`
  - `git diff --cached --check`: no whitespace errors

## Coverage Map

- recall / NDCG: existing `ecaz bench recall` artifacts in
  `review/30686-spire-phase9-quality-baseline` and
  `review/30687-spire-adaptive-nprobe`
- latency p50/p95/p99: existing `ecaz bench latency` artifacts in
  `review/30686-spire-phase9-quality-baseline` and
  `review/30687-spire-adaptive-nprobe`
- object bytes: existing `ecaz bench storage` artifacts in
  `review/30686-spire-phase9-quality-baseline`
- route counts: `ecaz bench spire-pipeline` via
  `ec_spire_index_scan_routing_snapshot` and
  `ec_spire_index_scan_pipeline_snapshot`
- candidate counts: `ecaz bench spire-pipeline` via
  `ec_spire_index_scan_pipeline_snapshot`
- heap rerank rows: `ecaz bench spire-pipeline` via
  `ec_spire_index_scan_pipeline_snapshot`
- local remote-fanout count: `ecaz bench spire-pipeline` via
  `ec_spire_index_scan_pipeline_snapshot`
- remote PID fanout count: `ecaz bench spire-pipeline --include-remote` or
  `--remote-selected-pids ...` via `ec_spire_remote_pipeline_steps`
