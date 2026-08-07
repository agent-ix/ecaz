# Task 185 gateway attribution manifest

- Task bucket / packet: `reviews/task-185/002-gateway-attribution/`
- Implementation head: `801bf5d78`
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
- Suite command: `/home/peter/.ecaz/task185-bin/ecaz bench suite run --config artifacts/gateway-attribution-100k-suite.json`
- Run directory: `/home/peter/.ecaz/clusters/task185-gateway-attribution-100k` (removed after all cited artifacts were captured)
- Suite result: one step succeeded, exit code 0, duration 2,126,724 ms; no missing expected artifacts
- Runner commit: `5487636f976a27a6e08c4ae16980470b0140ebbe-dirty` (stable copied executable; source was dirty only because this packet update was not yet committed)
- Build provenance: release profile with `distann-head-attribution-benchmark`; feature-build diagnostic, not a featureless production-latency run
- Installed extension SHA: `be1871b620be91e6a2424918cebf84a2b6df67c3` (the driver correction is CLI-only and does not alter the installed trace endpoint)
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

- `run/suite-manifest.json` — immutable suite provenance and expected-artifact status; SHA-256 `9104a32a9d6a9586d1950174672677f730187f43a2a35694a5a9961f8a55d903`
- `run/results.jsonl` — normalized suite rows; SHA-256 `1846ea1ae9486c5a377e2069de130fbb30b16f7b8e1955d8de8ef9b1a7e9a09e`
- `run/training-landmarks-control-100k/distann-multinode-summary.log` — topology, provenance, coverage, build, recall, latency, storage, head, and engagement lines; SHA-256 `4213ce35ab94df256cb52b84839cf54e6bad178a51c5ea5c33ad0f95f2ab5ad5`
- `run/training-landmarks-control-100k/physical-control-gateway-trace.json` — 200 disjoint-training traces; SHA-256 `572e39e1eb3403f0950af57f9b1b80403004e812ef6461a008511529e8a893cb`
- `run/training-landmarks-control-100k/physical-control-gateway-analysis.json` — exact truth join, per-seed coverage/marginals, redundancy, and hard-query summary; SHA-256 `808b610729423786dab1591805a08cd1b904f844ce8b99e1d3cdb6a29c52522d`
- `run/training-landmarks-control-100k/physical-head-membership.json` — fixed 4,096-member head; SHA-256 `b10f32bbe9cde0b318fcefc8aaa6a081c7198c31cf990f98e9fda50b56b33686`
- `run/training-landmarks-control-100k/physical-control-recall.log` — recall table and CI; SHA-256 `0ac11acc9c7b073d17abfcaecfe0e6d0e68f562b08981ec5038da085ea310328`
- `run/training-landmarks-control-100k/physical-control-latency.log` — one warm latency sample; SHA-256 `4fc16f7c8f549c097841c642891c7b178d37d2e1a94af058dda89eb77178c83f`

Key trace audit: 200 disjoint-training traces, each with 32 seeds and query
IDs 1--200 corresponding to source rows 201--400; 7,056 unique expansions;
95 shared-expansion events; 0 zero-hit queries; 0 malformed records; origin
mask histogram `1:1145, 2:297, 4:825, 5:22, 8:4694, 9:21, 10:7, 12:45`.

Truth-join audit: exact inner-product top-10 over the 100,000-row corpus
(`corpus_sha256=07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`)
found 1,523/2,000 truth ids (76.15%), with three zero-success queries and 30
redundant seed truth hits. The logical-id-to-global-vec-id reconstruction was
checked against every member of the persisted 4,096-row head. The corpus and
any truth cache remain regenerable inputs and are not committed.

Key benchmark lines: recall@32 `0.9205`, CI `[0.9078, 0.9316]`; warm latency
`41.20 ms` (count 1); physical generation bytes `2,496,626,688`; head sample
count `4096`; topology `owners=3`, `remote_verified=2`; engagement `pass=true`.
These metrics are control context only. No policy promotion or production
latency claim is made from this feature-build diagnostic.
