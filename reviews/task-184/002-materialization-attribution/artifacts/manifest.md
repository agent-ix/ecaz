# Task 184 materialization-attribution artifacts

Packet: `reviews/task-184/002-materialization-attribution/`

Implementation head: `0f4b1d44c7211f6a0017577551b054b1c45825fe`

Lane: benchmark-only materialization attribution on PG18.

Fixture: retained Task 183 production policy on a fresh isolated 100k physical
generation: three local owners, trained exact 4,096-entry head, 32 seeds,
BW4/H100, RaBitQ neighbor traversal, exact final ranking, staged corpus and
query files from `/home/peter/dev/ecaz/data/staged-current`.

Timestamp: 2026-07-19 America/Los_Angeles.

## Validation

- `cargo-check-attribution.log`
  - command: `cargo check --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
  - result: pass
- `cargo-check-production.log`
  - command: `cargo check --no-default-features --features pg18`
  - result: pass; no warnings
- `cargo-check-cli.log`
  - command: `cargo check -p ecaz-cli`
  - result: pass; only the pre-existing unused `path` warning
- `cargo-test-stage-counters.log`
  - command: `cargo test --lib --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark' am::ec_distann::stage_counters::tests::counters_accumulate_nested_samples_and_reset -- --exact --nocapture`
  - result: 1 passed, 0 failed
- `cargo-test-cli-materialization-work.log`
  - command: `cargo test -p ecaz-cli distann_materialization_work -- --nocapture`
  - result: 1 passed, 0 failed
- `cargo-test-cli-suite-parser.log`
  - command: `cargo test -p ecaz-cli distann_local_multinode_expands_task183_stage_profile -- --nocapture`
  - result: 1 passed, 0 failed
- `materialization-profile-100k-suite.json`
  - runner: `ecaz bench suite`
  - isolation: fresh one-index-per-table physical generation
  - work: 200 held-out recall queries / 2,000 distinct top-10 trials and 50
    timed latency queries after 10 warmups, concurrency 1

## Installed release and suite

- installed measurement head:
  `9d216fe8e1065e926a880d7bec660802545cff83`
- installed `ecaz.so` SHA-256:
  `7d435d4de632fa2f4b7892d223049cbc33a035fded0dd3fdd87313a4780a60b6`
- release CLI SHA-256:
  `ac8debb4c6e1c976f6ca474e31b1ebfa44f38a26f30b6c87106f4e481850a35a`
- `implementation-install.log`
  - command: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
  - result: pass; 1,119 SQL entities
  - SHA-256: `e16619e88c1a91aba3156d506807e1773963e87928393a13c26360d19aacee29`
- `cli-release-build.log`
  - command: `cargo build --release -p ecaz-cli`
  - result: pass; only the pre-existing unused `path` warning
  - SHA-256: `81eb33c4c26e25cb273c4e951eca7b29c6db69232e72c502e66fdab6f75481f9`
- `suite-audit.log`
  - command: `target/release/ecaz bench suite audit --config reviews/task-184/002-materialization-attribution/artifacts/materialization-profile-100k-suite.json`
  - result: pass; one step
  - SHA-256: `0f5c094625e7b01f602eb183aebfcc69174384d16118d057cc583114e40696fb`
- `suite-dry-run.log`
  - command: `target/release/ecaz bench suite run --config reviews/task-184/002-materialization-attribution/artifacts/materialization-profile-100k-suite.json --artifact-dir reviews/task-184/002-materialization-attribution/artifacts/run --dry-run`
  - result: intended retained-policy physical fixture with attribution enabled
  - SHA-256: `720dbd096a5591db3efd2372b62ddcd64a1a763bd20842fcbfb44cedb3d3ee0c`
- suite command: `target/release/ecaz bench suite run --config reviews/task-184/002-materialization-attribution/artifacts/materialization-profile-100k-suite.json --artifact-dir reviews/task-184/002-materialization-attribution/artifacts/run`
  - result: completed 1, failed 0, skipped 0, missing 0, stale 0
  - isolated one-index-per-table physical generation; three local PG18 owners
  - `suite-status.log` SHA-256:
    `9b8760240263f4c4a634e9cb920bd7c8ba7b2756ba394dc90432f032f28f43ba`
  - `suite-report.md` SHA-256:
    `41d09eec8d7d0336c33e4ff9f8c211cd23f212e7501233020d82261cabc77fbf`
  - `run/results.jsonl` SHA-256:
    `70fce9a592815e7cb84e86af916f3bdf7bc78a3378a52b8ebee325a33602b142`
  - `run/suite-manifest.json` SHA-256:
    `99d585883a71fcc1c140fd207735b77c3216108a3226ee46df0c19a97092c3c4`
  - summary SHA-256:
    `8692947827609db33513d0a2400767eb8ea92af28db2cf8ddb4f55909024d8d8`
  - physical recall SHA-256:
    `56b6e9f11dbb30bd82032c2edb1246269d145329c02f7b52615d569355c07e77`
  - physical latency SHA-256:
    `8dce29a0f45729ae720ddc684efcfddce6d72faaed866dfa8461b8a1e6e88930`
  - single recall SHA-256:
    `16875e47ee8c9991c77b10212985239c9d4cfc6b2dcfe41da0334367d89d8550`
  - single latency SHA-256:
    `6ae3c5589856669d6046f90fc56040937ee175921745057b22f38e90c196c2e1`

## Key results

- physical recall: 0.9625; 95% CI 0.9532--0.9700; 200 queries / 2,000
  distinct top-10 trials
- latency: mean 38.10 ms, p50 37.30 ms, p95 48.80 ms, p99 53.70 ms,
  max 55.80 ms; 50 timed after 10 warmups, concurrency 1
- remote materialization: 25.444944 ms/query; request wait 25.369306 ms;
  owner endpoint critical 22.728272 ms
- summed owner work: endpoint 39.980501 ms, payload SQL 32.276778 ms,
  open/validate 6.682875 ms, node lookup 1.010468 ms
- coordinator decode/map/association: 0.059556 / 0.002160 / 0.040305
  ms/query
- ranked/associated 40 rows/query; executor/client consumed 10; remote
  requested/returned/installed 26.84; remote consumed 6.64; logical payload
  bytes 496,003/query; zero tombstones
- storage: physical generation 2,496,626,688 bytes; control 24,576 bytes;
  coordinator source 1,666,260,992 bytes; single index 854,810,624 bytes;
  single source 2,519,465,984 bytes
- construction: physical 878,965 ms; publish 1,005,840 ms; single 405,723 ms
- topology, real remote materialization, query separation, and unanimous
  installed release provenance passed.

## Candidate selection

Pre-register only MAT-01/MAT-04 bounded global-ranked-window incremental
payload materialization, batch size 10, against the unchanged eager path. The
request defines its deterministic deepening, correctness matrix, failure
semantics, and work cap. No other candidate family is eligible in this task.
