# Task 185 gateway attribution manifest

- Task bucket / packet: `reviews/task-185/002-gateway-attribution/`
- Implementation / benchmark head: `be1871b620be91e6a2424918cebf84a2b6df67c3`
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
- Suite command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config artifacts/gateway-attribution-100k-suite.json`
- Run directory: `/home/peter/.ecaz/clusters/task185-gateway-attribution-100k` (removed after all cited artifacts were captured)
- Suite result: one step succeeded, exit code 0, duration 2,114,406 ms; no missing expected artifacts
- Build provenance: release profile with `distann-head-attribution-benchmark`; feature-build diagnostic, not a featureless production-latency run
- Query SHA-256: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Training slice: rows 201--400, SHA-256 `30f11df03f6e988adfe531e2bf54b75b8515fa207fee1212dd0774acffec7471`; evaluation slice: rows 1--200
- Surface: one physical three-owner sharded generation, shared-table fixture, no replica, two remote owners verified; `materialization_batch_size=0`

## Validation artifacts

The following validation artifacts are packet-local and record commands/results:

- `pg18-feature-check.log`
- `gateway-trace-test.log`
- `cli-check.log`
- `suite-audit.log`
- `suite-dry-run.log`

## Captured run artifacts

- `run/suite-manifest.json` — immutable suite provenance and expected-artifact status; SHA-256 `2e514ae321f12815deba40ee59e6f704c8abaf99b015702ca1dfc509bbd281f3`
- `run/results.jsonl` — normalized suite rows; SHA-256 `336054b5983f2a46c9fd38842b49d6d20dccc7e28b3c8f0ebaea4d22e925aefd`
- `run/training-landmarks-control-100k/distann-multinode-summary.log` — topology, provenance, coverage, build, recall, latency, storage, head, and engagement lines; SHA-256 `cb437c21db767c7e9ee9915cafd3c4d9d01cb6ee4f832fbb9c2360eab4aafa4a`
- `run/training-landmarks-control-100k/physical-control-gateway-trace.json` — 200 query traces; SHA-256 `02619bc2cc54c22f0595e55c63dc5d5b1fa08035c0fffefcbe407ed8a8579d8f`
- `run/training-landmarks-control-100k/physical-head-membership.json` — fixed 4,096-member head; SHA-256 `b10f32bbe9cde0b318fcefc8aaa6a081c7198c31cf990f98e9fda50b56b33686`
- `run/training-landmarks-control-100k/physical-control-recall.log` — recall table and CI; SHA-256 `b21ec36d0337733b90e773329e31ddfe293c87c064b164c625ba4e789bc4f360`
- `run/training-landmarks-control-100k/physical-control-latency.log` — one warm latency sample; SHA-256 `261ab946335602b6b341f3bdf3e0b13a1512c0b86319d7867f3a9351e11dad0f`

Key trace audit: 200 traces, each with 32 seeds; 6,584 unique expansions;
98 shared-expansion events; 0 zero-hit queries; 0 malformed records; origin
mask histogram `1:1551, 2:256, 4:1090, 5:42, 8:3589, 9:37, 10:1, 12:18`.

Key benchmark lines: recall@32 `0.9205`, CI `[0.9078, 0.9316]`; warm latency
`40.90 ms` (count 1); physical generation bytes `2,496,626,688`; head sample
count `4096`; topology `owners=3`, `remote_verified=2`; engagement `pass=true`.
These metrics are control context only. No policy promotion or production
latency claim is made from this feature-build diagnostic.
