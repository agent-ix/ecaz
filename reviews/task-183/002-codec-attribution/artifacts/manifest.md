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
- Installed extension measurement head:
  `c9011fb8b1e0e11ec2b94f2e45b29c0f0299f714`.
- Extension install: PASS. Command: `cargo pgrx install --release --pg-config
  /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features
  --features 'pg18 pg_test distann-head-attribution-benchmark'`; artifact:
  `implementation-install.log`; SHA-256:
  `3d2333fb12d21ce5ad013634c8d802623c978acd5e39d4ef010889338bc0148b`.
- Initial release CLI build: PASS. Command: `cargo build --release -p ecaz-cli`;
  artifact: `cli-release-build.log`; SHA-256:
  `ea43012aa364c0ec2b800cc12735d0c3bf61e8216bf8910a33e6d5ec07095e18`.
- First suite attempt: setup-only failure before generation construction or
  measurement. The 100k bulk load triggered autovacuum, whose relation lock
  correctly caused `ec_distann_begin_epoch_build` to fail closed with
  `EC_BUILD_BUSY`. No recall, latency, storage, or codec result was produced.
- Fixture remediation head: `04cad857e`. Fresh physical fixture source tables
  now disable table/TOAST autovacuum and run explicit `ANALYZE dm` before index
  and epoch construction. This removes the setup race without weakening the
  production build gate. CLI check: PASS; artifact `setup-fix-check.log`;
  SHA-256:
  `8f26adaae1b065d68466a4031f17c4af08a7d543f53e2ef45f56f95376ba7bad`.
- Remediated release CLI build: PASS. Command:
  `cargo build --release -p ecaz-cli`; artifact `cli-release-rebuild.log`;
  SHA-256:
  `12ee14a6e8a345709fb5dcd369fc5479b5217098ed19180e19c5402e86f1ae75`.
- Release measurement: pending. Every node must attest the installed extension
  head above and release profile unanimously; the release runner contains the
  fixture remediation at `04cad857e`.

No measurement result is claimed yet. The suite must emit equal aggregate seed
ID digests for the two trained arms or terminate before their recall and
latency rows become decision-grade.
