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

The installed-release SHA, commands, artifact digests, and cited results will
be appended after the suite completes.
