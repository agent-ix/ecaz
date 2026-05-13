# Review Request: SPIRE Pipeline Query Metrics

## Summary

Partial progress on the Phase 12.9 row:

> Add or extend `ecaz bench spire-pipeline` for distributed recall, latency,
> and counter capture across local instances.

This slice extends `ecaz bench spire-pipeline` with optional coordinator query
metrics:

- `--include-query-metrics` runs the coordinator KNN SQL for each sampled query
  and reports latency min/p50/p95/p99/max per `nprobe` sweep value.
- `--include-recall` loads exact local truth for the sampled queries and
  reports recall@k in the same `Coordinator query metrics` section.
- `--query-metric-k` controls the query metric limit independently from the
  remote pipeline diagnostic `--top-k`.

The existing routing, local pipeline, optional remote fanout, and
`--remote-tuple-transport` controls remain in the same command, so future
packet-local readiness runs can collect query metrics and pipeline counters
from one CLI entry point instead of coordinating separate scripts.

This packet does not claim production benchmark results. It only adds the CLI
harness surface and pure/parser coverage.

## Files

- `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`
- `crates/ecaz-cli/src/cli.rs`
- `crates/ecaz-cli/README.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`
- `review/30965-spire-pipeline-query-metrics/artifacts/manifest.md`

## Validation

Packet-local logs are in `artifacts/` and indexed by
`artifacts/manifest.md`.

- `cargo test -p ecaz-cli spire_pipeline`
- `cargo check --no-default-features --features pg18`
- `git diff --check f86f690c^ f86f690c`

No live PostgreSQL fixture was run for this slice. The covered surface is CLI
parsing, report rendering, query-matrix validation, recall rendering, and the
PG18 build/check path.

## Reviewer Focus

- Confirm `--include-query-metrics` and `--include-recall` are scoped as
  coordinator query metrics inside `spire-pipeline`, not as a replacement for
  the broader final readiness bundle.
- Confirm `--query-metric-k` is independent from remote diagnostic `--top-k`.
- Confirm the report keeps routing/local/remote counters and query metrics in
  distinct sections.
- Confirm the tracker update remains a sub-bullet under the still-open Phase
  12.9 benchmark row.
