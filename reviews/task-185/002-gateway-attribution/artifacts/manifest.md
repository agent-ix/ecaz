# Task 185 gateway attribution manifest

- Task bucket / packet: `reviews/task-185/002-gateway-attribution/`
- Implementation head: `436cae964`
- Lane: Phase 1 gateway/basin attribution only
- Suite config: `artifacts/gateway-attribution-100k-suite.json`
- Planned scale: 100k (`ec_real_100k`)
- Planned topology: three-owner physical sharded generation; one index per
  suite step; run directory under `/home/peter/.ecaz/clusters/`
- Frozen scan: cap 4096, exact head scoring, 32 seeds, graph degree 32, BW4,
  H100, RaBitQ traversal, exact final ranking
- Input split: evaluation rows 1--200; disjoint training rows 201--400
- Feature profile: `distann-head-attribution-benchmark`, diagnostic only
- Timestamp: 2026-08-07 America/Los_Angeles

## Validation artifacts

The following logs are packet-local and record commands/results; no benchmark
result is asserted until the suite run lands:

- `pg18-feature-check.log`
- `gateway-trace-test.log`
- `cli-check.log`
- `suite-audit.log`
- `suite-dry-run.log`

The live suite will add `run/suite-manifest.json`, `run/results.jsonl`, the
physical summary/recall/latency logs, and the generated `*-gateway-trace.json`.
