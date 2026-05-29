# Review Request: SPIRE Phase 13e Pooling Gate Report

## Summary

This slice implements the connection-pooling evidence gate in `ecaz bench suite report` without implementing pooling itself. The report now derives a `SPIRE Connection Pooling Gate` table from parsed `spire-pipeline` query metrics and production read profile rows.

The gate marks pooling as a candidate only when the profile evidence satisfies the Phase 13e threshold:

- connect/TLS p50 is at least 1 ms per query, or
- connect/TLS p95 is at least 15% of read p95 latency.

If real AWS profile evidence stays below both thresholds, the report labels pooling `pooling_not_justified`. If profile rows exist without matching latency p95 evidence, the report labels the row `missing_latency_p95` rather than guessing.

## Key Evidence

- `artifacts/pooling-gate-report.md`: generated suite report includes `SPIRE Connection Pooling Gate`.
- `artifacts/pooling-gate-spire-pipeline.log`: synthetic report fixture with both a not-justified row and a candidate row.
- `artifacts/pooling-gate-results.jsonl`: structured parsed rows emitted by the report command.
- `artifacts/manifest.md`: packet-local provenance for commands, fixture scope, and cited result lines.

Key report rows:

- `| synthetic-profile | 8 | 0.500 | 1.000 | 10.000 | 0.1000 | pooling_not_justified |`
- `| synthetic-profile | 16 | 1.200 | 1.600 | 10.000 | 0.1600 | pooling_candidate |`

## Validation

- `cargo test -p ecaz-cli suite`
- `cargo check -p ecaz-cli`
- `cargo build -p ecaz-cli`
- `cargo fmt --all -- --check`
- `target/debug/ecaz --database postgres bench suite report --manifest reviews/task-30/974-spire-phase13e-pooling-gate-report/artifacts/pooling-gate-suite-manifest.json --results-output reviews/task-30/974-spire-phase13e-pooling-gate-report/artifacts/pooling-gate-results.jsonl`

## Remaining Phase 13e Work

- Run the AWS correctness tier against real remote placements and capture selected PIDs, dispatch count, connect/TLS time, candidate time, heap time, payload bytes, merge time, and timeout/cancel counts.
- Run the representative AWS tier and capture p50/p95/p99 latency plus recall.
- Use the new report gate against real AWS profile rows before deciding whether bounded per-backend pooling is justified.
