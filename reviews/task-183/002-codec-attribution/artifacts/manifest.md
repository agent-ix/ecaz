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
- Release command: `target/release/ecaz bench suite run --config
  reviews/task-183/002-codec-attribution/artifacts/codec-attribution-suite.json`.
- Execution date: 2026-07-17 America/Los_Angeles.
- Release measurement: PASS. Completed 1, failed 0, skipped 0, missing 0,
  stale 0; duration 3,264,447 ms. Every node unanimously attested installed
  release extension SHA `c9011fb8b1e0e11ec2b94f2e45b29c0f0299f714`.
  The release runner contains the fixture remediation at `04cad857e`.
- Runner report records `04cad857e...-dirty` because the failed-attempt
  packet-local suite manifest was modified when the release CLI was rebuilt;
  all Rust sources were committed and clean at `04cad857e`. Installed extension
  provenance is clean and unanimous, and the measured config hash is unchanged.
- Suite status/audit: PASS. Artifacts: `run/status.log`, `run/audit.log`.

## Phase 1 result

- The trained RaBitQ and exact-neighbor arms emitted the identical aggregate
  ordered seed-ID digest
  `488caa73ad3f6c22864f9af309569ba4fe6edd72c8d535e71eec7bff78af6d50`.
- Trained RaBitQ: recall 0.9625 (95% CI 0.9532--0.9700); warm
  p50/p95/p99/max 43.8/55.6/62.3/63.2 ms.
- Same-seed exact neighbor: recall 0.9605 (0.9510--0.9682); warm
  p50/p95/p99/max 113.1/141.7/173.9/182.7 ms.
- Exact-neighbor delta: -0.0020 recall with overlapping intervals; +69.3 ms
  and 2.58x RaBitQ p50. No positive residual codec contribution is measured.
- Owner-scan RaBitQ oracle: recall 0.9970 (0.9935--0.9986); warm
  p50/p95/p99/max 2566.0/2601.3/2613.8/2614.3 ms. Remaining recall headroom
  over trained RaBitQ is +0.0345 at about 58.6x p50.
- Shared physical generation bytes: 2,496,626,688; control bytes: 24,576;
  coordinator source bytes: 1,666,260,992; single index bytes: 854,810,624.
- Shared head: 4,096 samples; 25,280,512 sample bytes; 611,691 graph bytes;
  25,892,203 cached-head estimate.
- Physical construction/publish: 907,537 / 1,041,779 ms. Single comparator
  construction: 416,431 ms.
- All Ready/Published rows had `non_owned=0`, `orphans=0`; physical serving,
  topology, two remote-owner materialization probes, and all engagement gates
  passed.

Decision: exact-neighbor is a Phase 1 NO-GO. Proceed to Phase 2 fixed-budget
coverage; do not pursue a new neighbor codec from this result.

## Durable result artifacts

- Suite manifest SHA-256:
  `c728ffd3a8004ea7190e6b21ac94133fcc064fb234a83809da12a6949312e4b4`.
- Results JSONL SHA-256:
  `5f5161051a207c5c760af6a1c882525d7c02481df04a6785dd1e00c7f15c8d0f`.
- Report SHA-256:
  `ad39d9817cc0afb21461e92ee1e5b8256fea6e7f21e73abebf3965ea042402f7`.
- Status SHA-256:
  `21f98b5638799654d05ec684c3d01a9469fb059673965d960277bbe0320d8bc1`.
- Audit SHA-256:
  `8524d98d4ca6cf564e9fe5d1682e7b79b3d75b1f03e63786afd9d8adf472ae56`.
- Compact step summary SHA-256:
  `60b4058fa82b10e7824542b66b8a1fd71ee39b2d858c5153e339febd4dd70afb`.
- Cited raw-log SHA-256 values:
  - `physical-trained-rabitq-recall.log`:
    `94db5df2cb934b393646c4b9f5617f141aeacd2a4d1f779f7d46d6c9148a56e4`
  - `physical-trained-rabitq-latency.log`:
    `9fb9dddc7a041930ad278a1c4603aaade2ab65b4e6719d02da0245ee6373d926`
  - `physical-trained-exact-neighbor-recall.log`:
    `02977986913b83386b0baa122040f0c665ee6a9c2a3d4f8e2dd7e7e6d2c2193d`
  - `physical-trained-exact-neighbor-latency.log`:
    `aa06cff6275422b3c67582c5bd1657b8fcd6dbe8ccd3ecd0f44c0acf27140b96`
  - `physical-owner-oracle-recall.log`:
    `611bb6879546d13ece61bb60df8e983789350f47750c0657b325f7f51f44ccc7`
  - `physical-owner-oracle-latency.log`:
    `d652006b039a616587bb2c8c8ffc642cc997a7860baa5dd62ea8e45c7230b9f7`
- PostgreSQL node
  logs, the full driver log, single-index raw logs, truth cache, and live run
  directory are not committed.

This packet claims only the 100k same-seed codec attribution and owner reference
above. Phase 2 policy selection remains unmeasured.
