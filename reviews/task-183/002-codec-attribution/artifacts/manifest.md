# Task 183 codec-attribution manifest

- Implementation head: `229e7d7a5`
- Task bucket / packet: `reviews/task-183/002-codec-attribution/`
- Lane: local PG18, three physical owners, one fresh 100k generation
- Fixture: `ec_real_100k`; held-out evaluation rows 1--200; disjoint training
  rows 201--400
- Storage format: persisted RaBitQ neighbor codes with exact final rerank
- Matrix: trained persisted-head RaBitQ vs same-seed exact-neighbor vs O(N)
  owner-scan RaBitQ diagnostic
- Fixed work: cap 4,096; 32 seeds; BW4/H100; graph degree 32; top-10
- Latency: 50 warm iterations after 10 warmups, concurrency 1
- Suite config: `codec-attribution-suite.json`
- Isolation: all three variants share one immutable three-owner physical
  generation; the single-index comparator is built once by the suite step
- Corpus TSVs, query TSVs, truth cache, node logs, and live run directory are
  not committed

## Validation

- PG18 benchmark-feature compile: PASS. Command:
  `cargo check --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`;
  artifact: `pg18-feature-check.log`.
- CLI compile: PASS. Command: `cargo check -p ecaz-cli`; artifact:
  `cli-check.log`. It reports one pre-existing unused-field warning.
- Fail-closed digest unit tests: PASS, 2 passed / 0 failed. Command:
  `cargo test -p ecaz-cli same_seed_digest`; artifact: `digest-tests.log`.
- Suite dry run: PASS, one `trained-codec-100k` step expanded with all three
  pre-registered variants. Command: `target/debug/ecaz bench suite run --config
  reviews/task-183/002-codec-attribution/artifacts/codec-attribution-suite.json
  --dry-run`; artifacts: `dry-run.log`, `run/suite-manifest.json`.
- Config SHA-256:
  `2a04870350df772bd77007e6e5c0b5511fae8f14e5e235e014a0411b773ec27f`.
- Dry-run suite-manifest SHA-256:
  `e988719e195f445e9332b7884c37ed454dde5670447daea4a4e2639d1146bd07`.
- Release measurement: pending

No measurement result is claimed yet. The suite must emit equal aggregate seed
ID digests for the two trained arms or terminate before their recall and
latency rows become decision-grade.
